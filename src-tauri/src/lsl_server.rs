/// LSL HTTP 中继服务
/// 
/// 工作原理：
///   1. 本模块在本地启动一个极简 TCP HTTP 服务器（不依赖 axum/hyper）
///   2. SL 内的 HUD 通过 llHTTPRequest 发 POST /chat 到用户公网 IP:PORT
///   3. 服务器解析 JSON body，提取 sender 和 text
///   4. 通过 Tauri 事件系统向前端推送 "log-update" 事件（与日志监听一致）
///   5. 前端在 [附近] Tab 中显示并翻译该消息

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use crate::constants::EVENT_LOG_UPDATE;

const NEARBY_SOURCE: &str = "chat.txt";
const READ_TIMEOUT_SECS: u64 = 5;

/// 用于从外部停止服务器
pub static LSL_SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize, Clone)]
struct LogPayload {
    source: String,
    msg:    String,
}

/// 启动 LSL 接收 HTTP 服务器，返回监听地址字符串
pub fn start(port: u16, app: AppHandle) -> Result<String, String> {
    if LSL_SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("LSL server is already running".into());
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).map_err(|e| e.to_string())?;
    let listen_addr = listener.local_addr().map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        listener.set_nonblocking(true).ok();
        loop {
            if !LSL_SERVER_RUNNING.load(Ordering::Relaxed) {
                break;
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    let app = app.clone();
                    std::thread::spawn(move || handle_connection(stream, app));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    Ok(format!("{}:{}", get_local_ip(), listen_addr.port()))
}

/// 停止服务器
pub fn stop() {
    LSL_SERVER_RUNNING.store(false, Ordering::SeqCst);
}

/// 处理单个 TCP 连接：解析 HTTP 请求，提取 JSON body，推送事件
fn handle_connection(mut stream: TcpStream, app: AppHandle) {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(READ_TIMEOUT_SECS))).ok();

    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let raw = String::from_utf8_lossy(&buf[..n]);

    // 健康检查
    if raw.contains("GET /ping") {
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK");
        return;
    }

    // 仅处理 POST /chat
    if !raw.contains("POST /chat") {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        return;
    }

    // 提取 body（HTTP header 和 body 以 \r\n\r\n 分隔）
    let body = match raw.find("\r\n\r\n") {
        Some(idx) => raw[idx + 4..].trim().to_string(),
        None      => return,
    };

    // 解析 JSON：{"sender":"xxx","text":"yyy"}
    let sender = extract_json_str(&body, "sender").unwrap_or_default();
    let text   = extract_json_str(&body, "text").unwrap_or_default();

    if sender.is_empty() || text.is_empty() {
        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
        return;
    }

    // 构造与文件监听一致的 payload 格式：  "SenderName: message"
    let msg = format!("{}: {}", sender, text);
    let _ = app.emit(EVENT_LOG_UPDATE, LogPayload {
        source: NEARBY_SOURCE.to_string(),
        msg,
    });

    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK");
}

/// 极简 JSON 字符串字段提取（避免引入 serde_json 解析依赖）
fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let key_pos = json.find(&pattern)?;
    let after_key = &json[key_pos + pattern.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();

    if after_colon.starts_with('"') {
        let content = &after_colon[1..];
        let end = content.find('"')?;
        Some(content[..end].replace("\\\"", "\"").replace("\\n", "\n"))
    } else {
        None
    }
}

/// 尝试获取本机局域网 IP
fn get_local_ip() -> String {
    // 通过连接外部地址（不实际发包）探测本机出口 IP
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}
