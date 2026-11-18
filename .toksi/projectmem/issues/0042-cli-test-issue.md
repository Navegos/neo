# Issue #0042 - CLI test issue

**Created:** 2026-10-31T11:59:29Z
**Location:** `src/main.rs:1`
**Commit:** `40504ec`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T11:59:29Z] Opened: CLI test issue
- [2026-10-31T11:59:36Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T11:59:43Z] Fix confirmed: Fixed assertion in test_foo
