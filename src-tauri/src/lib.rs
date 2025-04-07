// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import db module
mod db;
mod llm; // Add the new llm module

// Use the chat_with_llm command from the llm module
use llm::chat_with_llm;
use endpoints::chat; // Import chat module from endpoints

// Re-export types we need for the API
pub use db::{Chat, Message, ApiChat, ApiMessage};
pub use endpoints::chat::{ChatCompletionRequest, ChatCompletionObject}; // Re-export LLM types
use tauri::{AppHandle, Emitter};

#[tauri::command]
async fn test(msg: String) -> Result<String, String> {
    // print message to console
    println!("Hello, world!");
    // return message wrappped with "Message: "
    Ok(format!("Message: {}", msg))
}

#[tauri::command]
async fn get_chats() -> Result<Vec<ApiChat>, String> {
    db::query_chats()
        .await
        .map(|chats| chats.into_iter().map(ApiChat::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_chat(name: String) -> Result<ApiChat, String> {
    // Call the database function to create a chat
    let chat_id = db::create_chat(name.clone())
        .await
        .map_err(|e| format!("Failed to create chat: {}", e.to_string()))?;
    
    // Since we may not immediately get the update from the subscription,
    // create a temporary object to return
    let current_time = db::get_current_timestamp();
    
    Ok(ApiChat {
        id: chat_id,
        name,
        created_at: current_time,
    })
}

#[tauri::command]
async fn get_messages(chat_id: String) -> Result<Vec<ApiMessage>, String> {
    db::query_messages(chat_id)
        .await
        .map(|messages| messages.into_iter().map(ApiMessage::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_message(chat_id: String, text: String, sender_name: String) -> Result<ApiMessage, String> {
    // Log the parameters received from the client
    println!("add_message called with parameters: chat_id='{}', text='{}', sender_name='{}'", 
             chat_id, text, sender_name);
    
    // Call the database function to add a message
    let message_id = db::add_message(chat_id.clone(), text.clone(), sender_name.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // Since we may not immediately get the update from the subscription,
    // create a temporary object to return
    let current_time = db::get_current_timestamp();
    
    // If sender is user, generate an AI response and add it to the database
    if sender_name == "user" {
        // Generate an AI response using the LLM
        println!("Generating AI response for message: {}", text);
        
        // Create a ChatCompletionRequest with system and user messages
        let request = endpoints::chat::ChatCompletionRequest {
            model: Some("llama2-7b-chat".to_string()),  // Use a default model
            messages: vec![
                endpoints::chat::ChatCompletionRequestMessage::new_system_message(
                    "You are a helpful assistant.",
                    None
                ),
                endpoints::chat::ChatCompletionRequestMessage::new_user_message(
                    endpoints::chat::ChatCompletionUserMessageContent::Text(text.clone()),
                    None
                ),
            ],
            temperature: Some(0.7),
            max_completion_tokens: Some(128),
            stream: Some(false),
            ..Default::default()
        };
        
        // Call the LLM with our request
        match llm::chat_with_llm(request).await {
            Ok(response) => {
                // Extract the assistant's response
                if let Some(choice) = response.choices.first() {
                    if let Some(content) = &choice.message.content {
                        // Add the AI response to the database
                        println!("Saving AI response to database: {}", content);
                        let _ = db::add_message(chat_id.clone(), content.clone(), "ai".to_string())
                            .await
                            .map_err(|e| {
                                eprintln!("Error saving AI response: {}", e);
                                // We don't return this error as the original user message was saved successfully
                            });
                    }
                }
            },
            Err(e) => {
                // Log the error but don't fail the original message addition
                eprintln!("Error generating AI response: {}", e);
            }
        }
    }
    
    Ok(ApiMessage {
        id: message_id,
        chat_id,
        text,
        sender: sender_name,
        timestamp: current_time,
        sequence_number: 0, // This will be updated when we get the subscription update
    })
}

#[tauri::command]
async fn update_chat(chat_id: String, name: String) -> Result<(), String> {
    // Call the database function to update the chat
    db::update_chat(chat_id, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_chat(chat_id: String) -> Result<(), String> {
    db::delete_chat(chat_id)
        .await
        .map_err(|e| e.to_string())
}

// New command to subscribe to chat updates
#[tauri::command]
async fn subscribe_to_chat_updates(window: AppHandle) -> Result<(), String> {
    // Try to get a receiver for chat updates
    match db::get_chat_update_receiver() {
        Ok(mut receiver) => {
            // Start a background task to forward updates to the frontend
            tokio::spawn(async move {
                while let Ok(chat) = receiver.recv().await {
                    // Emit the chat update to all windows
                    let _ = window.emit("chat-update", chat);
                }
            });
            Ok(())
        },
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            // Create a separate blocking task to initialize the database
            // We'll use a simple blocking approach here to avoid runtime issues
            println!("Starting database initialization...");
            
            // Tauri already sets up a runtime, so we should use it
            tauri::async_runtime::spawn(async {
                match db::get_db().await {
                    Ok(_) => println!("Database initialized successfully from tauri::async_runtime"),
                    Err(e) => eprintln!("Failed to initialize database: {}", e),
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_chats, 
            create_chat,
            get_messages,
            add_message,
            update_chat,
            subscribe_to_chat_updates,
            test,
            delete_chat,
            chat_with_llm
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
