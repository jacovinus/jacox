---
name: Frontend Development Best Practices
description: Guidelines for frontend development in Stepbit, including package management and testing.
---
# Frontend Development Best Practices

This skill outlines the standards for the Stepbit React/Vite frontend.

## Environment Requirements

- **Node.js**: Always use Node 24. A `.nvmrc` file is provided in the frontend directory.
- **React**: Project is standardized on React 19.
- **Package Manager**: Use `pnpm` (version 10.x).


## Package Management
  - `pnpm add <pkg>`: Add a production dependency.
  - `pnpm add -D <pkg>`: Add a dev dependency.
  - `pnpm install`: Install all dependencies.

## Testing Strategy

- **Framework**: Use `Vitest` for unit and component testing.
- **Library**: Use `React Testing Library` for DOM-based assertions.
- **TDD**: Write tests before implementing new features or fixing bugs.

## Design System
- Use the predefined tokens in `index.css`.
- Prefer `framer-motion` for complex animations.
- Ensure all interactive elements have unique IDs for browser testing.
