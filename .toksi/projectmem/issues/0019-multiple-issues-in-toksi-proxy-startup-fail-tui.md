# Issue #0019 - Multiple issues in toksi: proxy startup fail, TUI hangs, agent wiring, dashboard telemetry

**Created:** 2026-11-01T14:17:53Z
**Location:** `crates/toksi`
**Commit:** `1d68c24`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T14:17:53Z] Opened: Multiple issues in toksi: proxy startup fail, TUI hangs, agent wiring, dashboard telemetry
- [2026-11-01T15:31:03Z] Fix confirmed: Fixed proxy serve args and parser, decoupled TUI probe thread, unified antigravity 4-directory topology, purged tokless MCP runners, wired GEMINI.md in-tree rules
- [2026-10-30T20:32:56Z] Opened: CLI test issue
- [2026-10-30T20:33:02Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T20:33:08Z] Fix confirmed: Fixed assertion in test_foo
