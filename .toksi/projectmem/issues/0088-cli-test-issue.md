# Issue #0088 - CLI test issue

**Created:** 2026-11-05T20:29:30Z
**Location:** `src/main.rs:1`
**Commit:** `765c83e`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-05T20:29:30Z] Opened: CLI test issue
- [2026-11-05T20:29:38Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-11-05T20:29:46Z] Fix confirmed: Fixed assertion in test_foo
