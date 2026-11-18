# Issue #0039 - CLI test issue

**Created:** 2026-10-31T10:54:16Z
**Location:** `src/main.rs:1`
**Commit:** `6ce377f`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T10:54:16Z] Opened: CLI test issue
- [2026-10-31T10:54:23Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T10:54:29Z] Fix confirmed: Fixed assertion in test_foo
