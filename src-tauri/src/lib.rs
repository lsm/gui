// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static CONVERSATIONS: Lazy<Mutex<Vec<Conversation>>> = Lazy::new(|| {
    Mutex::new(vec![
        Conversation { id: 1, name: "Chat about AI".to_string() },
        Conversation { id: 2, name: "Project planning".to_string() },
        Conversation { id: 3, name: "Travel ideas".to_string() },
        Conversation { id: 4, name: "Book recommendations".to_string() },
        Conversation { id: 5, name: "Coding help".to_string() },
    ])
});

#[derive(Serialize, Deserialize, Clone)]
pub struct Conversation {
    id: u32,
    name: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_conversations() -> Vec<Conversation> {
    // Get conversation list from global state
    CONVERSATIONS.lock().unwrap().clone()
}

#[tauri::command]
fn create_conversation(name: String) -> Conversation {
    let mut conversations = CONVERSATIONS.lock().unwrap();
    
    // Generate new ID (simply take the max ID + 1)
    let new_id = conversations
        .iter()
        .map(|c| c.id)
        .max()
        .unwrap_or(0) + 1;
    
    // Create new conversation
    let new_conversation = Conversation {
        id: new_id,
        name,
    };
    
    // Add to the list
    conversations.push(new_conversation.clone());
    
    // Return the newly created conversation
    new_conversation
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_conversations, create_conversation])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
