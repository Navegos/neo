# Issue #0004 - RTK tool missing opencode2 wiring, amp permission wiring, rtk_location_for, pi extension normalization, and kilo/cline foreign backup parity

**Created:** 2026-10-26T23:23:38Z
**Location:** `crates/tokless/src/tools/rtk.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:23:38Z] Opened: RTK tool missing opencode2 wiring, amp permission wiring, rtk_location_for, pi extension normalization, and kilo/cline foreign backup parity
- [2026-10-27T02:03:39Z] Fix confirmed: Fixed OpenCode BYOK fields, stale v2 path read, and RTK location in kilo and rtk.rs
- [2026-10-29T01:21:15Z] Opened: CLI test issue
- [2026-10-29T01:21:20Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T01:21:25Z] Fix confirmed: Fixed assertion in test_foo
