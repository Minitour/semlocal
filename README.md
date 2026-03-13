# semlocal

[![Release](https://github.com/Minitour/semlocal/actions/workflows/release.yml/badge.svg)](https://github.com/Minitour/semlocal/actions/workflows/release.yml)
[![npm](https://img.shields.io/npm/v/semlocal)](https://www.npmjs.com/package/semlocal)

Local semantic search for the command line. Write, search, and delete text using vector embeddings — no backend, no API keys, everything stays on your machine.

## Install

```bash
npm install -g semlocal
```

## Usage

### Store text

```bash
semlocal write "Rust is a systems programming language focused on safety"
# prints: a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

### Search

```bash
semlocal search "safe low-level language"
# [0.87] a1b2c3d4-... Rust is a systems programming language focused on safety
```

Return results as JSON:

```bash
semlocal search "safe low-level language" --json --top 3
```

### Delete an entry

```bash
semlocal delete a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

### Custom storage directory

By default the index is stored in `.semlocal/` in the current working directory. Use `--src` to change this:

```bash
semlocal write "hello world" --src ~/my-index
semlocal search "greeting" --src ~/my-index
```

## How it works

semlocal uses [FastEmbed](https://github.com/Anush008/fastembed-rs) (ONNX Runtime) to generate 384-dimensional vector embeddings with the `all-MiniLM-L6-v2` model. Embeddings are stored in a local SQLite database. Search is performed via brute-force cosine similarity over all stored entries.

### First run

The embedding model (~25 MB) is downloaded automatically on first use and cached in `~/.cache/fastembed/`. Subsequent runs start instantly.

## Platforms

Pre-built binaries are provided for:

| OS | x64 | arm64 |
|---|---|---|
| Linux | ✓ | ✓ |
| macOS | | ✓ |
| Windows | ✓ | |

## License

MIT
