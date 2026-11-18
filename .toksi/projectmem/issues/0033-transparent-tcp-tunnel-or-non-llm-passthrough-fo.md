# Issue #0033 - Transparent TCP tunnel or non-LLM passthrough for Antigravity auth/eligibility, proxy only LLM generation requests

**Created:** 2026-11-04T16:52:09Z
**Location:** `crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)`
**Commit:** `15bb1ea`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T16:52:09Z] Opened: Transparent TCP tunnel or non-LLM passthrough for Antigravity auth/eligibility, proxy only LLM generation requests
- [2026-10-31T00:47:01Z] Opened: CLI test issue
- [2026-10-31T00:47:07Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T00:47:14Z] Fix confirmed: Fixed assertion in test_foo
