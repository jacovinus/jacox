---
name: Jacox MCP Integration
description: Guidance on integrating the Model Context Protocol (MCP) into the Jacox ecosystem, allowing models to interact with local and remote tools.
---
# Jacox MCP Integration Skill

This skill provides patterns for exposing and managing MCP tools within the Jacox frontend and backend.

## Architecture

1. **LLMOS Proxy**: Jacox acts as a proxy/orchestrator, discovering tools from LLMOS and presenting them to the user.
2. **Frontend Management**: A "Tools" view in Jacox allows users to see registered MCP tools and their capabilities.

## Implementation Patterns

### Backend (Proxying)
Jacox should expose an endpoint `GET /api/llm/mcp/tools` that fetches tool definitions from LLMOS.

### Frontend (Tool Widget)
Create a `ToolCard` component that displays:
- Tool name and description.
- Input schema (JSON Schema).
- A "Test" button to execute the tool directly.

## Best Practices
- **Security**: Always validate that the user has the necessary permissions to call a specific MCP tool.
- **Caching**: Cache tool definitions on the frontend to avoid redundant API calls to LLMOS.
