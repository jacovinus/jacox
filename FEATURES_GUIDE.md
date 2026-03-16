# Stepbit: Features & Usage Guide 📘

This guide provides a deep-dive into every capability of the Stepbit LLM Server, including real-world examples and architectural use cases.

---

## 1. Multi-Provider LLM Engine 🧠
Stepbit abstracts away the differences between various LLM providers. You can switch between cloud (OpenAI, Anthropic) and local (Ollama) models with a single click or config change.

### 🛠 Configuration Example
```yaml
llm:
  provider: "ollama"  # Change to "openai", "anthropic", or "copilot"
  model: "mistral"
```

### 🎯 Step-by-Step: Switching Providers
1. Open the **Sidebar**.
2. Click on the current provider name at the bottom.
3. Select your desired provider from the list.
4. The dashboard will instantly update to show available models for that provider.

---

## 2. Local Memory Layer (DuckDB) 🦆
Every conversation is persisted in a high-performance, local DuckDB database (`chat.db`). This provides instant history retrieval and analytical capabilities.

### 🛠 CLI Example
```bash
# List all saved sessions from the local database
cargo run -- session list
```

### 🎯 Use Case: Analytical Insights
Since data is in DuckDB, you can run complex SQL queries over your chat history to find patterns, common topics, or token usage trends.

---

## 3. Real-time WebSocket Streaming ⚡
Stepbit supports token-by-token streaming via WebSockets, providing a fluid "typing" effect.

### 🎯 Live Status Indicators
During a stream, you will see real-time status updates:
- `Thinking...`: The model is generating a response.
- `Searching: <tool>...`: The model is engaging an MCP tool.
- `Finalizing...`: Compiling the final answer.

---

## 4. Internal Service Security (Rolling Tokens) 🛡️
When communicating with **stepbit-core**, Stepbit implements a **Chained Request Security** handshake. After every successful request, stepbit-core rotates its internal token and provides a new one via the `X-Next-Token` header.

### 🛠 Handshake Mechanics
1. **Initial**: Stepbit uses the Master API Key.
2. **Handshake**: stepbit-core validates and sends back a `X-Next-Token`.
3. **Chain**: Stepbit uses that specific token for the next request.
4. **Safety**: If the chain breaks, it automatically re-syncs using the Master Key.

---

## 5. Session Portability (Import/Export) 📥📤
Export your history to human-readable `.txt` files and import them anywhere.

### 🛠 How to Export/Import
- **Export**: Click the download icon in the chat header.
- **Import**: Click the upload icon in the Sidebar's session list and select your `.txt` file.

---

## 6. Cognitive Pipelines (The Reasoning Hub) 🧠
Pipelines transform LLMs from simple chat bots into structured reasoning agents.

### 🎯 Tutorial: Executing your first Pipeline
1. Navigate to the **Pipelines** tab.
2. Click **Execute** on a pipeline card.
3. Observe the **Reasoning Trace Viewer**: It shows each stage (McpTool -> Verification -> Synthesis) as it happens.
4. Review the **Final Answer**: A structured response based on the pipeline's logic.

---

## 7. Internet Search Tool 🌐
Give local models access to the real world. Stepbit uses a custom scraper to fetch grounding data.

### 🛠 Example Prompt
"Who won the match yesterday?" -> Stepbit will automatically perform an `internet_search` to find the latest news.

---

## 8. High-Fidelity Data Visualization (Charts) 📊
Stepbit renders interactive charts directly in chat. If the model detects data trends, it will output a JSON block that Stepbit transforms into a premium visualization.

### 🎯 Supported Types
- **Line Charts**: Ideal for temporal data.
- **Bar Charts**: Best for comparisons.
- **Horizontal Bars**: Used for detailed metrics like DuckDB memory profiling.

---

## 9. Skills Library 📚
A persistent library of reusable prompts. Use it to store personas like "Expert Coder" or "SQL Analyst".

### 🎯 Step-by-Step: Importing a Persona
1. Go to **Skills** tab.
2. Click **Import from URL**.
3. Paste: `https://raw.githubusercontent.com/jacovinus/stepbit/master/skills/coding_expert.md`
4. The persona is now saved. Click **Copy** and paste it into any chat.

---

## 10. Live Data Analyst (Snapshot Mode) 📸
stepbit-core can analyze your active `chat.db` without causing locks or latency in your chat sessions.

### 🛠 How it works
1. You trigger a pipeline that requires DB access.
2. stepbit-core detects the lock and creates a **Temporary Snapshot**.
3. It attaches this snapshot as a `READ_ONLY` database.
4. The pipeline performs the analysis and reports back, ensuring zero downtime for the main application.

---

## 11. Reasoning Playground (Advanced DAG) 🛰️
The Reasoning Playground is a high-fidelity editor for building ad-hoc AI agents. Unlike the deterministic pipelines, the playground allows for free-form graph sketching.

### 🎯 Key Interactive Features
- **Draggable Canvas**: Use the SVG-based board to position your reasoning nodes (LLM, MCP, DB Query).
- **Dynamic Connections**: Click and drag between nodes to establish data flow.
- **Variable Injection**: Any node can reference another via `{{node_id.output}}`.
- **Live Execution Feedback**: Watch nodes light up in **Orange (Running)**, **Green (Success)**, or **Pink (Error)** as the backend executor traverses the graph.
- **SSE Streaming**: Full implementation of Server-Sent Events ensures that you see the results of each node as they happen, without waiting for the entire graph to complete.
- **Enhanced Execution Log**: A vertical, scrollable log provides high-bandwidth feedback for complex outputs.
- **Node Inspector & Formatted Results**: A dedicated 450px sidebar allows for deep inspection of JSON or text results with syntax-aware formatting.

---

## 12. Pluggable Infrastructure 🔌
Stepbit is designed to work with or without `stepbit-core`. 
- **Standalone**: All standard chat and search features work.
- **Integrated**: Connect `stepbit-core` to unlock the **Pipelines Hub**, **Reasoning Graphs**, and **Advanced MCP tools**.

Built with 🦀 and 🎨 for superior AI orchestration.
