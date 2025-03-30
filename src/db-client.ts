import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function subscribeToUpdates(
  setMessages: (messages: any[]) => void,
  setChats: (chats: any[]) => void,
  selectedChatId: string | null
) {
  // Set up subscription to database updates
  await invoke("subscribe_to_db_updates");
  
  // Listen for database events - different versions of SpaceTimeDB use different event naming
  // We'll set up multiple listeners to handle different possibilities
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
            invoke("get_messages", { chat_id: selectedChatId })
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
  
  const setupChatListeners = async () => {
    const listeners: (() => void)[] = [];
    
    // Try different event name formats
    const chatEventNames = [
      "spacetimedb:tableupdate:chat", 
      "spacetimedb:tableupdate:Chat",
      "table:Chat", 
      "table:chat"
    ];
    
    for (const eventName of chatEventNames) {
      try {
        const unsubscribe = await listen(eventName, (event) => {
          invoke("get_chats")
            .then((chats) => setChats(chats as any[]));
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
  const cleanupChatListeners = await setupChatListeners();
  
  // Return cleanup function
  return () => {
    cleanupMessageListeners();
    cleanupChatListeners();
  };
} 