# Issue #0047 - CLI test issue

**Created:** 2026-10-31T14:47:03Z
**Location:** `src/main.rs:1`
**Commit:** `c01d263`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T14:47:03Z] Opened: CLI test issue
- [2026-10-31T14:47:10Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T14:47:17Z] Fix confirmed: Fixed assertion in test_foo
