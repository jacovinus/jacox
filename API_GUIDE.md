# Stepbit API Guide 🛠️

This guide documents the REST and WebSocket endpoints available in the Stepbit LLM Server. 

---

## 🏗️ Authentication & Security

Stepbit uses a hybrid authentication model to balance ease of use with enterprise-grade security.

### 1. Bearer Token (Standard)
Most REST endpoints require an `Authorization: Bearer <API_KEY>` header.
- **Default Key**: `sk-dev-key-123` (Change this in `config.yaml`).

### 2. Chained Handshake (Remote stepbit-core)
When communicating with a remote `stepbit-core` instance, Stepbit uses a rotating token mechanism.
- **Header**: `X-API-Key` is used for the initial handshake.
- **Rotation**: stepbit-core provides a `X-Next-Token` in the response header. 
- **Next Request**: Stepbit must use that specific token.
- **Verification**: This prevents replay attacks and ensures zero-trust connectivity.

---

## 🧠 Cognitive Pipelines API

Execute structured reasoning workflows programmatically.

### `POST /api/pipelines/:id/execute`
Executes a pre-defined pipeline by its database ID.
- **Headers**: `Content-Type: application/json`
- **Body**: 
  ```json
  {
    "question": "What are the top 3 sessions with the most messages?",
    "rlm_enabled": false
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "final_answer": "The top 3 sessions are...",
    "trace": ["McpToolStage: executed query", "SynthesisStage: compiled answer"],
    "tool_calls": [...]
  }
  ```

---

## 🏗️ Reasoning Graph API

Build and execute ad-hoc reasoning chains using a Directed Acyclic Graph (DAG).

### `POST /v1/reasoning/execute`
Executes a graph and waits for completion (Blocking).

### `POST /v1/reasoning/execute/stream` (Recommended)
Executes a graph and streams lifecycle events via **Server-Sent Events (SSE)**.
- **Events**:
  - `node_started`: `{"type": "node_started", "node_id": "..."}`
  - `node_completed`: `{"type": "node_completed", "node_id": "...", "result": {...}}`
  - `error`: `{"type": "error", "error": "..."}`
- **Key Features**: 
  - **Template Substitution**: Use `{{node_id.output}}` to link data flows.
  - **Parallel Execution**: Independent nodes execute concurrently.
  - **Fallback Simulation**: Gracefully returns mock results if no LLM engine is loaded.

---

---

## 💬 Chat & Streaming

### `WS /ws/chat/{session_id}`
Real-time, bidirectional communication.

**Client JSON**:
```json
{
  "type": "message",
  "content": "Analyze my DB usage",
  "search": true
}
```

**Server Enums**:
- `chunk`: A single token of text.
- `status`: A human-readable action (e.g. "Analyzing...").
- `trace`: A reasoning step from a pipeline.
- `done`: Signal for stream termination.

---

## ⚙️ Configuration

### `GET /config/active-provider`
Returns the current LLM orchestration state.
```json
{
  "provider": "ollama",
  "model": "mistral:latest",
  "status": "online"
}
```

---

## 📈 System Health

### `GET /health`
Verifies that the Rust kernel and DuckDB engine are active.
```json
{
  "status": "healthy",
  "api": "connected",
  "database": "connected",
  "stepbit-core": "connected"
}
```

For full request/response schemas, refer to the technical documentation within the `docs/` folder.
