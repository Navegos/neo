---
name: caveman-commit
description: >
  Write a Conventional Commits message compressed to intent only. Use for
  "write a commit", "commit message", /commit or /caveman-commit.
---

Write commit messages terse and exact. Conventional Commits format. No fluff. Why over what.

## Rules

**Subject line:**
- `<type>(<scope>): <imperative summary>` (`<scope>` optional)
- Types: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `build`, `ci`, `style`, `revert`
- Imperative mood (`add`, `fix`, `remove`), <=50 chars when possible (hard cap 72), no trailing period

**Body (only if needed):**
- Skip when subject is self-explanatory
- Include for non-obvious *why*, breaking changes, migration notes, linked issues (`Closes #42`, `Refs #17`)
- Wrap at 72 chars; use `-` bullets

**Exclude:**
- Filler (`"This commit does X"`, `"I"`, `"we"`, `"now"`), AI attribution unless required by project rule, emoji, restating file names already in scope

## Auto-Clarity & Boundaries

Always include full body for breaking changes, security fixes, data migrations, or reverts. Only generates the commit message as a paste-ready code block; does not run `git commit` or stage files.
