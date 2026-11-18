# Issue #0098 - CLI test issue

**Created:** 2026-11-06T00:12:41Z
**Location:** `src/main.rs:1`
**Commit:** `21d49bf`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-06T00:12:41Z] Opened: CLI test issue
- [2026-11-06T00:12:51Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-06T00:13:00Z] Fix confirmed: Fixed assertion in test_foo
