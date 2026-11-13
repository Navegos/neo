---
name: caveman-stats
description: >
  Show recorded output and cache-read token usage and mode attribution for
  the current session, or locate the host's native usage report. Trigger: /caveman-stats.
---

# Caveman Stats

Report recorded output and cache-read tokens, response counts, and mode attribution where tracked by the host.

- Do not estimate or fabricate token savings percentages, dollar savings, or net rule overhead without a paired baseline comparison (`Savings remain unknown`).
- On Gemini CLI, direct the user to `/stats model` or `/stats session`.
- Original vs compressed memory files are compared strictly by measured byte size.
- Statusline badge reflects active mode only: `[CAVEMAN]`, `[ULTRACAVE]`, `[MEGACAVE]`, or `[CAVEMAN:COMMIT|REVIEW|COMPRESS]`; durable `off` renders nothing.
