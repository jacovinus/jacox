---
description: Expert guidance on Stepbit Frontend (React/Vite) architecture, design system, and real-time visualization.
---

# Stepbit Frontend Expert Skill

The Stepbit frontend is a premium, high-performance React application designed for AI orchestration.

## Tech Stack
- **Core**: React 19, Vite 6, TypeScript.
- **Styling**: Tailwind CSS 4 with a custom Monokai-Gruvbox theme.
- **Animations**: `framer-motion` for fluid graph and UI transitions.
- **State & Data**: `@tanstack/react-query` for API state, custom hooks for WebSockets.

## Component Architecture
- **Reasoning Playground**: Uses `framer-motion` and custom SVG logic to render interactive reasoning graphs.
- **MCP Tool Hub**: Dynamic interface for exploring tool schemas and capabilities.
- **SQL Explorer**: Integrated `CodeMirror` with SQL syntax highlighting for DuckDB interaction.
- **Markdown Core**: Enhanced with `rehype-raw` to support live CSS/HTML injection from LLM responses.

## Connection & Security
- **API Client**: Uses standard `axios` with relative path resolution to support direct connection to the backend.
- **Authentication**: Automatically manages the stepbit-core rolling token via `localStorage` and `x-next-token` interception.
- **Environment**: Requires Node.js 24+ and `pnpm` 10+ for development.

## UX Principles
- **Aesthetic Excellence**: Strictly adheres to the dark-mode aesthetic with semi-transparent elements and aqua-pink highlights.
- **Responsiveness**: All interactive elements target 60FPS animations.
- **Feedback**: Real-time "Thinking", "Searching", and "Executing" states are mandatory for all long-running tasks.

## Integration Checklist
1. Ensure `VITE_API_BASE_URL` points to the correct backend IP (default `http://127.0.0.1:8080/api`).
2. Use **relative paths** in API calls (e.g., `api.get('sessions')`) to ensure they append correctly to the baseURL.
3. Leverage `useChatStream` for all high-fidelity chat interactions.
