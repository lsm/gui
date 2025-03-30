import { createSignal, onMount, createEffect, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { subscribeToUpdates } from "./db-client";
import { Sidebar, ChatView, ChatControlBar } from "./components";
import { Message, Conversation } from "./types";
import "./App.css";

function App() {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [inputValue, setInputValue] = createSignal("");
  const [selectedItem, setSelectedItem] = createSignal<string | null>(null);
  const [items, setItems] = createSignal<Conversation[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [loadingMessages, setLoadingMessages] = createSignal(false);
  const [newConversationName, setNewConversationName] = createSignal("");
  const [showNewConversation, setShowNewConversation] = createSignal(false);
  
  // Load conversation list from database
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
  async function loadMessages(conversationId: string) {
    try {
      setLoadingMessages(true);
      const conversationMessages = await invoke<Message[]>("get_messages", { conversation_id: conversationId });
      setMessages(conversationMessages);
    } catch (error) {
      console.error(`Error loading messages for conversation ${conversationId}:`, error);
    } finally {
      setLoadingMessages(false);
    }
  }
  
  // Handle conversation selection
  function handleSelectConversation(id: string) {
    setSelectedItem(id);
  }
  
  // Load initial conversations
  onMount(async () => {
    await loadConversations();
    
    // Set up real-time updates
    const cleanup = await subscribeToUpdates(
      setMessages,
      setItems,
      selectedItem()
    );
    
    onCleanup(cleanup);
  });
  
  // Load messages when selected conversation changes
  createEffect(() => {
    const currentConversation = selectedItem();
    if (currentConversation !== null) {
      loadMessages(currentConversation);
      
      // Update window title with current conversation name
      const selectedConversationName = items().find(item => item.id === currentConversation)?.name || "Chat";
      updateWindowTitle(selectedConversationName);
    }
  });
  
  // Update the native window title
  async function updateWindowTitle(title: string) {
    try {
      const currentWindow = Window.getCurrent();
      await currentWindow.setTitle(`${title} - Chat App`);
    } catch (error) {
      console.error("Error updating window title:", error);
    }
  }

  async function sendMessage(e: Event) {
    e.preventDefault();
    
    if (!inputValue().trim() || selectedItem() === null) return;
    
    try {
      // Add user message to chat
      const userMessage = await invoke<Message>("add_message", {
        conversation_id: selectedItem(),
        text: inputValue(),
        sender_name: "user"
      });
      
      setMessages([...messages(), userMessage]);
      setInputValue("");
      
      // For demo purposes, we'll use a simple response
      // In a real app, you'd call your AI service here
      const aiResponse = await invoke<Message>("add_message", {
        conversation_id: selectedItem(),
        text: `You said: "${userMessage.text}". This is a simulated AI response.`,
        sender_name: "ai"
      });
      
      setMessages([...messages(), aiResponse]);
    } catch (error) {
      console.error("Error sending message:", error);
    }
  }
  
  async function createNewConversation(e?: Event) {
    if (e) e.preventDefault();
    
    try {
      const newConversation = await invoke<Conversation>("create_conversation", { 
        name: newConversationName() || "New Chat" 
      });
      
      // Select the newly created conversation
      setSelectedItem(newConversation.id);
      
      // Reset the form state
      setNewConversationName("");
      setShowNewConversation(false);
    } catch (error) {
      console.error("Error creating conversation:", error);
    }
  }

  return (
    <div class="app-container">
      {/* Sidebar */}
      <Sidebar 
        items={items()}
        loading={loading()} 
        selectedItem={selectedItem()} 
        onSelectConversation={handleSelectConversation}
        onCreateNewConversation={createNewConversation}
      />
      
      {/* Main chat area */}
      <div class="main-content">
        <ChatControlBar onCreateNewConversation={createNewConversation} />
        
        <ChatView 
          messages={messages()}
          loadingMessages={loadingMessages()}
          inputValue={inputValue()}
          onInputChange={setInputValue}
          onSendMessage={sendMessage}
          disabled={selectedItem() === null}
        />
      </div>
    </div>
  );
}

export default App;
