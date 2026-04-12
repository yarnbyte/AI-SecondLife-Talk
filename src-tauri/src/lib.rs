pub mod constants;
pub mod watcher;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::ShortcutState;
use tokio::sync::oneshot;

// ── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn start_listen_log(path_override: Option<String>, app: AppHandle) -> Result<String, String> {
    let target_dir = std::path::PathBuf::from(
        path_override.unwrap_or_else(|| constants::GAME_LOG_DIR_WIN.to_string()),
    );
    watcher::LogWatcherService::start_watching(app, target_dir).map(|_| "Started".to_string())
}

#[tauri::command]
fn stop_listen_log() {
    watcher::LogWatcherService::stop_watching();
}

#[tauri::command]
fn copy_to_clipboard(text: String, app: AppHandle) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_folder_dialog(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::FilePath;

    let (tx, rx) = oneshot::channel::<Option<FilePath>>();
    app.dialog()
        .file()
        .set_title("选择 Firestorm 日志目录")
        .pick_folder(move |result| {
            let _ = tx.send(result);
        });

    match rx.await {
        Ok(Some(path)) => path
            .into_path()
            .ok()
            .map(|p| p.to_string_lossy().to_string()),
        _ => None,
    }
}

#[tauri::command]
fn get_default_log_dir() -> Option<String> {
    std::env::var("APPDATA")
        .ok()
        .map(|appdata| format!("{}\\Firestorm_x64", appdata))
}

/// 根据客户端类型返回对应的日志目录路径
#[tauri::command]
fn get_viewer_log_dir(viewer: String) -> Option<String> {
    let appdata = std::env::var("APPDATA").ok()?;
    let subdir = match viewer.as_str() {
        "firestorm" => "Firestorm_x64",
        "official" => "SecondLife",
        _ => return None, // custom：由前端自行处理
    };
    Some(format!("{}\\{}", appdata, subdir))
}

#[tauri::command]
fn list_accounts(log_dir: String) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(&log_dir) else {
        return vec![];
    };
    let mut accounts: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        // 过滤常见的非账号类配置目录，剩下的用户子目录都列出来供选择
        .filter(|name| {
            let lower = name.to_lowercase();
            !lower.starts_with('.')
                && lower != "logs"
                && lower != "browser_profile"
                && lower != "dictionaries"
                && lower != "user_settings"
                && lower != "crashreports"
        })
        .collect();
    accounts.sort();
    accounts
}

#[tauri::command]
fn append_translation_history(
    account: String,
    source: String,
    timestamp: String,
    sender: String,
    text: String,
    translated: String,
) -> Result<(), String> {
    use std::fs::{create_dir_all, OpenOptions};
    use std::io::Write;

    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    // 每个账号有独立的子目录
    let account_dir = format!("{}\\ai_sl_talk\\{}", appdata, account);

    if let Err(e) = create_dir_all(&account_dir) {
        return Err(e.to_string());
    }

    let file_name = if source.ends_with(".txt") || source.ends_with(".log") {
        source
    } else {
        format!("{}.txt", source)
    };

    let file_path = format!("{}\\{}", account_dir, file_name);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| e.to_string())?;

    let log_line = format!(
        "[{}] {}:\n原文: {}\n翻译: {}\n\n",
        timestamp,
        sender,
        text.trim(),
        translated.trim()
    );
    if let Err(e) = file.write_all(log_line.as_bytes()) {
        return Err(e.to_string());
    }

    Ok(())
}

#[tauri::command]
fn open_history_folder(account: String) -> Result<(), String> {
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    // 打开该账号的专属子目录；若不存在则创建并打开父目录
    let account_dir = format!("{}\\ai_sl_talk\\{}", appdata, account);
    let target_dir = if !account.is_empty() {
        let _ = std::fs::create_dir_all(&account_dir);
        account_dir
    } else {
        let base = format!("{}\\ai_sl_talk", appdata);
        let _ = std::fs::create_dir_all(&base);
        base
    };

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer")
        .arg(&target_dir)
        .spawn();

    Ok(())
}

#[tauri::command]
fn save_ui_state(account: String, state: String) -> Result<(), String> {
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    let account_dir = format!("{}\\ai_sl_talk\\{}", appdata, account);
    let _ = std::fs::create_dir_all(&account_dir);
    std::fs::write(format!("{}\\chat-state.json", account_dir), state)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_ui_state(account: String) -> Result<String, String> {
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    let target_file = format!("{}\\ai_sl_talk\\{}\\chat-state.json", appdata, account);
    std::fs::read_to_string(target_file).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_custom_i18n() -> Result<String, String> {
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    let target_file = format!("{}\\ai_sl_talk\\i18n.json", appdata);
    std::fs::read_to_string(target_file).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_default_i18n(content: String) -> Result<(), String> {
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    let target_dir = format!("{}\\ai_sl_talk", appdata);
    let _ = std::fs::create_dir_all(&target_dir);
    let target_file = format!("{}\\i18n.json", target_dir);
    if !std::path::Path::new(&target_file).exists() {
        let _ = std::fs::write(target_file, content);
    }
    Ok(())
}

// ── 命令区域 ─────────────────────────────────────────────────────────

#[tauri::command]
fn toggle_topmost(app: AppHandle, pin: bool) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_always_on_top(pin);
        // 置顶状态下脱离任务栏，转换为 ToolWindow 性质，从而直接免疫 Windows Win+D 的切屏和最小化打击
        let _ = win.set_skip_taskbar(pin);
    }
}

/// 显示主窗口（托盘点击时调用）
#[tauri::command]
fn show_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

// ── 构建系统托盘 ─────────────────────────────────────────────────────

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("AISLtalk — SL 无感翻译")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

// ── 应用入口 ─────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app_handle, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = app_handle.emit(constants::EVENT_SHORTCUT_TRIGGER, ());
                    }
                })
                .build(),
        )
        .setup(|app| {
            build_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_listen_log,
            stop_listen_log,
            copy_to_clipboard,
            open_folder_dialog,
            get_default_log_dir,
            get_viewer_log_dir,
            list_accounts,
            show_main_window,
            toggle_topmost,
            append_translation_history,
            open_history_folder,
            save_ui_state,
            load_ui_state,
            load_custom_i18n,
            save_default_i18n
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
