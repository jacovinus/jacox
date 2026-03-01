export interface Session {
  id: string; // UUID
  name: string;
  created_at: string;
  updated_at: string;
  metadata: Record<string, any>;
}

export interface Message {
  id: number;
  session_id: string;
  role: 'system' | 'user' | 'assistant';
  content: string;
  model: string | null;
  token_count: number | null;
  created_at: string;
  metadata: Record<string, any>;
}

export interface CreateSessionRequest {
  name: string;
  metadata?: Record<string, any>;
}

export interface UpdateSessionRequest {
  name?: string;
  metadata?: Record<string, any>;
}

export interface CreateMessageRequest {
  role: string;
  content: string;
  model?: string;
  token_count?: number;
  metadata?: Record<string, any>;
}

export interface WsServerMessage {
  type: 'chunk' | 'done' | 'error' | 'status';
  content: string;
}

export interface WsClientMessage {
  type: 'message';
  content: string;
  stream?: boolean;
}

export interface PaginationQuery {
  limit?: number;
  offset?: number;
}
export interface SystemStats {
  total_sessions: number;
  total_messages: number;
  total_tokens: number;
  db_size_bytes: number;
}
