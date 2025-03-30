// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Conversation struct to represent a chat conversation
#[derive(Serialize, Deserialize, Clone)]
pub struct Conversation {
    id: u32,
    name: String,
}

// Message struct to represent a chat message
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    id: u32,
    text: String,
    sender: String, // "user" or "ai"
}

// In-memory storage for conversations
static CONVERSATIONS: Lazy<Mutex<Vec<Conversation>>> = Lazy::new(|| {
    Mutex::new(vec![
        Conversation { id: 1, name: "Chat about AI".to_string() },
        Conversation { id: 2, name: "Project planning".to_string() },
        Conversation { id: 3, name: "Travel ideas".to_string() },
        Conversation { id: 4, name: "Book recommendations".to_string() },
        Conversation { id: 5, name: "Coding help".to_string() },
    ])
});

// In-memory storage for messages
static MESSAGES: Lazy<Mutex<HashMap<u32, Vec<Message>>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Sample messages for conversation 1
    map.insert(1, vec![
        Message { id: 1, text: "What can you tell me about AI?".to_string(), sender: "user".to_string() },
        Message { id: 2, text: "Artificial Intelligence (AI) refers to systems designed to perform tasks that typically require human intelligence. These include learning, reasoning, problem-solving, perception, and language understanding.".to_string(), sender: "ai".to_string() },
    ]);
    
    // Sample messages for conversation 2
    map.insert(2, vec![
        Message { id: 1, text: "Let's plan our project timeline.".to_string(), sender: "user".to_string() },
        Message { id: 2, text: "Great! We should start by defining our objectives and milestones. What's the project scope?".to_string(), sender: "ai".to_string() },
    ]);
    
    // Sample messages for other conversations
    map.insert(3, vec![
        Message { id: 1, text: "I'm planning a trip to Japan. Any recommendations?".to_string(), sender: "user".to_string() },
        Message { id: 2, text: "Japan is a great destination! I'd recommend visiting Tokyo, Kyoto, and Osaka. Each city offers unique experiences from modern technology to traditional culture.".to_string(), sender: "ai".to_string() },
    ]);
    
    map.insert(4, vec![
        Message { id: 1, text: "Can you suggest some science fiction books?".to_string(), sender: "user".to_string() },
        Message { id: 2, text: "Some classic sci-fi books include 'Dune' by Frank Herbert, '1984' by George Orwell, and 'The Hitchhiker's Guide to the Galaxy' by Douglas Adams. For more recent works, consider 'The Three-Body Problem' by Liu Cixin.".to_string(), sender: "ai".to_string() },
    ]);
    
    map.insert(5, vec![
        Message { id: 1, text: "I'm having trouble with async functions in JavaScript.".to_string(), sender: "user".to_string() },
        Message { id: 2, text: "Async functions in JavaScript can be tricky! Remember that an async function always returns a Promise. You can use 'await' inside an async function to pause execution until the Promise resolves.".to_string(), sender: "ai".to_string() },
    ]);
    
    Mutex::new(map)
});

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
    
    // Initialize empty message list for this conversation
    MESSAGES.lock().unwrap().insert(new_id, Vec::new());
    
    // Return the newly created conversation
    new_conversation
}

#[tauri::command]
fn get_messages(conversation_id: u32) -> Vec<Message> {
    // Get messages for the specified conversation
    MESSAGES.lock()
        .unwrap()
        .get(&conversation_id)
        .cloned()
        .unwrap_or_default()
}

#[tauri::command]
fn add_message(conversation_id: u32, text: String, sender: String) -> Message {
    let mut messages = MESSAGES.lock().unwrap();
    
    // Get or create the message list for this conversation
    let conversation_messages = messages
        .entry(conversation_id)
        .or_insert_with(Vec::new);
    
    // Generate new message ID
    let new_id = conversation_messages
        .iter()
        .map(|m| m.id)
        .max()
        .unwrap_or(0) + 1;
    
    // Create the new message
    let new_message = Message {
        id: new_id,
        text,
        sender,
    };
    
    // Add to the list
    conversation_messages.push(new_message.clone());
    
    // Return the created message
    new_message
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_conversations, 
            create_conversation,
            get_messages,
            add_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
