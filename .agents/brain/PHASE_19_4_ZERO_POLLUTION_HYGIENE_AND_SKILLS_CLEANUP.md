# Phase 19.4 --- Zero-Pollution Adapt-Only Repo Hygiene & `.agents/skills/` Cleanup

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-NEEDS-FIX]

## Ready-to-Run Chat Prompt
```text
Read ALL 10 canonical workspace files FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. .toksi/projectmem/plan.md
5. .toksi/projectmem/PROJECT_MAP.md
6. .toksi/projectmem/summary.md
7. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
8. crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
9. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
10. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
Plus .agents/brain/PHASE_19_4_ZERO_POLLUTION_HYGIENE_AND_SKILLS_CLEANUP.md.

Execute Phase 19.4: Zero-Pollution Adapt-Only Repo Hygiene & `.agents/skills/` Cleanup (RESEARCHPLAN.txt:588).
Sources (inspect-vault/*): skills (<skills> Navegos/skills), advisory (<advisory> Navegos/advisory), skilldoctor, agentskills, skill-scanner, defenseclaw, upskill, agent-threat-rules, qdrant-skills, crewAIInc-skills, Essential-Hermes-Skills
Targets:
  - <neo>/.agents/skills/ (922 unadapted external skill directories verified on disk)
  - <neo>/agents/ (28 files) & <neo>/agents/rules/ (24 files) --- staged zero-bloat outside crates/toksi/
  - crates/toksi/src/rules/{skills_repo.rs,skill_validator.rs,skills_catalog.rs,cli.rs} (`toksi skills audit-neo`)
  - crates/toksi/src/commands/skills.rs
  - inspect-vault/skills (`Navegos/skills`) & inspect-vault/advisory (`Navegos/advisory`)
  - crates/toksi/tests/phase19_4_zero_pollution_hygiene_verification.rs (NEW)

Requirements:
1. Enforce the Strict Zero-Pollution Rule (`RESEARCHPLAN.txt:588`):
   - NEVER copy unnecessary raw files, third-party install scripts (`.sh`, `.ps1`, `.bat`), or foreign GitHub Actions `.github/workflows/*.yml` files from `inspect-vault/*` into `<neo>`.
   - Audit every directory in `<neo>/.agents/skills/` (currently `922` unadapted directories from upstream tool packs) plus `<neo>` root/crate folders.
2. Quarantine/Delete Unadapted Clutter & Adapt Strictly Required `<neo>` Skills:
   - Remove unadapted foreign framework skills not authored for `<neo>` / `toksi` (retaining only verified, adapted `<neo>` workspace skills such as `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `principles`, `caveman`, `ponytail`, `security`, etc.).
   - Adapt retained/imported skills (`qdrant-skills`, `crewAIInc-skills`, `Essential-Hermes-Skills` --- **explicitly quarantining/stripping offensive/unsafe patterns such as `reverse-skill` if non-compliant with `<enforce>`**), heal YAML frontmatter via `agentskills/skills-ref`, grade `>= 85/100` via `skilldoctor`, evaluate quality via `upskill`, wrap in `<enforce>...</enforce>`, and verify `CelVerdict::Allow` via `skill-scanner` + `defenseclaw` + ATR.
3. Enforce Zero Binary Bloat & Sync External Repositories:
   - Keep zero `.md`/`.yaml`/`.json` skills/rules embedded via `include_str!` inside `crates/toksi/`. Stage vetted assets in `<neo>/agents/` & `<neo>/agents/rules/`, sync cleanly into `<skills>` (`inspect-vault/skills` -> `Navegos/skills`), and validate zero-plaintext shards in `<advisory>` (`inspect-vault/advisory` -> `Navegos/advisory`).
4. Implement `toksi skills audit-neo` in `src/commands/skills.rs` / `src/rules/cli.rs` + continuous guard in `phase19_4_zero_pollution_hygiene_verification.rs` failing if unadapted foreign skill bloat or raw vault scripts exist in `<neo>`.
5. Update `.toksi/test-baseline.json` and all 6 canonical plan destinations; verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p toksi --test phase19_4_zero_pollution_hygiene_verification`.
```

---

## Technical Specifications & Details

### 1. Live Disk Clutter Audit (Verified 2026-11-21)
- `<neo>/.agents/skills/`: **922 directories** (e.g., `3d-animation-short-generator`, `accessing-mlflow`, `add-jit-kernel`, `cuopt-*`, `nemo-*`, `sglang-*`, `vllm-*`, etc. from external vaults), `0` script subdirs.
- `<neo>/agents/`: `28` files (`agents/skills/*`).
- `<neo>/agents/rules/`: `24` files.
- `<skills>` (`inspect-vault/skills`): `80` files (`rules/`, `skills/`).
- `<advisory>` (`inspect-vault/advisory`): `30` files.

### 2. Security & Hygiene Gate (`<enforce>`)
- Any imported skill with `curl | sh`, obfuscated base64 execution, raw socket shells, duplicate names/triggers, or broken YAML frontmatter fails closed and is quarantined/healed before entering `<neo>/agents/`, `<skills>`, or any agent config dir.

## Verification Criteria
- `toksi skills audit-neo` reports `0` unadapted foreign scripts, `0` unhealed `SKILL.md` frontmatters, `0` trigger collisions, and `<neo>/.agents/skills/` strictly pruned to adapted `<neo>` skills.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
