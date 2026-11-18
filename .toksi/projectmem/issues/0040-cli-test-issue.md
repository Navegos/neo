# Issue #0040 - CLI test issue

**Created:** 2026-10-31T11:24:48Z
**Location:** `src/main.rs:1`
**Commit:** `6ce377f`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-31T11:24:48Z] Opened: CLI test issue
- [2026-10-31T11:24:55Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T11:25:02Z] Fix confirmed: Fixed assertion in test_foo
