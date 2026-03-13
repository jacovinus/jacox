import api from './client';

export interface Skill {
  id: number;
  name: string;
  content: string;
  tags: string;
  source_url?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateSkillRequest {
  name: string;
  content: string;
  tags?: string;
  source_url?: string;
}

export interface UpdateSkillRequest {
  name?: string;
  content?: string;
  tags?: string;
}

export interface FetchUrlRequest {
  url: string;
  name: string;
  tags?: string;
}

export const skillsApi = {
  list: (limit = 50, offset = 0) =>
    api.get<Skill[]>('skills', { params: { limit, offset } }).then(res =>
      Array.isArray(res.data) ? res.data : []
    ),

  get: (id: number) =>
    api.get<Skill>(`skills/${id}`).then(res => res.data),

  create: (data: CreateSkillRequest) =>
    api.post<Skill>('skills', data).then(res => res.data),

  update: (id: number, data: UpdateSkillRequest) =>
    api.patch<Skill>(`skills/${id}`, data).then(res => res.data),

  delete: (id: number) =>
    api.delete(`skills/${id}`),

  fetchUrl: (data: FetchUrlRequest) =>
    api.post<Skill>('skills/fetch-url', data).then(res => res.data),
};
