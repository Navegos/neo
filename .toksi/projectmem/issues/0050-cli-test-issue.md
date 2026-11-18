# Issue #0050 - CLI test issue

**Created:** 2026-10-31T23:11:18Z
**Location:** `src/main.rs:1`
**Commit:** `9a69ecd`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T23:11:18Z] Opened: CLI test issue
- [2026-10-31T23:11:25Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T23:11:33Z] Fix confirmed: Fixed assertion in test_foo
