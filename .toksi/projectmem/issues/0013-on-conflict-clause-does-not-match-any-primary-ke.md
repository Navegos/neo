# Issue #0013 - ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash

**Created:** 2026-10-28T06:21:51Z
**Location:** `crates/tokless/src/icm/store.rs:77`
**Commit:** `a95dd89`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-28T06:21:51Z] Opened: ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash
- [2026-10-28T06:23:16Z] Fix confirmed: Add UNIQUE constraint directly on summary_hash column in memories table
- [2026-10-29T20:21:10Z] Opened: CLI test issue
- [2026-10-29T20:21:16Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T20:21:22Z] Fix confirmed: Fixed assertion in test_foo
