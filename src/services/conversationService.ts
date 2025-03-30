import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { Conversation, Message } from "../types";

/**
 * Load all conversations from the database
 */
export async function loadConversations(): Promise<Conversation[]> {
  try {
    const conversations = await invoke<Conversation[]>("get_conversations");
    return conversations;
  } catch (error) {
    console.error("Error loading conversations:", error);
    // In case of a closed channel error, try to restart the database connection
    if (error && typeof error === 'string' && error.includes('closed channel')) {
      try {
        // Try to re-subscribe to database updates which might help with reconnection
        await invoke("subscribe_to_db_updates");
        // Try again after re-subscribing
        const conversations = await invoke<Conversation[]>("get_conversations");
        return conversations;
      } catch (retryError) {
        console.error("Failed to reconnect to database:", retryError);
      }
    }
    return [];
  }
}

/**
 * Load messages for a specific conversation
 */
export async function loadMessages(conversationId: string): Promise<Message[]> {
  try {
    const conversationMessages = await invoke<Message[]>("get_messages", { conversation_id: conversationId });
    return conversationMessages;
  } catch (error) {
    console.error(`Error loading messages for conversation ${conversationId}:`, error);
    return [];
  }
}

/**
 * Create a new conversation
 */
export async function createConversation(name: string): Promise<Conversation> {
  try {
    // First try to re-establish the database connection
    try {
      const result = await invoke("test", { msg: "Hello, world!" });
      console.log("Test Result:", result);
      // await invoke("subscribe_to_db_updates");
    } catch (subError) {
      console.warn("Warning during resubscribe attempt:", subError);
      // Continue even if this fails - it might not be necessary
    }
    
    // Now attempt to create the conversation
    // const newConversation = await invoke<Conversation>("create_conversation", { 
    //   name: name || "New Chat" 
    // });
    // return newConversation;
  } catch (error) {
    console.error("Error creating conversation:", error);
    
    // If it's a closed channel error, we should try to reinitialize
    if (error && typeof error === 'string' && error.includes('closed channel')) {
      throw new Error(`Database connection error: ${error}. Please restart the application.`);
    }
    
    throw error; // Re-throw to allow the UI to handle it
  }
}

/**
 * Send a message in a conversation and get AI response
 */
export async function sendMessage(conversationId: string, text: string): Promise<[Message, Message]> {
  try {
    // Try to re-establish the database connection
    try {
      await invoke("subscribe_to_db_updates");
    } catch (subError) {
      console.warn("Warning during resubscribe attempt before sending message:", subError);
      // Continue even if this fails
    }
    
    // Add user message to chat
    const userMessage = await invoke<Message>("add_message", {
      conversation_id: conversationId,
      text: text,
      sender_name: "user"
    });
    
    // For demo purposes, we'll use a simple response
    // In a real app, you'd call your AI service here
    const aiResponse = await invoke<Message>("add_message", {
      conversation_id: conversationId,
      text: `You said: "${userMessage.text}". This is a simulated AI response.`,
      sender_name: "ai"
    });
    
    return [userMessage, aiResponse];
  } catch (error) {
    console.error("Error sending message:", error);
    
    // If it's a closed channel error, we should notify the user
    if (error && typeof error === 'string' && error.includes('closed channel')) {
      throw new Error(`Database connection error: ${error}. Please restart the application.`);
    }
    
    throw error;
  }
}

/**
 * Update the native window title
 */
export async function updateWindowTitle(title: string): Promise<void> {
  try {
    const currentWindow = Window.getCurrent();
    await currentWindow.setTitle(`${title} - Chat App`);
  } catch (error) {
    console.error("Error updating window title:", error);
  }
} 