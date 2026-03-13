# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-03-04

### Added
- **Dynamic Token Rotation (LLMOS)**: Implemented a cryptographically secure, rolling token handshake between Jacox and LLMOS.
- **Provider Self-Healing**: Added automatic Master Key fallback for LLMOS providers to recover from network-induced token desynchronization.
- **Skills Functionality**: New markdown-based interface for creating, fetching, and managing reusable AI skills.
- **DuckDB Memory Inspection**: Added detailed breakdown of memory usage in the Dashboard with Monokai-styled progress bars.
- **Database Purge**: New CLI command `database purge` and API endpoint `DELETE /sessions` to clear all data and reset sequences.
- **Model Support**: Metadata example for `ministral-3:8b` model.
- **Project Historian Skill**: New standard and skill for maintaining project history and documentation.
- **Dynamic Provider Switching**: Ability to switch between LLM providers (OpenAI, Anthropic, Ollama) directly from the frontend sidebar.
- **GitHub Copilot Integration**: Added support for GitHub Copilot as an LLM provider, including specialized headers and authentication handling.
- **Dynamic Model Selection**: Enabled choosing specific models from the frontend selector and settings page.
- **Dynamic Settings Dashboard**: Completely redesigned Settings page with live provider management, model picking, and connectivity status.
- **Backend Model Management**: Added per-provider active model state to `ProviderManager`.
- **Live CSS & HTML Rendering**: Integrated `rehype-raw` to enable active CSS injection and raw HTML support directly in chat messages.
- **Interactive Playgrounds**: Added a special `live-playground` class for centered demo areas in chat.
- **Real-time Model Header**: Added a model selector directly in the chat header for instant switching.

### Fixed
- **SQL Error**: Corrected DuckDB memory query column name from `memory_usage` to `memory_usage_bytes`.
- **Frontend Types**: Fixed TypeScript implicitly 'any' errors in `Dashboard.tsx` and improved type safety for memory stats.
- **State Management**: Fixed `get_active_provider_info` to return correct provider IDs instead of "provider-manager".
- **UI Synchronization**: Aligned polling intervals between Sidebar and Chat (5s) and fixed persistent "Loading..." states with backend default fallbacks.

### Changed
- **Database Schema Access**: Made `SCHEMA` string public in `connection.rs` to allow clean-slate re-initialization during purge.
- **Universal Provider Trait**: Added `default_model` method to `LlmProvider` trait for consistent UI synchronization.
- **Agent Instructions**: Updated `design-system` skill, `frontend-expert` skill, and core `system_prompt` to include Live CSS usage guidelines and visual interactivity standards.
