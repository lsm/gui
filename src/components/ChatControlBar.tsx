type ChatControlBarProps = {
  onCreateNewConversation: () => void;
};

export function ChatControlBar(props: ChatControlBarProps) {
  return (
    <div class="chat-control-bar">
      <div class="control-spacer"></div>
      <button 
        class="new-conversation-btn" 
        onClick={props.onCreateNewConversation}
        title="New Chat"
      >
        +
      </button>
    </div>
  );
} 