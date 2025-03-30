use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use surrealdb::sql::Thing;

// Define structs for chats and messages - usable directly by the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chat {
    #[serde(skip_serializing)]
    pub id: Option<Thing>,
    pub name: String,
    pub creator: String,
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[serde(skip_serializing)]
    pub id: Option<Thing>,
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

// API version of Message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiMessage {
    pub id: String,
    pub chat_id: String,
    pub sequence_number: u64,
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
}

impl From<Chat> for ApiChat {
    fn from(c: Chat) -> Self {
        Self {
            id: c.id.map_or_else(|| "unknown".to_string(), |t| t.id.to_string()),
            name: c.name,
            created_at: c.created_at,
        }
    }
}

impl From<Message> for ApiMessage {
    fn from(m: Message) -> Self {
        Self {
            id: m.id.map_or_else(|| "unknown".to_string(), |t| t.id.to_string()),
            chat_id: m.chat_id,
            sequence_number: m.sequence_number,
            text: m.text,
            sender: m.sender,
            timestamp: m.timestamp,
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
pub fn create_chat_data(name: String, creator: String) -> Chat {
    let name = validate_chat_name(name);
    
    // Create new chat without ID (SurrealDB will generate it)
    Chat {
        id: None,
        name,
        creator,
        created_at: get_current_timestamp(),
    }
}

// Create a new message record
pub fn create_message_data(
    chat_id: String,
    text: String,
    sender: String,
    sequence_number: u64,
) -> Message {
    // Create message without ID (SurrealDB will generate it)
    Message {
        id: None,
        chat_id,
        sequence_number,
        text,
        sender,
        timestamp: get_current_timestamp(),
    }
}

// Helper function to get current timestamp as u64
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
} 