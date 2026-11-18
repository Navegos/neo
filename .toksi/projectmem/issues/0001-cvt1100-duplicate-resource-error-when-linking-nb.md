# Issue #0001 - CVT1100 duplicate resource error when linking nbuild tests on Windows

**Created:** 2026-10-16T21:35:10Z
**Location:** `crates/nbuild/tests/test_basic.rs`
**Commit:** `4d62972`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-16T21:35:10Z] Opened: CVT1100 duplicate resource error when linking nbuild tests on Windows
- [2026-10-16T21:35:29Z] Attempt (worked): Remove winres from crates/build.rs and use build metadata to eliminate CVT1100 duplicate PE resource conflict
- [2026-10-16T21:39:16Z] Fix confirmed: Eliminated duplicate winres PE resource conflict across crates, all 75 tests pass
- [2026-10-29T00:50:49Z] Opened: CLI test issue
- [2026-10-29T00:50:54Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T00:50:59Z] Fix confirmed: Fixed assertion in test_foo
