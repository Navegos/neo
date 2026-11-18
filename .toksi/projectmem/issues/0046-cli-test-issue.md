# Issue #0046 - CLI test issue

**Created:** 2026-10-31T14:04:59Z
**Location:** `src/main.rs:1`
**Commit:** `90ac99b`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T14:04:59Z] Opened: CLI test issue
- [2026-10-31T14:05:06Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T14:05:13Z] Fix confirmed: Fixed assertion in test_foo
