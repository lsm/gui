use llama_core::chat::{self}; // Import the chat module
use either::Either;
use std::path::PathBuf;
use anyhow::Result;
use crate::model_manager::ModelManager;

use endpoints::chat::{
    ChatCompletionObject,
    ChatCompletionRequest,
    // Import other types as needed
};

// Command to handle chat completion requests with llama-core
#[tauri::command]
pub async fn chat_with_llm(mut request: ChatCompletionRequest) -> Result<ChatCompletionObject, String> {
    println!("Received chat request for model: {}", request.model.as_deref().unwrap_or("unknown"));

    // If no model is specified, use the default downloaded model
    if request.model.is_none() {
        let model_manager = ModelManager::new().map_err(|e| e.to_string())?;
        let model_path = model_manager.get_model_path();
        
        if !model_path.exists() {
            return Err("Default model not found. Please wait for the model to download.".to_string());
        }
        
        // Set the model path as the model identifier
        request.model = Some(model_path.to_str().unwrap().to_string());
    }

    // Ensure stream is set to false - we don't support streaming responses yet
    request.stream = Some(false);
    
    // Call llama-core chat function with the proper request type
    match chat::chat(&mut request).await {
        Ok(Either::Right(completion)) => {
            println!("Received non-streaming response from llama-core");
            // Return the completion object directly
            Ok(completion)
        }
        Ok(Either::Left(_stream)) => {
            // Handle streaming response - Not implemented yet
            println!("Received streaming response from llama-core (not implemented)");
            Err("Streaming responses are not yet supported.".to_string())
        }
        Err(e) => {
            eprintln!("Error calling llama_core::chat: {}", e);
            Err(format!("LLM chat request failed: {}", e))
        }
    }
} 