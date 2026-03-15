import api from './client';

export interface McpTool {
  name: string;
  description: string;
  input_schema: any;
}

export const getMcpTools = async (): Promise<McpTool[]> => {
  const response = await api.get('llm/mcp/tools');
  return response.data;
};

export const executeReasoning = async (graph: any): Promise<any> => {
  const response = await api.post('llm/reasoning/execute', graph);
  return response.data;
};

export const executeReasoningStream = async (graph: any, onEvent: (event: any) => void): Promise<void> => {
  const response = await fetch(`${import.meta.env.VITE_API_BASE_URL || '/api'}/llm/reasoning/execute/stream`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${localStorage.getItem('jacox_api_key') || 'sk-dev-key-123'}`
    },
    body: JSON.stringify(graph)
  });

  if (!response.body) return;
  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n');
    
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        try {
          const data = JSON.parse(line.slice(6));
          onEvent(data);
        } catch (e) {
          console.error('Error parsing SSE event:', e);
        }
      }
    }
  }
};
