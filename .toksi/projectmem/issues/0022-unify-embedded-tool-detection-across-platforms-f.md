# Issue #0022 - Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom

**Created:** 2026-11-01T18:39:51Z
**Location:** `crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)`
**Commit:** `1d68c24`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T18:39:51Z] Opened: Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom
- [2026-11-01T18:40:04Z] Attempt (partial): Update doctor_stats, versions, doctor_probe, projectmem, and rtk tool to resolve in-tree toksi binary for embedded tools
- [2026-11-01T18:47:53Z] Fix confirmed: Unified embedded tool resolution across Windows, Linux, and Mac: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom
- [2026-10-30T21:38:38Z] Opened: CLI test issue
- [2026-10-30T21:38:45Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T21:38:52Z] Fix confirmed: Fixed assertion in test_foo
