use spacetimedb::{table, reducer, Table, ReducerContext, Identity};
use uuid::Uuid;

// Define tables for conversations and messages
#[table(name = conversation, public)]
pub struct Conversation {
    #[primary_key]
    pub id: String,  // Store UUID as string
    pub name: String,
    pub creator: Identity,
    pub created_at: u64,
}

#[table(name = message, public)]
pub struct Message {
    #[primary_key]
    pub id: String,  // Store UUID as string
    pub conversation_id: String,  // Reference to conversation UUID
    pub sequence_number: u64,
    pub text: String,
    pub sender: Identity,
    pub timestamp: u64,
}

/// Takes a name and checks and returns default name if empty.
fn validate_convo_name(name: String) -> String {
    if name.is_empty() {
        return "No Name".to_string();
    } else {
        return name;
    }
}

// Define reducers (functions that modify the database)
#[reducer]
pub fn create_conversation(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_convo_name(name);
    
    // Generate a new UUID for the conversation
    let conversation_id = Uuid::new_v4().to_string();
    
    // Create and insert new conversation
    ctx.db.conversation().insert(Conversation {
        id: conversation_id,
        name,
        creator: ctx.sender,
        created_at: get_current_timestamp(),
    });
    
    Ok(())
}

// Non-reducer function to create a conversation and return the ID
pub fn create_conversation_with_id(ctx: &ReducerContext, name: String) -> Result<String, String> {
    let name = validate_convo_name(name);
    
    // Generate a new UUID for the conversation
    let conversation_id = Uuid::new_v4().to_string();
    
    // Create and insert new conversation
    ctx.db.conversation().insert(Conversation {
        id: conversation_id.clone(),
        name,
        creator: ctx.sender,
        created_at: get_current_timestamp(),
    });
    
    Ok(conversation_id)
}

#[reducer]
pub fn set_conversation_name(ctx: &ReducerContext, conversation_id: String, name: String) -> Result<(), String> {
    let name = validate_convo_name(name);
    
    // Find the conversation with the given ID using the primary key index
    if let Some(conversation) = ctx.db.conversation().id().find(conversation_id) {
        // Create updated conversation
        let updated = Conversation {
            id: conversation.id,
            name,
            creator: conversation.creator,
            created_at: conversation.created_at,
        };
        
        // Update the conversation
        ctx.db.conversation().id().update(updated);
        return Ok(());
    } else {
        return Err("Conversation not found".to_string());
    }
}

#[reducer]
pub fn add_message(
    ctx: &ReducerContext,
    conversation_id: String,
    text: String,
) -> Result<(), String> {
    // Find the conversation to validate it exists
    if ctx.db.conversation().id().find(conversation_id.clone()).is_none() {
        return Err("Conversation not found".to_string());
    }
    
    // Generate a new UUID for the message
    let message_id = Uuid::new_v4().to_string();
    
    // Get the next sequence number
    let sequence_number = ctx.db.message()
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .count() as u64 + 1;
    
    // Insert message
    ctx.db.message().insert(Message {
        id: message_id,
        conversation_id,
        sequence_number,
        text,
        sender: ctx.sender,
        timestamp: get_current_timestamp(),
    });
    
    Ok(())
}

// Non-reducer function to add a message and return the ID
pub fn add_message_with_id(
    ctx: &ReducerContext,
    conversation_id: String,
    text: String,
) -> Result<String, String> {
    // Find the conversation to validate it exists
    if ctx.db.conversation().id().find(conversation_id.clone()).is_none() {
        return Err("Conversation not found".to_string());
    }
    
    // Generate a new UUID for the message
    let message_id = Uuid::new_v4().to_string();
    
    // Get the next sequence number
    let sequence_number = ctx.db.message()
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .count() as u64 + 1;
    
    // Insert message
    ctx.db.message().insert(Message {
        id: message_id.clone(),
        conversation_id,
        sequence_number,
        text,
        sender: ctx.sender,
        timestamp: get_current_timestamp(),
    });
    
    Ok(message_id)
}

#[reducer]
pub fn delete_conversation(ctx: &ReducerContext, conversation_id: String) -> Result<(), String> {
    // Find the conversation with the given ID
    if let Some(_) = ctx.db.conversation().id().find(conversation_id.clone()) {
        // Delete all messages in the conversation
        for message in ctx.db.message().iter().filter(|m| m.conversation_id == conversation_id).collect::<Vec<_>>() {
            ctx.db.message().id().delete(message.id.clone());
        }
        
        // Delete the conversation
        ctx.db.conversation().id().delete(conversation_id);
        return Ok(());
    } else {
        return Err("Conversation not found".to_string());
    }
}

// Helper function to get current timestamp as u64
fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
} 