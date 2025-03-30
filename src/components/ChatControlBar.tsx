interface ChatControlBarProps {
  onCreateNewChat: () => void;
}

export function ChatControlBar(props: ChatControlBarProps) {
  return (
    <div class="chat-control-bar">
      <div class="control-spacer"></div>
      <button 
        class="new-chat-btn" 
        onClick={props.onCreateNewChat}
        title="New Chat"
      >
        +
      </button>
    </div>
  );
} 