---
name: ponytail-help
description: >
  Quick reference for ponytail levels, skills, and commands. One-shot display.
  Trigger: /ponytail-help.
---

# Ponytail Help

One-shot reference card: do not change mode, write flag files, or persist state.

## Levels & Skills

- `/ponytail lite` | `/ponytail` (`full`, default) | `/ponytail ultra` | `/ponytail off`
- `/ponytail-review`: quality and simplicity review of current changes
- `/ponytail-audit`: whole-repo quality and bloat audit
- `/ponytail-debt`: tracked ledger of `shortcut:` and `ponytail:` comments
- `/ponytail-gain`: published Ponytail 5 benchmark scoreboard
- `/ponytail-help`: this card

## Statusline & Configuration

- Default mode resolution: `PONYTAIL_DEFAULT_MODE` env var (`off|lite|full|ultra`) > config file (`~/.config/ponytail/config.json`) > `full`.
- Statusline flag: `.ponytail-active` renders `[PONYTAIL]`, `[PONYTAIL:LITE]`, `[PONYTAIL:ULTRA]` (amber), or `[PONYTAIL:REVIEW]`.
