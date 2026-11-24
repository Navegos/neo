# Project Memory Summary (Wave 20 / v7.0)

_Synchronized Wave 20 / v7.0 across the **3-repository ecosystem** (`<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, `<skills>` `Navegos/skills`) covering **493 unique mandated vaults** (`294` Waves 1–18 + `138` Wave 19 + `61` Wave 20 `[DEEP-RESEARCH-DONE]`), **49 `.agents/brain/*.md` guides** (`INDEX` + `PHASE_01`..`PHASE_20_6`), real **`<toksi-siagent>`** (`toksi _worker siagent`, **22 Orchestrator Modes** across 7 Cognitive Domains `[toksi_siagent_router.*]`), **29 slash commands**, decoupled **10-Tab UI**, per-service `~/.toksi/services/*.db`, **11 AI/ML Pillars**, **Distributed Cluster Stack (`src/cluster/`, `toksi cluster`)**, and **OpenTelemetry Rust (`gen_ai.*`)**._

## Current Status
- Deep-researched all **493 vaults** (`[DEEP-RESEARCH-DONE]`) across Sections `2.1`–`2.17`.
- Implemented & verified all Wave 19–20 execution phases (`PHASE_19_1`..`PHASE_20_6`) across `497` `src/**/*.rs` (`541` total `.rs` files in `crates/toksi`, `0` unsafe blocks under `#![deny(unsafe_code)]`) and `43` integration test suites tracked in `.toksi/test-baseline.json`: `ctxmode` & `aimemory` renames, `toksi _worker <role>` subprocesses + Named Pipe/UDS `src/ipc/`, `docker/Dockerfile.test` + `docker/compose.test.yaml` + `sbxenv.yaml`/Kit v3, `<neo>/.agents/skills/` hygiene cleanup, Wave 19 Workforce/`ANP`/ATR/Training plane, real `<toksi-siagent>` execution (local `loop_engine.rs` stubs removed), 22 modes, 5 efforts (`Max..Low`) + sampling matrix + auto `/v1` + dual `agy`/ADC, `daemon heal|dashboard|service`, `~/.toksi/services/*.db`, decoupled 10-Tab UI (`toksi cluster`, `/api/otel/status`, `Quotas`, `Activity Room`), 11-pillar AI/ML vaults (`liteparse`, `hyperflare-inference`, `kvpress`, `deepeval`, `ragas`, `AutoFA`, `PurpleLlama`), `src/cluster/` (`toksi cluster`), and `src/telemetry/otel.rs`.

## Key Architecture Pointers
- `crates/toksi/research/RESEARCHPLAN.md` — Authoritative Wave 20 / v7.0 blueprint (`493` `[DEEP-RESEARCH-DONE]` vaults, 8-state matrix, Sections 0–13)
- `crates/toksi/CONSOLIDATION_PLAN.md` — Master Wave 20 / v7.0 consolidation plan & Phase `01`..`20.6` prompts
- `crates/toksi/PLAN.md` — Technical standards, `_worker` IPC topology, detached `:9100`, `~/.toksi/services/*.db`, cluster stack, 10-tab UI, & 29 slash commands
- `.toksi/projectmem/{plan.md,PROJECT_MAP.md,events.jsonl}` — Canonical project memory plan & repository layout
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` — 48-phase execution index (**49** `.agents/brain/*.md` guides with 8-State Progress Taxonomy)
