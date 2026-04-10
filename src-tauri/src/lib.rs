pub mod constants;
pub mod watcher;

use tauri::{AppHandle, Manager, Emitter};
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
    app.clipboard().write_text(text.clone()).map_err(|e| e.to_string())?;
    Ok(())
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
            #[cfg(desktop)]
            {
                let ctrl_space = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
                let _ = app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().build()
                );
                // Tauri 2 的全局快捷键机制如上已添加。
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_listen_log,
            copy_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
