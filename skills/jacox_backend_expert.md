---
description: Expert guidance on Stepbit Backend (Rust/Actix-web) architecture, DuckDB integration, and LLMOS proxying.
---

# Stepbit Backend Expert Skill

Stepbit is a high-performance orchestration server written in Rust, using `actix-web` and `DuckDB`.

## Technical Stack
- **Framework**: `actix-web` for high-throughput HTTP and WebSockets.
- **Database**: `DuckDB` used as an analytical conversation memory store (`chat.db`).
- **Authentication**: Custom `ApiKeyAuth` middleware with support for static keys and rolling LLMOS tokens.

## Architecture & Data Flow
- **Request Proxying**: The backend acts as a gateway for LLMOS, injecting security headers and managing the rolling token handshake.
- **MCP Integration**: Proxies `GET /api/llm/mcp/tools` to LLMOS for tool discovery.
- **Reasoning Execution**: Proxies `POST /api/llm/reasoning/execute` to the LLMOS remote DAG engine.
- **WebSocket Hub**: Manages real-time bidirectional communication (`/ws`) for streaming responses and system status.

## Database Integration
- **DuckDB Service**: Centralized in `src/db/service.rs`.
- **Schema Management**: Dynamic schema initialization and migration.
- **SQL Explorer**: Exposes the `POST /api/query` endpoint for raw SQL execution (restricted to allowed tables).

## Best Practices
- **Conn Pool**: Always use the synchronized `DbPool` (Mutex-wrapped connection) to prevent concurrent access issues with DuckDB.
- **Middleware**: New routes requiring authentication must be added to the `/api` or `/sessions` scopes wrapped by `ApiKeyAuth`.
- **Health Checks**: `/api/health` should reflect both API and Database connectivity status.
- **TDD**: Implement regression tests for new controllers in the `tests/` directory.
