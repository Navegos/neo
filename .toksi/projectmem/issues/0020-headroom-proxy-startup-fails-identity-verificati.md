# Issue #0020 - Headroom proxy startup fails identity verification when routing disabled

**Created:** 2026-11-01T15:56:07Z
**Location:** `crates/toksi/src/headroom/proxy.rs:498 (run_proxy_foreground)`
**Commit:** `1d68c24`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T15:56:07Z] Opened: Headroom proxy startup fails identity verification when routing disabled
- [2026-11-01T15:57:40Z] Attempt (worked): Remove proxy_routing_enabled check in run_proxy_foreground, enable routing in start_proxy, extend identity timeout and error fallback on Windows
- [2026-11-01T15:59:19Z] Fix confirmed: Prevent premature exit in run_proxy_foreground by removing routing_enabled guard on serve mode, enable routing preference on start_proxy, and improve Windows identity verification fallback/timeouts
- [2026-10-30T21:03:11Z] Opened: CLI test issue
- [2026-10-30T21:03:18Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T21:03:25Z] Fix confirmed: Fixed assertion in test_foo
