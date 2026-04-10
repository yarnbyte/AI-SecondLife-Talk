use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tauri::AppHandle;

pub struct LogWatcherService;

impl LogWatcherService {
    /// 开启异步文件轮询监听（骨架，后续接入真实增量读取逻辑）
    pub fn start_watching(_app_handle: AppHandle, _target_dir: PathBuf) -> Result<(), String> {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(2));
            // TODO: 接入真实的 Firestorm .txt 增量读取
        });
        Ok(())
    }
}
