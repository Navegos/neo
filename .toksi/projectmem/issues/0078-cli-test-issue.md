# Issue #0078 - CLI test issue

**Created:** 2026-11-04T23:35:10Z
**Location:** `src/main.rs:1`
**Commit:** `702dd59`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T23:35:10Z] Opened: CLI test issue
- [2026-11-04T23:35:18Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T23:35:25Z] Fix confirmed: Fixed assertion in test_foo
