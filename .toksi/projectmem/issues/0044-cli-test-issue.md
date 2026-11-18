# Issue #0044 - CLI test issue

**Created:** 2026-10-31T13:14:12Z
**Location:** `src/main.rs:1`
**Commit:** `732fa9f`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T13:14:12Z] Opened: CLI test issue
- [2026-10-31T13:14:19Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T13:14:26Z] Fix confirmed: Fixed assertion in test_foo
