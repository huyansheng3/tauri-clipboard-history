// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use arboard::Clipboard;
use chrono::Local;
use parking_lot::Mutex;
use serde::Serialize;
use std::{sync::Arc, thread, time::Duration};

#[derive(Debug, Serialize, Clone)]
pub struct ClipboardEntry {
    content: String,
    timestamp: String,
}

// 使用Arc和Mutex来存储剪贴板历史
struct ClipboardState {
    history: Vec<ClipboardEntry>,
    last_content: String,
}

impl ClipboardState {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            last_content: String::new(),
        }
    }
}

lazy_static::lazy_static! {
    static ref CLIPBOARD_STATE: Arc<Mutex<ClipboardState>> = Arc::new(Mutex::new(ClipboardState::new()));
}

#[tauri::command]
fn get_clipboard_history() -> Vec<ClipboardEntry> {
    CLIPBOARD_STATE.lock().history.clone()
}

// 原有的greet函数保持不变
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn start_clipboard_monitor() {
    thread::spawn(|| {
        let mut clipboard = Clipboard::new().unwrap();
        
        loop {
            if let Ok(content) = clipboard.get_text() {
                let mut state = CLIPBOARD_STATE.lock();
                
                // 只有当内容变化时才添加新记录
                if !content.is_empty() && content != state.last_content {
                    state.last_content = content.clone();
                    
                    let entry = ClipboardEntry {
                        content,
                        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    
                    state.history.push(entry);
                }
            }
            
            thread::sleep(Duration::from_millis(500));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    start_clipboard_monitor();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_clipboard_history])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
