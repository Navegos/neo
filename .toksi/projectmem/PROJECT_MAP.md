# Toksi Project Map (Wave 18 / v5.0)

- **Repository Root**: `d:/AI/neo`
- **Primary Rust Crate**: `crates/toksi` (`crates/toksi/Cargo.toml`)
- **Architecture & Phase Specs**:
  - `crates/toksi/CONSOLIDATION_PLAN.md` (Wave 18 / v5.0 Master Plan — 67 vaults, 34 phases, ready prompts)
  - `crates/toksi/PLAN.md` (Wave 18 / v5.0 Architectural Blueprint & 8-port topology)
  - `crates/toksi/research/RESEARCHPLAN.md` (Wave 18 Technical Blueprints: 67 vaults, `src/plugins/`, `defenseclaw`/`skill-scanner`, `:9098`, ADC, Effort, Security, Plan Checkpoints)
  - `crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md` (90+ ecosystems, Categories 1–5)
  - `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` + 34 `PHASE_*.md` files (35 total markdown guides covering `PHASE_01`..`PHASE_18_4`)
  - `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/}` (Canonical workspace memory root)

## Crate Module Map (`crates/toksi/src/`)
| Directory / Module | Role |
|---|---|
| `src/main.rs`, `src/lib.rs`, `src/commands/` | CLI entrypoint (`toksi plugin`, `toksi security`, ...) & **13** universal `< 5ms` slash commands (`/byok`..`/rtk`, `/security`, `/plugin`, `/resume`) |
| `src/config/` *(planned Wave 17–18)*, `src/util/ports.rs` | Canonical `~/.toksi/config.toml` parser (`[plugins]`, `[security]` `defenseclaw_fail_closed` / `skill_scanner_policy`), port allocation & `AGY_ADC_AUTH=true` config |
| `src/keyring/` (`adc.rs` Wave 18.3) | OS Keychain/DPAPI encrypted secrets & Google Cloud Application Default Credentials (ADC) token exchange |
| `src/db/` | Unified `~/.toksi/toksi.db` (`byok_profiles` w/ `gcp_project`/`gcp_location`, `plugins_registry`) & repo `<repo>/.toksi/` SQLite (`memory.db`, `symbols.db`, `test-baseline.json`) |
| `src/headroom/` (`effort.rs` Wave 18.3) | Pure-Rust AST/TOON & `HeadroomEffortCompressor` (`Low`/`Medium`/`High`, `live_zone`, `CcrStore`, `SmartCrusher`, `TokenEconomizer`, `agent-opt`, `mcp_pruner`, `chroma`/`lance`) serving `:8787` AND invoked by `:9098`, `:9099`, `:18787`, and `src/agent_runtime/` |
| `src/gateway/` (`effort_router.rs`, `vertex_adc.rs`, `gemini_wire.rs`, `proxy.rs`) | `:8788..:8790`, `:9098` Protocol Bus, `:9099` Enterprise AI Gateway & `:18787` BYOK/Vertex ADC Gateway — each enforcing full `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) before dispatch/cache |
| `src/rtk/` | High-speed TOML filter & TEE recovery engine |
| `src/projectmem/` (`plan_checkpoint.rs`, `wiki_sync.rs`) | JSONL events, derived `summary.md`, `plan.md` sync & atomic `plan_checkpoints/<plan_id>.json` resume |
| `src/contextmode/` | FTS5 BM25 sandbox analyzer & chunking engine (`<repo>/.toksi/context/`) |
| `src/icm/`, `src/aimemory/` (`lance_store.rs`) | Hierarchical memory + hybrid BM25/vector & Lance columnar store (`<repo>/.toksi/ai-memory/`) |
| `src/codegraph/` (`kernel/langs.rs`, `openwiki.rs`) | 45-language Tree-sitter indexer (`langs.rs`), call graph, blast radius & OpenWiki synthesizer |
| `src/rules/` (`skill_validator.rs`, `skills_catalog.rs`, `embedded/`) | In-tree `caveman`, `ponytail`, `principles` + `agentskills` YAML frontmatter auto-healer & deduplicator |
| `src/security/` (`defenseclaw`, `hookguard`, `skill_scanner`, `cel_decision`, `pipeline_taint`, `bytecode_audit`, `sarif`, `audit`, `yara_mcp`, `analyzer`, `sage_rules`, `skill_audit`, `osv`, `semgrep_engine`, `external_analyzers`) | `<enforce>` policy guard, `defenseclaw` fail-closed hookguard & audit, `skill-scanner` multi-engine/CEL policy, & built-in 41-language AST static code security analyzer + OSV lockfile scanner |
| `src/plugins/` (`mod`, `manifest`, `loader`, `hooks`, `slash_router`, `cli_bridge`, `analyzer_plugin`) | Toksi Unified Plugin System loading 7 mount surfaces (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP`, `agent_to_agent`) under `<enforce>` |
| `src/agent_runtime/` (`acp.rs`, `checkpoint.rs`, `deep_planning.rs`, `fleet.rs`, `graph_checkpoint.rs`, `guardrails.rs`, `modes.rs`, `zeroclaw_core.rs`) | Pure-Rust agent orchestrator wired out of the box ("off the back") via `DirectWireBus` & `:8787`/`:9098`/`:9099`/`:18787` to all gateways, Protocol Bus, `HeadroomEffortCompressor`, `src/plugins/`, and all 67 vaults (bridging ACP, LangGraph, DeepAgents, Bosun, Cline, KiloCode, ZeroClaw, AlphaCode) |
| `src/telemetry/` (`tokview.rs`, `traces.rs`), `src/tui/`, `src/dashboard/` | Token/effort attribution (`tokview`), 9-tab mouse-capable Ratatui TUI, `:9100` HTMX/Alpine Web Cockpit |
| `src/env_injector/`, `src/tools/`, `src/agents/` | Non-destructive `<enforce>` rule injection & multi-agent adapters |

## Key Verification Paths
- `crates/toksi/tests/` — Integration suites (`phase1`..`phase18_4`), `benchmarks.rs`, `e2e_tests.rs`, `keyring_db_byok_verification.rs`.
- `inspect-vault/` — **67** audited upstream research repositories (`36` prior + `31` Wave 18, read-only reference vault).
