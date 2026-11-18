# Issue #0029 - RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json

**Created:** 2026-11-03T21:32:42Z
**Location:** `crates/toksi/src/agents/antigravity.rs:210 (antigravity_hooks_file)`
**Commit:** `41764ac`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-03T21:32:42Z] Opened: RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json
- [2026-11-03T22:29:37Z] Attempt (worked): Implemented run_ctx_hook with ctx-hook, context-mode-hook, and context-mode aliases in main.rs and ctx.rs
- [2026-11-03T22:29:49Z] Fix confirmed: Added ctx-hook and context-mode aliases in toksi CLI, installed fresh binary to C:/Programs/neo/bin/toksi.exe
- [2026-10-31T00:02:21Z] Opened: CLI test issue
- [2026-10-31T00:02:27Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T00:02:33Z] Fix confirmed: Fixed assertion in test_foo
