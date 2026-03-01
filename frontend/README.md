# Jacox Frontend: Premium AI Intelligence Interface 🌌

The Jacox frontend is a high-performance, local-first React application designed for seamless interaction with local and cloud LLMs. It prioritizes aesthetic excellence (Monokai-Gruvbox) and technical robustness.

## 🛠 Tech Stack
- **Framework**: React 19 + Vite
- **Styling**: Tailwind CSS 4 (Custom design system)
- **Data Fetching**: @tanstack/react-query
- **Real-time**: WebSocket (actix-ws compatible)
- **Icons**: Lucide React
- **Visualization**: Recharts

---

## 🏗 Architecture Overview

### 1. Streaming Engine (`useChatStream.ts`)
The core of the interaction logic. This hook manages:
- **WebSocket Lifecycle**: Automatic connection/reconnection per session.
- **Buffer Management**: Accumulates partial tokens into fluid messages.
- **Status Signaling**: Handles `status` messages from the server (e.g., "Searching...") and manages `isWaiting`/`isStreaming` states.
- **Process Control**: Provides a `cancel()` function to abort backend tasks instantly.

### 2. Message Rendering (`MarkdownContent.tsx`)
A hardened markdown renderer that transforms raw LLM output into a premium UI:
- **GFM Support**: Full GitHub Flavored Markdown.
- **Code Interceptors**: Automatically detects and renders special block types:
    - **Live SVGs**: Renders SVG code as interactive graphics.
    - **Charts**: Intercepts `role: "chart"` JSON to render visualizations.
- **Raw Toggle**: Users can switch between rendered and raw views at the message level.

### 3. Data Visualization (`ChartComponent.tsx`)
Built with **Recharts**, this component supports advanced data formats:
- **Standard Format**: Label-value pairs.
- **Dense/Bucketed Format**: Supports `values` and `subLabels` arrays for high-resolution temporal data (e.g., hourly prices within a day bucket).
- **Interactive Tooltips**: Custom tooltip logic to handle granular sub-data.

---

## 🎨 Design System
The UI follows a strict **Monokai-Gruvbox Fusion** palette, defined in the project's CSS variables and Tailwind config. Key characteristics:
- **Glassmorphism**: Heavy use of semi-transparent backgrounds and blurs.
- **Premium Gradients**: Subtle aqua and pink accents for interactive elements.
- **Micro-animations**: Pulse effects for "Searching" states and bounce animations for loading bubbles.

---

## 🚀 Development

### Installation
```bash
pnpm install
```

### Run Dev Server
```bash
pnpm dev
```

### Build for Production
```bash
pnpm build
```

---

## 🔌 WebSocket Protocol
Messages exchanged with the backend:

**Outbound (ClientMsg):**
- `type: "message"`: Send a new user prompt.
- `type: "cancel"`: Abort active processing.

**Inbound (ServerMsg):**
- `type: "chunk"`: A text fragment of the response.
- `type: "status"`: A status update (e.g., "Thinking...", "Searching...").
- `type: "done"`: Signaling terminal state of a response.
- `type: "error"`: Error details if a failure occurs.
