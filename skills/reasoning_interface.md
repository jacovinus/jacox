---
name: Stepbit Reasoning Interface
description: Guidelines for building the UI/UX for the Reasoning Graph Engine, including visualization and interaction patterns.
---
# Stepbit Reasoning Interface Skill

This skill focuses on the frontend presentation of the Reasoning Graph.

## Visualization Patterns

1. **Graph View**: Use a library like `react-flow` or a custom SVG-based renderer to show the DAG.
    - **Nodes**: Colored by type (`LlmGeneration` = blue, `McpToolCall` = green, etc.).
    - **Edges**: Directed arrows showing execution flow.
2. **Live Execution**: Highlight the active node(s) during execution.
3. **Node Inspector**: Clicking a node shows its payload, input context, and execution results.

## Execution Workflow

1. **Builder**: A drag-and-drop or checklist-based UI to construct a `ReasoningGraph`.
2. **Execution Sidebar**: Shows logs and status of each node in real-time.

## UI/UX Rules
- **Feedback**: Always show a loading state for in-progress nodes.
- **Errors**: Distinctly highlight failed nodes and prevent execution of child nodes visually.
