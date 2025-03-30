// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import db module
mod db;

// Re-export types we need for the API
pub use db::{Conversation, Message, ApiConversation};

#[tauri::command]
async fn subscribe_to_db_updates() -> Result<(), String> {
    db::subscribe_to_updates()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_conversations() -> Result<Vec<ApiConversation>, String> {
    db::query_conversations()
        .await
        .map(|conversations| conversations.into_iter().map(ApiConversation::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_conversation(name: String) -> Result<ApiConversation, String> {
    // Call the database function to create a conversation
    let conversation_id = db::create_conversation(name.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // Since we may not immediately get the update from the subscription,
    // create a temporary object to return
    let current_time = db::get_current_timestamp();
    
    Ok(ApiConversation {
        id: conversation_id,
        name,
        created_at: current_time,
    })
}

#[tauri::command]
async fn get_messages(conversation_id: String) -> Result<Vec<Message>, String> {
    db::query_messages(conversation_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_message(conversation_id: String, text: String, sender_name: String) -> Result<Message, String> {
    // Call the database function to add a message
    let message_id = db::add_message(conversation_id.clone(), text.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // Since we may not immediately get the update from the subscription,
    // create a temporary object to return
    let current_time = db::get_current_timestamp();
    
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
                    db::init_database().await.expect("Failed to initialize database");
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
