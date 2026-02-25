# Jacox: Local-First LLM Chat Server 🚀

**Jacox** is a lightweight LLM chat server built in **Rust** with **DuckDB** for lightning-fast conversation memory. It acts as an optimized bridge between your frontends and Large Language Models (OpenAI, Anthropic, or local Ollama).

## ✨ Features
- **Pluggable LLMs**: Switch providers (OpenAI, Anthropic, Ollama) via YAML.
- **Embedded Memory**: All chat history stored locally in DuckDB (`chat.db`).
- **Interactive Playground**: Built-in visual dashboard at `/playground`.
- **Session Portability**: Export/Import chat histories via `.txt` (CLI & REST).
- **Deployment Ready**: Docker Compose, Standalone Binary, or Static Linking support.

> [!IMPORTANT]
> **Check out the [Features & Usage Guide](FEATURES_GUIDE.md) for use cases, and the [Deployment Guide](DEPLOYMENT_GUIDE.md) for production setup.**

## 🚀 Quick Start

### 1. Installation
Ensure you have the Rust toolchain installed.
```bash
git clone https://github.com/jacovinus/jacox.git
cd jacox
cargo build --release
```

### 2. Configuration
Copy `config.yaml` and add your API keys.
```yaml
llm:
  provider: "ollama" # or "openai", "anthropic"
  model: "llama3.2"
  openai:
    api_key: "sk-..."
```

### 3. Running the Server
```bash
cargo run -- serve
```

## 🛠 Usage

### CLI Interface
```bash
# List chat sessions
cargo run -- session list

# Create a new session
cargo run -- session create --name "Research Project"

# Chat in the terminal (Interactive)
cargo run -- chat --session <UUID>

# Export/Import (Portability)
cargo run -- session export <UUID>
cargo run -- session import --path my_chat.txt
```

### OpenAI Compatibility
Point any OpenAI client to Jacox:
```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
  }'
```

### Deployment
For production or portable use:
- **Docker**: `docker compose up -d`
- **Standalone**: Run `./package.sh` to generate a `dist/` bundle.
- **Guide**: See [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) for details.

## 🏗 Architecture
Jacox uses **DuckDB** for analytical storage of chat messages, allowing for complex RAG-like queries in the future. The **Actix-web** layer ensures high concurrency with minimal CPU overhead.

---
Built with 🦀 by the Jacox team.
