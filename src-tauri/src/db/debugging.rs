use crate::db::schema::{Chat, Message};
use surrealdb::engine::local::{Db};
use surrealdb::Surreal;

pub async fn debug_query_chats(db: &Surreal<Db>) -> Result<(), String> {
    // DEBUG: Query all chats for debugging
    println!("DEBUG: Querying all available chats");
    let all_chats_result = db.query("SELECT * FROM chat").await;
    match all_chats_result {
        Ok(mut result) => {
            let all_chats: Vec<Chat> = match result.take(0) {
                Ok(chats) => chats,
                Err(e) => {
                    println!("Error extracting all chats: {}", e);
                    vec![] // Empty vector on error
                }
            };
            println!("DEBUG: Found {} chats:", all_chats.len());
            for (i, chat) in all_chats.iter().enumerate() {
                println!("  {}: ID: {:?}, Name: {}", i, chat.id, chat.name);
            }
            Ok(())
        },
        Err(e) => {
            println!("Error querying all chats: {}", e);
            Err(e.to_string())
        }
    }
}

pub async fn debug_query_messages(db: &Surreal<Db>, chat_id: String) -> Result<(), String> {
    // DEBUG: Query all messages for debugging
    println!("DEBUG: Querying all messages for chat: {}", chat_id);
    let messages_result = db.query("SELECT * FROM message WHERE chat_id = $id")
        .bind(("id", chat_id.clone()))
        .await;
    match messages_result {
        Ok(mut result) => {
            let messages: Vec<Message> = match result.take(0) {
                Ok(msgs) => msgs,
                Err(e) => {
                    eprintln!("Error extracting messages: {}", e);
                    vec![]
                }
            };
            println!("DEBUG: Found {} messages:", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                println!("  {}: ID: {:?}, Text: {}", i, msg.id, msg.text);
            }
            Ok(())
        },
        Err(e) => {
            eprintln!("Error querying messages: {}", e);
            Err(e.to_string())
        }
    }
}