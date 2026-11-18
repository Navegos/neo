# Issue #0034 - Bypass non-LLM authentication and ConnectRPC management endpoints directly without intercepting headers or proxying, capturing only LLM processing in and out

**Created:** 2026-11-04T16:55:58Z
**Location:** `crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)`
**Commit:** `15bb1ea`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T16:55:58Z] Opened: Bypass non-LLM authentication and ConnectRPC management endpoints directly without intercepting headers or proxying, capturing only LLM processing in and out
- [2026-10-31T09:25:09Z] Opened: CLI test issue
- [2026-10-31T09:25:16Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T09:25:22Z] Fix confirmed: Fixed assertion in test_foo
