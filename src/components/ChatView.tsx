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
    <div class="flex flex-col h-full bg-chat-bg relative">
      <div class="flex-1 overflow-y-auto py-2.5 pb-24 flex flex-col mb-31 bg-chat-bg scroll-smooth overscroll-contain" ref={messagesContainer}>
        {props.loadingMessages ? (
          <div class="p-4 text-text-secondary text-sm text-center italic my-5">Loading messages...</div>
        ) : (
          <For each={props.messages}>
            {(message) => (
              <div class={`max-w-full py-3 shadow-none leading-relaxed text-[15px] border-b-0 m-0 w-full relative ${normalizeSender(message.sender) === 'user' ? 'self-end bg-transparent text-text-primary py-2.5 pr-[5%] pl-0 text-right' : 'self-center bg-transparent text-text-primary py-2.5 px-[10%] flex justify-center'}`}>
                <div class={`break-words leading-normal max-w-[90%] ${normalizeSender(message.sender) === 'user' ? 'bg-chat-user-message-bg p-3 rounded-[18px] rounded-br-1 relative inline-block shadow-[0_2px_6px_var(--color-chat-message-shadow)] max-w-full text-left' : 'bg-transparent p-3 relative inline-block max-w-[650px] text-left'}`}>
                  {message.text}
                </div>
              </div>
            )}
          </For>
        )}
      </div>
      
      {/* Message input */}
      <form class="flex px-[5%] py-4 pb-6 bg-chat-bg border-t border-border shadow-[0_-4px_20px_var(--color-chat-shadow)] fixed bottom-0 left-0 right-0 z-100 w-[calc(100%-260px)] ml-[260px] box-border" onSubmit={handleSubmit}>
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
          class="flex-1 w-full border border-border p-3.5 rounded-lg bg-chat-input-bg text-text-primary text-[0.95em] shadow-[0_0_10px_rgba(0,0,0,0.05)] transition-all duration-200 mr-0 resize-none min-h-6 max-h-[200px] leading-normal focus:border-accent-primary focus:shadow-[0_0_0_3px_rgba(156,163,175,0.2)] disabled:bg-[rgba(40,40,40,0.5)] disabled:text-[rgba(255,255,255,0.5)] disabled:cursor-not-allowed"
        />
      </form>
    </div>
  );
} 