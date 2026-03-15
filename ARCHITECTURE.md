# 🏗️ Jacox Architecture

Jacox is a high-performance LLM orchestration platform designed with a modular, pluggable architecture. It bridges the gap between raw LLM capabilities and premium, user-centric AI applications.

---

## 🛰️ System Overview

Jacox follows a modern client-server architecture:

- **Backend (Rust)**: Built with `actix-web`, it handles heavy lifting, database orchestration, and secure communication with LLM providers.
- **Reasoning Core (Local LLMOS)**: Jacox is designed to run alongside `jac_llmos`, providing a "Local-First" cognitive stack for complex problem-solving.
- **Frontend (React 19)**: A high-fidelity Dashboard and Chat interface built with Vite and Tailwind CSS.

---

## 🧩 Core Components

### 1. API Layer (`src/api/`)
Handles the communication between the frontend and the core engine.
- **REST Endpoints**: CRUD operations for sessions, messages, skills, and pipelines.
- **WebSocket Engine**: Real-time streaming of tokens and reasoning traces with cancellation support.
- **OpenAI Proxy**: Provides a `/v1/chat/completions` endpoint for compatibility with OpenAI-compatible tools.

### 2. LLM Engine (`src/llm/`)
A pluggable provider system that abstracts the complexity of different AI backends:
- **Core (Local Reasoning)**: `jac_llmos` integration. This is the **primary, mission-critical connection** that enables the Pipelines Hub, Reasoning Graphs, and advanced MCP tools.
- **Local Fallbacks**: Ollama for standard chat tasks.
- **Cloud Providers**: OpenAI, Anthropic, GitHub Copilot.

### 3. Data Layer (`src/db/`)
Powered by **DuckDB**, a high-performance analytical database.
- **Local Memory**: Persistent storage for all conversations in `chat.db`.
- **Analytical Snapshotting**: Allows `jac_llmos` to perform non-blocking analysis on live data by attaching read-only snapshots.

### 4. Reasoning Engine & Pipelines (`src/llm/llmos.rs`)
The "Cognitive Core" of Jacox:
- **DAG Executor**: A Directed Acyclic Graph architecture that manages node lifecycle and data flow.
- **Cognitive Pipelines**: Deterministic, multi-stage workflows (Search -> Analyze -> Synthesize).
- **Variable Resolution**: Supports `{{node_id.output}}` syntax for chaining outputs between nodes.

---

## 🔄 Data Flow

```mermaid
sequenceDiagram
    participant User as 👤 User
    participant Jacox as 🦀 Jacox (8080)
    participant DB as 🦆 DuckDB (chat.db)
    participant LLMOS as 🧠 LLMOS (8081)
    
    User->>Jacox: Ask "Analyze my history"
    Jacox->>DB: Fetch Session Context
    Jacox->>LLMOS: Trigger Pipeline Request
    Note over LLMOS: Detects DB Lock
    LLMOS->>DB: Attach Snapshot (Read-Only)
    LLMOS->>LLMOS: Execute reasoning stages
    LLMOS-->>Jacox: SSE Stream Results
    Jacox-->>User: Visual Reasoning Trace
```

---

## 🛡️ Security Architecture

### Rolling Handshake
Internal communication between Jacox and LLMOS is protected by a rotating token mechanism:
1. **Initial Handshake**: Jacox uses the master `API_KEY`.
2. **Token Rotation**: LLMOS returns a new `X-Next-Token` in every response header.
3. **Chained Authentication**: The next request must use the new token, preventing replay attacks.

---

## 🎨 Visualization Layer
Jacox interprets specific structured outputs from LLMs to render:
- **Interactive Charts**: Line, Bar, and Area charts for data trends.
- **Live SVGs**: Dynamic graphic generation.
- **Code Previews**: Live HTML/CSS rendering for UI experimentation.

---

Built with performance and aesthetics as first-class citizens.
