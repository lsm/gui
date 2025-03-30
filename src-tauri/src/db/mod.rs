// Database module for Focus app

// Import the submodules
mod client;
mod schema;

// Re-export types needed by the API
pub use schema::{Conversation, Message, ApiConversation, get_current_timestamp};
pub use client::*; 