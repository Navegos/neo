---
name: ponytail-debt
description: >
  List every `shortcut:` comment (and older `ponytail:` ones) as a debt ledger.
  One-shot report, changes nothing. Trigger: /ponytail-debt.
---

Collect every deliberate deferral (`shortcut:` or older `ponytail:` comment) into a debt ledger. One-shot report; change nothing.

## Scan & Output

- Scan repo while respecting `.gitignore` (`(#|//|/[*]) ?(shortcut|ponytail):`). Skip non-deferral hits (e.g., keyboard shortcuts).
- Format per file: `<file>:<line>, <what was simplified>. ceiling: <limit>. upgrade: <trigger>.`
- Tag missing upgrade paths with `no-trigger`.
- End with `<N> markers, <M> with no trigger.` or `No shortcut debt. Clean ledger.`
