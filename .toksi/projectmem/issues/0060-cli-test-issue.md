# Issue #0060 - CLI test issue

**Created:** 2026-11-02T20:20:30Z
**Location:** `src/main.rs:1`
**Commit:** `e0bc5f7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-02T20:20:30Z] Opened: CLI test issue
- [2026-11-02T20:20:38Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-02T20:20:46Z] Fix confirmed: Fixed assertion in test_foo
