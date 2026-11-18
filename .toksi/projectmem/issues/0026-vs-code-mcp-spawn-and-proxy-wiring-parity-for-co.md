# Issue #0026 - VS Code MCP spawn and proxy wiring parity for Copilot and Antigravity streaming

**Created:** 2026-11-02T19:38:39Z
**Location:** `crates/toksi/src/agents/copilot.rs:2478 (detect_copilot_proxy)`
**Commit:** `e0bc5f7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-02T19:38:39Z] Opened: VS Code MCP spawn and proxy wiring parity for Copilot and Antigravity streaming
- [2026-11-02T19:43:24Z] Attempt (worked): Fixed VS Code extension detection, Copilot proxy autostart/lifecycle wiring, run-mcp proxy ensure, and HTTP/1.1 chunked streaming in byok_gateway
- [2026-11-02T19:44:03Z] Fix confirmed: Resolved Copilot proxy startup, VS Code MCP proxy boot, and Antigravity streaming timeouts in Toksi
- [2026-10-30T23:49:59Z] Opened: CLI test issue
- [2026-10-30T23:50:06Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T23:50:12Z] Fix confirmed: Fixed assertion in test_foo
