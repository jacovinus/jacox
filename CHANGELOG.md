# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
