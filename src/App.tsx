import { createSignal, onMount, createEffect, onCleanup } from "solid-js";
import { subscribeToUpdates } from "./db-client";
import { Sidebar, ChatView, ChatControlBar, ConfirmationModal } from "./components";
import { Message, Chat } from "./types";
import { loadChats, loadMessages, createChat, sendMessage, updateWindowTitle, deleteChat } from "./services/chatService";

function App() {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [inputValue, setInputValue] = createSignal("");
  const [selectedItem, setSelectedItem] = createSignal<string | null>(null);
  const [items, setItems] = createSignal<Chat[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [loadingMessages, setLoadingMessages] = createSignal(false);
  const [newChatName, setNewChatName] = createSignal("");
  const [showNewChat, setShowNewChat] = createSignal(false);
  
  // State for confirmation modal
  const [isConfirmModalOpen, setIsConfirmModalOpen] = createSignal(false);
  const [chatIdToDelete, setChatIdToDelete] = createSignal<string | null>(null);
  const [chatNameToDelete, setChatNameToDelete] = createSignal<string>("");
  
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
      setItems,
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

  // Listen for changes in items to update the window title if the selected chat's name changes
  createEffect(() => {
    const currentChat = selectedItem();
    const chatsList = items();
    
    if (currentChat !== null) {
      const selectedChatName = chatsList.find(item => item.id === currentChat)?.name || "Chat";
      updateWindowTitle(selectedChatName);
    }
  });

  async function handleSendMessage(e: Event) {
    e.preventDefault();
    console.log("handleSendMessage called", {
      inputValue: inputValue(),
      hasText: !!inputValue().trim(),
      selectedItem: selectedItem()
    });
    
    if (!inputValue().trim() || selectedItem() === null) {
      console.log("Early return: no input or no selected chat");
      return;
    }
    
    try {
      console.log("Calling sendMessage with", {
        chatId: selectedItem()!,
        text: inputValue()
      });
      const [userMessage, aiResponse] = await sendMessage(selectedItem()!, inputValue());
      
      console.log("Messages received", {
        userMessage,
        aiResponse
      });
      
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

  // Function to initiate chat deletion (opens modal)
  async function handleDeleteChat() {
    const currentChatId = selectedItem();
    if (!currentChatId) {
      console.warn("No chat selected for deletion.");
      return;
    }

    const chatToDelete = items().find(chat => chat.id === currentChatId);
    if (!chatToDelete) {
      console.warn(`Chat with ID ${currentChatId} not found in local state.`);
      return;
    }

    // Set state to open the modal
    setChatIdToDelete(currentChatId);
    setChatNameToDelete(chatToDelete.name || "this chat");
    setIsConfirmModalOpen(true);
  }

  // Function called when user confirms deletion in the modal
  async function confirmDelete() {
    const idToDelete = chatIdToDelete();
    if (!idToDelete) return; // Should not happen if modal is open correctly

    setIsConfirmModalOpen(false); // Close modal immediately

    try {
      await deleteChat(idToDelete);

      // Update local state
      const currentItems = items(); // Capture current items before filtering
      const remainingChats = currentItems.filter(chat => chat.id !== idToDelete);
      setItems(remainingChats);

      // Select next chat or clear selection
      if (selectedItem() === idToDelete) {
        if (remainingChats.length > 0) {
          // Find the index of the deleted chat in the *original* list
          const deletedIndex = currentItems.findIndex(chat => chat.id === idToDelete);
          // Select the previous chat, or the first chat if the deleted one was the first
          const nextIndex = Math.max(0, deletedIndex - 1);
          setSelectedItem(remainingChats[nextIndex]?.id || null);
        } else {
          setSelectedItem(null);
          setMessages([]); // Clear messages if no chats left
          updateWindowTitle("Chat"); // Reset window title
        }
      } // Selection remains if a different chat was selected
      
    } catch (error) {
      console.error(`Error deleting chat ${idToDelete}:`, error);
      const errorMessage = error instanceof Error ? error.message : "Failed to delete chat. Please try again.";
      alert(`Error: ${errorMessage}`);
    } finally {
      // Reset deletion state
      setChatIdToDelete(null);
      setChatNameToDelete("");
    }
  }

  // Function called when user cancels deletion in the modal
  function cancelDelete() {
    setIsConfirmModalOpen(false);
    setChatIdToDelete(null);
    setChatNameToDelete("");
  }

  return (
    <div class="flex h-screen w-full overflow-hidden bg-bg-primary text-text-primary">
      {/* Sidebar */}
      <Sidebar 
        items={items()}
        loading={loading()} 
        selectedItem={selectedItem()} 
        onSelectChat={handleSelectChat}
        onCreateNewChat={handleCreateNewChat}
      />
      
      {/* Main chat area */}
      <div class="flex-1 flex flex-col overflow-hidden bg-chat-bg relative">
        <ChatControlBar onDeleteChat={handleDeleteChat} />
        
        <ChatView 
          messages={messages()}
          loadingMessages={loadingMessages()}
          inputValue={inputValue()}
          onInputChange={setInputValue}
          onSendMessage={handleSendMessage}
          disabled={selectedItem() === null}
        />
      </div>

      {/* Confirmation Modal */}
      <ConfirmationModal 
        isOpen={isConfirmModalOpen()}
        title="Confirm Deletion"
        message={`Are you sure you want to delete "${chatNameToDelete()}"? This action cannot be undone.`}
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
      />
    </div>
  );
}

export default App;
