# Issue #0070 - CLI test issue

**Created:** 2026-11-04T21:57:15Z
**Location:** `src/main.rs:1`
**Commit:** `1ddf24a`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T21:57:15Z] Opened: CLI test issue
- [2026-11-04T21:57:23Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-04T21:57:31Z] Fix confirmed: Fixed assertion in test_foo
