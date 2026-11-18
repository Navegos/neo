# Issue #0064 - CLI test issue

**Created:** 2026-11-04T13:52:18Z
**Location:** `src/main.rs:1`
**Commit:** `f8da2d3`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T13:52:18Z] Opened: CLI test issue
- [2026-11-04T13:52:25Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T13:52:33Z] Fix confirmed: Fixed assertion in test_foo
