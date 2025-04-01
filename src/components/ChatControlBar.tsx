interface ChatControlBarProps {
  onCreateNewChat: () => void;
}

export function ChatControlBar(props: ChatControlBarProps) {
  return (
    <div class="flex justify-end items-center px-3 py-1.5 bg-secondary border-b border-border">
      <div class="flex-1"></div>
      <button 
        class="w-6 h-6 bg-transparent border-none flex items-center justify-center text-accent-primary hover:text-text-primary hover:bg-hover-bg rounded-[3px] text-base leading-none pb-0.5 transition-all duration-200 cursor-pointer static" 
        onClick={props.onCreateNewChat}
        title="New Chat"
      >
        +
      </button>
    </div>
  );
} 