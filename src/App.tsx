import { createSignal, onMount, createEffect, onCleanup } from "solid-js";
import { subscribeToUpdates } from "./db-client";
import { Sidebar, ChatView, ChatControlBar } from "./components";
import { Message, Chat } from "./types";
import { loadChats, loadMessages, createChat, sendMessage, updateWindowTitle } from "./services/chatService";
import "./App.css";

function App() {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [inputValue, setInputValue] = createSignal("");
  const [selectedItem, setSelectedItem] = createSignal<string | null>(null);
  const [items, setItems] = createSignal<Chat[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [loadingMessages, setLoadingMessages] = createSignal(false);
  const [newChatName, setNewChatName] = createSignal("");
  const [showNewChat, setShowNewChat] = createSignal(false);
  
  // Load chat list from database
  async function fetchChats() {
    try {
      setLoading(true);
      const chats = await loadChats();
      setItems(chats);
      // Select the first chat by default
      if (chats.length > 0 && selectedItem() === null) {
        setSelectedItem(chats[0].id);
      }
    } catch (error) {
      console.error("Error loading chats:", error);
    } finally {
      setLoading(false);
    }
  }
  
  // Load messages for the selected chat
  async function fetchMessages(chatId: string) {
    try {
      setLoadingMessages(true);
      const chatMessages = await loadMessages(chatId);
      setMessages(chatMessages);
    } catch (error) {
      console.error(`Error loading messages for chat ${chatId}:`, error);
    } finally {
      setLoadingMessages(false);
    }
  }
  
  // Handle chat selection
  function handleSelectChat(id: string) {
    setSelectedItem(id);
  }
  
  // Load initial chats
  onMount(async () => {
    await fetchChats();
    
    // Set up real-time updates
    const cleanup = await subscribeToUpdates(
      setMessages,
      setItems,
      selectedItem()
    );
    
    onCleanup(cleanup);
  });
  
  // Load messages when selected chat changes
  createEffect(() => {
    const currentChat = selectedItem();
    if (currentChat !== null) {
      fetchMessages(currentChat);
      
      // Update window title with current chat name
      const selectedChatName = items().find(item => item.id === currentChat)?.name || "Chat";
      updateWindowTitle(selectedChatName);
    }
  });

  async function handleSendMessage(e: Event) {
    e.preventDefault();
    
    if (!inputValue().trim() || selectedItem() === null) return;
    
    try {
      const [userMessage, aiResponse] = await sendMessage(selectedItem()!, inputValue());
      
      // Update messages state with both messages
      setMessages([...messages(), userMessage, aiResponse]);
      setInputValue("");
    } catch (error) {
      console.error("Error sending message:", error);
      
      // Display a user-friendly error message
      const errorMessage = error instanceof Error ? error.message : "Failed to send message. Please try again.";
      alert(`Error: ${errorMessage}`);
    }
  }
  
  async function handleCreateNewChat(e?: Event) {
    if (e) e.preventDefault();
    
    try {
      const newChat = await createChat(newChatName() || "New Chat");
      
      // Select the newly created chat
      setSelectedItem(newChat.id);
      
      // Reset the form state
      setNewChatName("");
      setShowNewChat(false);
    } catch (error) {
      console.error("Error creating chat:", error);
      
      // Display a user-friendly error message
      const errorMessage = error instanceof Error ? error.message : "Failed to create chat. Please try again.";
      alert(`Error: ${errorMessage}`);
      
      // Try to refresh chats in case there was an issue
      fetchChats();
    }
  }

  return (
    <div class="app-container">
      {/* Sidebar */}
      <Sidebar 
        items={items()}
        loading={loading()} 
        selectedItem={selectedItem()} 
        onSelectChat={handleSelectChat}
        onCreateNewChat={handleCreateNewChat}
      />
      
      {/* Main chat area */}
      <div class="main-content">
        <ChatControlBar onCreateNewChat={handleCreateNewChat} />
        
        <ChatView 
          messages={messages()}
          loadingMessages={loadingMessages()}
          inputValue={inputValue()}
          onInputChange={setInputValue}
          onSendMessage={handleSendMessage}
          disabled={selectedItem() === null}
        />
      </div>
    </div>
  );
}

export default App;
