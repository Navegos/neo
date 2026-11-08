# Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement.
Targets:
  - crates/toksi/src/gateway/agent_cli/mod.rs
  - crates/toksi/src/gateway/agent_cli/tier_check.rs
  - crates/toksi/src/gateway/agent_cli/{antigravity.rs, copilot.rs, claude.rs, codex.rs, cursor.rs, grok.rs}
  - crates/toksi/src/gateway/providers/aggregators/bifrost.rs
  - crates/toksi/src/gateway/loadbalancer.rs
  - crates/toksi/src/gateway/keys.rs
  - crates/toksi/src/gateway/routing.rs

Requirements:
1. First-Class Local Agent CLI Providers:
   - Secure and register all 6 Local Agent CLIs as first-class peer BYOK providers:
     * AntigravityCli (agy.exe)
     * CopilotCli (copilot.exe)
     * ClaudeCli (claude)
     * CodexCli (codex)
     * CursorBridge (cursor)
     * GrokCli (grok.exe)
   - When authenticated via OS Keyring/session tokens, smart-launch the CLI helper process in the background and verify chat endpoint reachability (/livez / ConnectRPC).
2. Bifrost Upstream Provider Integration:
   - Integrate Bifrost provider client into `gateway/providers/aggregators/bifrost.rs` with health probes, routing, and load balancing.
3. Strict Paid Quota Enforcement:
   - Implement `tier_check.rs` checking account billing/subscription status for all providers.
   - Mandate: all BYOK providers must have an active paid commercial tier or subscription quota. Free accounts are disqualified from active gateway rotation.
   - Graceful Inform & Zero-Fail Policy: Never panic, crash, or fail with a breaking bug. Log an informative notice: `[INFO] Provider '<Name>' is on a Free Tier (unbilled/rate-limited). Paid quota required for BYOK gateway pool. Bypassing from active pool without error.`
   - Gracefully fall back to another active paid provider in the cascade or direct local agent execution.
4. Gateway Activation Guard:
   - Gateway engages when `active_paid_providers >= 1`.
   - If zero providers are configured, requests pass through gracefully without throwing errors or breaking local agent execution.
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Paid Tier Check Guard in `tier_check.rs`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderBillingTier {
    PaidCommercial,
    PaidSubscription,
    FreeTierDisqualified,
    Unknown,
}

pub fn check_provider_tier(provider_name: &str, metadata: &serde_json::Value) -> ProviderBillingTier {
    match provider_name {
        "antigravity" | "gemini_agy" => {
            // Check paid GCP project billing or Gemini Code Assist Enterprise/Standard
            if metadata.get("billing_enabled").and_then(|v| v.as_bool()).unwrap_or(false)
                || metadata.get("tier").and_then(|v| v.as_str()) == Some("paid")
            {
                ProviderBillingTier::PaidCommercial
            } else {
                ProviderBillingTier::FreeTierDisqualified
            }
        }
        "copilot" | "copilot_cli" => {
            // Check Copilot Business, Enterprise, or Pro
            let plan = metadata.get("plan").and_then(|v| v.as_str()).unwrap_or("");
            if plan.contains("business") || plan.contains("enterprise") || plan.contains("individual") {
                ProviderBillingTier::PaidSubscription
            } else {
                ProviderBillingTier::FreeTierDisqualified
            }
        }
        "claude" | "claude_cli" => {
            // Check Claude Pro, Team, or Enterprise
            let plan = metadata.get("plan").and_then(|v| v.as_str()).unwrap_or("");
            if plan.contains("pro") || plan.contains("team") || plan.contains("enterprise") {
                ProviderBillingTier::PaidSubscription
            } else {
                ProviderBillingTier::FreeTierDisqualified
            }
        }
        "codex" | "codex_cli" => {
            // Check OpenAI paid billing tier or ChatGPT Plus/Team/Enterprise
            let plan = metadata.get("plan").and_then(|v| v.as_str()).unwrap_or("");
            let has_credits = metadata.get("has_paid_credits").and_then(|v| v.as_bool()).unwrap_or(false);
            if has_credits || plan.contains("plus") || plan.contains("team") || plan.contains("enterprise") || plan.contains("paid") {
                ProviderBillingTier::PaidCommercial
            } else {
                ProviderBillingTier::FreeTierDisqualified
            }
        }
        "cursor" | "cursor_bridge" => {
            // Check Cursor Pro or Business
            let plan = metadata.get("plan").and_then(|v| v.as_str()).unwrap_or("");
            if plan.contains("pro") || plan.contains("business") {
                ProviderBillingTier::PaidSubscription
            } else {
                ProviderBillingTier::FreeTierDisqualified
            }
        }
        "grok" | "grok_cli" => {
            // Check Grok paid subscription or active API credits
            let plan = metadata.get("plan").and_then(|v| v.as_str()).unwrap_or("");
            let has_credits = metadata.get("has_paid_credits").and_then(|v| v.as_bool()).unwrap_or(false);
            if has_credits || plan.contains("premium") || plan.contains("supergrok") || plan.contains("paid") {
                ProviderBillingTier::PaidCommercial
            } else {
                ProviderBillingTier::FreeTierDisqualified
            }
        }
        _ => ProviderBillingTier::PaidCommercial,
    }
}
```
