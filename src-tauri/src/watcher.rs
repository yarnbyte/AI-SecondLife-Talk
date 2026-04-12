use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::constants::EVENT_LOG_UPDATE;
use tauri::{AppHandle, Emitter};

// 记录每个被监听文件的已读字节偏移量
type OffsetMap = Arc<Mutex<HashMap<PathBuf, u64>>>;

use std::sync::atomic::{AtomicBool, Ordering};

static IS_WATCHING: AtomicBool = AtomicBool::new(false);

pub struct LogWatcherService;

impl LogWatcherService {
    /// 启动监听：对 target_dir 下的所有 .log / .txt 文件进行增量轮询
    pub fn start_watching(app_handle: AppHandle, target_dir: PathBuf) -> Result<(), String> {
        if !target_dir.is_dir() {
            return Err(format!("聊天记录日志目录不存在: {}", target_dir.display()));
        }

        // 防多指/重复点击产生无限线程
        if IS_WATCHING.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let offsets: OffsetMap = Arc::new(Mutex::new(HashMap::new()));

        // 启动前先初始化所有现存文件的 offset 到末尾，避免重复读历史消息
        Self::init_offsets(&target_dir, &offsets);

        thread::spawn(move || {
            use notify::{RecursiveMode, Watcher};
            use std::sync::mpsc::channel;

            let (tx, rx) = channel();

            // 使用跨平台的系统原生文件通知监听器
            let mut watcher = match notify::recommended_watcher(tx) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("初始化 watcher 失败: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&target_dir, RecursiveMode::NonRecursive) {
                eprintln!("监听目录失败: {}", e);
                return;
            }

            // 阻塞等待系统发出的文件修改事件
            loop {
                if !IS_WATCHING.load(Ordering::SeqCst) {
                    break;
                }

                match rx.recv_timeout(std::time::Duration::from_millis(500)) {
                    Ok(Ok(event)) => {
                        for path in event.paths {
                            Self::poll_single_file(&path, &offsets, &app_handle);
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break, // 通道断开则退出线程
                }
            }
        });

        Ok(())
    }

    /// 停止监听
    pub fn stop_watching() {
        IS_WATCHING.store(false, Ordering::SeqCst);
    }

    /// 初始化：把目录下所有日志文件的 offset 设置到当前末尾
    fn init_offsets(dir: &PathBuf, offsets: &OffsetMap) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut map = offsets.lock().unwrap();
        for entry in entries.flatten() {
            let path = entry.path();
            if Self::is_log_file(&path) {
                if let Ok(meta) = std::fs::metadata(&path) {
                    map.insert(path, meta.len());
                }
            }
        }
    }

    /// 仅处理单个触发了修改事件的文件，避免全盘遍历带来的开销
    fn poll_single_file(path: &PathBuf, offsets: &OffsetMap, app: &AppHandle) {
        if !Self::is_log_file(path) {
            return;
        }

        let current_offset = {
            let map = offsets.lock().unwrap();
            *map.get(path).unwrap_or(&0)
        };

        let Ok(mut file) = File::open(path) else {
            return;
        };

        // 获取最新文件长度
        let Ok(meta) = file.metadata() else { return };
        let file_len = meta.len();

        if file_len <= current_offset {
            return;
        }

        // 移动到上次读取的位置
        let _ = file.seek(SeekFrom::Start(current_offset));

        use serde::Serialize;
        #[derive(Serialize, Clone)]
        struct LogMessage {
            source: String,
            msg: String,
        }

        // 使用 read_to_string 保证不管是不是 CRLF 换行都不会导致偏移量计算出错
        let mut read_buf = String::new();
        let mut reader = BufReader::new(file);
        let Ok(_) = reader.read_to_string(&mut read_buf) else {
            return;
        };

        // 以换行为界，把完整内容分发送
        for line in read_buf.lines() {
            if let Some(msg) = Self::parse_log_line(line) {
                let source_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("chat.txt")
                    .to_string();

                let _ = app.emit(
                    EVENT_LOG_UPDATE,
                    LogMessage {
                        source: source_name,
                        msg,
                    },
                );
            }
        }

        // 记录新的文件末尾位置
        offsets.lock().unwrap().insert(path.clone(), file_len);
    }

    /// 判断是否为 Firestorm 日志文件（.log 或 .txt）
    fn is_log_file(path: &PathBuf) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("log") | Some("txt")
        )
    }

    /// 解析 Firestorm 日志行，提取 "发送者: 内容"
    /// Firestorm 格式：[yyyy/MM/dd HH:mm]  SenderName: message
    /// 或者：         yyyy/MM/dd HH:mm  SenderName: message
    fn parse_log_line(line: &str) -> Option<String> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        // 跳过系统噪音行（不含冒号的时间戳行、空行等）
        // Firestorm 行格式：以 [ 开头或数字日期开头，后跟时间，再跟发送者和消息
        let msg_part = if line.starts_with('[') {
            // 格式: [2026/04/10 13:00]  SenderName: message
            let close = line.find(']')?;
            line[close + 1..].trim()
        } else if line.chars().next()?.is_ascii_digit() {
            // 格式: 2026/04/10 13:00  SenderName: message
            // 跳过日期时间部分（前 16 个字符）
            let rest = line.get(16..)?.trim();
            rest
        } else {
            return None;
        };

        // msg_part 现在是 "SenderName: message"
        if !msg_part.contains(": ") {
            return None;
        }

        // 过滤系统消息（Firestorm 典型噪音）
        let lower = msg_part.to_lowercase();
        if lower.starts_with("secondlife:") || lower.contains("teleport") && !lower.contains(": ") {
            return None;
        }

        Some(msg_part.to_string())
    }
}
