# Issue #0099 - CLI test issue

**Created:** 2026-11-06T00:23:17Z
**Location:** `src/main.rs:1`
**Commit:** `21d49bf`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-06T00:23:17Z] Opened: CLI test issue
- [2026-11-06T00:23:25Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-06T00:23:33Z] Fix confirmed: Fixed assertion in test_foo
