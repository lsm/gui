export type Message = {
  id: string;
  conversation_id: string;
  text: string;
  sender: string;
  timestamp: number;
  sequence_number: number;
};

export type Conversation = {
  id: string;
  name: string;
  created_at: number;
}; 