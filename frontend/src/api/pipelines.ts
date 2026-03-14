import client from './client';

export interface Pipeline {
  id: number;
  name: string;
  definition: any;
  created_at: string;
  updated_at: string;
}

export interface PipelineExecuteResult {
  final_answer: string;
  trace: string[];
  tool_calls: any[];
  intermediate_results: any[];
}

export interface LlmosStatus {
  online: boolean;
  message: string;
}

export const pipelinesApi = {
  list: async (limit = 50, offset = 0): Promise<Pipeline[]> => {
    const response = await client.get(`/pipelines?limit=${limit}&offset=${offset}`);
    return response.data;
  },

  get: async (id: number): Promise<Pipeline> => {
    const response = await client.get(`/pipelines/${id}`);
    return response.data;
  },

  create: async (name: string, definition: any): Promise<Pipeline> => {
    const response = await client.post('/pipelines', { name, definition });
    return response.data;
  },

  update: async (id: number, name: string, definition: any): Promise<Pipeline> => {
    const response = await client.patch(`/pipelines/${id}`, { name, definition });
    return response.data;
  },

  delete: async (id: number): Promise<void> => {
    await client.delete(`/pipelines/${id}`);
  },

  execute: async (id: number, question: string): Promise<PipelineExecuteResult> => {
    const response = await client.post(`/pipelines/${id}/execute`, { question });
    return response.data;
  },

  getLlmosStatus: async (): Promise<LlmosStatus> => {
    const response = await client.get('/llmos/status');
    return response.data;
  }
};
