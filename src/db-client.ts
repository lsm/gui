import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function subscribeToUpdates(
  setMessages: (messages: any[]) => void,
  setChats: (chats: any[]) => void,
  selectedChatId: string | null
) {
  // Set up subscription to database updates
  await invoke("subscribe_to_db_updates");
  
  // Set up subscription to chat updates via the new mechanism
  await invoke("subscribe_to_chat_updates");
  
  // Listen for chat update events
  const chatUpdateUnsubscribe = await listen("chat-update", (event) => {
    console.log("Received chat update:", event);
    
    // Refresh the chat list when we get an update
    invoke("get_chats")
      .then((chats) => setChats(chats as any[]));
  });
  
  // Listen for database events for messages
  const setupMessageListeners = async () => {
    const listeners: (() => void)[] = [];
    
    // Try different event name formats
    const messageEventNames = [
      "spacetimedb:tableupdate:message", 
      "spacetimedb:tableupdate:Message",
      "table:Message", 
      "table:message"
    ];
    
    for (const eventName of messageEventNames) {
      try {
        const unsubscribe = await listen(eventName, (event) => {
          if (selectedChatId) {
            invoke("get_messages", { chatId: selectedChatId })
              .then((messages) => setMessages(messages as any[]));
          }
        });
        listeners.push(unsubscribe);
        console.log(`Successfully subscribed to ${eventName}`);
      } catch (e) {
        console.log(`Failed to subscribe to ${eventName}: ${e}`);
      }
    }
    
    return () => listeners.forEach(unsub => unsub());
  };
  
  const cleanupMessageListeners = await setupMessageListeners();
  
  // Return cleanup function
  return () => {
    chatUpdateUnsubscribe();
    cleanupMessageListeners();
  };
} 