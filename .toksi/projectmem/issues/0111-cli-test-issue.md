# Issue #0111 - CLI test issue

**Created:** 2026-11-08T01:35:47Z
**Location:** `src/main.rs:1`
**Commit:** `749f1b0`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-08T01:35:47Z] Opened: CLI test issue
- [2026-11-08T01:35:57Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-08T01:36:07Z] Fix confirmed: Fixed assertion in test_foo
