interface ChatControlBarProps {
  onDeleteChat: () => void;
}

export function ChatControlBar(props: ChatControlBarProps) {
  return (
    <div class="flex justify-end items-center px-3 py-1.5 bg-secondary border-b border-border">
      <div class="flex-1"></div>
      <button 
        class="w-6 h-6 bg-transparent border-none flex items-center justify-center text-accent-primary hover:text-text-primary hover:bg-hover-bg rounded-[3px] text-base leading-none pb-0.5 transition-all duration-200 cursor-pointer static" 
        onClick={props.onDeleteChat}
        title="Delete Chat"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  );
} 