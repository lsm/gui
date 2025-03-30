// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use spacetimedb::Identity;

// Import your schema and client modules
mod db_schema;
mod db_client;

// Public API types that match the internal schema types but with simpler types for the frontend
#[derive(Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: String,
    pub name: String,
    pub created_at: u64,
}

impl From<db_schema::Conversation> for Conversation {
    fn from(c: db_schema::Conversation) -> Self {
        Self {
            id: c.id,
            name: c.name,
            created_at: c.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
    pub sequence_number: u64,
}

impl From<db_schema::Message> for Message {
    fn from(m: db_schema::Message) -> Self {
        Self {
            id: m.id,
            conversation_id: m.conversation_id,
            text: m.text,
            sender: format!("{}", m.sender),
            timestamp: m.timestamp,
            sequence_number: m.sequence_number,
        }
    }
}

// Helper function to convert Identity to String
fn identity_to_string(identity: Identity) -> String {
    format!("{}", identity)
}

#[tauri::command]
async fn subscribe_to_db_updates() -> Result<(), String> {
    db_client::subscribe_to_updates()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_conversations() -> Result<Vec<Conversation>, String> {
    db_client::query_conversations()
        .await
        .map(|conversations| conversations.into_iter().map(Conversation::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_conversation(name: String) -> Result<Conversation, String> {
    // Call the SpaceTimeDB function to create a conversation
    let conversation_id = db_client::create_conversation(name.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // Since we may not immediately get the update from the subscription,
    // create a temporary object to return
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Ok(Conversation {
        id: conversation_id,
        name,
        created_at: current_time,
    })
}

#[tauri::command]
async fn get_messages(conversation_id: String) -> Result<Vec<Message>, String> {
    db_client::query_messages(conversation_id)
        .await
        .map(|messages| messages.into_iter().map(Message::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_message(conversation_id: String, text: String, sender_name: String) -> Result<Message, String> {
    // Call the SpaceTimeDB function to add a message
    let message_id = db_client::add_message(conversation_id.clone(), text.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // Since we may not immediately get the update from the subscription,
    // create a temporary object to return
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Ok(Message {
        id: message_id,
        conversation_id,
        text,
        sender: sender_name,
        timestamp: current_time,
        sequence_number: 0, // This will be updated when we get the subscription update
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            // Initialize database
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    db_client::init_database().await.expect("Failed to initialize database");
                });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_conversations, 
            create_conversation,
            get_messages,
            add_message,
            subscribe_to_db_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
