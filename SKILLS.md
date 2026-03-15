# Jacox Skills 📚

**Skills** are reusable, named markdown snippets that live in the Jacox database. Think of them as your personal prompt library: system prompts, role definitions, code templates, or any text you want to inject into a conversation without re-typing it.

---

## 🏗️ What is a Skill?

| Field | Description | Example |
| :--- | :--- | :--- |
| `Name` | A unique identifier. | `"Senior React Architect"` |
| `Content` | The markdown/text prompt. | `"You are a senior React developer..."` |
| `Tags` | Comma-separated labels. | `"frontend, react, arch"` |
| `Source URL` | The origin if imported. | `https://github.com/...` |

---

## 🎯 Practical Tutorial: persona Management

### Step 1: Create a "Coding Expert" Persona
1. Navigate to the **Skills** tab in the sidebar.
2. Click **New Skill**.
3. Name: `Coding Expert`.
4. Tags: `code, review`.
5. Content: 
   ```markdown
   As an Expert Developer, review the following code for:
   - Security (OWASP Top 10)
   - Performance (Big O complexity)
   - Readability (Clean Code principles)
   ```
6. Click **Create Skill**.

### Step 2: Import from the Community
1. Click **Import from URL**.
2. Enter a raw GitHub URL (e.g., a system prompt from a repo).
3. Jacox automatically fetches the content and saves it as a reusable skill.

### Step 3: Usage in Chat
1. Find your `Coding Expert` skill card.
2. Click the **Copy** (clipboard) icon.
3. Paste it into your chat message or as the initial System Prompt.

---

## 🛠️ Advanced: Skills API

For power users, Jacox provides a full REST API for skill management.

### List All Skills
```bash
curl -X GET http://localhost:8080/api/skills \
  -H "Authorization: Bearer sk-dev-key-123"
```

### Create a New Skill
```bash
curl -X POST http://localhost:8080/api/skills \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{
    "name": "SQL Analyst",
    "content": "You are a professional SQL data analyst...",
    "tags": "data, db"
  }'
```

### Fetch from URL Programmatically
```bash
curl -X POST http://localhost:8080/api/skills/fetch-url \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-dev-key-123" \
  -d '{
    "url": "https://raw.githubusercontent.com/.../readme.md",
    "name": "Project Documentation"
  }'
```

---

## 💡 Best Practices

1. **Tag Aggressively**: Use tags like `code`, `personal`, `work` to filter your library as it grows.
2. **Versioned Personas**: Create skills like `React Expert v1` and `React Expert v2` to compare subtle variations in model responses.
3. **Template Blocks**: Store structured templates (e.g., "Bug Report", "Feature Spec") to ensure consistency across your team's outputs.

Built with 🦀 and 🎨 for superior AI orchestration.
