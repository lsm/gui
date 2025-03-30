// Database module for Focus app

// Import the submodules
mod client;
pub mod schema;

// Re-export types needed by the API
pub use schema::{Chat, Message, ApiChat, ApiMessage, get_current_timestamp};
pub use client::*; 