# Phase 18.4 — Legacy `.projectmem/` Repair, Documentation Integrity & Final Workspace Verification

## Ready-to-Run Chat Prompt

```text
Execute Phase 18.4 of the Toksi Unified Consolidation Plan (Repository-Wide Documentation Integrity, `<repo>/.toksi/projectmem/` Repair, and End-to-End Verification Gate).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_4_LEGACY_PROJECTMEM_REPAIR_AND_MASTER_DOC_SYNC.md first.

Your goals in Rust (`crates/toksi`) & Repo Verification:
1. Enforce Single Canonical ProjectMem Storage & Repair Guard (`src/projectmem/` & `src/doctor/`):
   - Verify `toksi doctor` and `toksi projectmem migrate` detect any loose `.projectmem/`, `.codegraph/`, `.context-mode/`, `.icm/`, `.ai-memory/`, `.rtk/` directories at repo root or nested subcrates, atomically migrate rows/events/checkpoints into `<repo>/.toksi/<tool>/`, and leave zero loose dot-folders outside `<repo>/.toksi/` (except legacy `.projectmem` test fixtures created strictly inside temporary `tempfile::TempDir` sandboxes).
   - Ensure `toksi rules audit-docs` verifies zero stale pre-read references (`(?<!\.toksi/)\.projectmem/`) remain in `.agents/brain/*.md`, `crates/toksi/CONSOLIDATION_PLAN.md`, `crates/toksi/PLAN.md`, `crates/toksi/README.md`, and `.toksi/projectmem/*.md`.
2. Automated Documentation Hygiene Test (`crates/toksi/tests/phase18_4_doc_and_checkpoint_hygiene.rs`):
   - Assert exact count and completeness of `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_18_4` (35 files).
   - Assert every `PHASE_*.md` chat prompt pre-reads `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`.
   - Assert `crates/toksi/README.md`, `PLAN.md`, `CONSOLIDATION_PLAN.md`, and `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}` match current codebase module names (zero phantom `crates/tokless/*` paths), 8-port topology (`:8787..:8790, :9098, :9099, :9100, :18787`), **13** slash commands (`/byok`..`/rtk`, `/security`, `/plugin`, `/resume`), Unified Plugin System (`src/plugins/`, `plugins_registry`), `defenseclaw` + `skill-scanner` security modules, and all **67** audited vaults.
3. Run Full Workspace Verification Gate & Update `.toksi/test-baseline.json`:
   - `cargo check --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace` (saving structured baseline diff to `<repo>/.toksi/test-baseline.json`).
   - Print final Wave 18 / v5.0 parity report and confirm zero blockers remain.
```

---

## Technical Specifications & Details

### 1. Canonical File & Architecture Invariants Checked in Phase 18.4
- **Workspace Memory Root**: Always `<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/<plan_id>.json}`. Never bare `.projectmem/` or `~/.toksi/projectmem/`.
- **Legacy Path Cleanup**: Eliminate stale `crates/tokless/*` documentation references in favor of `crates/toksi/*`.
- **Master Architecture Alignment**:
  - Audited Vaults: **67** (`36` Waves 1–17 + `31` Wave 18 incl. `defenseclaw`, `skill-scanner`).
  - Daemon Topology: **8 Ports** (`:8787` Headroom, `:8788` Grok OAuth, `:8789` Copilot VS Code, `:8790` Copilot CLI, `:9098` Protocol Bus, `:9099` Gateway, `:9100` Web Cockpit, `:18787` BYOK Gateway).
  - Universal Slash Commands: **13** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/plugin`, `/resume`).
  - Unified Plugins & Fail-Closed Security: `src/plugins/*`, `commands/plugin.rs`, SQLite `plugins_registry`, `[plugins]`/`[security]` in `~/.toksi/config.toml`, `defenseclaw` + `skill-scanner` integration in `src/security/*`.
  - Phases & Brain Files: **35** files in `.agents/brain/` (`INDEX_AND_EXECUTION_GUIDE.md` + 34 `PHASE_*.md` files covering Phases 1–14, 15.1–15.6, 16.1–16.3, 17.1–17.7, 18.1–18.4).
  - Verification Gate: `cargo check --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + incremental `<repo>/.toksi/test-baseline.json`.
