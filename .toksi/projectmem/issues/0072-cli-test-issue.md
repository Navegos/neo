# Issue #0072 - CLI test issue

**Created:** 2026-11-04T22:21:12Z
**Location:** `src/main.rs:1`
**Commit:** `b3b21ff`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T22:21:12Z] Opened: CLI test issue
- [2026-11-04T22:21:20Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T22:21:27Z] Fix confirmed: Fixed assertion in test_foo
