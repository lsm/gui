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
  // Handle the submit event
  const handleSubmit = (e: Event) => {
    e.preventDefault();
    console.log("Form submitted", {
      inputValue: props.inputValue,
      hasText: !!props.inputValue.trim(),
      disabled: props.disabled
    });
    
    // Only proceed if there's input
    if (props.inputValue.trim()) {
      console.log("Calling onSendMessage");
      props.onSendMessage(e);
    }
  };

  // Helper function to normalize sender type for proper styling
  const normalizeSender = (sender: string): "user" | "ai" => {
    // If sender starts with "user-", it's a user message
    if (sender.startsWith("user-") || sender === "user") {
      return "user";
    }
    // Otherwise treat as AI
    return "ai";
  };

  return (
    <div class="chat-container">
      <div class="chat-messages">
        {props.loadingMessages ? (
          <div class="loading-state">Loading messages...</div>
        ) : (
          <For each={props.messages}>
            {(message) => (
              <div class={`message ${normalizeSender(message.sender)}`}>
                <div class="message-content">{message.text}</div>
              </div>
            )}
          </For>
        )}
      </div>
      
      {/* Message input */}
      <form class="chat-input" onSubmit={handleSubmit}>
        <input
          type="text"
          value={props.inputValue}
          onInput={(e) => props.onInputChange(e.currentTarget.value)}
          placeholder="Type your message here..."
          disabled={props.disabled}
        />
        <button 
          type="submit" 
          disabled={props.disabled || !props.inputValue.trim()}
        >
          <span>Send</span>
        </button>
      </form>
    </div>
  );
} 