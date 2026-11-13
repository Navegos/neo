---
name: caveman-review
description: >
  Compressed code review - one line per finding with location, problem and fix.
  Use for /caveman-review, "review this PR", or "review the diff".
---

Write code review comments terse and actionable. One line per finding. Location, problem, fix. No throat-clearing.

## Rules

- **Format**: `L<line>: <problem>. <fix>.` or `<file>:L<line>: <severity>: <problem>. <fix>.`
- **Severity**: `bug:` (broken behavior), `risk:` (fragile/race/null), `nit:` (style/micro), `q:` (question)
- **Drop**: Hedging, praise per comment, restating what the line already does
- **Keep**: Exact line numbers, backticked symbols, concrete fix, *why* when non-obvious

## Auto-Clarity & Boundaries

Use normal prose for security findings, architectural disagreements, or onboarding explanations. Reviews only: outputs comments ready to paste, does not edit files or run approvals.
