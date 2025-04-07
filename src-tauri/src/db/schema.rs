use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

// Define the types of authors that can create messages or chats
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AuthorType {
    User,
    Assistant,
    Tool,
    System,
}

// Author as a separate entity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    #[serde(skip_serializing)]
    pub id: Option<RecordId>,
    pub kind: AuthorType,
    pub name: String,
}

// Define structs for chats and messages - usable directly by the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chat {
    #[serde(skip_serializing)]
    pub id: Option<RecordId>,
    pub name: String,
    #[serde(rename = "author")]
    pub author_id: Option<RecordId>,
    pub created_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[serde(skip_serializing)]
    pub id: Option<RecordId>,
    #[serde(rename = "chat")]
    pub chat_id: Option<RecordId>,
    pub sequence_number: u64,
    pub text: String,
    #[serde(rename = "author")]
    pub author_id: Option<RecordId>,
    pub created_at: Datetime,
}

// API types with expanded author information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiChat {
    pub id: String,
    pub name: String,
    pub author: Option<Author>,
    pub created_at: Datetime,
}

// API version of Message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiMessage {
    pub id: String,
    pub chat_id: String,
    pub sequence_number: u64,
    pub text: String,
    pub author: Option<Author>,
    pub created_at: Datetime,
}

impl From<Chat> for ApiChat {
    fn from(c: Chat) -> Self {
        Self {
            id: c.id.map_or_else(|| "unknown".to_string(), |t| t.to_string()),
            name: c.name,
            author: None, // Will need to be populated separately with an author query
            created_at: c.created_at,
        }
    }
}

impl From<Message> for ApiMessage {
    fn from(m: Message) -> Self {
        Self {
            id: m.id.map_or_else(|| "unknown".to_string(), |t| t.to_string()),
            chat_id: m.chat_id.map_or_else(|| "unknown".to_string(), |t| t.to_string()),
            sequence_number: m.sequence_number,
            text: m.text,
            author: None, // Will need to be populated separately with an author query
            created_at: m.created_at,
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
pub fn create_chat_data(name: String, author_id: Option<RecordId>) -> Chat {
    let name = validate_chat_name(name);
    
    // Create new chat without ID (SurrealDB will generate it)
    Chat {
        id: None,
        name,
        author_id,
        created_at: get_current_timestamp(),
    }
}

// Create a new message record
pub fn create_message_data(
    chat_id: Option<RecordId>,
    text: String,
    author_id: Option<RecordId>,
    sequence_number: u64,
) -> Message {
    // Create message without ID (SurrealDB will generate it)
    Message {
        id: None,
        chat_id,
        sequence_number,
        text,
        author_id,
        created_at: get_current_timestamp(),
    }
}

// Helper function to get current timestamp as Datetime
pub fn get_current_timestamp() -> Datetime {
    Datetime::from(chrono::Utc::now())
}