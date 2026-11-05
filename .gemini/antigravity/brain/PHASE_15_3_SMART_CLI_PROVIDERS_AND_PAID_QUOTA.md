# Phase 15.3: Smart Local Agent CLI Providers & Paid Quota Enforcement

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, and .gemini/antigravity/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md first to understand the global master architecture and BYOK routing.

Execute Phase 15.3: Smart Local Agent CLI Providers & Paid Quota Enforcement.
Targets:
  - crates/toksi/src/gateway/agent_cli/mod.rs
  - crates/toksi/src/gateway/agent_cli/tier_check.rs
  - crates/toksi/src/gateway/agent_cli/antigravity.rs
  - crates/toksi/src/gateway/agent_cli/copilot.rs
  - crates/toksi/src/gateway/agent_cli/claude.rs
  - crates/toksi/src/gateway/agent_cli/cursor.rs
  - crates/toksi/src/gateway/agent_cli/grok.rs
  - crates/toksi/src/gateway/providers/mod.rs
  - crates/toksi/src/gateway/loadbalancer.rs
  - crates/toksi/src/gateway/keys.rs

Requirements:
1. First-Class Local Agent CLI Providers:
   - Secure and register all 5 Local Agent CLIs as first-class peer BYOK providers:
     * `AntigravityCli` (`agy.exe` / Google Gemini Code Assist)
     * `CopilotCli` (`copilot.exe` / GitHub Copilot CLI)
     * `ClaudeCli` (`claude` / Claude Code CLI)
     * `CursorBridge` (`cursor` / Cursor IDE RPC Bridge)
     * `GrokCli` (`grok.exe` / xAI Grok CLI)
   - Smart Background Launch: Query OS Keyring (`advapi32.dll` on Windows, Keychain on macOS, Secret Service on Linux) and local session files (`~/.gemini/`, `~/.claude.json`, `gh auth token`). Auto-launch CLI helper in background detached mode and verify chat reachability (`/livez`, ConnectRPC health check).
2. Strict Paid Quota Enforcement (No Free Accounts for BYOK Providers):
   - Mandate: All BYOK providers (remote API keys and Local Agent CLI bridges) MUST have an active commercial paid tier or subscription quota.
   - Detect and disqualify free-tier or trial accounts (e.g. unbilled Google AI Studio trial keys with 15 RPM throttles, expired OpenAI trial credits, free Copilot trials, or unbilled accounts).
   - Graceful Inform & Zero-Fail Policy:
     * When a free tier is detected, Toksi NEVER crashes, panics, fails, or throws breaking errors.
     * Log a clear non-blocking notice:
       `[INFO] Provider '<Name>' is on a Free Tier (unbilled/rate-limited). Paid quota required for BYOK gateway pool. Bypassing from active pool without error.`
     * Mark provider as `FreeTierDisqualified` in SQLite `provider_keys`, and gracefully fall back to the next healthy paid provider or allow direct local agent execution.
3. Gateway Activation Guard & Zero-Provider Passthrough:
   - LLM routing gateway and load balancer only engage active dispatch when `active_paid_provider_count >= 1`.
   - If only an agent CLI is launched (e.g. `agy.exe` or `copilot.exe`), route cleanly through it without requiring external credit card API keys.
   - If zero providers configured, pass through traffic gracefully without errors.
4. Upstream Vault Audit & Bifrost Parity:
   - Verify all 39+ BYOK providers from `neo/inspect-vault/bifrost` (including `Bifrost` self-hosted/aggregator, `AWS SageMaker`, `AWS Bedrock Mantle`).
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. `tier_check.rs` Implementation
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaTier {
    PaidCommercial,
    FreeTierDisqualified,
    Unknown,
}

pub fn verify_provider_quota_tier(provider: &str, key_or_session: &str) -> QuotaTier {
    // 1. Antigravity: Check Google Cloud paid billing project or Gemini Code Assist subscription
    // 2. Copilot: Check active GitHub Copilot Business / Enterprise / Pro subscription
    // 3. Claude Code: Check Claude Pro, Team, or Enterprise plan
    // 4. Cursor: Check Cursor Pro or Business active subscription
    // 5. Grok: Check positive credit balance / commercial tier
    // If free tier or unbilled, return QuotaTier::FreeTierDisqualified
}
```

### 2. Smart Background CLI Bridge Launcher
In `crates/toksi/src/gateway/agent_cli/mod.rs`:
- Check OS Keyring for stored tokens.
- Check local session configs (`~/.gemini/`, `~/.claude.json`, etc.).
- If credentials exist and bridge is not running, spawn detached child process and poll `/livez` or RPC ping until ready.

