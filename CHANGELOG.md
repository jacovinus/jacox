# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-03-15

### Added
- **SSE Streaming Reasoning**: Implemented end-to-end Server-Sent Events (SSE) support for the Reasoning Playground.
- **Enhanced Execution Log**: Replaced the small results grid with a vertical, scrollable execution log for high-bandwidth feedback.
- **Sidebar Revamp**: Expanded the Node Inspector to 450px and added a dedicated formatting viewer for execution results (JSON/Text).
- **Stream Proxy**: Added an Actix-web proxy to bridge SSE requests from the frontend to the `jac_llmos` engine.

### Fixed
- **Trace Visibility**: Resolved issues where large reasoning outputs were truncated in the UI.
- **Node Selection UX**: Optimized the inspector layout to show status, configuration, and results in a structured single-column view.

## [0.3.1] - 2026-03-15

### Added
- **Interactive Reasoning Playground**: Completely overhauled the placeholder component into a production-ready graph editor.
- **Draggable DAG Canvas**: Added a high-performance SVG-based canvas with `framer-motion` for drag-and-drop node orchestration.
- **Dynamic Node Editor**: Integrated a sidebar with live JSON payload editing and connection management.
- **Visual Execution Trace**: Real-time status indicators (pending/running/success) at the node level during execution.

### Fixed
- **Variable Resolution**: Frontend now correctly supports `{{node_id.output}}` syntax for chaining reasoning outputs.
- **API Synchronization**: Aligned the playground with the latest `jac_llmos` Reasoning Engine specification.

## [0.3.0] - 2026-03-14

### Added
- **Cognitive Pipelines Integration**: Full end-to-end support for multi-stage reasoning workflows.
- **DuckDB Snapshotting**: Mandatory snapshotting policy for `jac_llmos` ensures non-blocking analytical access to `chat.db` while `stepbit` is active.
- **Pluggable Architecture**: Implemented `jac_llmos` as an optional, pluggable tool. The UI now gracefully handles service connectivity states.
- **Pipeline Hub (Frontend)**: A new premium page for managing pipelines with real-time connectivity and trace viewing.
- **Transparent Views**: `jac_llmos` now automatically creates views for `stepbit` tables (`messages`, `sessions`, `pipelines`) in the `main` schema for simplified querying.
- **Enhanced Authentication**: Support for rotating tokens (`X-Next-Token`) and `X-API-Key` headers.

### Changed
- **UTF-8 Safety**: Refactored `LlmEngine` to use a byte buffer for streaming, preventing corruption of multibyte characters (emojis, etc.) in pipeline results.
- **Proactive Model Loading**: `jac_llmos` now pre-loads models specified in pipeline stages to reduce cold-start latency.

### Fixed
- **Detailed Execution Reporting**: The Pipeline Execution Modal now extracts the full backend error trace.
- **DuckDB Lock Contention**: Resolved via the new snapshot-fallback attachment mechanism.
- **Middleware Compatibility**: Resolved issues with Bearer token vs X-API-Key resolution in Actix-web.

## [0.2.0] - 2026-03-13

### Added
- **MCP Tool Registry**: New dedicated page for exploring registered Model Context Protocol tools and their input schemas.
- **Reasoning Playground**: Interactive, animated graph visualization based on `framer-motion` for building and executing multi-step reasoning tasks.
- **LLMOS Remote Engine**: Full integration with the LLMOS Remote Reasoning Engine, supporting parallel node execution and state resolution.
- **Node 24 & React 19**: Standardized the entire frontend repository on Node.js 24 and React 19 for improved performance and security.
- **Automated Stack Runner**: Enhanced `run_stack.sh` with Node version enforcement (`nvm`), robust health checks, and improved logging.

### Fixed
- **API Connectivity**: Standardized all frontend API calls to use relative paths, resolving URL resolution issues when running without a proxy.
- **Auth Resilience**: Correctly exempted health check and root paths in the backend authentication middleware.
- **Sidebar UX**: Integrated MCP and Reasoning features into the primary sidebar navigation.

## [0.1.0] - 2026-03-04

### Fixed
- **SQL Error**: Corrected DuckDB memory query column name from `memory_usage` to `memory_usage_bytes`.
- **Frontend Types**: Fixed TypeScript implicitly 'any' errors in `Dashboard.tsx` and improved type safety for memory stats.
- **State Management**: Fixed `get_active_provider_info` to return correct provider IDs instead of "provider-manager".
- **UI Synchronization**: Aligned polling intervals between Sidebar and Chat (5s) and fixed persistent "Loading..." states with backend default fallbacks.

### Changed
- **Database Schema Access**: Made `SCHEMA` string public in `connection.rs` to allow clean-slate re-initialization during purge.
- **Universal Provider Trait**: Added `default_model` method to `LlmProvider` trait for consistent UI synchronization.
- **Agent Instructions**: Updated `design-system` skill, `frontend-expert` skill, and core `system_prompt` to include Live CSS usage guidelines and visual interactivity standards.
