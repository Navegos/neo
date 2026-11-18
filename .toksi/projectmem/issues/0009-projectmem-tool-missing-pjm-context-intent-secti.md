# Issue #0009 - ProjectMem tool missing pjm context + intent sections + global gotchas, kilo/opencode2 extensions, before_agent_start/transform hooks, and git worktree post-commit hook

**Created:** 2026-10-26T23:25:23Z
**Location:** `crates/tokless/src/tools/projectmem.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:25:23Z] Opened: ProjectMem tool missing pjm context + intent sections + global gotchas, kilo/opencode2 extensions, before_agent_start/transform hooks, and git worktree post-commit hook
- [2026-10-27T02:46:58Z] Fix confirmed: Projectmem tool equipped with session push, kilo/opencode2 extensions, and rule files refresh
- [2026-10-29T19:28:51Z] Opened: CLI test issue
- [2026-10-29T19:29:02Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T19:29:07Z] Fix confirmed: Fixed assertion in test_foo
