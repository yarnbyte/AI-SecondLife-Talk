pub mod constants;
pub mod watcher;

use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{ShortcutState};

#[tauri::command]
fn start_listen_log(path_override: Option<String>, app: AppHandle) -> Result<String, String> {
    let target_dir = std::path::PathBuf::from(
        path_override.unwrap_or_else(|| constants::GAME_LOG_DIR_WIN.to_string())
    );
    watcher::LogWatcherService::start_watching(app, target_dir).map(|_| "Started".to_string())
}

#[tauri::command]
fn copy_to_clipboard(text: String, app: AppHandle) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

/// 弹出系统文件夹选择对话框（使用 tauri-plugin-dialog 原生 API）
#[tauri::command]
async fn open_folder_dialog(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::FilePath;
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel::<Option<FilePath>>();

    app.dialog()
        .file()
        .set_title("选择 Firestorm 日志目录")
        .pick_folder(move |result| {
            let _ = tx.send(result);
        });

    match rx.await {
        Ok(Some(path)) => path.into_path().ok().map(|p| p.to_string_lossy().to_string()),
        _ => None,
    }
}

/// 获取 Firestorm 日志的默认真实路径（解析 %APPDATA%，不是字面字符串）
#[tauri::command]
fn get_default_log_dir() -> Option<String> {
    std::env::var("APPDATA").ok().map(|appdata| {
        format!("{}\\Firestorm_x64", appdata)
    })
}

/// 扫描日志目录下所有以 _resident 结尾的子文件夹（即 SL 账号文件夹）
#[tauri::command]
fn list_accounts(log_dir: String) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(&log_dir) else { return vec![]; };
    let mut accounts: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| {
            let lower = name.to_lowercase();
            lower.ends_with("_resident")
        })
        .collect();
    accounts.sort();
    accounts
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(
            |app_handle, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    let _ = app_handle.emit(constants::EVENT_SHORTCUT_TRIGGER, ());
                }
            }
        ).build())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            start_listen_log,
            copy_to_clipboard,
            open_folder_dialog,
            list_accounts,
            get_default_log_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
