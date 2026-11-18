# Issue #0035 - Phase 08: Purge external runtime downloaders and rewire agent harnesses to in-tree modules

**Created:** 2026-11-05T14:39:48Z
**Location:** `crates/toksi/src/util/`
**Commit:** `2ef4758`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-05T14:39:48Z] Opened: Phase 08: Purge external runtime downloaders and rewire agent harnesses to in-tree modules
- [2026-11-06T00:50:00Z] Attempt (worked): Purged downloader references, simplified headroom paths, wired MCP aliases to in-tree toksi, verified 5/5 phase 8 tests and 321 unit tests pass
- [2026-11-06T00:50:17Z] Fix confirmed: Deleted runtime downloaders (npminstall, nodewin, nodelinux, uv, projectmem), wired 17 agents and tools to native in-tree Rust modules, enforced system detection policy via which::which, 0 compiler warnings, all tests pass
- [2026-10-31T09:37:43Z] Opened: CLI test issue
- [2026-10-31T09:37:50Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T09:37:56Z] Fix confirmed: Fixed assertion in test_foo
