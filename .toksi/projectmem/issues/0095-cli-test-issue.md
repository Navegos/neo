# Issue #0095 - CLI test issue

**Created:** 2026-11-05T23:30:45Z
**Location:** `src/main.rs:1`
**Commit:** `1ffc749`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-05T23:30:45Z] Opened: CLI test issue
- [2026-11-05T23:30:54Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-05T23:31:02Z] Fix confirmed: Fixed assertion in test_foo
