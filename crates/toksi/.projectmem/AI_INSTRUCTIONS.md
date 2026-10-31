# projectmem AI Instructions

<!-- projectmem-instructions: 1 -->

These instructions are MANDATORY for all AI coding agents working in this project. Failure to follow them means your work is incomplete and the audit trail is corrupted.

This file is stable operating guidance. Do not rewrite it unless the user asks or projectmem itself changes.

## Start of every session

**Step 1 — Identify your mode by reading `.projectmem/summary.md` and `.projectmem/PROJECT_MAP.md`.**

- **Setup Mode** — `summary.md` and/or `PROJECT_MAP.md` still contain the **placeholder text** from `pjm init`.
- **Maintenance Mode** — `summary.md` AND `PROJECT_MAP.md` contain real project content.

**Step 2 — Read these files (or call the MCP equivalents):**

| File | MCP tool | Purpose |
| --- | --- | --- |
| `.projectmem/AI_INSTRUCTIONS.md` | `get_instructions()` | Workflow rules (this file) |
| `.projectmem/summary.md` | `get_summary()` | Distilled project memory (what HAPPENED) |
| `.projectmem/PROJECT_MAP.md` | `get_project_map()` | Structural layout |
| `.projectmem/plan.md` | `get_plan()` | Ideas + plans (what we INTEND to do) |

## MANDATORY Triggers — You MUST act on these automatically

| Trigger | MCP tool | CLI command |
| --- | --- | --- |
| Bug, error, or unexpected behavior | `log_issue(summary, location)` | `pjm log "<text>" --at "<file:line>"` |
| Fix attempt FAILED | `record_attempt(summary, outcome="failed")` | `pjm attempt "<text>" --failed --at "<file:line>"` |
| Fix attempt PARTIAL | `record_attempt(summary, outcome="partial")` | `pjm attempt "<text>" --partial --at "<file:line>"` |
| Fix attempt WORKED | `record_attempt(summary, outcome="worked")` | `pjm attempt "<text>" --worked --at "<file:line>"` |
| Fix confirmed — close the issue | `record_fix(summary)` | `pjm fix "<text>" --at "<file:line>"` |
| Architectural / design decision | `add_decision(summary)` | `pjm decision "<text>" --at "<file:line>"` |
| Gotcha / setup detail / constraint discovered | `add_note(summary)` | `pjm note "<text>" --at "<file:line>"` |
| Before finishing the session | `get_summary()` | `pjm show` |

All write tools auto-append to `events.jsonl` AND auto-regenerate `summary.md`.
