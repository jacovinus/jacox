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

---

## 📚 Skills

A **Skill** is a named markdown snippet stored in the local database. Skills are copy-paste-ready and can be imported from any public URL.

All endpoints require `Authorization: Bearer <API_KEY>`.

### `GET /api/skills`
List all skills with pagination.
- **Query**: `?limit=50&offset=0`
- **Response**: `200 OK` — JSON array of Skill objects.

### `POST /api/skills`
Create a new skill.
- **Body**:
  ```json
  {
    "name": "Senior Code Reviewer",
    "content": "You are a senior engineer…",
    "tags": "code, review",
    "source_url": null
  }
  ```
- **Response**: `201 Created` with the new Skill object.

### `GET /api/skills/:id`
Get a single skill by its numeric ID.
- **Response**: `200 OK` with Skill object, or `404 Not Found`.

### `PATCH /api/skills/:id`
Partially update a skill. All fields are optional.
- **Body**: `{ "name"?: "…", "content"?: "…", "tags"?: "…" }`
- **Response**: `200 OK` with updated Skill object.

### `DELETE /api/skills/:id`
Permanently delete a skill.
- **Response**: `204 No Content`.

### `POST /api/skills/fetch-url`
Fetch content from a public URL and store it as a skill in one step.
- **Body**:
  ```json
  {
    "url": "https://raw.githubusercontent.com/sindresorhus/awesome/main/readme.md",
    "name": "Awesome README",
    "tags": "reference"
  }
  ```
- **Response**: `201 Created` with the new Skill object.

> `text/html` pages are tag-stripped to plain text. `text/plain` and `text/markdown` are stored as-is.

#### Skill Object Schema
```json
{
  "id": 1,
  "name": "Senior Code Reviewer",
  "content": "You are a senior engineer…",
  "tags": "code, review",
  "source_url": null,
  "created_at": "2026-03-13T15:00:00Z",
  "updated_at": "2026-03-13T15:00:00Z"
}
```
