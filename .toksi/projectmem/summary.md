# Project Memory Summary (Wave 21 / v8.0)

_Synchronized Wave 21 / v8.0 across the **3-repository ecosystem** (`<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, `<skills>` `Navegos/skills`) covering **495 unique mandated vaults** (`294` Waves 1–18 + `138` Wave 19 + `61` Wave 20 = `493` Wave 20 + `2` Wave 21 reconciled: `Zoo-Code`, `agent-orchestrator` `[DEEP-RESEARCH-DONE]`), **55 `.agents/brain/*.md` guides** (`INDEX` + `PHASE_01`..`PHASE_21_6`), real **`<toksi-siagent>`** (**22 Orchestrator Modes** across 7 Cognitive Domains `[toksi_siagent_router.*]`), **29 slash commands**, decoupled **10-Tab UI**, per-service `~/.toksi/services/*.db`, **11 AI/ML Pillars**, **Distributed Cluster Stack (`src/cluster/`, `toksi cluster`)**, **OpenTelemetry Rust (`gen_ai.*`)**, **Single `NetworkService` Manager + On-Demand `<toksi-siagent>`**, **Codex/Grok-Build Chat TUI**, **13-IDE Plugin Host**, and **Multi-Agent Orchestration Resilience & Cross-Chat Resume**._

## Current Status
- Deep-researched all **495 vaults** (`[DEEP-RESEARCH-DONE]`) across Sections `2.1`–`2.17` (`493` Wave 20 + `Zoo-Code` + `agent-orchestrator`).
- Waves 1–20 (`PHASE_01`..`PHASE_20_6`, `48` phases) are `[PLAN-IMPLEMENTED]`/`[PLAN-FIXED]` across `497` `src/**/*.rs` (`541` total `.rs` files in `crates/toksi`, `0` unsafe blocks under `#![deny(unsafe_code)]`) and `43` integration test suites tracked in `.toksi/test-baseline.json`: `ctxmode` & `aimemory` renames, `toksi _worker <role>` subprocesses + Named Pipe/UDS `src/ipc/`, `docker/Dockerfile.test` + `docker/compose.test.yaml` + `sbxenv.yaml`/Kit v3, `<neo>/.agents/skills/` hygiene cleanup, 22 modes, 5 efforts, sampling matrix, decoupled 10-Tab UI, `~/.toksi/services/*.db`, 11-pillar AI/ML vaults, `src/cluster/` (`toksi cluster`), and `src/telemetry/otel.rs`.
- **Wave 21 (`PHASE_21_1`..`PHASE_21_6`) Planned (`[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]`):**
  1. `PHASE_21_1`: Single dedicated `toksi _worker network-manager` (`NetworkService` owning daemon listening ports to prevent `os error 10048`) + decoupling `<toksi-siagent>` from `daemon start` into an on-demand process.
  2. `PHASE_21_2`: Codex + Grok-Build grade streaming `ratatui` Chat TUI (`src/agent_runtime/chat_ui.rs`) executing all `toksi` CLI commands and 22 Orchestrator Modes.
  3. `PHASE_21_3`: External 13-IDE plugin host protocol (`src/tools/ide_plugin_host.rs`, `toksi ide-bridge`, `:9098`).
  4. `PHASE_21_4`: `Zoo-Code` (Semble, `DCG`, `.roomodes`), `agent-orchestrator` (per-task git worktree Kanban lanes), 11-pillar AI/ML deep adapters, and `total_mandated_vault_count()` = `495`.
  5. `PHASE_21_5`: Multi-Agent Orchestration Resilience & Cross-Chat Resume (`src/agent_runtime/orchestration_resume.rs`, write-ahead `orchestration_journal.jsonl`, disjoint `worker_leases.json`, 5-store atomic flush).
  6. `PHASE_21_6`: `phase21_1_to_21_6_verification.rs` + `.toksi/test-baseline.json` refresh + 55-file 8-state sync.

## Key Architecture Pointers
- `crates/toksi/research/RESEARCHPLAN.md` — Authoritative Wave 21 / v8.0 blueprint (`495` `[DEEP-RESEARCH-DONE]` vaults, 8-state matrix, Sections 0–14)
- `crates/toksi/CONSOLIDATION_PLAN.md` — Master Wave 21 / v8.0 consolidation plan & Phase `01`..`21.6` prompts
- `crates/toksi/PLAN.md` — Technical standards, `NetworkService` + `_worker` IPC topology, detached `:9100`, `~/.toksi/services/*.db`, cluster stack, 10-tab UI, & 29 slash commands
- `crates/toksi/README.md` — Complete Wave 21 CLI, config, agent, bridge, and verification guide
- `.toksi/projectmem/{plan.md,PROJECT_MAP.md,events.jsonl,plan_checkpoints/}` — Canonical project memory plan, checkpoints & repository layout
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` — 54-phase execution index (**55** `.agents/brain/*.md` guides with 8-State Progress Taxonomy)
