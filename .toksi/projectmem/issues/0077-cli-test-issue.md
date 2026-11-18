# Issue #0077 - CLI test issue

**Created:** 2026-11-04T23:25:56Z
**Location:** `src/main.rs:1`
**Commit:** `0f4d66e`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T23:25:56Z] Opened: CLI test issue
- [2026-11-04T23:26:04Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T23:26:12Z] Fix confirmed: Fixed assertion in test_foo
