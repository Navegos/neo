# Issue #0109 - CLI test issue

**Created:** 2026-11-06T02:37:59Z
**Location:** `src/main.rs:1`
**Commit:** `f17f06d`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-06T02:37:59Z] Opened: CLI test issue
- [2026-11-06T02:38:09Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-06T02:38:19Z] Fix confirmed: Fixed assertion in test_foo
