---
name: Stepbit MCP Integration
description: Guidance on integrating the Model Context Protocol (MCP) into the Stepbit ecosystem, allowing models to interact with local and remote tools.
---
# Stepbit MCP Integration Skill

This skill provides patterns for exposing and managing MCP tools within the Stepbit frontend and backend.

## Architecture

1. **LLMOS Proxy**: Stepbit acts as a proxy/orchestrator, discovering tools from LLMOS and presenting them to the user.
2. **Frontend Management**: A "Tools" view in Stepbit allows users to see registered MCP tools and their capabilities.

## Implementation Patterns

### Backend (Proxying)
Stepbit should expose an endpoint `GET /api/llm/mcp/tools` that fetches tool definitions from LLMOS.

### Frontend (Tool Widget)
Create a `ToolCard` component that displays:
- Tool name and description.
- Input schema (JSON Schema).
- A "Test" button to execute the tool directly.

## Best Practices
- **Security**: Always validate that the user has the necessary permissions to call a specific MCP tool.
- **Caching**: Cache tool definitions on the frontend to avoid redundant API calls to LLMOS.
