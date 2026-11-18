# Issue #0023 - Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI

**Created:** 2026-11-01T19:19:31Z
**Location:** `crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)`
**Commit:** `fddf486`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-01T19:19:31Z] Opened: Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI
- [2026-11-01T19:19:56Z] Attempt (partial): Check Windows registry env for Antigravity, enable Copilot CLI provider config wired check, and update TUI/Dashboard to reflect proxy routing state
- [2026-11-01T19:52:49Z] Fix confirmed: Fixed Antigravity & Copilot Headroom proxy routing and status detection across dashboard, TUI, and CLI
- [2026-10-30T23:27:35Z] Opened: CLI test issue
- [2026-10-30T23:27:43Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T23:27:49Z] Fix confirmed: Fixed assertion in test_foo
