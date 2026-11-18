# Issue #0062 - CLI test issue

**Created:** 2026-11-04T11:52:11Z
**Location:** `src/main.rs:1`
**Commit:** `50201a4`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T11:52:11Z] Opened: CLI test issue
- [2026-11-04T11:52:19Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T11:52:27Z] Fix confirmed: Fixed assertion in test_foo
