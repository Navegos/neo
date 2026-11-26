# Phase 17.5: Native `toksi` Coding CLI & Swarm (`src/agent_runtime/`), `DirectWireBus`, & 16 In-Chat Slash Commands

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Section 13.3), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md first.

Execute Phase 17.5: Native `toksi` Coding CLI & Swarm (`src/agent_runtime/`), `DirectWireBus`, & 16 In-Chat Slash Commands.
Sources:
  - neo/inspect-vault/hermes-agent
  - neo/inspect-vault/openclaw
  - neo/inspect-vault/openclaw-rs
  - neo/inspect-vault/mofa
  - neo/inspect-vault/mofaclaw
  - neo/inspect-vault/crewAI
  - neo/inspect-vault/codex (codex-rs)
  - neo/inspect-vault/{crustly, crush, goose, grok-build, gemini-cli}
Targets:
  - crates/toksi/src/agent_runtime/{mod.rs, loop_engine.rs, hermes.rs, openclaw.rs, mofa.rs, crew.rs, scheduler.rs, sandbox.rs, patch.rs, slash.rs, state.rs}
  - crates/toksi/src/commands/{agent_runtime.rs, plugin.rs, mod.rs} (preserve `commands/agent.rs` install/health/fix lifecycle!)
  - crates/toksi/src/plugins/{mod.rs, slash_router.rs, hooks.rs}
  - crates/toksi/src/tools/{direct_wire.rs, multi_protocol.rs}
  - crates/toksi/src/headroom/proxy.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly (Windows Restricted Token / Job Objects helpers in `crates/toksi-sys`); zero `.unwrap()`/`.expect()`/`panic!()`.
2. CLI Surface Preservation: Keep existing `toksi agent list|install|health|mismatch|upgrade|fix|restore|remove|edit` intact in `commands/agent.rs`; expose top-level `toksi chat`, `toksi run "<prompt>"`, `toksi exec "<prompt>"`, and `toksi swarm "<goal>"` (with `toksi agent chat|run|swarm|exec` aliases).
3. Fast Non-Blocking Slash Commands (`< 5ms`): In-chat slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`) MUST intercept both native `toksi chat` AND external hooked agent chats (`agy`, `claude`, `codex`, `cursor`, `grok`, `opencode`, etc.) cleanly without hangs, aborts, or infinite loops.

Requirements:
1. Build Native Coding CLI & Swarm Engine (`crates/toksi/src/agent_runtime/`):
   - **`hermes.rs`**: `TurnFacadeMixin` durable turn leases, `RUN_BUDGET_WRAPUP_NOTICE` (80% wall-clock notice), `RunawayStreamWatch` (`MIN_FRAGMENT_LENGTH=400`, `REPEAT_WINDOW=60`, `MIN_REPEAT_COUNT=5`, `STOP_PATH_MIN_CHARS=16000`), `StopGateVerdict` post-answer barrier, `ContextEngine` `[CONTEXT COMPACTION â€” REFERENCE ONLY]`, `StreamingContextScrubber`, `MoAChatCompletions`, `_BackgroundReviewRun`.
   - **`openclaw.rs` + `state.rs`**: 6-detector `ToolLoopDetector` (`generic_repeat`, `argument_churn`, `unknown_tool_repeat=10`, `known_poll_no_progress`, `global_circuit_breaker=30`, `ping_pong`, normalized SHA-256 history size `30`, `CRITICAL=20`), `PendingMessageQueue` rollback, periodic `advisor` fork, BLAKE2b-256 `EventLog` + CRDT merge (`openclaw-rs`).
   - **`mofa.rs` + `crew.rs` + `scheduler.rs`**: `MoFAAgent`/`AgentContext`/`InterruptSignal`, `SubtaskDAG` (`petgraph::DiGraph`) + wave `ParallelScheduler`, `SwarmHITLGate`, `TaskAnalyzer`, `ReflectionAgent` (max 3 rounds), `ProcessMode`, `ReasoningPlan`, `InterceptionPoint`/`HookAborted`, causal `EventBus`, single-sleep `CronService` + `HEARTBEAT.md`.
   - **`sandbox.rs` + `patch.rs`**: Native 3-OS sandbox (`bwrap`/seccomp/`Landlock`, macOS `.sbpl` `sandbox-exec`, Windows `WindowsRestrictedToken` + Job Objects via `toksi-sys`), `PermissionPolicy`, `ssrf_guard`, `SHELL_METACHARS` RBAC, Lark `ApplyPatchTool`, `HashlineEditTool`, unified `HookEventName` matrix, TOML/MD slash loader.
   - Wire directly out of the box ("off the back", zero manual config) via `DirectWireBus` and local `:8787`/`:9098`/`:9099`/`:18787` endpoints to Enterprise AI Gateway, BYOK & Vertex ADC Gateway, Protocol Bus (`MCP`/`A2A`/`ACP`/`SSE`), shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`), `src/plugins/`, `src/security/{adr,policy_opa,policy_cedar,iam_attestation,guardrails_validators,llm_guard}.rs`, `src/agent_runtime/sandbox.rs`, `src/commands/guard.rs` kill-switch, and all **495** consolidated `inspect-vault/*` tools forming the **35-component `<toksi-siagent>`** (`PHASE_18_1`â€“`PHASE_18_6`, **43** `.agents/brain/*.md` guides, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, externalized zero-bloat `<skills>` & `<advisory>`).
2. Ship `DirectWireBus` (`src/tools/direct_wire.rs`) & Upgrade `tools/multi_protocol.rs`:
   - `DirectWireBus`: Zero-config-file in-process interception for tools & agent-to-agent messages (`direct-wired` mode alongside optional `config-wired`).
   - Real `A2aTaskStore` v1.0 JSON-RPC 2.0 (`SendMessage`/`GetTask` + `"message/send"`/`"tasks/get"`, SHA-256 constant-time bearer check, 1 MiB cap, `(peer, messageId)` dedupe) + `SessionActorQueue` ACP v2.0.
3. Implement 16 In-Chat Slash Commands (`src/agent_runtime/slash.rs` + `src/plugins/slash_router.rs` + `headroom/proxy.rs`):
   - `/byok [filter]` -> lists user-configured BYOK providers & stores active BYOK step.
   - `/model [filter]` -> lists models discovered for the currently selected BYOK first.
   - `/effort [low|medium|high]` -> selects thinking effort AFTER model & scales Headroom compression (ignoring gracefully if unsupported).
   - `/byokmodel <byok> <model> [effort]` -> one-shot atomic persistence into `~/.toksi/toksi.db` (`byok_profiles` / `agent_profile_bindings`) + `~/.toksi/config.toml` sync.
   - `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk` -> instant `< 5ms` direct tool execution returning formatted structured response inside active chat turn (including when hooked through external agents like `agy`).
   - `/security [scan [path]|scan-skills [path]|guardrails|audit-evidence|adr|rules|audit-skills]` -> instant 41-language SAST/OSV + blind signature scan + ADR inspection + `skill-scanner` + `defenseclaw` + `<enforce>` + `agentskills` YAML frontmatter check (`PHASE_18_2`, `PHASE_18_5`).
   - `/advisory [search <q>|validate|prune-fp|build-shards|publish-pr|set-dir <path>]` -> O(1) prefix search against `Navegos/advisory` + 12-DB `UpstreamDeduplicator` + `DisclosureGate` PR workflow (`PHASE_18_2`).
   - `/skills [sync|list|import <dir>|heal|grade|scan|push|set-dir <path>]` -> externalized `Navegos/skills` + user skills sync, `skilldoctor` grading, `agentskills` healing, and `<enforce>` vetting (`PHASE_18_2`).
   - `/sandbox [run|create|exec|ls|inspect|stop|rm|policy|secret]` -> launch or enforce policy on isolated 3-OS native / `sandbox-runtime` / `sbx` agent sandboxes (`PHASE_18_5`).
   - `/plugin [list|load <id>|unload <id>|reload <id>|info <id>|scan <path>|enable <id>|disable <id>]` -> instant Unified Plugin System lifecycle dispatch & `<enforce>` pre-load gate (`PHASE_18_2`).
   - `/resume [plan_id]` -> hydrate atomic `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` and resume without re-running completed steps (`PHASE_18_3`).
4. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Verification Criteria
- `toksi run "echo ok"` and `toksi swarm "analyze"` execute with `ToolLoopDetector`, `RunawayStreamWatch`, and OS sandbox guards active without hangs.
- Typing `/byokmodel`, `/byok` -> `/model` -> `/effort`, and `/projectmem status` completes in `< 5ms`, updates `byok_profiles` / `agent_profile_bindings`, and respects unsupported effort gracefully.
- A2A JSON-RPC 2.0 `SendMessage`/`GetTask` and ACP `SessionActorQueue` round-trip cleanly over `DirectWireBus` and HTTP/SSE.

---

## Wave 18 (v5.2) Synchronization Addendum
- Expand `< 5ms` universal slash commands to **16**: `/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume` (plus plugin-registered `/plugin:*` mounts via `src/plugins/slash_router.rs`).
- Expose `:9098` Protocol Bus (`MCP` / `A2A` / `ACP` `agent-client-protocol` + `src/plugins/` mounts) and persist safe `temporal`/`restate`/`langgraph` plan checkpoints inside `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` across **35-component `<toksi-siagent>`** (**495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, `PHASE_18_1`â€“`PHASE_18_6`).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** `16` core slash commands implemented on disk (`CORE_SLASH_COMMANDS.len() == 16` kept green during Wave 19 doc sync); `PHASE_19_1`/`19_5`/`19_6` expand runtime to `21` commands (`/ctxmode`, `/workforce`, `/spec`, `/train`, `/serve`, `/voice`).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
