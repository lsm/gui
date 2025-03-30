import { createSignal, For, onMount, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Define types for our application
type Message = {
  id: number;
  text: string;
  sender: "user" | "ai";
};

type Conversation = {
  id: number;
  name: string;
};

function App() {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [inputValue, setInputValue] = createSignal("");
  const [selectedItem, setSelectedItem] = createSignal<number | null>(null);
  const [items, setItems] = createSignal<Conversation[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [loadingMessages, setLoadingMessages] = createSignal(false);
  const [newConversationName, setNewConversationName] = createSignal("");
  const [showNewConversation, setShowNewConversation] = createSignal(false);
  
  // Load conversation list from Rust API
  async function loadConversations() {
    try {
      setLoading(true);
      const conversations = await invoke<Conversation[]>("get_conversations");
      setItems(conversations);
      // Select the first conversation by default
      if (conversations.length > 0 && selectedItem() === null) {
        setSelectedItem(conversations[0].id);
      }
    } catch (error) {
      console.error("Error loading conversations:", error);
    } finally {
      setLoading(false);
    }
  }
  
  // Load messages for the selected conversation
  async function loadMessages(conversationId: number) {
    try {
      setLoadingMessages(true);
      const conversationMessages = await invoke<Message[]>("get_messages", { conversationId });
      setMessages(conversationMessages);
    } catch (error) {
      console.error(`Error loading messages for conversation ${conversationId}:`, error);
    } finally {
      setLoadingMessages(false);
    }
  }
  
  // Handle conversation selection
  function handleSelectConversation(id: number) {
    setSelectedItem(id);
  }
  
  // Load initial conversations
  onMount(() => {
    loadConversations();
  });
  
  // Load messages when selected conversation changes
  createEffect(() => {
    const currentConversation = selectedItem();
    if (currentConversation !== null) {
      loadMessages(currentConversation);
    }
  });

  async function sendMessage(e: Event) {
    e.preventDefault();
    
    if (!inputValue().trim() || selectedItem() === null) return;
    
    try {
      // Add user message to chat
      const userMessage = await invoke<Message>("add_message", {
        conversationId: selectedItem(),
        text: inputValue(),
        sender: "user"
      });
      
      setMessages([...messages(), userMessage]);
      setInputValue("");
      
      // For demo purposes, we'll use a simple response
      // In a real app, you'd call your AI service here
      const aiResponse = await invoke<Message>("add_message", {
        conversationId: selectedItem(),
        text: `You said: "${userMessage.text}". This is a simulated AI response.`,
        sender: "ai"
      });
      
      setMessages([...messages(), aiResponse]);
    } catch (error) {
      console.error("Error sending message:", error);
    }
  }
  
  async function createNewConversation(e: Event) {
    e.preventDefault();
    
    if (!newConversationName().trim()) return;
    
    try {
      const newConversation = await invoke<Conversation>("create_conversation", { 
        name: newConversationName() 
      });
      
      // Reload the conversation list
      await loadConversations();
      
      // Select the newly created conversation
      setSelectedItem(newConversation.id);
      
      // Reset the form
      setNewConversationName("");
      setShowNewConversation(false);
    } catch (error) {
      console.error("Error creating conversation:", error);
    }
  }

  return (
    <div class="app-container">
      {/* Sidebar */}
      <div class="sidebar">
        <div class="sidebar-header">
          <h2>Conversations</h2>
          <button 
            class="new-conversation-btn" 
            onClick={() => setShowNewConversation(!showNewConversation())}
            title="New Conversation"
          >
            +
          </button>
        </div>
        
        <Show when={showNewConversation()}>
          <form class="new-conversation-form" onSubmit={createNewConversation}>
            <input
              type="text"
              value={newConversationName()}
              onInput={(e) => setNewConversationName(e.currentTarget.value)}
              placeholder="Conversation name..."
            />
            <button type="submit">Create</button>
          </form>
        </Show>
        
        <div class="item-list">
          {loading() ? (
            <div class="loading-state">Loading conversations...</div>
          ) : (
            <For each={items()}>
              {(item) => (
                <div 
                  class={`item ${selectedItem() === item.id ? 'selected' : ''}`}
                  onClick={() => handleSelectConversation(item.id)}
                >
                  <div class="item-name">{item.name}</div>
                </div>
              )}
            </For>
          )}
        </div>
      </div>
      
      {/* Main chat area */}
      <div class="main-content">
        <div class="chat-container">
          <div class="chat-messages">
            {loadingMessages() ? (
              <div class="loading-state">Loading messages...</div>
            ) : (
              <For each={messages()}>
                {(message) => (
                  <div class={`message ${message.sender}`}>
                    <div class="message-content">{message.text}</div>
                  </div>
                )}
              </For>
            )}
          </div>
          
          {/* Message input */}
          <form class="chat-input" onSubmit={sendMessage}>
            <input
              type="text"
              value={inputValue()}
              onInput={(e) => setInputValue(e.currentTarget.value)}
              placeholder="Type your message here..."
              disabled={selectedItem() === null}
            />
            <button type="submit" disabled={selectedItem() === null}>
              <span>Send</span>
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default App;
