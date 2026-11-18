# Issue #0082 - CLI test issue

**Created:** 2026-11-05T15:04:45Z
**Location:** `src/main.rs:1`
**Commit:** `2ef4758`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-05T15:04:45Z] Opened: CLI test issue
- [2026-11-05T15:04:54Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-05T15:05:01Z] Fix confirmed: Fixed assertion in test_foo
