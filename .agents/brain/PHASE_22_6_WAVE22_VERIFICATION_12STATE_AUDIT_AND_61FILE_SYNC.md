# Phase 22.6: Wave 22 Integration Verification, Pure-ASCII Audit & 12-State 61-File Synchronization

> **Status (Wave 22 / v9.0 Audit):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```markdown
Execute Phase 22.6 to validate all Wave 22 implementations end-to-end, audit strict pure 7-bit ASCII across the entire repo, refresh `.toksi/test-baseline.json`, and promote all 61 `.agents/brain/*.md` guides + 7 master docs to `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]`.

Mandatory Pre-Read Set:
- `crates/toksi/research/RESEARCHPLAN.md` & `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md` & `crates/toksi/README.md`
- `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`
- `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`
- `.agents/rules/ascii-only.md`
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`

Multi-Agent Resume Rule: Single coordinator owns final verification, `.toksi/test-baseline.json`, and 12-State badge promotion. Never mark `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` before all commands pass after the final edit.

Targets:
1. Create `crates/toksi/tests/phase22_1_to_22_6_verification.rs` covering:
   - Double-keystroke press filter (`KeyEventKind::Press`) & line editor + Pause/Resume
   - Headroom-central `:8787` routing + Gateway matching (zero synthetic `"gpt-6.2-astra"` 502s) + Git-Engineer `.git/config` awareness
   - `r3bl-open-core` (`r3bl_tui`) TUI + hover ergonomics
   - Dynamic `.git/config` repo discovery (no hardcoded `neo_dir`), `<neo>/agents/rules/`, and lossless `refactor-router`
   - `MojibakeLab`/`python-ftfy` `heal-encoding` + 0 non-ASCII bytes in `crates/toksi/src/`
   - Collision-free service ports, Copilot -> `:8787` consolidation, standalone/MCP/`<toksi-siagent>` SAST, plugin mounts, bidirectional DB/TOML BYOK sync, and `total_mandated_vault_count() == 502`
2. Run full verification gates:
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace` (live-safe `TOKSI_HOME` `TempDir` + `:0` isolated)
   - `cargo run -p toksi -- test --all --force` (refresh `.toksi/test-baseline.json`)
   - `cargo run -p toksi -- heal-encoding --check` (0 non-ASCII violations, 0 `\r\n`, 0 BOM, 0 mojibake)
3. Promote `PHASE_01`..`PHASE_22_6`, `INDEX_AND_EXECUTION_GUIDE.md`, and all 7 master docs (`RESEARCHPLAN.md`, `CONSOLIDATION_PLAN.md`, `PLAN.md`, `README.md`, `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}`) to verified `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]`.

Verification:
- All workspace targets compile with 0 errors and 0 warnings; all test suites pass; 61 `.agents/brain/*.md` guides match the 12-State Progress Taxonomy cleanly.
```

## Technical Specifications & Verification Criteria
- **Zero Regression**: `phase18_4_doc_and_checkpoint_hygiene`, `phase19_1_to_19_6_verification`, `phase20_1_to_20_6_verification`, `phase21_1_to_21_6_verification`, and `phase22_1_to_22_6_verification` all pass.
- **Zero-Data-Loss Check**: Confirm `git status` clean of unintended `.agents/skills/` pollution and zero stale bare `.projectmem/` pre-read refs.
