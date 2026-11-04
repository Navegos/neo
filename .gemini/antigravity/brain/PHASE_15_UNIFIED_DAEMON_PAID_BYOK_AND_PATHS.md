# Phase 15: Unified 5-Port Daemon, Paid Quota Enforcement & Smart Agent CLI Providers

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, .projectmem/plan.md, and .gemini/antigravity/brain/PHASE_15_UNIFIED_DAEMON_PAID_BYOK_AND_PATHS.md first to understand the global master architecture and cross-tool coupling.

Execute Phase 15: Unified 5-Port Daemon, Paid Quota Enforcement & Smart Agent CLI Providers.
Targets:
  - crates/toksi/src/commands/daemon.rs
  - crates/toksi/src/headroom/{proxy.rs, copilot_daemon.rs}
  - crates/toksi/src/gateway/{mod.rs, providers/mod.rs, agent_cli/mod.rs, agent_cli/tier_check.rs, loadbalancer.rs, keys.rs}
  - crates/toksi/src/util/paths.rs
  - crates/toksi/src/db/mod.rs
  - crates/toksi/src/rules/{mod.rs, sync.rs}
  - crates/toksi/src/main.rs

Requirements:
1. Unified 5-Port Daemon Supervision:
   - Upgrade `toksi daemon start` to supervise all 5 dedicated endpoints simultaneously:
     * Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
     * Port 8789: GitHub Copilot VS Code Proxy (DEFAULT_COPILOT_PROXY_PORT)
     * Port 8790: GitHub Copilot CLI Proxy (DEFAULT_COPILOT_CLI_PROXY_PORT)
     * Port 9099: Enterprise AI Gateway & Drop-in SDKs (/v1/chat/completions, /v1/messages, /metrics)
     * Port 9100: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)
   - Ensure `is_daemon_running()`, `daemon_status()`, and `daemon_stop()` handle all 5 ports cleanly with automatic teardown of Windows HKCU\Environment variables.
2. First-Class Local Agent CLI Providers & Smart Background Launch:
   - Secure and register all 5 Local Agent CLIs as first-class peer BYOK providers:
     * AntigravityCli (agy.exe)
     * CopilotCli (copilot.exe)
     * ClaudeCli (claude)
     * CursorBridge (cursor)
     * GrokCli (grok.exe)
   - When authenticated via OS Keyring/session tokens, smart-launch the CLI helper process in the background in detached mode and verify chat endpoint reachability (/livez / ConnectRPC).
3. Paid Quota Enforcement & Free-Tier Disqualification:
   - Implement `tier_check.rs` checking account billing/subscription status for all providers.
   - Enforce mandate: all BYOK providers must have an active paid commercial tier or subscription quota. Free accounts are disqualified from the active gateway rotation pool.
   - Graceful Inform & Zero-Fail Policy: Never panic, crash, or fail with a breaking bug. Log an informative notice ([INFO] Provider 'X' is on Free tier. Paid account required for BYOK rotation; bypassing gracefully) and smoothly fall back to another paid provider or direct agent execution.
   - Gateway engages when active_paid_providers >= 1. If zero configured, pass through traffic gracefully without errors.
4. Ground-Truth Token Accounting:
   - Extract real prompt/completion/cached token usage directly from live SSE response streams and upstream response headers (x-ratelimit-remaining-tokens, anthropic-ratelimit-requests-remaining, etc.) in `telemetry/collector.rs`. Zero virtual/synthetic math.
5. Rules, Skills & Instructions Sync:
   - Implement universal synchronizer updating `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md` across both project workspace directories and user config homes for all 17 agents.
6. Cross-Platform Forward-Slash Normalization:
   - Implement `normalize_path()` converting all backslashes to `/` across all path utilities, logs, database records, and CLI outputs for Git Bash / MSYS2 compatibility.
7. Database Location:
   - Ensure default database path is `~/.toksi/toksi.db` (users/.toksi) with automated startup migration from legacy paths.
8. Verification:
   - Ensure 0 compiler warnings, all tests pass, and Copilot ports 8789/8790 launch correctly.
```

---

## Detailed Architectural Requirements

### 1. Dedicated 5-Port Supervision
In `crates/toksi/src/commands/daemon.rs`:
- Upgrade `daemon_run_foreground()`:
  * Check and spawn `proxy::start_proxy()` on Port 8787.
  * Check and spawn `copilot_daemon::ensure_copilot_proxy_up()` on Port 8789.
  * Check and spawn Copilot CLI proxy on Port 8790.
  * Check and spawn Enterprise Gateway on Port 9099.
  * Check and spawn Web UI Dashboard on Port 9100.
- Update `daemon_status()` to report health status of all 5 listeners.
- Update `daemon_stop()` to terminate all background listeners and cleanly teardown `HKCU\Environment`.

### 2. First-Class Local Agent CLI Providers
In `crates/toksi/src/gateway/agent_cli/`:
- Secure registration of:
  * `AntigravityCli`: routes Google Cloud Code / Gemini requests through local authenticated `agy` session.
  * `CopilotCli`: routes Copilot chat/completion requests through local `copilot` CLI session.
  * `ClaudeCli`: routes Claude Messages through `~/.claude.json` local session.
  * `CursorBridge`: routes completions through Cursor IDE language model RPC socket.
  * `GrokCli`: routes completions through local `grok` CLI session.
- Implement smart detached background launch with process supervisor.

### 3. Paid Quota Enforcement Policy
In `crates/toksi/src/gateway/agent_cli/tier_check.rs`:
- Quota and plan validation:
  * Google Gemini: requires paid GCP project billing or Gemini Code Assist license.
  * GitHub Copilot: requires active Business, Enterprise, or Pro subscription.
  * Claude Code: requires Pro, Team, or Enterprise plan.
  * Cursor: requires Pro or Business subscription.
  * xAI Grok: requires active paid API credit balance.
- Free trial accounts or zero-credit accounts are marked `FreeTierDisqualified`.
- Emits non-blocking info notice. Zero crashes, zero panics, zero breaking bugs.

### 4. Forward-Slash Path Normalization
In `crates/toksi/src/util/paths.rs`:
- Global `normalize_path(p: &str) -> String` replacing all `\` with `/`.
- Ensure all path helpers return forward-slash strings.

### 5. Verification
Run:
```bash
cargo check -p toksi
cargo test -p toksi
```

