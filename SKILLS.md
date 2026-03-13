# Jacox Skills 📚

**Skills** are reusable, named markdown snippets that live in the Jacox database. Think of them as your personal prompt library: system prompts, role definitions, code templates, analysis frameworks, or any text you want to inject into a conversation without re-typing it.

---

## What Is a Skill?

A **Skill** has four fields:

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Human-readable label, e.g. `"Senior Code Reviewer"` |
| `content` | Markdown text | The actual content — could be a system prompt, a template, instructions, etc. |
| `tags` | Comma-separated string | Optional labels to group/filter skills, e.g. `"code, review, python"` |
| `source_url` | Optional URL | Populated automatically when a skill is imported from a URL |

---

## The Workflow

```
┌──────────────────────────────────────────────────────────┐
│                     Skill Library                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Review   │  │ Explain  │  │ Summary  │  ← saved      │
│  │ Prompt   │  │ Code     │  │ Template │    skills     │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘              │
│       │              │              │                     │
│       └──────────────┴──────────────┘                   │
│                      ↓                                   │
│              📋 Copy to Clipboard                        │
│                      ↓                                   │
│              Paste into Chat / System Prompt             │
└──────────────────────────────────────────────────────────┘
```

1. **Create** a skill once — write your prompt/template in the markdown editor.
2. **Tag** it so you can find it later (`code`, `review`, `analysis`, etc.).
3. **Copy** the skill content to the clipboard with a single click.
4. **Paste** it into a chat message, another tool, or a system configuration.
5. **Edit or delete** it whenever your workflow evolves.

---

## Creating a Skill

### Via the UI

1. Navigate to **Skills** in the sidebar.
2. Click **New Skill**.
3. Fill in the **name**, optional **tags**, and the **content** in the markdown editor (monospace, full markdown supported).
4. Hit **Create Skill**.

### Via the API

```bash
curl -X POST http://localhost:8080/api/skills \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{
    "name": "Senior Code Reviewer",
    "content": "You are an experienced senior engineer. Review the following code for:\n- Correctness\n- Performance\n- Security\n- Readability\n\nProvide inline comments and a severity rating (low / medium / high) for each issue.",
    "tags": "code, review"
  }'
```

---

## Importing a Skill from a URL

You can point Jacox at any public URL (a raw GitHub file, a personal website, a documentation page) and it will fetch and store the content as a skill.

### Via the UI

1. Open **New Skill** and click **Import from URL**.
2. Paste the URL (e.g. a raw `.md` file on GitHub).
3. Click **Fetch & Save** — the content is fetched, stored, and the card appears immediately.

### Via the API

```bash
curl -X POST http://localhost:8080/api/skills/fetch-url \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{
    "name": "Awesome README",
    "url": "https://raw.githubusercontent.com/sindresorhus/awesome/main/readme.md",
    "tags": "reference"
  }'
```

> **Content-Type handling:**  
> If the URL returns `text/markdown` or plain text, it is stored as-is.  
> If it returns `text/html`, basic tag stripping is applied to extract readable text.

---

## Editing & Deleting Skills

### Via the UI

- **Edit**: Click the ✏️ pencil icon on any skill card, modify the name, tags, or content, and save.
- **Delete**: Click the 🗑️ trash icon. The deletion is immediate (no soft-delete).

### Via the API

```bash
# Update name and/or content
curl -X PATCH http://localhost:8080/api/skills/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{"name": "Updated Name", "tags": "code, updated"}'

# Delete
curl -X DELETE http://localhost:8080/api/skills/1 \
  -H "Authorization: Bearer sk-dev-key-123"
```

---

## Using Skills in a Conversation

Skills are **passive** — Jacox does not inject them automatically. This is intentional: you stay in full control of what goes into every prompt.

The recommended workflow:

1. Start a new chat session.
2. Open the **Skills** page in a side tab.
3. Find the skill you want.
4. Click **Copy** (clipboard icon) on the card.
5. Paste it into the first message of your chat as a system context block, or prepend it to your user message.

### Example

You have a skill called `"Python Expert"` that contains:

```
You are an expert Python engineer with deep knowledge of CPython internals,
async programming, and the scientific Python ecosystem (NumPy, Pandas, PyTorch).
Always prefer idiomatic, PEP-8 compliant code and explain your reasoning.
```

You paste this into the chat before asking your question:

```
[Skill: Python Expert]
You are an expert Python engineer…

Question: Why is my asyncio event loop blocking on this CPU-bound function?
```

The model uses the skill content as context for the entire reply.

---

## Searching & Filtering

The Skills page has a live **search bar** that filters across `name`, `content`, and `tags` simultaneously. This lets you find the right skill quickly even with a large library.

---

## Data Storage

Skills are stored in the local **DuckDB** database (`chat.db`) alongside sessions and messages. The schema is:

```sql
CREATE TABLE skills (
    id         BIGINT PRIMARY KEY DEFAULT nextval('seq_skills_id'),
    name       VARCHAR NOT NULL,
    content    TEXT NOT NULL,
    tags       VARCHAR DEFAULT '',
    source_url VARCHAR,                         -- set when imported from URL
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

They persist across server restarts and are included in any DuckDB backup you make of `chat.db`.

---

## Example Use Cases

| Use Case | Skill Content |
|----------|--------------|
| **System persona** | A detailed role definition: "You are a principal SRE at a Fortune 500 company…" |
| **Code review template** | A checklist for reviewers listing security, performance, style points |
| **Meeting summary format** | Instructions for formatting transcripts into action items and owners |
| **Translation prompt** | "Translate the following text to Brazilian Portuguese, preserving technical terminology" |
| **Data analysis framework** | Step-by-step instructions for analyzing CSV data |
| **Imported reference doc** | Raw GitHub README, official documentation page fetched via URL import |

---

## API Reference Summary

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/skills` | List all skills (`?limit=50&offset=0`) |
| `POST` | `/api/skills` | Create a skill |
| `GET` | `/api/skills/:id` | Get a single skill |
| `PATCH` | `/api/skills/:id` | Update name, content, or tags |
| `DELETE` | `/api/skills/:id` | Delete a skill |
| `POST` | `/api/skills/fetch-url` | Fetch from a URL and store as a skill |

See [API_GUIDE.md](./API_GUIDE.md) for full request/response schemas.
