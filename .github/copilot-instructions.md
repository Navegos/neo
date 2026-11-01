# Agent Instructions

Apply on every coding task:

**Skills (apply):**
- Principles — think, simplify, edit surgically, verify.
- Response Style (caveman) — terse prose, full technical accuracy.
- Build Discipline (ponytail) — reuse first, write only what must exist.

**Tools (call):**
- Code Index (codegraph) — MCP `codegraph_explore` for structure, flows, dependencies.
- Context Tools (context-mode) — MCP `ctx_*` for large, uncertain, multi-source analysis.
- Project Memory (projectmem) — MCP tools log issues, attempts, fixes, decisions, gotchas.

## Context Tools (context-mode)

MCP tools `ctx_*` keep large or uncertain analysis out of context.

### Rules

| Tool | Role | Replaces |
|------|------|----------|
| `ctx_execute` | Run code in sandbox. Only stdout enters context. | Bash for analysis tasks |
| `ctx_execute_file` | Process file in sandbox. Raw bytes never leave. | Read on large files (>200 lines) |
| `ctx_batch_execute` | Run N commands + auto-index output. Search in same call. Concurrency 1-8. | Multiple Bash + grep |
| `ctx_index` | Chunk markdown/text into FTS5. Queryable via `ctx_search`. | Manual grep over pasted content |
| `ctx_search` | Multi-strategy search across indexed content + session memory. Typo correction. | Re-asking user, re-deriving |
| `ctx_fetch_and_index` | Fetch URL, chunk and index. Cache 24h; `ttl: 0` or `force: true` bypasses cache. Supports `requests` + `concurrency: 1-8`. | Webfetch |

### Examples

```shell
ctx_execute(language:"shell", code:"git log --oneline -20 | wc -l")
```

```javascript
ctx_execute(language:"javascript", code:`
  const fs = require('node:fs');
  const files = fs.readdirSync('src').filter(f => f.endsWith('.ts'));
  const counts = files.map(f => ({
    file: f,
    lines: fs.readFileSync('src/' + f, 'utf8').split('\n').length
  }));
  console.log(JSON.stringify(counts));
`)
```

```javascript
ctx_execute_file(path:"app.log", language:"javascript", code:`
  const lines = FILE_CONTENT.split('\n');
  const errs = lines.filter(l => /ERROR|FATAL/.test(l));
  console.log(errs.length + ' errors');
`)
```

```javascript
ctx_batch_execute(commands:[
  {label:"diff", command:"git diff HEAD~1"},
  {label:"status", command:"git status"},
], queries:["failures","errors"], timeout:120000, concurrency:2,
 cwd:".", query_scope:"batch")
```

```javascript
ctx_fetch_and_index(requests:[
  {url:"https://docs.example.com/api", source:"api-docs"},
  {url:"https://docs.example.com/guide", source:"guide"},
], concurrency:4)
ctx_search(queries:["auth endpoint","rate limits"], source:"api-docs")
```

Use host tools for edits, Git writes, navigation, installs, and small output. Use context-mode for analysis and large command output. Sandbox writes do not affect host files.

Windows: `pwsh -NoProfile -Command`, absolute paths, `X:\` maps to `/x/`, quote spaces.

## Project Memory (projectmem)

MCP tools record what broke, what was tried, and what was decided, across sessions, in `.projectmem/`.

### Rules

| Event | Tool | Rule |
|-------|------|------|
| Bug found | `log_issue` | Trigger: bug, unclear behavior, hypothesis, or investigation topic. |
| Fix tried | `record_attempt` | `outcome`: `worked`, `failed`, or `partial`. One call per attempt. |
| Fix proven | `record_fix` | Needs evidence: test passes, error gone, or user confirms. Closes issue. |
| Design choice | `add_decision` | Replaces older decision: set `supersedes` to old event id — the old one retires from the summary, history is never rewritten; two disagreeing decisions corrupt trust. |
| Gotcha found | `add_note` | Prefix `gotcha:`. Shared across projects. Also use for notes and key files future contributors should read first. |

- Log as soon as a trigger fires — before any other work.
- One call per event, decisions and gotchas alike; never batch writer calls.
- Use only the fixed event vocabulary; never invent new event types.
- Log every fix, small ones too. Keep entries short and specific: file, error, test name.
- All writers accept `location: "file:line"`.
- Never log secrets, credentials, customer data, tokens, or large transcripts.
- Edit `PROJECT_MAP.md` and `plan.md` directly. Never log plans as events — a user's idea or plan belongs in `plan.md`.
- Never edit `summary.md` (derived) or `events.jsonl` (append-only). Stale detail there → record the correction with a writer tool.
- Superseding a decision? Get the old event id from `events.jsonl`, pass it as `supersedes`.
- Trust recorded memory over recall; don't re-derive answers by re-reading `summary.md` or source files.
- Before exiting: memory must reflect what actually happened.

### Examples

```
log_issue(summary: "JWT parse fails on expired token", location: "auth/middleware.go:42")
record_attempt(summary: "Moved expiry check before signature verify", outcome: "failed", issue_id: "0001")
record_fix(summary: "Use leeway comparison at boundary", issue_id: "0001")
add_decision(summary: "Use pgx pool over database/sql", supersedes: "<old event id>")
add_note(summary: "gotcha: local CI skips integration tests without TEST_DB set")
```
