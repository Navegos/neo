# #0023 Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI

- 2026-11-01T19:19:31Z `issue`: Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)]
- 2026-11-01T19:19:56Z `attempt`: Check Windows registry env for Antigravity, enable Copilot CLI provider config wired check, and update TUI/Dashboard to reflect proxy routing state [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)] (partial)
- 2026-11-01T19:52:49Z `fix`: Fixed Antigravity & Copilot Headroom proxy routing and status detection across dashboard, TUI, and CLI
