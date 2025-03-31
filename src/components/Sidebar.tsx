import { For, createSignal } from "solid-js";
import { Chat } from "../types";
import { updateChatTitle } from "../services/chatService";

interface SidebarProps {
  items: Chat[];
  loading: boolean;
  selectedItem: string | null;
  onSelectChat: (id: string) => void;
  onCreateNewChat: () => void;
}

export function Sidebar(props: SidebarProps) {
  const [editingChatId, setEditingChatId] = createSignal<string | null>(null);
  const [editTitle, setEditTitle] = createSignal("");
  
  const handleDoubleClick = (chat: Chat) => {
    setEditingChatId(chat.id);
    setEditTitle(chat.name);
  };
  
  const handleTitleChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    setEditTitle(target.value);
  };
  
  const handleTitleKeyDown = async (e: KeyboardEvent, chatId: string) => {
    if (e.key === "Enter") {
      e.preventDefault();
      await saveChatTitle(chatId);
    } else if (e.key === "Escape") {
      setEditingChatId(null);
    }
  };
  
  const handleTitleBlur = async (chatId: string) => {
    await saveChatTitle(chatId);
  };
  
  const saveChatTitle = async (chatId: string) => {
    const newTitle = editTitle().trim();
    if (newTitle) {
      try {
        await updateChatTitle(chatId, newTitle);
        setEditingChatId(null);
      } catch (error) {
        console.error("Failed to update chat title:", error);
        // Optionally show an error message to the user
      }
    } else {
      setEditingChatId(null);
    }
  };

  return (
    <aside class="sidebar">
      <div class="sidebar-header">
        <h2>Chats</h2>
      </div>
      
      <div class="item-list">
        {props.loading ? (
          <div class="loading-state">Loading chats...</div>
        ) : (
          <For each={props.items}>
            {(item) => (
              <div 
                class={`item ${props.selectedItem === item.id ? 'selected' : ''}`}
                onClick={() => props.onSelectChat(item.id)}
              >
                {editingChatId() === item.id ? (
                  <input
                    type="text"
                    class="edit-title-input"
                    value={editTitle()}
                    onInput={handleTitleChange}
                    onKeyDown={(e) => handleTitleKeyDown(e, item.id)}
                    onBlur={() => handleTitleBlur(item.id)}
                    // Focus the input element when it appears with a short timeout
                    ref={(el) => { 
                      setTimeout(() => {
                        el.focus();
                        // Set cursor position to the end of the text
                        const len = el.value.length;
                        el.setSelectionRange(len, len);
                      }, 0);
                    }}
                  />
                ) : (
                  <div 
                    class="item-name"
                    onDblClick={() => handleDoubleClick(item)}
                  >{item.name}</div>
                )}
              </div>
            )}
          </For>
        )}
      </div>
    </aside>
  );
} 