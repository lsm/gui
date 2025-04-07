use crate::db::schema::{create_chat_data, create_message_data, ApiChat, ApiMessage, Author, AuthorType, Chat, Message};
use surrealdb::engine::local::{RocksDb, Db};
use surrealdb::Surreal;
use surrealdb::{Datetime, RecordId};
use chrono::{DateTime, Utc};
use tokio::sync::{OnceCell, broadcast};
use uuid::Uuid;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::error::Error;
use futures::StreamExt;
use std::str::FromStr;

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
        DEFINE TABLE author SCHEMAFULL;
        DEFINE FIELD kind ON author TYPE string;
        DEFINE FIELD name ON author TYPE string;
        
        DEFINE TABLE chat SCHEMAFULL;
        DEFINE FIELD name ON chat TYPE string;
        DEFINE FIELD author ON chat TYPE record<author>;
        DEFINE FIELD created_at ON chat TYPE datetime;
        
        DEFINE TABLE message SCHEMAFULL;
        DEFINE FIELD chat ON message TYPE record<chat>;
        DEFINE FIELD sequence_number ON message TYPE number;
        DEFINE FIELD text ON message TYPE string;
        DEFINE FIELD author ON message TYPE record<author>;
        DEFINE FIELD created_at ON message TYPE datetime;
        
        DEFINE INDEX chat_idx ON TABLE message FIELDS chat;
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
                                        id: chat.id.map_or_else(
                                            || "unknown".to_string(),
                                            |record_id| record_id.to_string(),
                                        ),
                                        name: chat.name.clone(),
                                        author: None, // We don't have author data here
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

// Query all chats with their authors
pub async fn query_chats() -> Result<Vec<ApiChat>, Box<dyn Error + Send + Sync>> {
    let db = get_db().await?;
    
    // Query all chats with their authors
    let mut result = db.query("SELECT *, author.* FROM chat")
        .await?;
    
    // This will get the chats with joined author data
    let chats: Vec<serde_json::Value> = result.take(0)?;
    
    // Convert to ApiChat format
    let api_chats: Vec<ApiChat> = chats.into_iter()
        .filter_map(|chat_value| {
            let chat_id = chat_value.get("id")?.get("id")?.as_str()?;
            
            // Extract author data
            let author_data = chat_value.get("author")?;
            let author_kind_str = author_data.get("kind")?.as_str()?;
            let author_kind = match author_kind_str {
                "User" => AuthorType::User,
                "Assistant" => AuthorType::Assistant,
                "Tool" => AuthorType::Tool,
                "System" => AuthorType::System,
                _ => return None,
            };
            
            let author = Author {
                id: None, // We don't need the ID for API responses
                kind: author_kind,
                name: author_data.get("name")?.as_str()?.to_string(),
            };
            
            // Parse the datetime string from SurrealDB
            let created_at_str = chat_value.get("created_at")?.as_str()?;
            let datetime = match DateTime::parse_from_rfc3339(created_at_str) {
                Ok(dt) => Datetime::from(dt.with_timezone(&Utc)),
                Err(_) => return None,
            };
            
            Some(ApiChat {
                id: chat_id.to_string(),
                name: chat_value.get("name")?.as_str()?.to_string(),
                author: Some(author),
                created_at: datetime,
            })
        })
        .collect();
    
    Ok(api_chats)
}

// Query messages for a specific chat
pub async fn query_messages(chat_id: String) -> Result<Vec<ApiMessage>, Box<dyn Error + Send + Sync>> {
    let db = get_db().await?;
    
    // Convert string to RecordId if needed
    let record_id = if chat_id.contains(':') {
        RecordId::from_str(&chat_id).ok()
    } else {
        RecordId::from_str(&format!("chat:{}", chat_id)).ok()
    };
    
    // Return empty array if record_id is invalid
    let record_id = match record_id {
        Some(id) => id,
        None => return Ok(vec![]),
    };
    
    // Query messages for the given chat with their authors
    let mut result = db.query("SELECT *, author.* FROM message WHERE chat = $id ORDER BY sequence_number")
        .bind(("id", record_id))
        .await?;
    
    // This will get messages with joined author data
    let messages: Vec<serde_json::Value> = result.take(0)?;
    
    // Convert to ApiMessage format
    let api_messages: Vec<ApiMessage> = messages.into_iter()
        .filter_map(|msg_value| {
            let msg_id = msg_value.get("id")?.get("id")?.as_str()?;
            
            // Extract author data
            let author_data = msg_value.get("author")?;
            let author_kind_str = author_data.get("kind")?.as_str()?;
            let author_kind = match author_kind_str {
                "User" => AuthorType::User,
                "Assistant" => AuthorType::Assistant,
                "Tool" => AuthorType::Tool,
                "System" => AuthorType::System,
                _ => return None,
            };
            
            let author = Author {
                id: None, // We don't need the ID for API responses
                kind: author_kind,
                name: author_data.get("name")?.as_str()?.to_string(),
            };
            
            // Parse the datetime string from SurrealDB
            let created_at_str = msg_value.get("created_at")?.as_str()?;
            let datetime = match DateTime::parse_from_rfc3339(created_at_str) {
                Ok(dt) => Datetime::from(dt.with_timezone(&Utc)),
                Err(_) => return None,
            };
            
            Some(ApiMessage {
                id: msg_id.to_string(),
                chat_id: msg_value.get("chat")?.get("id")?.as_str()?.to_string(),
                sequence_number: msg_value.get("sequence_number")?.as_u64()?,
                text: msg_value.get("text")?.as_str()?.to_string(),
                author: Some(author),
                created_at: datetime,
            })
        })
        .collect();
    
    Ok(api_messages)
}

// Create a new chat
pub async fn create_chat(name: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Get DB connection, this will auto-reinitialize if needed 
    let db = get_db().await?;
    
    // Create a user author
    let author = Author {
        id: None,
        kind: AuthorType::User,
        name: format!("User-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("anonymous")),
    };
    
    // Insert the author first
    println!("Creating author");
    let created_author: Option<Author> = db.create("author")
        .content(author)
        .await?;
    
    let author_id = match created_author {
        Some(author) => author.id,
        None => return Err("Failed to create author record".into()),
    };
    
    // Create a new chat with reference to the author
    let chat = create_chat_data(name, author_id);
    
    // Insert into the database with error tracing
    println!("Attempting to create chat in database");
    let created: Option<Chat> = db.create("chat")
        .content(chat)
        .await?;
        
    // Extract the ID from the created record
    let chat_id = match created {
        Some(record) => record.id.map_or_else(
            || "unknown".to_string(), 
            |record_id| record_id.to_string()
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
    
    // Convert string to RecordId
    let chat_record_id = if chat_id.contains(':') {
        RecordId::from_str(&chat_id).map_err(|e| format!("Invalid chat ID format: {}", e))?
    } else {
        RecordId::from_str(&format!("chat:{}", chat_id)).map_err(|e| format!("Invalid chat ID format: {}", e))?
    };
    
    // Check if chat exists
    println!("Checking if chat exists: {}", chat_id);
    let chat_result: Result<Option<Chat>, surrealdb::Error> = db.select(chat_record_id.clone()).await;
    
    let _chat = match chat_result {
        Ok(maybe_chat) => match maybe_chat {
            Some(chat) => {
                println!("Found chat: {:?}", chat.name);
                chat
            },
            None => {
                eprintln!("Chat not found with ID {}", chat_id);
                return Err(format!("Chat not found with ID {}", chat_id).into());
            }
        },
        Err(e) => {
            eprintln!("Error checking chat: {}", e);
            return Err(format!("Failed to verify chat: {}", e).into());
        }
    };
    
    // Get the next sequence number
    println!("Querying messages for chat: {}", chat_id);
    let query_result = db.query("SELECT * FROM message WHERE chat = $id")
        .bind(("id", chat_record_id.clone()))
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
    
    // Create an author for the message
    let author = Author {
        id: None,
        kind: AuthorType::User,
        name: sender_name,
    };
    
    // Insert the author first
    println!("Creating author");
    let created_author: Option<Author> = db.create("author")
        .content(author)
        .await?;
    
    let author_id = match created_author {
        Some(author) => author.id,
        None => return Err("Failed to create author record".into()),
    };
    
    // Create a new message with reference to author
    let message = create_message_data(
        Some(chat_record_id),
        text,
        author_id,
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
            |record_id| record_id.to_string()
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

// Delete a chat and its messages
pub async fn delete_chat(chat_id: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = get_db().await?;
    
    // Delete associated messages first using a query
    println!("Deleting messages for chat: {}", chat_id);
    let mut delete_messages_result = db
        .query("DELETE message WHERE chat_id = $id")
        .bind(("id", chat_id.clone())) // Clone chat_id here as it's used again below
        .await?;
    
    // Check the result of the delete query (optional, but good practice)
    let _deleted_messages: Vec<Message> = delete_messages_result.take(0)?;
    println!("Messages deleted for chat: {}", chat_id);

    // Delete the chat itself
    println!("Deleting chat: {}", chat_id);
    // Handle potential "table:id" format
    let parts: Vec<&str> = chat_id.split(':').collect();
    let (table, id) = if parts.len() > 1 {
        (parts[0], parts[1])
    } else {
        ("chat", parts[0]) // Assume "chat" table if not specified
    };

    let deleted_chat: Option<Chat> = db
        .delete((table, id))
        .await?;

    if deleted_chat.is_some() {
        println!("Successfully deleted chat: {}", chat_id);
        Ok(())
    } else {
        println!("Chat not found or already deleted: {}", chat_id);
        // Consider if this should be an error or not. For idempotency, Ok might be fine.
        Ok(())
        // Err(format!("Chat with ID {} not found", chat_id).into())
    }
}

