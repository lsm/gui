import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { Chat, Message } from "../types";

/**
 * Load all chats from the database
 */
export async function loadChats(): Promise<Chat[]> {
  try {
    const chats = await invoke<Chat[]>("get_chats");
    return chats;
  } catch (error) {
    console.error("Error loading chats:", error);
    // In case of a closed channel error, try to restart the database connection
    if (error && typeof error === 'string' && error.includes('closed channel')) {
      try {
        // Try to re-subscribe to database updates which might help with reconnection
        await invoke("subscribe_to_db_updates");
        // Try again after re-subscribing
        const chats = await invoke<Chat[]>("get_chats");
        return chats;
      } catch (retryError) {
        console.error("Failed to reconnect to database:", retryError);
      }
    }
    return [];
  }
}

/**
 * Load messages for a specific chat
 */
export async function loadMessages(chatId: string): Promise<Message[]> {
  try {
    const chatMessages = await invoke<Message[]>("get_messages", { chat_id: chatId });
    return chatMessages;
  } catch (error) {
    console.error(`Error loading messages for chat ${chatId}:`, error);
    return [];
  }
}

/**
 * Create a new chat
 */
export async function createChat(name: string): Promise<Chat> {
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
    
    // Now attempt to create the chat
    const newChat = await invoke<Chat>("create_chat", { 
      name: name || "New Chat" 
    });
    return newChat;
  } catch (error) {
    console.error("Error creating chat:", error);
    
    // If it's a closed channel error, we should try to reinitialize
    if (error && typeof error === 'string' && error.includes('closed channel')) {
      throw new Error(`Database connection error: ${error}. Please restart the application.`);
    }
    
    throw error; // Re-throw to allow the UI to handle it
  }
}

/**
 * Send a message in a chat and get AI response
 */
export async function sendMessage(chatId: string, text: string): Promise<[Message, Message]> {
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
      chat_id: chatId,
      text: text,
      sender_name: "user"
    });
    
    // For demo purposes, we'll use a simple response
    // In a real app, you'd call your AI service here
    const aiResponse = await invoke<Message>("add_message", {
      chat_id: chatId,
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