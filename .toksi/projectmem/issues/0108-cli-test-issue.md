# Issue #0108 - CLI test issue

**Created:** 2026-11-06T02:25:38Z
**Location:** `src/main.rs:1`
**Commit:** `f17f06d`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-06T02:25:38Z] Opened: CLI test issue
- [2026-11-06T02:25:48Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-06T02:25:58Z] Fix confirmed: Fixed assertion in test_foo
