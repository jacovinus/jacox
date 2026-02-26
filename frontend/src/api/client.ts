import axios from 'axios';

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
});

api.interceptors.request.use((config) => {
  const apiKey = localStorage.getItem('jacox_api_key') || 'sk-dev-key-123';
  if (apiKey) {
    config.headers.Authorization = `Bearer ${apiKey}`;
  }
  return config;
});

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (!error.response) {
            console.error('Network Error / Backend Unreachable');
        } else if (error.response.status >= 500) {
            console.error('Backend Server Error:', error.response.status);
        }
        return Promise.reject(error);
    }
);

export default api;
