---
description: Expert guidance on stepbit-core (Lightweight Local LLM Operating System) internals, API protocols, and reasoning engine.
---

# stepbit-core Expert Skill

stepbit-core is a lightweight, high-performance LLM operating system with a focus on local inference, observability, and security.

## Core Architecture
- **Kernel**: Orchestrates components (Scheduler, Inference Engine, MCP, Reasoning).
- **Scheduler**: Token-based with High/Normal/Background priority and dynamic budgets.
- **Inference**: Integrates `llama.cpp` with continuous batching support.
- **Reasoning Engine**: DAG-based engine in `src/reasoning` for multi-step graph execution.

## API Protocols
- **OpenAI Compatible**: `POST /v1/chat/completions`, `GET /v1/models`.
- **Chained Security**: Uses a rotating `X-Next-Token` handshake. Each successful response provides the token for the next request.
- **MCP Discovery**: `GET /v1/mcp/tools` lists available tools and their JSON schemas.
- **Remote Reasoning**: `POST /v1/reasoning/execute` accepts a `ReasoningGraph` (nodes and edges) and executes it in parallel.

## Development Patterns
- **TDD Requirement**: All new kernel or API features must be verified with integration tests in `tests/`.
- **Error Handling**: Uses `StatusCode` and custom `Error` types with structured logging.
- **Metrics**: Exposed at `/metrics` for Prometheus.
- **Health Checks**: `/health` (liveness) and `/ready` (ready to infer) are exempted from auth.

## Integration Guide
When connecting to stepbit-core:
1. Initialize connection using the `LLMOS_API_KEY` (default `sk-dev-key-123`).
2. Capture `x-next-token` from every response and use it for the next `Authorization: Bearer <token>` header.
3. Use `/v1/mcp/tools` to verify availability of specialized engines like DuckDB.
