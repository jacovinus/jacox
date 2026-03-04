# Jacox API Guide 🛠️

This guide documents the REST and WebSocket endpoints available in the Jacox LLM Server. All REST endpoints require an `Authorization: Bearer <API_KEY>` header (default: `sk-dev-key-123`).

---

## 🏗️ Session Management
Manage conversation sessions and their metadata.

### `POST /sessions`
Create a new chat session.
- **Body**: `{ "name": "Project Research", "metadata": {} }`
- **Response**: `201 Created` with Session object.

### `GET /sessions`
List all sessions with pagination.
- **Query**: `?limit=20&offset=0`
- **Response**: `200 OK` with JSON array of sessions.

### `GET /sessions/{id}`
Retrieve details for a specific session.

### `PATCH /sessions/{id}`
Update session name or metadata.

### `DELETE /sessions/{id}`
Delete a specific session and all its messages.

### `DELETE /sessions`
**Purge All Data**: Instantly clears all sessions and messages from the database.

---

## 💬 Messages & Chat
Interact with models within a session.

### `POST /sessions/{id}/messages`
Add a message and trigger an LLM completion (Synchronous).
- **Body**: 
  ```json
  {
    "role": "user",
    "content": "Hello!",
    "model": "mistral" (optional),
    "metadata": {} (optional)
  }
  ```
- **Response**: `201 Created` with the assistant's response message.

### `GET /sessions/{id}/messages`
Retrieve message history for a session.
- **Query**: `?limit=50&offset=0`

### `GET /sessions/{id}/export`
Export the entire session history as a plain-text file.

### `POST /sessions/import`
Import a session from a raw text body (matching the export format).

---

## ⚡ WebSocket Streaming
Real-time, token-by-token streaming with tool support.

### `WS /ws/chat/{session_id}`
Establish a persistent connection for interactive chat.

**Client Messages**:
- **Message**: `{ "type": "message", "content": "Hello!", "search": true, "reason": true }`
- **Cancel**: `{ "type": "cancel" }` - Aborts the current generation/search.

**Server Messages**:
- **Status**: `{ "type": "status", "content": "Searching: internet_search..." }`
- **Chunk**: `{ "type": "chunk", "content": "To..." }`
- **Done**: `{ "type": "done" }`
- **Error**: `{ "type": "error", "content": "..." }`

---

## ⚙️ Configuration & Providers
Manage LLM providers and active models.

### `GET /config/providers`
List all configured providers and their status.

### `GET /config/active-provider`
Get detailed info about the currently active provider, including supported and active models.

### `POST /config/active-provider`
Switch the active provider.
- **Body**: `{ "provider_id": "openai" }`

### `POST /config/active-model`
Switch the active model for the current provider.
- **Body**: `{ "model_id": "gpt-4o" }`

---

## 📊 Analytics & Health

### `GET /sessions/stats`
Retrieve DuckDB storage statistics and memory breakdown.

### `GET /health`
Check API and Database connectivity.
- **Response**: `{ "status": "healthy", "api": "connected", "database": "connected" }`

---

## 🔄 OpenAI Compatibility
Jacox provides a drop-in 1:1 replacement for OpenAI's Chat Completions.
- **Endpoint**: `POST /v1/chat/completions`
- **Compatibility**: Supports `model`, `messages`, `tools`, and `stream: false`.
