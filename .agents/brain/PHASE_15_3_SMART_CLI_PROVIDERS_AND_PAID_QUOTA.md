# Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/bifrost is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new provider integrations, aggregator features, and quota handlers into consolidated toksi.

Execute Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement.
Source: neo/inspect-vault/bifrost (BYOK provider registry & aggregators)
Targets:
  - crates/toksi/src/gateway/agent_cli/mod.rs
  - crates/toksi/src/gateway/agent_cli/tier_check.rs
  - crates/toksi/src/gateway/agent_cli/antigravity.rs
  - crates/toksi/src/gateway/agent_cli/copilot.rs
  - crates/toksi/src/gateway/agent_cli/claude.rs
  - crates/toksi/src/gateway/agent_cli/codex.rs
  - crates/toksi/src/gateway/agent_cli/cursor.rs
  - crates/toksi/src/gateway/agent_cli/grok.rs
  - crates/toksi/src/gateway/providers/aggregators/bifrost.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Strict Paid Quota Enforcement: All BYOK providers must have an active paid commercial tier. Free-tier accounts (unbilled AI Studio trial keys, expired credits, free Copilot trials) are strictly disqualified from active BYOK gateway rotation.
4. Graceful Inform & Zero-Fail Policy: Detecting an unpaid account MUST NEVER crash, fail, bug out, or panic. Emit a clear non-blocking informative log:
   `[INFO] Provider '<Name>' is on a Free Tier (unbilled/rate-limited). Paid quota required for BYOK gateway pool. Bypassing from active pool without error.`
   Gracefully fall back to the next healthy paid provider in the cascade or route the agent directly.
5. Gateway Activation Safety Guard: The LLM routing gateway and load balancer only engage active dispatch when at least one healthy paid BYOK provider is configured. If zero are configured, requests pass through cleanly without error.

Requirements:
1. Register 6 Local Agent CLIs as First-Class BYOK Providers:
   - `AntigravityCli` (`agy.exe` / Google Gemini Code Assist)
   - `CopilotCli` (`copilot.exe` / GitHub Copilot CLI)
   - `ClaudeCli` (`claude` / Claude Code CLI)
   - `CodexCli` (`codex` / OpenAI Codex CLI)
   - `CursorBridge` (`cursor` / Cursor IDE RPC Bridge)
   - `GrokCli` (`grok.exe` / xAI Grok CLI)
2. Smart Background Launch:
   - Query OS Keyring credentials and local session states.
   - When authenticated, launch CLI bridge process in detached background mode (`CREATE_NO_WINDOW` on Windows, `setsid()` on Unix).
   - Verify chat message reachability (`/livez` and ConnectRPC health checks).
3. Billing & Subscription Validator (`tier_check.rs`):
   - Check paid commercial tier vs free/unbilled trial for all providers.
   - Flag free-tier accounts as `FreeTierDisqualified` in SQLite `provider_keys`.
4. Bifrost BYOK Provider Integration:
   - Implement `gateway/providers/aggregators/bifrost.rs` with health probes, failover routing, and load balancing.
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Smart CLI Bridge Manager** (`src/gateway/agent_cli/mod.rs`): Spawns agent CLI child processes in detached mode without console popup windows, querying `/livez` to verify readiness.
- **Paid Quota Validator** (`src/gateway/agent_cli/tier_check.rs`): Inspects provider responses and account metadata to ensure active commercial billing. Disqualifies free/unbilled keys from the gateway pool without panicking.
- **Bifrost Aggregator Client** (`src/gateway/providers/aggregators/bifrost.rs`): Connects to self-hosted or remote Bifrost gateway instances as a peer aggregator.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- agent_cli` passes 100%.
- Unbilled mock provider key logs informative bypass notice and successfully activates next paid provider in cascade.
