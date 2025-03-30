import { createSignal, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Define types for our application
type Message = {
  id: number;
  text: string;
  sender: "user" | "ai";
};

type Item = {
  id: number;
  name: string;
  preview: string;
};

function App() {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [inputValue, setInputValue] = createSignal("");
  const [selectedItem, setSelectedItem] = createSignal<number | null>(null);
  
  // Sample items for the sidebar - replace with your actual data
  const [items] = createSignal<Item[]>([
    { id: 1, name: "Item 1", preview: "Preview 1" },
    { id: 2, name: "Item 2", preview: "Preview 2" },
    { id: 3, name: "Item 3", preview: "Preview 3" },
    { id: 4, name: "Item 4", preview: "Preview 4" },
    { id: 5, name: "Item 5", preview: "Preview 5" },
  ]);

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

  return (
    <div class="app-container">
      {/* Sidebar */}
      <div class="sidebar">
        <h2>Items</h2>
        <div class="item-list">
          <For each={items()}>
            {(item) => (
              <div 
                class={`item ${selectedItem() === item.id ? 'selected' : ''}`}
                onClick={() => setSelectedItem(item.id)}
              >
                <div class="item-preview">{item.preview}</div>
                <div class="item-name">{item.name}</div>
              </div>
            )}
          </For>
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
            <button type="submit">Send</button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default App;
