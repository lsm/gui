import { For } from "solid-js";
import { Conversation } from "../types";

type SidebarProps = {
  items: Conversation[];
  loading: boolean;
  selectedItem: string | null;
  onSelectConversation: (id: string) => void;
  onCreateNewConversation: () => void;
};

export function Sidebar(props: SidebarProps) {
  return (
    <div class="sidebar">
      <div class="sidebar-header">
        <h2>Conversations</h2>
      </div>
      
      <div class="item-list">
        {props.loading ? (
          <div class="loading-state">Loading conversations...</div>
        ) : (
          <For each={props.items}>
            {(item) => (
              <div 
                class={`item ${props.selectedItem === item.id ? 'selected' : ''}`}
                onClick={() => props.onSelectConversation(item.id)}
              >
                <div class="item-name">{item.name}</div>
              </div>
            )}
          </For>
        )}
      </div>
    </div>
  );
} 