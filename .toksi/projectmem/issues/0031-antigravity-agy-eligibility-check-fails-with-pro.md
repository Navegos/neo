# Issue #0031 - Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing

**Created:** 2026-11-04T15:37:18Z
**Location:** `crates/toksi/src/headroom/proxy.rs:625 (serve_headroom_connection)`
**Commit:** `15bb1ea`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T15:37:18Z] Opened: Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing
- [2026-11-04T16:01:04Z] Attempt (worked): Added is_proto_or_connect_request, format_connect_error, translated upstream HTML errors into ConnectRPC JSON error format, and routed all cloudcode endpoints to aicode_url
- [2026-11-04T16:01:21Z] Fix confirmed: Preserve binary Protocol Buffer framing in and out, translate upstream 502/HTML errors to structured ConnectRPC error JSON (eliminating proto syntax error), and route all Cloud Code/Jetski RPCs to aicode.googleapis.com
- [2026-10-31T00:17:27Z] Opened: CLI test issue
- [2026-10-31T00:17:33Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T00:17:40Z] Fix confirmed: Fixed assertion in test_foo
