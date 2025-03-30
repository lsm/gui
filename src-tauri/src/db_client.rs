use crate::db_schema::{Conversation, Message, create_conversation_data, create_message_data, get_current_timestamp};
use surrealdb::engine::local::{RocksDb, Db};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::OnceCell;
use uuid::Uuid;
use std::path::PathBuf;

// Define namespace and database name
const NAMESPACE: &str = "chat_app";
const DATABASE: &str = "chat_db";
const DB_PATH: &str = "chat_data";

// Static instance for the SurrealDB client
static DB: OnceCell<Surreal<Db>> = OnceCell::const_new();

// Initialize the database client
pub async fn init_database() -> Result<(), Box<dyn std::error::Error>> {
    if DB.initialized() {
        return Ok(());
    }

    // Create a path for the database
    let db_path = PathBuf::from(DB_PATH);
    
    // Create database instance with RocksDB as the storage engine
    let db = Surreal::new::<RocksDb>(db_path).await?;
    
    // Sign in as root
    db.signin(Root {
        username: "root",
        password: "root",
    }).await?;
    
    // Select a namespace and database
    db.use_ns(NAMESPACE).use_db(DATABASE).await?;
    
    // Create schema for the tables
    db.query(r#"
        DEFINE TABLE conversation SCHEMAFULL;
        DEFINE FIELD id ON conversation TYPE string;
        DEFINE FIELD name ON conversation TYPE string;
        DEFINE FIELD creator ON conversation TYPE string;
        DEFINE FIELD created_at ON conversation TYPE number;
        
        DEFINE TABLE message SCHEMAFULL;
        DEFINE FIELD id ON message TYPE string;
        DEFINE FIELD conversation_id ON message TYPE string;
        DEFINE FIELD sequence_number ON message TYPE number;
        DEFINE FIELD text ON message TYPE string;
        DEFINE FIELD sender ON message TYPE string;
        DEFINE FIELD timestamp ON message TYPE number;
        
        DEFINE INDEX conversation_id ON TABLE message FIELDS conversation_id;
    "#).await?;
    
    // Set the database instance
    DB.set(db).map_err(|_| "Failed to initialize database".to_string())?;
    
    println!("SurrealDB initialized successfully");
    Ok(())
}

// Get database instance
pub async fn get_db() -> Result<&'static Surreal<Db>, Box<dyn std::error::Error>> {
    match DB.get() {
        Some(db) => Ok(db),
        None => Err("Database not initialized".into()),
    }
}

// No subscription needed with SurrealDB - function provided for API compatibility
pub async fn subscribe_to_updates() -> Result<(), Box<dyn std::error::Error>> {
    // SurrealDB doesn't have a built-in subscription mechanism
    println!("Note: SurrealDB doesn't have built-in subscriptions. Consider implementing polling or WebSockets.");
    Ok(())
}

// Query all conversations
pub async fn query_conversations() -> Result<Vec<Conversation>, Box<dyn std::error::Error>> {
    let db = get_db().await?;
    
    // Query all conversations
    let conversations: Vec<Conversation> = db.select("conversation").await?;
    
    Ok(conversations)
}

// Query messages for a specific conversation
pub async fn query_messages(conversation_id: String) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let db = get_db().await?;
    
    // Clone the conversation_id for the query
    let id_for_query = conversation_id.clone();
    
    // Query messages for the given conversation
    let mut result = db.query("SELECT * FROM message WHERE conversation_id = $id ORDER BY sequence_number")
        .bind(("id", id_for_query))
        .await?;
    let messages: Vec<Message> = result.take(0)?;
    
    Ok(messages)
}

// Create a new conversation
pub async fn create_conversation(name: String) -> Result<String, Box<dyn std::error::Error>> {
    let db = get_db().await?;
    
    // Generate a pseudo-random user ID
    let user_id = format!("user-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("anonymous"));
    
    // Create a new conversation
    let (conversation_id, conversation) = create_conversation_data(name, user_id);
    
    // Insert into the database
    let _: Option<Conversation> = db.create(("conversation", conversation_id.clone()))
        .content(conversation)
        .await?;
    
    println!("Created conversation with ID: {}", conversation_id);
    
    Ok(conversation_id)
}

// Add a new message
pub async fn add_message(conversation_id: String, text: String) -> Result<String, Box<dyn std::error::Error>> {
    let db = get_db().await?;
    
    // Clone the conversation_id for checking and queries
    let id_for_check = conversation_id.clone();
    
    // Check if conversation exists
    let conversation: Option<Conversation> = db.select(("conversation", id_for_check)).await?;
    if conversation.is_none() {
        return Err("Conversation not found".into());
    }
    
    // Clone for querying messages
    let id_for_query = conversation_id.clone();
    
    // Get the next sequence number
    let mut result = db.query("SELECT * FROM message WHERE conversation_id = $id")
        .bind(("id", id_for_query))
        .await?;
    let messages: Vec<Message> = result.take(0)?;
    
    let sequence_number = messages.len() as u64 + 1;
    
    // Create a sender ID
    let sender = format!("user-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("anonymous"));
    
    // Create a new message
    let (message_id, message) = create_message_data(
        conversation_id,
        text,
        sender,
        sequence_number
    );
    
    // Insert into the database
    let _: Option<Message> = db.create(("message", message_id.clone()))
        .content(message)
        .await?;
    
    println!("Created message with ID: {}", message_id);
    
    Ok(message_id)
}

// Delete a conversation and its messages
pub async fn delete_conversation(conversation_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let db = get_db().await?;
    
    // Clone the conversation_id for the query
    let id_for_query = conversation_id.clone();
    
    // Delete all messages in the conversation
    db.query("DELETE message WHERE conversation_id = $id")
        .bind(("id", id_for_query))
        .await?;
    
    // Delete the conversation - with type annotation
    let _: Option<Conversation> = db.delete(("conversation", conversation_id)).await?;
    
    Ok(())
} 