import { For, createSignal } from "solid-js";
import { Chat } from "../types";
import { updateChatTitle } from "../services/chatService";
import { ModelDownloadProgress } from "./ModelDownloadProgress";

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
    <aside class="w-[260px] bg-sidebar-bg overflow-hidden flex flex-col scroll-smooth overscroll-contain">
      <div class="flex justify-between items-center p-4 border-b border-sidebar-border relative">
        <h2 class="p-0 m-0 text-sm tracking-[0.5px] text-sidebar-header-text font-medium">Chats</h2>
        <button 
          onClick={props.onCreateNewChat}
          class="text-sidebar-header-text hover:bg-sidebar-item-hover p-1 rounded"
          title="New Chat"
        >
          {/* Basic plus icon for now */}
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
          </svg>
        </button>
      </div>
      
      <div class="flex-1 overflow-y-auto p-2">
        {props.loading ? (
          <div class="p-4 text-text-secondary text-sm text-center italic my-5">Loading chats...</div>
        ) : (
          <For each={props.items}>
            {(item) => (
              <div 
                class={`p-3 cursor-pointer rounded-md transition-colors duration-200 my-0.5 flex items-center min-h-6 hover:bg-sidebar-item-hover ${
                  props.selectedItem === item.id ? 'bg-sidebar-item-selected' : ''
                }`}
                onClick={() => props.onSelectChat(item.id)}
              >
                {editingChatId() === item.id ? (
                  <input
                    type="text"
                    class="edit-title-input"
                    style={{
                      "height": "20px",
                      "line-height": "20px",
                      "padding": "0"
                    }}
                    value={editTitle()}
                    onInput={handleTitleChange}
                    onKeyDown={(e) => handleTitleKeyDown(e, item.id)}
                    onBlur={() => handleTitleBlur(item.id)}
                    ref={(el) => { 
                      setTimeout(() => {
                        el.focus();
                        const len = el.value.length;
                        el.setSelectionRange(len, len);
                      }, 0);
                    }}
                  />
                ) : (
                  <div 
                    class="whitespace-nowrap overflow-hidden text-ellipsis w-full text-left text-sm"
                    style={{
                      "height": "20px",
                      "line-height": "20px"
                    }}
                    onDblClick={() => handleDoubleClick(item)}
                  >{item.name}</div>
                )}
              </div>
            )}
          </For>
        )}
      </div>
      
      {/* Model download progress at bottom of sidebar */}
      <div class="mt-auto">
        <ModelDownloadProgress />
      </div>
    </aside>
  );
} 