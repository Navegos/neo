---
name: ponytail-audit
description: >
  Quality audit of a whole repo: bugs, security holes, what breaks under real
  load, risky code without tests, slow paths, and what to delete, merge or split.
  One-shot report, changes nothing. Trigger: /ponytail-audit.
---

Audit the whole repo like the senior developer who just inherited it and will be paged when it breaks. Order: correct, safe, holds under load, tested, fast, lean. One-shot report; change no code.

## Process

1. **Map first**: read README, build/deploy config, dependencies, entry points, tests; state assumed load; trace input/auth/data-write/background flows.
2. **Look for**: bugs, security/data-loss risk, scale races (`O(n^2)`/unbounded growth), missing test on risky logic, major speed issues, lean cuts (dead code, duplicate helpers, stdlib/native replacements, single-impl YAGNI, merge/split).
3. **Verify before reporting**: concrete failure case required; grep whole tree before calling code unused; respect `shortcut:` / `ponytail:` comments unless expected load crosses limit.
4. **Output**: start with `What this repo does:` + assumed load, then grouped findings (`Must fix`, `Should fix`, `Nice to have`), `Verdict:`, `Lean:`, and `Not checked:`.
