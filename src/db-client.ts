import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function subscribeToUpdates(
  setMessages: (messages: any[]) => void,
  setConversations: (conversations: any[]) => void,
  selectedConversationId: string | null
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
          if (selectedConversationId) {
            invoke("get_messages", { conversation_id: selectedConversationId })
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
  
  const setupConversationListeners = async () => {
    const listeners: (() => void)[] = [];
    
    // Try different event name formats
    const conversationEventNames = [
      "spacetimedb:tableupdate:conversation", 
      "spacetimedb:tableupdate:Conversation",
      "table:Conversation", 
      "table:conversation"
    ];
    
    for (const eventName of conversationEventNames) {
      try {
        const unsubscribe = await listen(eventName, (event) => {
          invoke("get_conversations")
            .then((conversations) => setConversations(conversations as any[]));
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
  const cleanupConversationListeners = await setupConversationListeners();
  
  // Return cleanup function
  return () => {
    cleanupMessageListeners();
    cleanupConversationListeners();
  };
} 