use std::sync::{Arc, Mutex, Once};
use crate::db_schema::{Conversation, Message};
use uuid::Uuid;
use spacetimedb::Identity;

// Mock connection type for simulation
pub struct MockConnection;

// Singleton pattern for database client
static INIT: Once = Once::new();
static CLIENT: Mutex<Option<Arc<MockConnection>>> = Mutex::new(None);

// Constants
const HOST: &str = "http://localhost:3000";

// Initialize the database client
pub async fn init_database() -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        // Connect anonymously (SpaceTimeDB will generate an identity)
        println!("Connecting to SpaceTimeDB...");
        init_connection();
    });
    
    Ok(())
}

// Initialize connection
fn init_connection() {
    // In a real implementation, we would connect to SpaceTimeDB
    // For this demo, we'll just create a mock connection
    println!("Simulating connection to SpaceTimeDB at {}", HOST);
    
    // Create mock connection
    let mock_connection = MockConnection;
    let mut client_guard = CLIENT.lock().unwrap();
    *client_guard = Some(Arc::new(mock_connection));
}

// Get client instance - mock for this demo
pub fn get_client() -> Arc<MockConnection> {
    // In a real implementation, this would return the actual client
    let client_guard = CLIENT.lock().unwrap();
    client_guard.as_ref().unwrap_or_else(|| panic!("Database client not initialized")).clone()
}

// Subscribe to database updates - mock for this demo
pub async fn subscribe_to_updates() -> Result<(), Box<dyn std::error::Error>> {
    // In a real implementation, this would subscribe to SpaceTimeDB tables
    println!("Simulating subscription to conversation and message tables");
    
    Ok(())
}

// Query all conversations - mock implementation
pub async fn query_conversations() -> Result<Vec<Conversation>, Box<dyn std::error::Error>> {
    // In a real implementation, this would query the database
    // For this demo, return mock data
    let sender_identity = Identity::__dummy();
    
    Ok(vec![
        Conversation {
            id: "1".to_string(),
            name: "General Chat".to_string(),
            creator: sender_identity,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    ])
}

// Query messages for a specific conversation - mock implementation
pub async fn query_messages(conversation_id: String) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    // In a real implementation, this would query the database
    // For this demo, return mock data
    let sender_identity = Identity::__dummy();
    
    Ok(vec![
        Message {
            id: "1".to_string(),
            conversation_id,
            sequence_number: 1,
            text: "Welcome to the chat!".to_string(),
            sender: sender_identity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    ])
}

// Create a new conversation - mock implementation
pub async fn create_conversation(name: String) -> Result<String, Box<dyn std::error::Error>> {
    // In a real implementation, this would call a reducer on SpaceTimeDB
    println!("Creating conversation: {}", name);
    
    // Generate a UUID for the mock conversation
    let conversation_id = Uuid::new_v4().to_string();
    println!("Created conversation with ID: {}", conversation_id);
    
    Ok(conversation_id)
}

// Add a new message - mock implementation
pub async fn add_message(conversation_id: String, text: String) -> Result<String, Box<dyn std::error::Error>> {
    // In a real implementation, this would call a reducer on SpaceTimeDB
    println!("Adding message to conversation {}: {}", conversation_id, text);
    
    // Generate a UUID for the mock message
    let message_id = Uuid::new_v4().to_string();
    println!("Created message with ID: {}", message_id);
    
    Ok(message_id)
} 