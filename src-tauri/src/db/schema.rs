use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

// Define structs for chats and messages - usable directly by the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub sequence_number: u64,
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
}

// Simplified API types when needed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiChat {
    pub id: String,
    pub name: String, 
    pub created_at: u64,
}

impl From<Chat> for ApiChat {
    fn from(c: Chat) -> Self {
        Self {
            id: c.id,
            name: c.name,
            created_at: c.created_at,
        }
    }
}

/// Takes a name and checks and returns default name if empty.
pub fn validate_chat_name(name: String) -> String {
    if name.is_empty() {
        return "No Name".to_string();
    } else {
        return name;
    }
}

// Create a new chat record
pub fn create_chat_data(name: String, creator: String) -> (String, Chat) {
    let name = validate_chat_name(name);
    
    // Generate a new UUID for the chat
    let chat_id = Uuid::new_v4().to_string();
    
    // Create new chat
    let chat = Chat {
        id: chat_id.clone(),
        name,
        creator,
        created_at: get_current_timestamp(),
    };
    
    (chat_id, chat)
}

// Create a new message record
pub fn create_message_data(
    chat_id: String,
    text: String,
    sender: String,
    sequence_number: u64,
) -> (String, Message) {
    // Generate a new UUID for the message
    let message_id = Uuid::new_v4().to_string();
    
    // Create message
    let message = Message {
        id: message_id.clone(),
        chat_id,
        sequence_number,
        text,
        sender,
        timestamp: get_current_timestamp(),
    };
    
    (message_id, message)
}

// Helper function to get current timestamp as u64
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
} 