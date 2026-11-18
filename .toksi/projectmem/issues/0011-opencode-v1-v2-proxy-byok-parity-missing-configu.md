# Issue #0011 - OpenCode v1/v2 proxy & BYOK parity missing: ConfigureOpenCodeProxyV2, RemoveOpenCodeProxyV2, OpenCodeProxyV2Wired, retrieve tool stash, and proxy_agent opencode2 spec

**Created:** 2026-10-26T23:26:16Z
**Location:** `crates/tokless/src/agents/opencode.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:26:16Z] Opened: OpenCode v1/v2 proxy & BYOK parity missing: ConfigureOpenCodeProxyV2, RemoveOpenCodeProxyV2, OpenCodeProxyV2Wired, retrieve tool stash, and proxy_agent opencode2 spec
- [2026-10-27T02:45:30Z] Fix confirmed: Verified and integrated OpenCode v1 and v2 proxy and BYOK parity with complete specs and stash handling
- [2026-10-29T19:53:29Z] Opened: CLI test issue
- [2026-10-29T19:53:35Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T19:53:41Z] Fix confirmed: Fixed assertion in test_foo
