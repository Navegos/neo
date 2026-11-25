# Phase 09: Legacy Healing & Binary Un-Hijack

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_09_LEGACY_HEALING_UNHIJACK.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/tokless is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and healing logic into consolidated toksi.

Execute Phase 09: Legacy Healing & Binary Un-Hijack.
Source: neo/inspect-vault/tokless
Target: crates/toksi/src/env_injector/heal.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors and anyhow::Result with .context() in CLI.
3. Universal Forward-Slash (/) Standard: Use normalize_path() for all filesystem traversals and restored paths.
4. Non-Destructive Operation: Safely restore original binaries from `.real` backups, clean up hijack shims, and never delete non-shim files.

Requirements:
1. Implement `crates/toksi/src/env_injector/heal.rs` + `agents/manager.rs` healing to audit and auto-heal hijacked binaries, shims, and legacy config names:
   - Identify `.real` binaries created by legacy proxy injectors (e.g. `agy.real`, `copilot.real`, `grok.real`).
   - Atomically restore `.real` binaries to their original filenames (`agy.exe`, `copilot.exe`, `grok.exe`).
   - Remove lingering `.cmd`, `.bat`, or shell script shims that hijacked `PATH`.
   - Clean up hijacked symlinks or junction points across Windows, Linux, and macOS.
   - Purge legacy strings/keys across configs & codebase: `tokless_*` (`paths.rs`), `toksi-projectmem`, `toksi-codegraph`, `toksi-codegraph-index`, `agy-hook codegraph-index`, `rtk hook cursor` (`main.rs:340`), `toksi-codegraph-init.js`.
   - Migrate legacy `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp`, `.agents/mcp` folders to `skills/<tool>`, standardize hook invocation to `<tool>-hook <agent>`, and merge nested crate `.toksi/` / `.<agent>/` DBs/files cleanly into root `<repo>/.toksi/<tool>` / `<repo>/.<agent>`.
2. Hook into `toksi doctor --fix` and `toksi daemon stop` (plus PATH verification via `check_executable_in_path`), aligned with **35-component `<toksi-siagent>`** (`DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, **495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, and externalized zero-bloat `<skills>` & `<advisory>`).
3. Ensure 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Binary Discovery & Detection** (`src/env_injector/heal.rs`): Traverses known agent CLI install locations (AppData, user bin, home directories) to detect `.real` backup files.
- **Atomic Restoration**: Renames `.real` files back to their original binary names, ensuring file permissions and executable bits are preserved.
- **Wrapper & Shim Purging**: Deletes shell scripts or Windows batch wrappers placed in `PATH` by previous versions that attempted intrusive proxy shimming.

### 2. Disciplinary Standards & Invariants
- All file operations run with explicit error handling; never panic on read-only files or permission errors.
- Always log actions clearly with `tracing::info!` so the user has complete visibility into what was restored.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- heal` passes 100%.
- Running `toksi doctor --fix` detects mock `.real` binaries and successfully restores them.

---

## Wave 18 (v5.2) Synchronization Addendum
- Pair `heal.rs` binary un-hijack with `skilldoctor` + `agentskills` YAML frontmatter healing (`PHASE_18_2`), legacy `.projectmem/` -> `<repo>/.toksi/projectmem/` consolidation (`PHASE_18_4`), and `<enforce>` plugin verification across **495 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, and externalized `<skills>` & `<advisory>`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
