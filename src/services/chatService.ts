import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { Chat, Message, Author, AuthorType } from "../types";

/**
 * Load all chats from the database
 */
export async function loadChats(): Promise<Chat[]> {
  try {
    const chats = await invoke<Chat[]>("get_chats");
    return chats;
  } catch (error) {
    console.error("Error loading chats:", error);
    return [];
  }
}

/**
 * Load messages for a specific chat
 */
export async function loadMessages(chatId: string): Promise<Message[]> {
  try {
    const chatMessages = await invoke<Message[]>("get_messages", { chatId });
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
  console.log("sendMessage called with chatId:", chatId);
  
  try {
    // Create a user author for the message
    const userAuthor: Author = {
      kind: AuthorType.User,
      name: "You"
    };
    
    // Add user message to chat
    console.log("Sending user message with chatId:", chatId);
    const userMessage = await invoke<Message>("add_message", { 
      chatId,
      text,
      senderName: userAuthor.name // This will be used to create the Author in the backend
    });
    console.log("User message added:", userMessage);
    
    // Create an AI author for the response
    const aiAuthor: Author = {
      kind: AuthorType.Assistant,
      name: "Assistant"
    };
    
    // Add AI response
    console.log("Sending AI response with chatId:", chatId);
    const aiResponse = await invoke<Message>("add_message", { 
      chatId,
      text: `You said: "${userMessage.text}". This is a simulated AI response.`,
      senderName: aiAuthor.name // This will be used to create the Author in the backend
    });
    console.log("AI response added:", aiResponse);
    
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
 * Update a chat's title
 */
export async function updateChatTitle(chatId: string, title: string): Promise<void> {
  try {
    console.log("Updating chat title for:", chatId, "new title:", title);
    await invoke("update_chat", { 
      chatId,
      name: title
    });
    
    // Update the window title as well
    await updateWindowTitle(title);
    
  } catch (error) {
    console.error("Error updating chat title:", error);
    
    // If it's a closed channel error, notify the user
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

/**
 * Delete a chat
 */
export async function deleteChat(chatId: string): Promise<void> {
  try {
    await invoke("delete_chat", { chatId });
  } catch (error) {
    console.error(`Error deleting chat ${chatId}:`, error);

    // Handle potential closed channel errors
    if (error && typeof error === 'string' && error.includes('closed channel')) {
      throw new Error(`Database connection error: ${error}. Please restart the application.`);
    }

    throw error; // Re-throw other errors
  }
} 