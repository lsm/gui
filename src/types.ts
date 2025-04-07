export enum AuthorType {
  User = "User",
  Assistant = "Assistant",
  Tool = "Tool",
  System = "System"
}

export type Author = {
  id?: string;
  kind: AuthorType;
  name: string;
};

export type Message = {
  id: string;
  chat_id: string;
  text: string;
  author: Author;
  created_at: string; // ISO format datetime
  sequence_number: number;
};

export type Chat = {
  id: string;
  name: string;
  author: Author;
  created_at: string; // ISO format datetime
}; 