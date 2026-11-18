# Issue #0024 - Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth

**Created:** 2026-11-01T20:33:19Z
**Location:** `crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)`
**Commit:** `fddf486`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T20:33:19Z] Opened: Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth
- [2026-11-01T20:38:50Z] Attempt (worked): Forward non-LLM requests directly to upstream service without payload modification and preserve query params
- [2026-11-01T20:39:06Z] Fix confirmed: Headroom proxy detects non-LLM requests, bypasses prompt modifications, preserves query strings, and routes Cloud Code endpoints to https://cloudcode-pa.googleapis.com
- [2026-10-30T23:34:49Z] Opened: CLI test issue
- [2026-10-30T23:34:56Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T23:35:02Z] Fix confirmed: Fixed assertion in test_foo
