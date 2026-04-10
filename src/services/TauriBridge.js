import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TAURI_EVENTS } from '../utils/constants';

export class TauriBridge {
    /**
     * 启动日志监听器
     * @param {string|null} customPath - 给定自定义路径，或者null以使用后端默认
     */
    static async startLogWatcher(customPath = null) {
        try {
            const config = { path_override: customPath };
            return await invoke('start_listen_log', config);
        } catch (error) {
            console.error("Failed to start log watcher:", error);
            throw error;
        }
    }

    /**
     * 写入剪贴板
     * @param {string} text 
     */
    static async copyToClipboard(text) {
        try {
            await invoke('copy_to_clipboard', { text });
        } catch (error) {
            console.error("Failed to write to clipboard:", error);
            throw error;
        }
    }

    /**
     * 监听来自后端的日志更新
     * @param {Function} callback (payload) => void 
     */
    static async onLogUpdate(callback) {
        return await listen(TAURI_EVENTS.LOG_UPDATE, (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听全局快捷键触发事件
     */
    static async onShortcutTrigger(callback) {
        return await listen(TAURI_EVENTS.SHORTCUT_TRIGGER, () => {
            callback();
        });
    }
}
