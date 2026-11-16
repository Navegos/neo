# Toksi Project Map (Wave 18 / v5.1)

- **Repository Root**: `d:/AI/neo` (Three-Repository Ecosystem: `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory` at `inspect-vault/advisory`, `<skills>` `Navegos/skills` at `inspect-vault/skills`)
- **Primary Rust Crate**: `crates/toksi` (`crates/toksi/Cargo.toml`)
- **Externalized Zero-Bloat Staging**: `<neo>/agents/` and `<neo>/rules/` (outside `crates/toksi/`, synced to `<skills>`)
- **Architecture & Phase Specs**:
  - `crates/toksi/CONSOLIDATION_PLAN.md` (Wave 18 / v5.1 Master Plan — 78 vaults, 35 phases, ready prompts)
  - `crates/toksi/PLAN.md` (Wave 18 / v5.1 Architectural Blueprint & 8-port topology)
  - `crates/toksi/research/RESEARCHPLAN.md` (Wave 18 Technical Blueprints: 78 vaults, 3 repos, `src/plugins/`, `src/validator/`, ADR/Sandbox/Kill-Switch, `defenseclaw`/`skill-scanner`, `:9098`, ADC, Effort, Security, Plan Checkpoints)
  - `crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md` (65/90+ ecosystems, Categories 1–5)
  - `crates/toksi/research/docker_sandboxes_sbx_cli_reference.md` (`sbx` 29-verb sandbox CLI parity)
  - `crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md` (`.agents` & vendor directory schemas + Category 1–3 hooks)
  - `crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md` (Zero-plaintext `BLAKE3` + `sparse::DFA`, `DisclosureGate`, 12-DB `UpstreamDeduplicator`, `CommandGuard`, & `Navegos/advisory` 3-tree shards)
  - `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` + 35 `PHASE_*.md` files (36 total markdown guides covering `PHASE_01`..`PHASE_18_5`)
  - `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/}` (Canonical workspace memory root)

## Crate Module Map (`crates/toksi/src/`)
| Directory / Module | Role |
|---|---|
| `src/main.rs`, `src/lib.rs`, `src/commands/` (`guard.rs`, `compile_blind.rs`, `advisory.rs`, `skills.rs`, `sandbox.rs`, `plugin.rs`, `security.rs`) | CLI entrypoint, `CommandGuard` + bypass kill-switch (`guard.rs`), & **16** universal `< 5ms` slash commands (`/byok`..`/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`) |
| `src/config/` *(planned Wave 17–18)*, `src/util/ports.rs` | Canonical `~/.toksi/config.toml` parser (`[repositories]`, `[plugins]`, `[security]` `kill_all_services_on_bypass`/`adr_enabled`/`defenseclaw_fail_closed`/`skill_scanner_policy`), port allocation & `AGY_ADC_AUTH=true` config |
| `src/keyring/` (`adc.rs` Wave 18.3) | OS Keychain/DPAPI encrypted secrets & Google Cloud Application Default Credentials (ADC) token exchange |
| `src/db/` | Unified `~/.toksi/toksi.db` (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`) & repo `<repo>/.toksi/` SQLite (`memory.db`, `symbols.db`, `test-baseline.json`) |
| `src/headroom/` (`effort.rs` Wave 18.3) | Pure-Rust AST/TOON & `HeadroomEffortCompressor` (`Low`/`Medium`/`High`, `live_zone`, `CcrStore`, `SmartCrusher`, `TokenEconomizer`, `agent-opt`, `mcp_pruner`, `chroma`/`lance`) serving `:8787` AND invoked by `:9098`, `:9099`, `:18787`, and `src/agent_runtime/` |
| `src/gateway/` (`effort_router.rs`, `vertex_adc.rs`, `gemini_wire.rs`, `proxy.rs`) | `:8788..:8790`, `:9098` Protocol Bus, `:9099` Enterprise AI Gateway & `:18787` BYOK/Vertex ADC Gateway — each enforcing full `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) before dispatch/cache |
| `src/rtk/` | High-speed TOML filter & TEE recovery engine |
| `src/projectmem/` (`plan_checkpoint.rs`, `wiki_sync.rs`) | JSONL events, derived `summary.md`, `plan.md` sync & atomic `plan_checkpoints/<plan_id>.json` resume |
| `src/contextmode/` | FTS5 BM25 sandbox analyzer & chunking engine (`<repo>/.toksi/context/`) |
| `src/icm/`, `src/aimemory/` (`lance_store.rs`) | Hierarchical memory + hybrid BM25/vector & Lance columnar store (`<repo>/.toksi/ai-memory/`) |
| `src/codegraph/` (`kernel/langs.rs`, `openwiki.rs`) | 45-language Tree-sitter indexer (`langs.rs`), call graph, blast radius & OpenWiki synthesizer |
| `src/rules/` (`skills_repo.rs`, `skill_validator.rs`, `skills_catalog.rs`, `sync.rs`) | Externalized `<skills>` sync (`Navegos/skills` + `~/.toksi/user-skills`, zero binary bloat), `skilldoctor` grading/fixing + `agentskills` YAML frontmatter healer & deduplicator |
| `src/security/` (`schema`, `polyglot`, `blind_compiler`, `scanner`, `advisory_client`, `secure_export`, `adr`, `defenseclaw`, `hookguard`, `skill_scanner`, `cel_decision`, `pipeline_taint`, `bytecode_audit`, `sarif`, `audit`, `yara_mcp`, `analyzer`, `sage_rules`, `skill_audit`, `osv`, `epss`, `semgrep_engine`, `external_analyzers`) | Zero-plaintext `BLAKE3`+`sparse::DFA` engine, `DisclosureGate` + 12-DB `UpstreamDeduplicator`, ADR (`adr.rs`), `defenseclaw` fail-closed hookguard & audit, `skill-scanner` multi-engine/CEL policy, & built-in 41-language AST static security analyzer + OSV/EPSS lockfile scanner |
| `src/validator/` (`mod`, `validate`, `fp_pruner`, `shard_builder`, `git_pr_driver`) | Native CVE 5.1 / OSV 1.7.4 / STIX 2.1 / SARIF validator, benign corpus FP pruner, 3-tree GitHub Pages shard builder, & `Navegos/advisory` REST PR driver |
| `src/plugins/` (`mod`, `manifest`, `loader`, `hooks`, `slash_router`, `cli_bridge`, `analyzer_plugin`) | Toksi Unified Plugin System loading 7 mount surfaces (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP`, `agent_to_agent`) under `<enforce>` |
| `src/agent_runtime/` (`acp.rs`, `sandbox.rs`, `codemode.rs`, `session.rs`, `checkpoint.rs`, `deep_planning.rs`, `fleet.rs`, `graph_checkpoint.rs`, `guardrails.rs`, `modes.rs`, `zeroclaw_core.rs`) | Pure-Rust agent orchestrator unifying `<neo>/inspect-vault/*<agent>` via `DirectWireBus` & `:8787`/`:9098`/`:9099`/`:18787` across all 78 vaults + 3-OS/`sandbox-runtime`/`sbx` isolation |
| `src/telemetry/` (`tokview.rs`, `traces.rs`), `src/tui/`, `src/dashboard/` | Token/effort attribution (`tokview`), 9-tab mouse-capable Ratatui TUI, `:9100` HTMX/Alpine Web Cockpit |
| `src/env_injector/`, `src/tools/`, `src/agents/` (`opencode.rs`) | Non-destructive `<enforce>` rule injection & multi-agent adapters |

## Key Verification Paths
- `crates/toksi/tests/` — Integration suites (`phase1`..`phase18_5`), `benchmarks.rs`, `e2e_tests.rs`, `keyring_db_byok_verification.rs`.
- `inspect-vault/` — **78** mandated upstream research repositories & companion repos (`36` prior + `42` Wave 18).
