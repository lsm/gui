export type Message = {
  id: string;
  chat_id: string;
  text: string;
  sender: string;
  timestamp: number;
  sequence_number: number;
};

export type Chat = {
  id: string;
  name: string;
  created_at: number;
}; 