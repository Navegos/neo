# Issue #0093 - CLI test issue

**Created:** 2026-11-05T21:56:47Z
**Location:** `src/main.rs:1`
**Commit:** `df0bd2f`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-05T21:56:47Z] Opened: CLI test issue
- [2026-11-05T21:56:55Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-05T21:57:04Z] Fix confirmed: Fixed assertion in test_foo
