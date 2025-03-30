use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

// Define structs for conversations and messages - usable directly by the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sequence_number: u64,
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
}

// Simplified API types when needed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConversation {
    pub id: String,
    pub name: String, 
    pub created_at: u64,
}

impl From<Conversation> for ApiConversation {
    fn from(c: Conversation) -> Self {
        Self {
            id: c.id,
            name: c.name,
            created_at: c.created_at,
        }
    }
}

/// Takes a name and checks and returns default name if empty.
pub fn validate_convo_name(name: String) -> String {
    if name.is_empty() {
        return "No Name".to_string();
    } else {
        return name;
    }
}

// Create a new conversation record
pub fn create_conversation_data(name: String, creator: String) -> (String, Conversation) {
    let name = validate_convo_name(name);
    
    // Generate a new UUID for the conversation
    let conversation_id = Uuid::new_v4().to_string();
    
    // Create new conversation
    let conversation = Conversation {
        id: conversation_id.clone(),
        name,
        creator,
        created_at: get_current_timestamp(),
    };
    
    (conversation_id, conversation)
}

// Create a new message record
pub fn create_message_data(
    conversation_id: String,
    text: String,
    sender: String,
    sequence_number: u64,
) -> (String, Message) {
    // Generate a new UUID for the message
    let message_id = Uuid::new_v4().to_string();
    
    // Create message
    let message = Message {
        id: message_id.clone(),
        conversation_id,
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