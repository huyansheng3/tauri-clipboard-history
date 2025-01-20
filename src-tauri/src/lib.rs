// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Local;
use image::{ImageBuffer, Rgba, ImageEncoder};
use parking_lot::Mutex;
use serde::Serialize;
use std::{sync::Arc, thread, time::Duration};
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ClipboardContent {
    Text { content: String },
    Image { 
        data: String, 
        width: usize,
        height: usize,
    },
    RichText { 
        content: String,
        emoji_images: Vec<EmojiImage>,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct ClipboardEntry {
    content: ClipboardContent,
    timestamp: String,
}

// 新增：表情图片结构
#[derive(Debug, Serialize, Clone)]
pub struct EmojiImage {
    data: String,     // base64 图片数据
    position: usize,  // 在文本中的位置
}

// 使用Arc和Mutex来存储剪贴板历史
struct ClipboardState {
    history: Vec<ClipboardEntry>,
    last_text_content: String,
    last_image_hash: u64,
    search_query: String,
}

impl ClipboardState {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            last_text_content: String::new(),
            last_image_hash: 0,
            search_query: String::new(),
        }
    }
}

lazy_static! {
    static ref CLIPBOARD_STATE: Arc<Mutex<ClipboardState>> = Arc::new(Mutex::new(ClipboardState::new()));
    // 微信表情图片的特征匹配
    static ref WECHAT_EMOJI_PATTERN: Regex = Regex::new(r"\[微信表情\]").unwrap();
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

// 添加新的搜索命令
#[tauri::command]
fn search_clipboard_history(query: String) -> Vec<ClipboardEntry> {
    let mut state = CLIPBOARD_STATE.lock();
    state.search_query = query.clone();
    
    if query.is_empty() {
        return state.history.clone();
    }
    
    state.history
        .iter()
        .filter(|entry| match &entry.content {
            ClipboardContent::Text { content } => {
                content.to_lowercase().contains(&query.to_lowercase())
            }
            ClipboardContent::Image { .. } => false, // 图片内容不参与搜索
            ClipboardContent::RichText { .. } => false, // 富文本不参与搜索
        })
        .cloned()
        .collect()
}

// 修改文本处理逻辑
fn process_clipboard_text(clipboard: &mut Clipboard) -> Option<ClipboardContent> {
    if let Ok(content) = clipboard.get_text() {
        if content.is_empty() {
            return None;
        }

        // 检查是否包含微信表情
        if WECHAT_EMOJI_PATTERN.is_match(&content) {
            // 尝试获取关联的图片
            if let Ok(image) = clipboard.get_image() {
                if let Some(base64) = image_to_base64(&image) {
                    // 先获取位置，因为这会借用 content
                    let position = content.find("[微信表情]").unwrap_or(0);
                    return Some(ClipboardContent::RichText {
                        content: content.clone(), // 克隆 content
                        emoji_images: vec![EmojiImage {
                            data: base64,
                            position,
                        }],
                    });
                }
            }
        }

        // 普通文本
        Some(ClipboardContent::Text { content })
    } else {
        None
    }
}

// 修改监控函数
fn start_clipboard_monitor() {
    thread::spawn(|| {
        let mut clipboard = Clipboard::new().unwrap();
        
        loop {
            let mut state = CLIPBOARD_STATE.lock();
            
            // 处理剪贴板内容
            if let Some(content) = process_clipboard_text(&mut clipboard) {
                match &content {
                    ClipboardContent::Text { content: text } => {
                        if text != &state.last_text_content {
                            state.last_text_content = text.clone();
                            state.history.push(ClipboardEntry {
                                content,
                                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            });
                        }
                    }
                    ClipboardContent::RichText { content: text, .. } => {
                        if text != &state.last_text_content {
                            state.last_text_content = text.clone();
                            state.history.push(ClipboardEntry {
                                content,
                                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            
            // 处理纯图片
            if let Ok(image) = clipboard.get_image() {
                let image_hash = calculate_image_hash(&image);
                if image_hash != state.last_image_hash {
                    state.last_image_hash = image_hash;
                    
                    if let Some(base64) = image_to_base64(&image) {
                        let entry = ClipboardEntry {
                            content: ClipboardContent::Image {
                                data: base64,
                                width: image.width,
                                height: image.height,
                            },
                            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        };
                        
                        state.history.push(entry);
                    }
                }
            }
            
            drop(state);
            thread::sleep(Duration::from_millis(500));
        }
    });
}

// 修改 image_to_base64 函数，使用新的 write_image 方法
fn image_to_base64(image: &ImageData) -> Option<String> {
    let bytes = &image.bytes;
    let width = image.width;
    let height = image.height;
    
    if let Some(img_buffer) = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, bytes.to_vec()) {
        let mut buffer = Vec::new();
        
        // 使用新的 write_image 方法
        if let Ok(_) = image::codecs::png::PngEncoder::new(&mut buffer)
            .write_image(
                &img_buffer,
                width as u32,
                height as u32,
                image::ColorType::Rgba8
            )
        {
            return Some(format!("data:image/png;base64,{}", BASE64.encode(&buffer)));
        }
    }
    None
}

// 辅助函数：计算图片哈希值用于去重
fn calculate_image_hash(image: &ImageData) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    image.bytes.hash(&mut hasher);
    hasher.finish()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    start_clipboard_monitor();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_clipboard_history,
            search_clipboard_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
