pub mod constants;
pub mod watcher;

use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
fn start_listen_log(path_override: Option<String>, app: AppHandle) -> Result<String, String> {
    // 默认路径或自定义路径
    let target_dir = std::path::PathBuf::from(
        path_override.unwrap_or_else(|| constants::GAME_LOG_DIR_WIN.to_string())
    );
    watcher::LogWatcherService::start_watching(app, target_dir).map(|_| "Started".to_string())
}

#[tauri::command]
fn copy_to_clipboard(text: String, app: AppHandle) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

/// 弹出系统文件夹选择对话框，返回用户选定的目录路径
#[tauri::command]
fn open_folder_dialog() -> Option<String> {
    use std::process::Command;
    // 使用 PowerShell 弹出选择对话框（Windows 通用方案）
    let output = Command::new("powershell")
        .args([
            "-Command",
            "[void][System.Reflection.Assembly]::LoadWithPartialName('System.windows.forms');\
            $dialog = New-Object System.Windows.Forms.FolderBrowserDialog;\
            $dialog.ShowNewFolderButton = $false;\
            if ($dialog.ShowDialog() -eq 'OK') { $dialog.SelectedPath }"
        ])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() { None } else { Some(path) }
}

/// 扫描日志目录下所有以 _resident 结尾的子文件夹（即SL账号文件夹）
#[tauri::command]
fn list_accounts(log_dir: String) -> Vec<String> {
    use std::fs;
    let Ok(entries) = fs::read_dir(&log_dir) else { return vec![]; };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.ends_with("_resident") || name.ends_with("_Resident"))
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(
            |app_handle, shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    // 当按下快捷键时，通知前端唤起输入框
                    let _ = app_handle.emit(constants::EVENT_SHORTCUT_TRIGGER, ());
                }
            }
        ).build())
        .setup(|app| {
            // 注册全局快捷键 Ctrl+Space 
            // 全局快捷键（Ctrl+Space）已在 plugin 的 with_handler 中注册
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_listen_log,
            copy_to_clipboard,
            open_folder_dialog,
            list_accounts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
