---
name: caveman-compress
description: >
  Compress a memory file such as CLAUDE.md or a todo list into caveman format
  to save input tokens, keeping a readable backup. Trigger: /caveman-compress.
---

# Caveman Compress

Compress natural language memory files (`CLAUDE.md`, todos, preferences) into terse caveman prose while keeping an out-of-tree human-readable backup.

## Boundaries

- Refuse files whose names or contents hold secrets or credentials (`.env`, `credentials`, private keys, cloud/API tokens).
- Require an explicit filepath; do not compress pasted text without naming or saving the file first.
- Write out-of-tree backups outside live auto-loaded directories so hosts do not re-ingest `.original.md`.

## Compression Rules

- **Remove**: articles (`a`, `an`, `the`), filler (`just`, `really`, `basically`, `actually`, `simply`), pleasantries, hedging, connective fluff.
- **Preserve EXACTLY**: fenced and indented code blocks, inline code, URLs, file paths, commands, technical terms, proper nouns, dates, versions, numeric values, env vars, headings, list hierarchy, tables, YAML frontmatter.
- **Compress**: short active-voice bullets, imperative instructions, merge redundant bullets, keep one illustrative example.
