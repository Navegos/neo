# Phase 18.4 --- Legacy `.projectmem/` Repair, Documentation Integrity & Final Workspace Verification

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt

```text
Execute Phase 18.4 of the Toksi Unified Consolidation Plan (Repository-Wide Documentation Integrity across all 43 `.agents/brain/*.md` guides, `<repo>/.toksi/projectmem/` Repair across 495 Mandated Vaults & 35-Component `<toksi-siagent>`, and End-to-End Verification Gate).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_4_LEGACY_PROJECTMEM_REPAIR_AND_MASTER_DOC_SYNC.md first.

Your goals in Rust (`crates/toksi`) & Repo Verification:
1. Enforce Single Canonical ProjectMem Storage & Repair Guard (`src/projectmem/` & `src/doctor/`):
   - Verify `toksi doctor` and `toksi projectmem migrate` detect any loose `.projectmem/`, `.codegraph/`, `.context-mode/`, `.icm/`, `.ai-memory/`, `.rtk/` directories at repo root or nested subcrates, atomically migrate rows/events/checkpoints into `<repo>/.toksi/<tool>/`, and leave zero loose dot-folders outside `<repo>/.toksi/` (except legacy `.projectmem` test fixtures created strictly inside temporary `tempfile::TempDir` sandboxes).
   - Ensure `toksi rules audit-docs` verifies zero stale pre-read references (`(?<!\.toksi/)\.projectmem/`) remain in `.agents/brain/*.md`, `crates/toksi/CONSOLIDATION_PLAN.md`, `crates/toksi/PLAN.md`, `crates/toksi/README.md`, and `.toksi/projectmem/*.md`.
2. Automated Documentation Hygiene Test (`crates/toksi/tests/phase18_4_doc_and_checkpoint_hygiene.rs`):
   - Assert exact count and completeness of `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_19_6` (**37 files**).
   - Assert every `PHASE_*.md` chat prompt pre-reads `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`.
   - Assert `crates/toksi/README.md`, `PLAN.md`, `CONSOLIDATION_PLAN.md`, `research/RESEARCHPLAN.md`, and `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}` match current codebase module names (zero phantom `crates/tokless/*` paths), 3-repository topology (`<neo>`, `<advisory>`, `<skills>` with zero binary bloat), 8-port topology (`:8787..:8790, :9098, :9099, :9100, :18787`), universal 7-entrypoint `HeadroomEffortCompressor` pass (`:8787`--`:8790`, `:9098`, `:9099`, `:18787`), **35-component `<toksi-siagent>`** out-of-the-box `DirectWireBus` bindings across all gateways/Protocol Bus/495 vaults, **16** slash commands (`/byok`..`/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`), Unified Plugin System (`src/plugins/`, `plugins_registry`), Zero-Plaintext Blind Engine + `validator/` + `adr.rs` + `sandbox.rs` (5 backends) + `guard.rs` kill-switch + `opa`/`cedar`/`spire` + `Guardrails`/`llm-guard` + `defenseclaw` + `skill-scanner`, and all **495** mandated vaults.
3. Run Full Workspace Verification Gate & Update `.toksi/test-baseline.json`:
   - `cargo check --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace` (saving structured baseline diff to `<repo>/.toksi/test-baseline.json`).
   - Print final Wave 18 / v5.2 parity report and confirm zero blockers remain.
```

---

## Technical Specifications & Details

### 1. Canonical File & Architecture Invariants Checked in Phase 18.4
- **Workspace Memory Root**: Always `<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/<plan_id>.json}`. Never bare `.projectmem/` or `~/.toksi/projectmem/`.
- **Legacy Path Cleanup**: Eliminate stale `crates/tokless/*` documentation references in favor of `crates/toksi/*`.
- **Master Architecture Alignment**:
  - Audited Vaults: **495** unique mandated vaults (493 Wave 20 + 2 Wave 21) (`292` `<deep-research>` vaults across Sections `2.1`--`2.10` + companion `<advisory>` & `<skills>` + `agents-structures` = `36` prior + `259` Wave 18 vaults).
  - `<toksi-siagent>` Super Intelligence Architecture: All **35** technical components (`Section 0.2`) wired out of the box via `DirectWireBus` and the shared 7-entrypoint `HeadroomEffortCompressor`.
  - Three-Repository Ecosystem: `<neo>` (`Navegos/neo`), `<advisory>` (`Navegos/advisory`), `<skills>` (`Navegos/skills`) with externalized skills/rules staged outside `crates/toksi/` (zero binary bloat).
  - Daemon Topology: **8 Ports** (`:8787` Headroom, `:8788` Grok OAuth, `:8789` Copilot VS Code, `:8790` Copilot CLI, `:9098` Protocol Bus, `:9099` Gateway, `:9100` Web Cockpit, `:18787` BYOK Gateway).
  - Universal Slash Commands: **16** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`).
  - Unified Plugins & Fail-Closed Security: `src/plugins/*`, `commands/plugin.rs`, SQLite `plugins_registry`, `[plugins]`/`[security]`/`[repositories]` in `~/.toksi/config.toml`, Zero-Plaintext Blind Engine + `src/validator/*` + `src/security/adr.rs` + `src/agent_runtime/sandbox.rs` (5 backends) + `src/commands/guard.rs` kill-switch + `opa`/`cedar`/`spire` + `Guardrails`/`llm-guard` + `defenseclaw` + `skill-scanner` integration.
  - Phases & Brain Files: **37** files in `.agents/brain/` (`INDEX_AND_EXECUTION_GUIDE.md` + 36 `PHASE_*.md` files covering Phases 1--14, 15.1--15.6, 16.1--16.3, 17.1--17.7, 18.1--18.6).
  - Verification Gate: `cargo check --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + incremental `<repo>/.toksi/test-baseline.json`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
