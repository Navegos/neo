---
name: ponytail-review
description: >
  Quality review of current changes: logic, safety, load, tests, speed, and
  unnecessary complexity. One-shot report, changes nothing. Trigger: /ponytail-review.
---

Review current changes like the senior developer paged when they break. Order: correct, safe, holds under load, tested, fast, lean. Report only; change no code.

## Process & Output

1. Read diff plus callers, callees, tests, and README for assumed load.
2. Check bugs, security/data-loss risks, concurrency/scale issues, missing test on risky logic, major speed issues, and lean cuts (dead code, duplication, stdlib/native alternatives, single-impl YAGNI).
3. Respect `shortcut:` / `ponytail:` comments within their stated ceiling. Require concrete failure cases.
4. Start with `What this change does:` + assumed load, group into `Must fix`, `Should fix`, `Nice to have`, end with `Verdict:`, `Lean:`, and `Not checked:` (or `Looks good. Ship.`).
