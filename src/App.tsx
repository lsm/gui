import { createSignal, onMount, createEffect, onCleanup } from "solid-js";
import { subscribeToUpdates } from "./db-client";
import { Sidebar, ChatView, ChatControlBar } from "./components";
import { Message, Conversation } from "./types";
import { loadConversations, loadMessages, createConversation, sendMessage, updateWindowTitle } from "./services/conversationService";
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
  async function fetchConversations() {
    try {
      setLoading(true);
      const conversations = await loadConversations();
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
  async function fetchMessages(conversationId: string) {
    try {
      setLoadingMessages(true);
      const conversationMessages = await loadMessages(conversationId);
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
    await fetchConversations();
    
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
      fetchMessages(currentConversation);
      
      // Update window title with current conversation name
      const selectedConversationName = items().find(item => item.id === currentConversation)?.name || "Chat";
      updateWindowTitle(selectedConversationName);
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
  
  async function handleCreateNewConversation(e?: Event) {
    if (e) e.preventDefault();
    
    try {
      const newConversation = await createConversation(newConversationName() || "New Chat");
      
      // Select the newly created conversation
      setSelectedItem(newConversation.id);
      
      // Reset the form state
      setNewConversationName("");
      setShowNewConversation(false);
    } catch (error) {
      console.error("Error creating conversation:", error);
      
      // Display a user-friendly error message
      const errorMessage = error instanceof Error ? error.message : "Failed to create conversation. Please try again.";
      alert(`Error: ${errorMessage}`);
      
      // Try to refresh conversations in case there was an issue
      fetchConversations();
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
        onCreateNewConversation={handleCreateNewConversation}
      />
      
      {/* Main chat area */}
      <div class="main-content">
        <ChatControlBar onCreateNewConversation={handleCreateNewConversation} />
        
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
