import { createSignal, For, onMount, Show } from "solid-js";
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
  
  onMount(() => {
    loadConversations();
  });

  async function sendMessage(e: Event) {
    e.preventDefault();
    
    if (!inputValue().trim()) return;
    
    // Add user message to chat
    const userMessage: Message = { 
      id: Date.now(), 
      text: inputValue(), 
      sender: "user" 
    };
    
    setMessages([...messages(), userMessage]);
    setInputValue("");
    
    try {
      // This is where you would call your language model API
      // For now we'll use the greet function from the template
      const response = await invoke("greet", { name: userMessage.text });
      
      // Add AI response to chat
      const aiMessage: Message = {
        id: Date.now() + 1,
        text: response as string,
        sender: "ai"
      };
      
      setMessages([...messages(), aiMessage]);
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
                  onClick={() => setSelectedItem(item.id)}
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
            <For each={messages()}>
              {(message) => (
                <div class={`message ${message.sender}`}>
                  <div class="message-content">{message.text}</div>
                </div>
              )}
            </For>
          </div>
          
          {/* Message input */}
          <form class="chat-input" onSubmit={sendMessage}>
            <input
              type="text"
              value={inputValue()}
              onInput={(e) => setInputValue(e.currentTarget.value)}
              placeholder="Type your message here..."
            />
            <button type="submit"><span>Send</span></button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default App;
