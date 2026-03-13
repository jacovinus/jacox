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
