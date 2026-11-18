# Issue #0010 - MCP adapters and opencode v2 regressions: opencode2 MCP writes v1 root shape instead of mcp.servers, codex MCP block parser missing, cursor MCP permissions missing

**Created:** 2026-10-26T23:25:31Z
**Location:** `crates/tokless/src/agents/mcp_adapters.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:25:31Z] Opened: MCP adapters and opencode v2 regressions: opencode2 MCP writes v1 root shape instead of mcp.servers, codex MCP block parser missing, cursor MCP permissions missing
- [2026-10-27T02:45:57Z] Fix confirmed: Verified and wired MCP adapters for opencode2 with mcp.servers, codex TOML block parser, and cursor MCP permissions
- [2026-10-29T19:45:08Z] Opened: CLI test issue
- [2026-10-29T19:45:14Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T19:45:20Z] Fix confirmed: Fixed assertion in test_foo
