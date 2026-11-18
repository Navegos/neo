# Issue #0027 - toksi proxy reports model unavailable in VS Code antigravity extension while terminal works

**Created:** 2026-11-03T11:51:03Z
**Location:** `crates/toksi/src/main.rs:252 (run)`
**Commit:** `e0bc5f7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-03T11:51:03Z] Opened: toksi proxy reports model unavailable in VS Code antigravity extension while terminal works
- [2026-11-03T12:02:50Z] Attempt (worked): Fixed proxy target resolution: routes Gemini API /v1beta and models to generativelanguage instead of cloudcode, added CORS OPTIONS preflight handler, and fixed /v1 path stripping in byok_target_path
- [2026-11-03T12:03:15Z] Fix confirmed: Proxy routes /v1beta and gemini endpoints to generativelanguage even when x-goog-api-client has antigravity header. Added CORS preflight 204 handler and fixed /v1 path preservation in byok_target_path. Tested by test_non_llm_uri_and_target_resolution.
- [2026-10-30T23:58:48Z] Opened: CLI test issue
- [2026-10-30T23:58:54Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T23:59:00Z] Fix confirmed: Fixed assertion in test_foo
