# Jacox: Features & Usage Guide 📘

This guide provides a deep-dive into every capability of the Jacox LLM Server, including real-world examples and architectural use cases.

---

## 1. Multi-Provider LLM Engine 🧠
Jacox abstracts away the differences between various LLM providers, allowing you to switch between cloud (OpenAI, Anthropic) and local (Ollama) models with a single config change.

### 🛠 Configuration Example
```yaml
llm:
  provider: "ollama"  # Just change this to "openai" or "anthropic"
  model: "mistral"
```

### 🎯 Use Case: Cost Optimization
Develop your application using a local model (Ollama + Mistral) to save costs. Once ready for production, switch to `gpt-4o` or `claude-3-5-sonnet` by simply updating the `provider` and adding your API key to `.env`.

---

## 2. Local Memory Layer (DuckDB) 🦆
Every conversation is persisted in a high-performance, local DuckDB database (`chat.db`). This provides instant history retrieval without the latency or privacy concerns of external vector stores.

### 🛠 CLI Example
```bash
# List all saved sessions from the local database
cargo run -- session list
```

### 🎯 Use Case: Privacy-First Personal Assistant
Build a chat application where data never leaves the user's machine unless it's sent to the LLM. The entire conversation history stays in `chat.db`, allowing for offline indexing and long-term memory.

---

## 3. Real-time WebSocket Streaming ⚡
Jacox supports token-by-token streaming via WebSockets. This provides a "live" feel similar to ChatGPT's interface.

### 🛠 Client Implementation (JS)
```javascript
const ws = new WebSocket("ws://localhost:8080/ws/chat/<SESSION_ID>?api_key=sk-dev-key-123");

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.type === "chunk") {
        console.log("Token:", data.content);
    }
};
```

### 🎯 Use Case: Real-time IDE Extensions
Integrate Jacox into a VS Code extension where the AI "types" its solution as it thinks, providing immediate feedback to the developer.

---

## 4. OpenAI Compatibility Proxy 🔄
Jacox acts as a drop-in proxy. Any application built for the OpenAI API can point to Jacox to use Ollama or Anthropic instead.

### 🛠 Integration Example (Python)
```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="sk-dev-key-123"
)

response = client.chat.completions.create(
  model="mistral", # This maps to your config.yaml provider
  messages=[{"role": "user", "content": "How are you?"}]
)
```

### 🎯 Use Case: Legacy Tool Modernization
Take an older tool that only supports OpenAI and "modernize" it to work with local Open Source models (Llama 3, Mistral) by updating the `base_url`.

### 🛡️ Internal Service Security (Rolling Tokens)
When communicating with **LLMOS**, Jacox implements a **Chained Request Security** handshake. After every successful request, LLMOS rotates its internal token and provides a new one to Jacox via the `X-Next-Token` header. This ensures that:
1.  **Replay Attacks** are impossible (sessions tokens die after one use).
2.  **Token Theft** is mitigated (internal keys are short-lived).
3.  **Self-Healing**: If a packet is lost, Jacox automatically uses the master API key to re-synchronize the chain.

---

## 5. Session Portability (Import/Export) 📥📤
Export your history to human-readable `.txt` files and import them into any other Jacox instance.

### 🛠 CLI Commands
```bash
# Export
cargo run -- session export <UUID> --path my_chat.txt

# Import
cargo run -- session import --path my_chat.txt
```

### 🌐 REST API Endpoints
- **Export**: `GET /sessions/{id}/export` (Returns plain text with `Content-Disposition` attachment)
- **Import**: `POST /sessions/import` (Send the `.txt` file content as the request body)

### 🎯 Use Case: Context Sharing
Developers can export a complex debugging session with an AI and share the `.txt` file with a teammate. The teammate can import it into their own Jacox instance via the CLI or UI to continue the conversation with the same context.

---

## 6. Interactive Playground 🛝
A built-in dashboard for rapid testing of your local models.

### 🛠 Access
Visit `http://localhost:8080/playground` to see the live chat interface.

### 🎯 Use Case: Model Benchmarking
Quickly switch models in `config.yaml` and test how they handle the same prompts side-by-side using the Playground UI.

---

## 7. Internet Search Tool 🌐
Jacox includes a built-in search tool that uses a custom Rust scraper to fetch grounding data from the web. This allows local models (like Mistral via Ollama) to answer questions about current events.

### 🛠 Example Prompt
"What is the current price of Bitcoin in Euros?"
-> The agent will trigger a `Searching: internet_search...` status and fetch live data.

### 🎯 Use Case: Real-time Grounding
Give your local models access to the "today" without needing a search API subscription (like Brave or Google).

---

## 8. High-Fidelity Data Visualization (Charts) 📊
If an LLM provides a JSON block with `role: "chart"`, Jacox will render it as a premium Recharts visualization.

### 🛠 JSON Format
```json
{
  "role": "chart",
  "type": "line",
  "title": "Bitcoin Trend",
  "data": [
    { "label": "Monday", "values": [50000, 51000, 52000], "subLabels": ["00:00", "08:00", "16:00"] }
  ],
  "xAxis": "label",
  "yAxis": "value"
}
```

### 🎯 Use Case: Financial Analysis
Visualize stock trends, crypto prices, or server telemetry directly in the chat interface with interactive tooltips and granular data buckets.

---

## 9. Skills Library 📚
A persistent library of reusable markdown snippets — system prompts, templates, role definitions, and more — that can be created, imported from a URL, and copied to the clipboard with a single click.

### 🛠 Creating a Skill (API)
```bash
curl -X POST http://localhost:8080/api/skills \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{
    "name": "Senior Code Reviewer",
    "content": "You are a senior engineer. Review for correctness, performance, and security.",
    "tags": "code, review"
  }'
```

### 🌐 Importing from a URL
```bash
curl -X POST http://localhost:8080/api/skills/fetch-url \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{"name": "Awesome README", "url": "https://raw.githubusercontent.com/sindresorhus/awesome/main/readme.md"}'
```

### 🎯 Use Case: Personal Prompt Library
Build a curated set of system personas ("Python Expert", "SQL Analyst", "Meeting Summarizer"). Before starting a new chat, open the **Skills** sidebar, copy the relevant persona to the clipboard, and paste it as the first message to instantly set the model's behavior without rewriting the same boilerplate every session.

For full documentation see [SKILLS.md](./SKILLS.md).
