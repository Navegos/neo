# Issue #0028 - VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream

**Created:** 2026-11-03T20:20:51Z
**Location:** `crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)`
**Commit:** `41764ac`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-03T20:20:51Z] Opened: VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream
- [2026-11-03T20:21:46Z] Fix confirmed: Added HTTP/1.1 keep-alive loop, Google Cloud Code route discrimination, stripped Content-Encoding on decompressed stream, and added VS Code settings candidate path
- [2026-11-03T20:53:41Z] Fix confirmed: Eliminated false chunking of HTTP/2 responses lacking Content-Length that broke protobuf JSON parsing with invalid value []. Wired GatewayProxyOrchestrator semantic cache into proxy loop.
- [2026-10-30T23:58:56Z] Opened: CLI test issue
- [2026-10-30T23:59:02Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T23:59:08Z] Fix confirmed: Fixed assertion in test_foo
