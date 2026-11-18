# Issue #0051 - CLI test issue

**Created:** 2026-11-01T15:00:56Z
**Location:** `src/main.rs:1`
**Commit:** `1d68c24`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T15:00:56Z] Opened: CLI test issue
- [2026-11-01T15:01:03Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-01T15:01:09Z] Fix confirmed: Fixed assertion in test_foo
