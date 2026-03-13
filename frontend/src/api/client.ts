import axios from 'axios';

const api = axios.create({
  baseURL: (import.meta.env.VITE_API_BASE_URL || '/api')
});

api.interceptors.request.use((config) => {
  // Prefer the rotated token if available for enhanced security
  const rotatedToken = localStorage.getItem('jacox_rotated_token');
  const apiKey = rotatedToken || localStorage.getItem('jacox_api_key') || 'sk-dev-key-123';
  
  if (apiKey) {
    config.headers = config.headers || {};
    config.headers['Authorization'] = `Bearer ${apiKey}`;
  }
  return config;
});

api.interceptors.response.use(
    (response) => {
        // Capture the next token for rotation
        const nextToken = response.headers['x-next-token'];
        if (nextToken) {
            localStorage.setItem('jacox_rotated_token', nextToken);
        }
        return response;
    },
    (error) => {
        // Even on error, the server might have rotated the token
        if (error.response?.headers?.['x-next-token']) {
            localStorage.setItem('jacox_rotated_token', error.response.headers['x-next-token']);
        }

        if (!error.response) {
            console.error('Network Error / Backend Unreachable');
        } else if (error.response.status === 401) {
            // If we get an unauthorized error, the rotated token might be stale.
            // Clear it to trigger a fallback to the static API key in the next request.
            localStorage.removeItem('jacox_rotated_token');
            console.warn('Rotated token stale or invalid. Clearing for fallback.');
        } else if (error.response.status >= 500) {
            console.error('Backend Server Error:', error.response.status);
        }
        return Promise.reject(error);
    }
);

export default api;
