# 🧭 Jacox Project Journey

This document serves as the project's roadmap and "travel journal," documenting our future plans, experimental ideas, and long-term vision.

---

## 🗺️ Current Horizon (Upcoming)

### 1. 📂 File System Intelligence
- **Contextual Knowledge**: Allow the agent to "index" specific project directories for faster, more accurate retrieval without full codebase searches.
- **RAG for Local Docs**: Implement a lightweight Retrieval-Augmented Generation layer using DuckDB's FTS5 or an external vec-store for local PDF/Markdown documentation.

### 2. 🔌 Plugin Ecosystem
- **Custom Tool Loading**: Allow users to drop `.rs` or `.js` files into a `plugins/` directory to instantly register new tools in the registry.
- **Webhooks**: Trigger external events (e.g., Slack/Discord notifications) based on chat events.

### 3. 🔐 Secure Encrypted Export/Import
- **Passkey Protection**: Implement AES-GCM or similar encryption for `.chat` exports, allowing users to secure their data with a custom access key.
- **Client-Side Decryption**: Ensure that imported files are only decrypted once the correct key is provided by the user, maintaining zero-knowledge principles.

### 4. 🤖 RLM Emulation System
- **Reward Modeling**: Utilize Rust's performance and DuckDB's analytical capabilities to emulate Reinforcement Learning (RLM) or Reward Model behaviors for response alignment/ranking.
- **Rule-based Logic Engine**: Build a hybrid engine that combines static rules with model-based scoring to ensure high-quality, aligned outputs from local LLMs.

### 5. 🎨 UI/UX Refinement
- **Theming Engine**: Support for custom Monokai-Gruvbox variations (e.g., "Deep Sea," "Rusty Gold").
- **Mobile-Responsive Optimization**: Ensure the dashboard and chat are fully functional on tablets and phones.

---

## 🔭 Future Frontiers (Ideas)

- **Multi-Agent Orchestration**: Allow "Director" agents to spawn "Specialist" agents for complex sub-tasks.
- **Voice Integration**: Local STT (Speech-to-Text) and TTS (Text-to-Speech) using models like Whisper and Piper.
- **Long-term Episodic Memory**: Move beyond session-based history to a persistent "world model" for the user.

---

## 📜 Log of Visions
*Every dream starts with a single line of code.*

- **2026-03-04**: Initialized the Journey. Focused on visual excellence and multi-provider stability.
