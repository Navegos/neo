# Issue #0016 - Phase 9: Legacy healing and binary un-hijack

**Created:** 2026-10-31T11:09:18Z
**Location:** `crates/tokless/CONSOLIDATION_PLAN.md:404`
**Commit:** `6ce377f`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T11:09:18Z] Opened: Phase 9: Legacy healing and binary un-hijack
- [2026-10-31T11:31:39Z] Attempt (worked): Implemented automated self-healing engine in env_injector::heal, purged destructive binary renaming and shims from antigravity, grok, and copilot, wired healing into init, doctor --fix, and daemon start
- [2026-10-31T11:34:14Z] Fix confirmed: Phase 9 complete: purged binary hijacking, automated self-healing engine restores .real binaries and deletes shims/state files, 232 unit tests and 5 phase 9 verification tests passing with 0 warnings
- [2026-10-30T19:21:27Z] Opened: CLI test issue
- [2026-10-30T19:21:33Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T19:21:39Z] Fix confirmed: Fixed assertion in test_foo
