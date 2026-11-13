---
name: caveman-help
description: >
  Quick-reference card for the three caveman modes and companion skills.
  Trigger: /caveman-help or "caveman help".
---

# Caveman Help

One-shot reference card: do not change mode, write flag files, or persist state.

## Modes

- `caveman` (`/caveman`): answer first, no fluff, every fact kept (default).
- `ultracave` (`/ultracave`, alias `/caveman ultra`): grammar stripped; fragments; each fact once.
- `megacave` (`/megacave`, alias `/caveman wenyan`): Classical Chinese 文言文; identifiers verbatim.
- `/caveman status`: reports active mode without changing it.

## Companion Skills

- `/caveman-commit`: Conventional Commits terse subject (<=50 chars) and why-only body.
- `/caveman-review`: one-line findings (`L<line>: <severity>: <problem>. <fix>.`).
- `/caveman-compress <file>`: compress markdown memory files while preserving code/paths/frontmatter.
- `/caveman-stats`: session output and cache-read token attribution; savings remain unmeasured without paired baseline.
- `cavecrew`: compressed subagent presets (`cavecrew-investigator`, `cavecrew-builder`, `cavecrew-reviewer`).
- Subsystems: `shrink` (tool catalog compressor), `browse` (a11y browser MCP), `.blocks` (`# /// block` verification spec).

## Configuration & Deactivation

- Deactivate: `"stop caveman"` or `"normal mode"`.
- Default mode resolution: `CAVEMAN_DEFAULT_MODE` env var > repo/user config > `caveman` (legacy aliases: `lite|full -> caveman`, `ultra -> ultracave`, `wenyan* -> megacave`, `off|manual`).
