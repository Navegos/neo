# Phase 17.5: Native `toksi` Coding CLI & Swarm (`src/agent_runtime/`), `DirectWireBus`, & 10 In-Chat Slash Commands

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Section 13.3), .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md first.

Execute Phase 17.5: Native `toksi` Coding CLI & Swarm (`src/agent_runtime/`), `DirectWireBus`, & 10 In-Chat Slash Commands.
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
  - crates/toksi/src/commands/{agent_runtime.rs, mod.rs} (preserve `commands/agent.rs` install/health/fix lifecycle!)
  - crates/toksi/src/tools/{direct_wire.rs, multi_protocol.rs}
  - crates/toksi/src/headroom/proxy.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly (Windows Restricted Token / Job Objects helpers in `crates/toksi-sys`); zero `.unwrap()`/`.expect()`/`panic!()`.
2. CLI Surface Preservation: Keep existing `toksi agent list|install|health|mismatch|upgrade|fix|restore|remove|edit` intact in `commands/agent.rs`; expose top-level `toksi chat`, `toksi run "<prompt>"`, `toksi exec "<prompt>"`, and `toksi swarm "<goal>"` (with `toksi agent chat|run|swarm|exec` aliases).
3. Fast Non-Blocking Slash Commands (`< 5ms`): In-chat slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`) MUST intercept both native `toksi chat` AND external hooked agent chats (`agy`, `claude`, `codex`, `cursor`, `grok`, etc.) cleanly without hangs, aborts, or infinite loops.

Requirements:
1. Build Native Coding CLI & Swarm Engine (`crates/toksi/src/agent_runtime/`):
   - **`hermes.rs`**: `TurnFacadeMixin` durable turn leases, `RUN_BUDGET_WRAPUP_NOTICE` (80% wall-clock notice), `RunawayStreamWatch` (`MIN_FRAGMENT_LENGTH=400`, `REPEAT_WINDOW=60`, `MIN_REPEAT_COUNT=5`, `STOP_PATH_MIN_CHARS=16000`), `StopGateVerdict` post-answer barrier, `ContextEngine` `[CONTEXT COMPACTION — REFERENCE ONLY]`, `StreamingContextScrubber`, `MoAChatCompletions`, `_BackgroundReviewRun`.
   - **`openclaw.rs` + `state.rs`**: 6-detector `ToolLoopDetector` (`generic_repeat`, `argument_churn`, `unknown_tool_repeat=10`, `known_poll_no_progress`, `global_circuit_breaker=30`, `ping_pong`, normalized SHA-256 history size `30`, `CRITICAL=20`), `PendingMessageQueue` rollback, periodic `advisor` fork, BLAKE2b-256 `EventLog` + CRDT merge (`openclaw-rs`).
   - **`mofa.rs` + `crew.rs` + `scheduler.rs`**: `MoFAAgent`/`AgentContext`/`InterruptSignal`, `SubtaskDAG` (`petgraph::DiGraph`) + wave `ParallelScheduler`, `SwarmHITLGate`, `TaskAnalyzer`, `ReflectionAgent` (max 3 rounds), `ProcessMode`, `ReasoningPlan`, `InterceptionPoint`/`HookAborted`, causal `EventBus`, single-sleep `CronService` + `HEARTBEAT.md`.
   - **`sandbox.rs` + `patch.rs`**: Native 3-OS sandbox (`bwrap`/seccomp/`Landlock`, macOS `.sbpl` `sandbox-exec`, Windows `WindowsRestrictedToken` + Job Objects via `toksi-sys`), `PermissionPolicy`, `ssrf_guard`, `SHELL_METACHARS` RBAC, Lark `ApplyPatchTool`, `HashlineEditTool`, unified `HookEventName` matrix, TOML/MD slash loader.
   - Wire directly to in-tree tools (`rtk`, `context`, `codegraph`, `projectmem`, `icm`, `ai-memory`) and configured Headroom `:8787` / Gateway `:9099` / `byok_profiles`.
2. Ship `DirectWireBus` (`src/tools/direct_wire.rs`) & Upgrade `tools/multi_protocol.rs`:
   - `DirectWireBus`: Zero-config-file in-process interception for tools & agent-to-agent messages (`direct-wired` mode alongside optional `config-wired`).
   - Real `A2aTaskStore` v1.0 JSON-RPC 2.0 (`SendMessage`/`GetTask` + `"message/send"`/`"tasks/get"`, SHA-256 constant-time bearer check, 1 MiB cap, `(peer, messageId)` dedupe) + `SessionActorQueue` ACP v2.0.
3. Implement 10 In-Chat Slash Commands (`src/agent_runtime/slash.rs` + `headroom/proxy.rs`):
   - `/byok [filter]` -> lists user-configured BYOK providers & stores active BYOK step.
   - `/model [filter]` -> lists models discovered for the currently selected BYOK first.
   - `/effort [none|low|medium|high|max|xhigh]` -> selects effort AFTER model (ignoring gracefully if unsupported).
   - `/byokmodel <byok> <model> [effort]` -> one-shot atomic persistence into `~/.toksi/toksi.db` (`byok_profiles` / `agent_profile_bindings`) + `~/.toksi/config.toml` sync.
   - `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk` -> instant `< 5ms` direct tool execution returning formatted structured response inside active chat turn (including when hooked through external agents like `agy`).
4. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Verification Criteria
- `toksi run "echo ok"` and `toksi swarm "analyze"` execute with `ToolLoopDetector`, `RunawayStreamWatch`, and OS sandbox guards active without hangs.
- Typing `/byokmodel`, `/byok` -> `/model` -> `/effort`, and `/projectmem status` completes in `< 5ms`, updates `byok_profiles` / `agent_profile_bindings`, and respects unsupported effort gracefully.
- A2A JSON-RPC 2.0 `SendMessage`/`GetTask` and ACP `SessionActorQueue` round-trip cleanly over `DirectWireBus` and HTTP/SSE.
