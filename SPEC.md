# Jacox - LLM Chat Server with Rust + DuckDB (Final Spec)

A pluggable, local LLM chat server built in Rust with DuckDB for conversation storage. Designed for frontends needing a local-first AI memory layer.

## Architecture Overview

```mermaid
graph TD;
    Client[React/CLI/App] -->|HTTP/REST| API[Jacox API Layer]
    Client -->|WebSocket| WS[Jacox WebSocket Handler]
    
    API --> Auth(Auth Middleware)
    WS --> Auth
    
    Auth --> Router{LlmProvider Factory}
    
    Router -.-> OpenAI[OpenAI API]
    Router -.-> Anthropic[Anthropic API]
    Router -.-> Ollama[Local Ollama]
    
    Router <-->|std::sync::Mutex Pool| DB[(DuckDB - chat.db)]
```

## Configuration

All configuration via `config.yaml`:

```yaml
server:
  host: "127.0.0.1"
  port: 8080

database:
  path: "./chat.db"

auth:
  api_keys:
    - "sk-dev-key-123"
  token_expiry_hours: 24

llm:
  provider: "ollama"  # openai, anthropic, ollama
  model: "llama3.2"
  
  openai:
    api_base: "https://api.openai.com/v1"
    api_key: "${OPENAI_API_KEY}"
    default_model: "gpt-4o"
  
  anthropic:
    api_base: "https://api.anthropic.com"
    api_key: "${ANTHROPIC_API_KEY}"
    default_model: "claude-3-5-sonnet-20241022"
  
  ollama:
    base_url: "http://localhost:11434"
    default_model: "llama3.2"

chat:
  max_history_messages: 50
  system_prompt: "You are a helpful assistant."
```

## Database Schema

```sql
-- Sessions: separates unique conversation threads
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    name STRING,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON DEFAULT '{}'
);

-- Messages: stores history for context injected into LLM
CREATE TABLE messages (
    id BIGINT PRIMARY KEY,
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    role STRING NOT NULL,  -- 'system', 'user', 'assistant'
    content TEXT NOT NULL,
    model STRING,          -- provider model name
    token_count INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON DEFAULT '{}'
);
```

## API Endpoints

### Authentication
Require `Authorization: Bearer <api_key>` header for all endpoints except `/health`.

### REST API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/sessions` | Create session |
| `GET` | `/sessions` | List sessions |
| `GET` | `/sessions/:id` | Get session details |
| `PATCH` | `/sessions/:id` | Update session (name, metadata) |
| `DELETE` | `/sessions/:id` | Delete session |
| `GET` | `/sessions/:id/messages` | Get history |
| `GET` | `/sessions/:id/export` | Export as .txt |
| `POST` | `/sessions/import` | Import from .txt |
| `GET` | `/api/stats` | System-wide statistics |
| `POST` | `/v1/chat/completions` | OpenAI compatible API |

### WebSocket

| Endpoint | Description |
|----------|-------------|
| `/ws/chat/:session_id` | Real-time streaming |

**WS Frame Format:**
- Client: 
    - `{ "type": "message", "content": "hello" }`
    - `{ "type": "cancel" }` - Aborts the active generation/tool task.
- Server: 
    - `{ "type": "chunk", "content": ".." }` -> Streaming content.
    - `{ "type": "status", "content": "Searching..." }` -> UI status pulse.
    - `{ "type": "done" }` -> End of response.
    - `{ "type": "error", "content": "..." }`

## LLM Provider Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, messages: &[Message], options: ChatOptions) -> Result<ChatResponse, LlmError>;
    async fn chat_streaming(&self, messages: &[Message], options: ChatOptions, tx: Sender<String>) -> Result<(), LlmError>;
    fn supported_models(&self) -> Vec<&str>;
}
```

## Frontend Features

The project includes a modern React-based admin dashboard and chat interface with the following capabilities:

### Rich Chat Interface
- **Markdown Rendering**: Full GitHub Flavored Markdown support.
- **Syntax Highlighting**: Code blocks with language detection and premium themes.
- **Table Support**: Formatted markdown tables with responsive scrolling.
- **Image/SVG Rendering**: Support for inline images from URLs and **Live SVG Preview** for code blocks.
- **Interactive Charts**: Responsive Bar/Line charts with "bucketed" high-resolution data support.
- **Thinking Indicators**: Real-time feedback for searching and processing states.
- **Process Control**: Ability to cancel long-running tasks via the UI.
- **Raw/Formatted Toggle**: Instantly switch between rendered markdown and raw plain text.
- **Dynamic Personalities**: Override system prompts per session via JSON metadata.

### Dashboard Statistics
- **Tokens Tracking**: Real-time estimation and reporting of token usage.
- **Byte-Perfect Storage**: Accurate DB size monitoring and display.
- **Smart Formatting**: K/M suffixes for high-volume metrics.
- **Health Checks**: Live status indicators for API and Database connectivity.

## Project Structure

```
src/
├── main.rs          # CLI entry & Actix server boot
├── lib.rs           # Project library modules
├── config/          # YAML Parsing & AppConfig
├── db/              # DuckDB service & connection pool
├── api/             # Actix routes, WebSocket & OpenAI adapter
│   ├── routes.rs    # Core REST endpoints
│   ├── websocket.rs # WebSocket stream engine
│   ├── routes_openai.rs # OpenAI compatibility proxy
│   └── middleware/  # Auth & Logging interceptors
├── llm/             # Provider implementations
└── cli/             # REPL & CLI commands (Clap)
```
