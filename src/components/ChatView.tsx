import { For, createEffect, onMount } from "solid-js";
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
  // Reference to the chat messages container
  let messagesContainer: HTMLDivElement | undefined;
  // Reference to the textarea
  let textareaRef: HTMLTextAreaElement | undefined;
  
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

  // Function to adjust textarea height
  const adjustTextareaHeight = () => {
    if (textareaRef) {
      textareaRef.style.height = '24px'; // Reset height to single line
      const scrollHeight = textareaRef.scrollHeight;
      textareaRef.style.height = Math.min(scrollHeight, 200) + 'px'; // Set new height, capped at 200px
    }
  };

  // Adjust height when input value changes
  createEffect(() => {
    // Use setTimeout to ensure the DOM has updated
    setTimeout(adjustTextareaHeight, 0);
  });

  // Helper function to normalize sender type for proper styling
  const normalizeSender = (sender: string): "user" | "ai" => {
    // If sender starts with "user-", it's a user message
    if (sender.startsWith("user-") || sender === "user") {
      return "user";
    }
    // Otherwise treat as AI
    return "ai";
  };
  
  // Scroll to bottom when new messages are added
  createEffect(() => {
    // Track messages to trigger effect when they change
    const messageLength = props.messages.length;
    if (messagesContainer && messageLength > 0) {
      // Use MutationObserver to detect when messages are fully rendered
      const observer = new MutationObserver(() => {
        messagesContainer!.scrollTop = messagesContainer!.scrollHeight;
        observer.disconnect();
      });
      
      observer.observe(messagesContainer, { childList: true, subtree: true });
      
      // Also use setTimeout as a fallback
      setTimeout(() => {
        messagesContainer!.scrollTop = messagesContainer!.scrollHeight;
      }, 150);
    }
  });
  
  // Scroll to bottom on initial load and when loading state changes
  createEffect(() => {
    // Track loading state changes
    const isLoading = props.loadingMessages;
    if (!isLoading && messagesContainer && props.messages.length > 0) {
      setTimeout(() => {
        messagesContainer!.scrollTop = messagesContainer!.scrollHeight;
      }, 150);
    }
  });
  
  // Initial scroll when component mounts
  onMount(() => {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  });

  return (
    <div class="chat-container">
      <div class="chat-messages" ref={messagesContainer}>
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
        <textarea
          ref={textareaRef}
          value={props.inputValue}
          onInput={(e) => {
            props.onInputChange(e.currentTarget.value);
            adjustTextareaHeight();
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleSubmit(e);
            }
          }}
          placeholder="Type your message here... (Shift+Enter for new line)"
          disabled={props.disabled}
        />
      </form>
    </div>
  );
} 