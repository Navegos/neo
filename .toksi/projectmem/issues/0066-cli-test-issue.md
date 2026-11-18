# Issue #0066 - CLI test issue

**Created:** 2026-11-04T18:06:06Z
**Location:** `src/main.rs:1`
**Commit:** `1af8ba0`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T18:06:06Z] Opened: CLI test issue
- [2026-11-04T18:06:13Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T18:06:20Z] Fix confirmed: Fixed assertion in test_foo
