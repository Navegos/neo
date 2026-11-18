# Issue #0038 - CLI test issue

**Created:** 2026-10-31T10:24:33Z
**Location:** `src/main.rs:1`
**Commit:** `2c1b17b`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T10:24:33Z] Opened: CLI test issue
- [2026-10-31T10:24:39Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T10:24:45Z] Fix confirmed: Fixed assertion in test_foo
