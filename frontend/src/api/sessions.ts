import api from './client';
import type { Session, CreateSessionRequest, Message, CreateMessageRequest, PaginationQuery } from '../types';

export const sessionsApi = {
  list: (params?: PaginationQuery) => 
    api.get<Session[]>('/sessions', { params }).then(res => Array.isArray(res.data) ? res.data : []),
    
  create: (data: CreateSessionRequest) => 
    api.post<Session>('/sessions', data).then(res => res.data),
    
  get: (id: string) => 
    api.get<Session>(`/sessions/${id}`).then(res => res.data),

  update: (id: string, data: Partial<CreateSessionRequest>) =>
    api.patch<Session>(`/sessions/${id}`, data).then(res => res.data),
    
  delete: (id: string) => 
    api.delete(`/sessions/${id}`).then(res => res.data),
    
  getMessages: (id: string, params?: PaginationQuery) => 
    api.get<Message[]>(`/sessions/${id}/messages`, { params }).then(res => res.data),
    
  addMessage: (id: string, data: CreateMessageRequest) => 
    api.post<Message>(`/sessions/${id}/messages`, data).then(res => res.data),
    
  export: (id: string) => 
    api.get(`/sessions/${id}/export`, { responseType: 'blob' }).then(res => res.data),
    
  import: (data: string) => 
    api.post<Session>('/sessions/import', data, {
      headers: { 'Content-Type': 'text/plain' }
    }).then(res => res.data),

  getStats: () =>
    api.get<any>('/sessions/stats').then(res => res.data),
};
