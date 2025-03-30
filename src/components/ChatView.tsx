import { For } from "solid-js";
import { Message } from "../types";

type ChatViewProps = {
  messages: Message[];
  loadingMessages: boolean;
  inputValue: string;
  onInputChange: (value: string) => void;
  onSendMessage: (e: Event) => void;
  disabled: boolean;
};

export function ChatView(props: ChatViewProps) {
  return (
    <div class="chat-container">
      <div class="chat-messages">
        {props.loadingMessages ? (
          <div class="loading-state">Loading messages...</div>
        ) : (
          <For each={props.messages}>
            {(message) => (
              <div class={`message ${message.sender}`}>
                <div class="message-content">{message.text}</div>
              </div>
            )}
          </For>
        )}
      </div>
      
      {/* Message input */}
      <form class="chat-input" onSubmit={props.onSendMessage}>
        <input
          type="text"
          value={props.inputValue}
          onInput={(e) => props.onInputChange(e.currentTarget.value)}
          placeholder="Type your message here..."
          disabled={props.disabled}
        />
      </form>
    </div>
  );
} 