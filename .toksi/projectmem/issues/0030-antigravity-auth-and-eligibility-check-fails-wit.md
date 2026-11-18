# Issue #0030 - Antigravity auth and eligibility check fails with proto syntax error line 1:1 invalid value due to ConnectRPC route discrimination, wrong upstream routing, and HTTP chunking of protobuf responses

**Created:** 2026-11-04T13:28:36Z
**Location:** `crates/toksi/src/headroom/proxy.rs:619 (serve_headroom_connection)`
**Commit:** `f8da2d3`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T13:28:36Z] Opened: Antigravity auth and eligibility check fails with proto syntax error line 1:1 invalid value due to ConnectRPC route discrimination, wrong upstream routing, and HTTP chunking of protobuf responses
- [2026-11-04T13:44:07Z] Fix confirmed: Expanded route discrimination for ConnectRPC and AICode service prefixes, routed aicode endpoints to aicode.googleapis.com, prevented false chunked transfer-encoding on unary protobuf/ConnectRPC responses, and expanded keyring targets for antigravity_user and jetski-standalone-oauth-token
- [2026-10-31T00:02:37Z] Opened: CLI test issue
- [2026-10-31T00:02:43Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T00:02:50Z] Fix confirmed: Fixed assertion in test_foo
