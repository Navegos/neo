---
name: cavecrew
description: >
  Delegate to compressed subagent presets (`cavecrew-investigator`, `cavecrew-builder`,
  `cavecrew-reviewer`) answering in ultracave voice so main context lasts longer.
---

# Cavecrew

Three ultracave subagent presets whose returned results use strict compact contracts:

- `cavecrew-investigator`: locate definitions/callers; returns `path:line — symbol — note` and `totals: <counts>` or `No match.`
- `cavecrew-builder`: surgical edits (<=2 files, known scope); returns `path:line-range — <change>` + `verified: re-read OK` or terminal marker (`too-big.`, `needs-confirm.`, `ambiguous.`, `regressed.`).
- `cavecrew-reviewer`: diff review; returns `path:line: <severity>: <problem>. <fix>.` sorted ascending, or `No issues.`

## Auto-Clarity & Safety

Verify absence claims (`No match.`, `No issues.`) with one targeted check before acting. Drop to plain prose for security warnings and irreversible-action confirmations.
