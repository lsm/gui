use crate::db::schema::{Chat, Message, create_chat_data, create_message_data, ApiChat};
use surrealdb::engine::local::{RocksDb, Db};
use surrealdb::Surreal;
use tokio::sync::{OnceCell, broadcast};
use uuid::Uuid;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::error::Error;
use futures::StreamExt;

// Define namespace and database name
const NAMESPACE: &str = "chat_app";
const DATABASE: &str = "chat_db";
const DB_PATH: &str = "chat_data";

// Static instance for the SurrealDB client
static DB: OnceCell<Surreal<Db>> = OnceCell::const_new();

// Channels for real-time updates
static CHAT_UPDATES: Lazy<Mutex<Option<broadcast::Sender<ApiChat>>>> = Lazy::new(|| Mutex::new(None));

// Initialize the database client
pub async fn init_database() -> Result<(), Box<dyn Error + Send + Sync>> {
    // If the DB is already initialized, just verify the connection
    if DB.initialized() {
        // Test that the connection is still valid by running a simple query
        let db = DB.get().ok_or("DB initialized but not available")?;
        // Simple health check
        let _: Option<serde_json::Value> = db.query("SELECT 1 FROM tb").await?.take(0)?;
        println!("Database connection verified");
        return Ok(());
    }

    println!("Initializing new database connection");
    
    // Create a path for the database
    let db_path = PathBuf::from(DB_PATH);
    
    // Create database instance with RocksDB as the storage engine
    let db = Surreal::new::<RocksDb>(db_path).await?;
    
    // Select a namespace and database
    db.use_ns(NAMESPACE).use_db(DATABASE).await?;
    
    // Create schema for the tables
    db.query(r#"
        DEFINE TABLE chat SCHEMAFULL;
        DEFINE FIELD id ON chat TYPE string;
        DEFINE FIELD name ON chat TYPE string;
        DEFINE FIELD creator ON chat TYPE string;
        DEFINE FIELD created_at ON chat TYPE number;
        
        DEFINE TABLE message SCHEMAFULL;
        DEFINE FIELD id ON message TYPE string;
        DEFINE FIELD chat_id ON message TYPE string;
        DEFINE FIELD sequence_number ON message TYPE number;
        DEFINE FIELD text ON message TYPE string;
        DEFINE FIELD sender ON message TYPE string;
        DEFINE FIELD timestamp ON message TYPE number;
        
        DEFINE INDEX chat_id ON TABLE message FIELDS chat_id;
    "#).await?;
    
    // Set the database instance
    match DB.set(db) {
        Ok(_) => {
            println!("SurrealDB initialized successfully");
            
            // Initialize broadcast channel for chat updates
            let (sender, _) = broadcast::channel::<ApiChat>(100);
            let mut chat_updates = CHAT_UPDATES.lock().unwrap();
            *chat_updates = Some(sender);
            
            // Initialize chat live query
            initialize_chat_live_query();
            
            // Initialize message live query if needed
            // initialize_message_live_query();
            
            Ok(())
        },
        Err(_) => {
            Err("Failed to set database instance. Another initialization might be in progress.".into())
        }
    }
}

// Initialize chat live query
fn initialize_chat_live_query() {
    tokio::spawn(async {
        // Skip error handling in the spawn to avoid Send issues
        if let Some(db) = DB.get() {
            // Create a live query on the chat table with explicit type annotation
            match db.select::<Vec<Chat>>("chat").live().await {
                Ok(mut stream) => {
                    println!("Started live query for chat table");
                    
                    // Process notifications using the Stream trait
                    while let Some(notification) = stream.next().await {
                        match notification {
                            Ok(notification) => {
                                // Process the notification based on its action
                                if let Ok(sender) = get_chat_update_sender() {
                                    // Convert the chat notification to ApiChat with proper type annotation
                                    let chat: Chat = notification.data;
                                    let api_chat = ApiChat {
                                        id: chat.id.as_ref().map_or_else(
                                            || "unknown".to_string(),
                                            |thing| thing.id.to_string(),
                                        ),
                                        name: chat.name.clone(),
                                        created_at: chat.created_at,
                                    };
                                    
                                    // Print first then send to avoid borrow after move
                                    println!("Chat update broadcasted via live query: {}", api_chat.name);
                                    
                                    // Send the update through the broadcast channel
                                    let _ = sender.send(api_chat);
                                }
                            },
                            Err(e) => {
                                eprintln!("Error in live query notification: {}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to set up live query for chat table: {}", e);
                }
            }
        }
    });
}

// Initialize message live query if needed
#[allow(dead_code)]
fn initialize_message_live_query() {
    tokio::spawn(async {
        // Skip error handling in the spawn to avoid Send issues
        if let Some(db) = DB.get() {
            // Create a live query on the message table with explicit type annotation
            match db.select::<Vec<Message>>("message").live().await {
                Ok(mut stream) => {
                    println!("Started live query for message table");
                    
                    // Process notifications using the Stream trait
                    while let Some(notification) = stream.next().await {
                        match notification {
                            Ok(_notification) => {
                                // Process message notifications if needed
                                // This would require a similar broadcast channel for messages
                                println!("Message update received");
                            },
                            Err(e) => {
                                eprintln!("Error in message live query notification: {}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to set up live query for message table: {}", e);
                }
            }
        }
    });
}

// Get a receiver for chat updates
pub fn get_chat_update_receiver() -> Result<broadcast::Receiver<ApiChat>, Box<dyn Error + Send + Sync>> {
    let sender = get_chat_update_sender()?;
    Ok(sender.subscribe())
}

// Helper to get the chat update sender
fn get_chat_update_sender() -> Result<broadcast::Sender<ApiChat>, Box<dyn Error + Send + Sync>> {
    let chat_updates = CHAT_UPDATES.lock().unwrap();
    match &*chat_updates {
        Some(sender) => Ok(sender.clone()),
        None => Err("Chat updates channel not initialized".into()),
    }
}

// Get database instance with auto-reconnect if needed
pub async fn get_db() -> Result<&'static Surreal<Db>, Box<dyn Error + Send + Sync>> {
    if !DB.initialized() {
        println!("Database not initialized, initializing now");
        init_database().await?;
    }
    
    match DB.get() {
        Some(db) => {
            // Try a simple query to verify the connection is still valid
            match db.query("SELECT 1 FROM tb").await {
                Ok(_) => Ok(db),
                Err(e) => {
                    // Connection issue detected, try to reinitialize
                    println!("Database connection issue detected: {}", e);
                    println!("Attempting to verify/reinitialize database connection");
                    init_database().await?;
                    
                    // If we reach here, we should have a valid DB connection
                    DB.get().ok_or_else(|| "Database still not available after reinitialization".into())
                }
            }
        },
        None => Err("Database not initialized".into()),
    }
}

// Query all chats
pub async fn query_chats() -> Result<Vec<Chat>, Box<dyn Error + Send + Sync>> {
    let db = get_db().await?;
    
    // Query all chats
    let chats: Vec<Chat> = db.select("chat").await?;
    
    Ok(chats)
}

// Query messages for a specific chat
pub async fn query_messages(chat_id: String) -> Result<Vec<Message>, Box<dyn Error + Send + Sync>> {
    let db = get_db().await?;
    
    // Query messages for the given chat
    let mut result = db.query("SELECT * FROM message WHERE chat_id = $id ORDER BY sequence_number")
        .bind(("id", chat_id))
        .await?;
    let messages: Vec<Message> = result.take(0)?;
    
    Ok(messages)
}

// Create a new chat
pub async fn create_chat(name: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Get DB connection, this will auto-reinitialize if needed 
    let db = get_db().await?;
    
    // Generate a pseudo-random user ID
    let user_id = format!("user-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("anonymous"));
    
    // Create a new chat
    let chat = create_chat_data(name, user_id);
    
    // Insert into the database with error tracing
    println!("Attempting to create chat in database");
    let created: Option<Chat> = db.create("chat")
        .content(chat)
        .await?;
        
    // Extract the ID from the created record
    let chat_id = match created {
        Some(record) => record.id.map_or_else(
            || "unknown".to_string(), 
            |thing| thing.id.to_string()
        ),
        None => return Err("Failed to create chat: No record returned".into()),
    };
        
    println!("Created chat with ID: {}", chat_id);
    Ok(chat_id)
}

// Add a new message
pub async fn add_message(chat_id: String, text: String, sender_name: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Get DB connection with auto-reconnect if needed
    let db = get_db().await?;
    
    // Check if chat exists - Extracting table and ID parts
    println!("Checking if chat exists: {}", chat_id);
    
    // The chat_id might be in form "chat:uuid" or just "uuid"
    let parts: Vec<&str> = chat_id.split(':').collect();
    let (table, id) = if parts.len() > 1 {
        (parts[0], parts[1])
    } else {
        ("chat", parts[0])
    };

    // debug_query_chats(db).await?;
    // debug_query_messages(db, chat_id.clone()).await?;
        
    // Query the chat with proper ID format
    println!("Looking for chat with table='{}', id='{}'", table, id);
    let chat_result: Result<Option<Chat>, surrealdb::Error> = db.select((table, id)).await;
    let _chat = match chat_result {
        Ok(maybe_chat) => match maybe_chat {
            Some(chat) => {
                println!("Found chat: {:?}", chat.name);
                chat
            },
            None => {
                eprintln!("Chat not found with ID {}", id);
                return Err(format!("Chat not found with ID {}", id).into());
            }
        },
        Err(e) => {
            eprintln!("Error checking chat: {}", e);
            return Err(format!("Failed to verify chat: {}", e).into());
        }
    };
    
    // Get the next sequence number
    println!("Querying messages for chat: {}", chat_id);
    let query_result = db.query("SELECT * FROM message WHERE chat_id = $id")
        .bind(("id", chat_id.clone()))
        .await;
    
    let messages: Vec<Message> = match query_result {
        Ok(mut result) => match result.take(0) {
            Ok(msgs) => msgs,
            Err(e) => {
                eprintln!("Error extracting messages: {}", e);
                return Err(format!("Failed to extract messages: {}", e).into());
            }
        },
        Err(e) => {
            eprintln!("Error querying messages: {}", e);
            return Err(format!("Failed to query messages: {}", e).into());
        }
    };
    
    let sequence_number = messages.len() as u64 + 1;
    
    // Create a new message
    let message = create_message_data(
        chat_id,
        text,
        sender_name,
        sequence_number
    );
    
    // Insert into the database
    println!("Creating message");
    let created: Option<Message> = db.create("message")
        .content(message)
        .await?;
    
    // Extract the ID from the created record
    let message_id = match created {
        Some(record) => record.id.map_or_else(
            || "unknown".to_string(), 
            |thing| thing.id.to_string()
        ),
        None => return Err("Failed to create message: No record returned".into()),
    };
    
    println!("Created message with ID: {}", message_id);
    Ok(message_id)
}

// Update a chat's properties
pub async fn update_chat(chat_id: String, name: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Get DB connection with auto-reconnect if needed
    let db = get_db().await?;
    
    // The chat_id might be in form "chat:uuid" or just "uuid"
    let parts: Vec<&str> = chat_id.split(':').collect();
    let (table, id) = if parts.len() > 1 {
        (parts[0], parts[1])
    } else {
        ("chat", parts[0])
    };
    
    // Update the chat with the new name
    println!("Updating chat {} with new name: {}", chat_id, name);
    let update_result: Result<Option<Chat>, surrealdb::Error> = db.update((table, id))
        .merge(serde_json::json!({
            "name": name
        }))
        .await;
    
    match update_result {
        Ok(_) => {
            println!("Successfully updated chat name");
            Ok(())
        },
        Err(e) => {
            eprintln!("Error updating chat: {}", e);
            Err(format!("Failed to update chat: {}", e).into())
        }
    }
}

