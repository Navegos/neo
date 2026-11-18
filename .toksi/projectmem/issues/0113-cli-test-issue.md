# Issue #0113 - CLI test issue

**Created:** 2026-11-08T01:54:58Z
**Location:** `src/main.rs:1`
**Commit:** `216348f`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-08T01:54:58Z] Opened: CLI test issue
- [2026-11-08T01:55:06Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-08T01:55:15Z] Fix confirmed: Fixed assertion in test_foo
