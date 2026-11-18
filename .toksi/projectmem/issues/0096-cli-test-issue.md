# Issue #0096 - CLI test issue

**Created:** 2026-11-05T23:49:35Z
**Location:** `src/main.rs:1`
**Commit:** `4bf46fb`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-05T23:49:35Z] Opened: CLI test issue
- [2026-11-05T23:49:45Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-05T23:49:54Z] Fix confirmed: Fixed assertion in test_foo
