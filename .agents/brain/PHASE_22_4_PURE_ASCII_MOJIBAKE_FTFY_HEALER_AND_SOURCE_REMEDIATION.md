# Phase 22.4: Strict Pure 7-Bit ASCII Guard, `MojibakeLab`/`python-ftfy` Auto-Healer & 118 `.rs` Source Remediation

> **Status (Wave 22 / v9.0 Audit):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```markdown
Implement Phase 22.4 across `crates/toksi/src/security/ascii_guard.rs`, `src/util/{mojibake,ftfy}.rs`, `src/commands/heal_encoding.rs`, and remediate all 118 non-ASCII `.rs` files in `crates/toksi/src/`.

Mandatory Pre-Read Set:
- `.agents/rules/ascii-only.md`
- `crates/toksi/research/RESEARCHPLAN.md` (Sections 1.3.4, 2.18, 3.6, 5.3, 12)
- `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md` & `crates/toksi/README.md`
- `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`
- `crates/toksi/research/{docker_cli_compose_yaml_full_reference,docker_sandboxes_sbx_cli_yaml_reference,folders_structures_default_knowledge_used_by_the_known_cli_agents_ides,full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture,open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator,toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping}.md`
- `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`
- `inspect-vault/{MojibakeLab,python-ftfy}`

Multi-Agent Resume Rule: Shard the 118 `.rs` files into disjoint module leases in `.toksi/projectmem/worker_leases.json`; run `cargo check -p toksi` after each module batch and flush to `orchestration_journal.jsonl`.

Targets:
1. `src/util/mojibake.rs` & `src/util/ftfy.rs`: Pure-Rust port of `MojibakeLab` encoding confusion scoring (UTF-8 vs CP1252/Latin-1 double/triple-encoding repair, C1 control normalization, NFC canonicalization) and `python-ftfy` `fix_text` pipeline.
2. `src/security/ascii_guard.rs` & `src/commands/heal_encoding.rs` (`toksi heal-encoding [--check|--apply] [--path <dir>]`): Enforce strict `0x00`-`0x7F` + `LF` + zero BOM across enforced files, emit exact substitution table diagnostics (`---`, `--`, `->`, `|--`, `` `-- ``, `\u{XXXX}` / `\uXXXX` escapes), and heal mojibake safely in docs without corrupting intentional binary blobs.
3. Remediate all non-ASCII bytes (`> 0x7F`) across `crates/toksi/src/` (`118` audited `.rs` files including `commands/top.rs`, `commands/rtk.rs`, `util/instructions.rs`, UI/comment/box-drawing strings -> `\u{XXXX}` or pure-ASCII prose/art).

Verification:
- Execute `toksi heal-encoding --check` against `crates/toksi/src/`, `crates/toksi/tests/`, `crates/toksi/research/*.md`, `.agents/**/*.md` and assert `0` violations in enforced paths and `0` mojibake in `README.md`.
- Run `cargo check -p toksi`, `cargo clippy -p toksi --tests -- -D warnings`, and `cargo test -p toksi heal_encoding`.
```

## Technical Specifications & Verification Criteria
- **Zero Intentional String Corruption**: Multi-byte Unicode test/UI fixtures use Rust native `\u{XXXX}` escapes rather than lossy replacement.
- **Verification**: Byte scan confirms `0` bytes `> 0x7F`, `0` `\r\n`, and `0` BOM across all `.rs` files under `crates/toksi/src/`.
