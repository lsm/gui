import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function subscribeToUpdates(
  setChats: (chats: any[]) => void,
) {
  // Set up subscription to chat updates via the new mechanism
  await invoke("subscribe_to_chat_updates");
  
  // Listen for chat update events
  const chatUpdateUnsubscribe = await listen("chat-update", (event) => {
    console.log("Received chat update:", event);
    
    // Refresh the chat list when we get an update
    invoke("get_chats")
      .then((chats) => setChats(chats as any[]));
  });
  
  return () => {
    chatUpdateUnsubscribe();
  };
} 