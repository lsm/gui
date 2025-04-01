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
    <aside class="w-[260px] bg-sidebar-bg overflow-y-auto flex flex-col">
      <div class="flex justify-between items-center p-4 border-b border-sidebar-border relative">
        <h2 class="p-0 m-0 text-sm tracking-[0.5px] text-sidebar-header-text font-medium">Chats</h2>
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
    </aside>
  );
} 