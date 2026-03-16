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
  type: 'message' | 'cancel';
  content: string;
  stream?: boolean;
  search?: boolean;
  reason?: boolean;
}

export interface PaginationQuery {
  limit?: number;
  offset?: number;
}
export interface MemoryUsageEntry {
  tag: string;
  usage_bytes: number;
}

export interface SystemStats {
  total_sessions: number;
  total_messages: number;
  total_tokens: number;
  db_size_bytes: number;
  memory_usage: MemoryUsageEntry[];
}

export interface ProviderInfo {
  id: string;
  active: boolean;
  supported_models: string[];
  status: 'online' | 'offline' | 'unverified';
}

export interface Pipeline {
  id: number;
  name: string;
  definition: {
    stages: Array<{
      stage_type: string;
      config: Record<string, any>;
    }>;
  };
  created_at: string;
  updated_at: string;
}

export interface PipelineExecuteResult {
  final_answer: string;
  trace: string[];
  tool_calls: any[];
  intermediate_results: any[];
}

export interface StepbitCoreStatus {
  online: boolean;
  message: string;
}
