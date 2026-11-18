# Issue #0021 - Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk

**Created:** 2026-11-01T18:18:09Z
**Location:** `crates/toksi/src/commands/rtk_hook.rs:52 (rtk_rewrite)`
**Commit:** `1d68c24`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T18:18:09Z] Opened: Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk
- [2026-11-01T18:19:30Z] Attempt (partial): Remove legacy external rtk binary fallback in rtk_rewrite and support toksi rtk prefix
- [2026-11-01T18:23:54Z] Fix confirmed: Purged external rtk binary fallback from rtk_rewrite; unsupported commands stay unmutated, supported commands rewrite to toksi rtk
- [2026-10-30T21:09:43Z] Opened: CLI test issue
- [2026-10-30T21:09:51Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T21:09:57Z] Fix confirmed: Fixed assertion in test_foo
