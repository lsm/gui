import { For } from "solid-js";
import { Chat } from "../types";

interface SidebarProps {
  items: Chat[];
  loading: boolean;
  selectedItem: string | null;
  onSelectChat: (id: string) => void;
  onCreateNewChat: () => void;
}

export function Sidebar(props: SidebarProps) {
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
                <div class="item-name">{item.name}</div>
              </div>
            )}
          </For>
        )}
      </div>
    </aside>
  );
} 