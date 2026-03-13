---
name: semlocal
description: Store, search, and delete text using local semantic search via the semlocal CLI. Use when the user asks to remember information, search for semantically similar text, build a local knowledge base, or work with vector embeddings locally.
---

# semlocal

Local semantic search CLI. Stores text as vector embeddings in a local SQLite database and retrieves entries by semantic similarity. No backend, no API keys.

## Prerequisites

Install globally via npm:

```bash
npm install -g semlocal
```

The embedding model (~25 MB) downloads automatically on first use and is cached at `~/.semlocal/models/`.

## Commands

### Write — store text

```bash
semlocal write "Text to store"
```

Returns a UUID for the stored entry. Also accepts piped input:

```bash
echo "some text" | semlocal write
cat file.txt | semlocal write -
```

### Search — find similar text

```bash
semlocal search "query text"
```

Output format: `[score] id content`

Options:
- `--top N` — number of results (default: 5)
- `--json` — output as JSON array with `id`, `score`, `content` fields

```bash
semlocal search "query" --top 3 --json
```

### Delete — remove an entry

```bash
semlocal delete <uuid>
```

Fails if the entry does not exist.

### Collections

All commands accept `--collection <name>` to partition entries. Defaults to `default` if omitted.

```bash
semlocal write "some fact" --collection notes
semlocal search "query" --collection notes
semlocal delete <uuid> --collection notes
```

Collections are implicit — created on first write, removed when their last entry is deleted.

### Custom index directory

All commands accept `--src <path>` to use a specific index directory instead of the default `.semlocal/` in the current working directory.

```bash
semlocal write "hello" --src ~/my-index
semlocal search "greeting" --src ~/my-index
```

## Usage Patterns

### Building a knowledge base

Store multiple entries, then query them:

```bash
semlocal write "Rust is a systems programming language focused on safety"
semlocal write "Python is great for data science and machine learning"
semlocal write "JavaScript runs in the browser and on the server via Node.js"

semlocal search "safe low-level language"
# [0.87] <id> Rust is a systems programming language focused on safety
```

### Ingesting file contents

```bash
cat notes.txt | semlocal write
```

### Machine-readable output

Use `--json` when parsing results programmatically:

```bash
semlocal search "query" --json --top 3
```

Returns:

```json
[
  { "id": "...", "score": 0.87, "content": "..." },
  { "id": "...", "score": 0.72, "content": "..." }
]
```

### Isolated indexes

Use `--src` to maintain separate indexes for different projects or contexts:

```bash
semlocal write "project A note" --src .semlocal-a
semlocal write "project B note" --src .semlocal-b
```

## Notes

- Embeddings use the `all-MiniLM-L6-v2` model (384 dimensions).
- Search is brute-force cosine similarity over all stored entries.
- The index is stored as a SQLite database at `<src>/store.db`.
- First run may take a few seconds while the model downloads.
