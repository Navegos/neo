# Toksi Master Execution Index & Brain Guide (Wave 18 / v5.1)

This directory (`d:/AI/neo/.agents/brain/`) holds the modular, non-overlapping implementation phases for the **Toksi Super-Intelligence & Zero-Regression Runtime Consolidation (Wave 18 / v5.1)** spanning all **78 mandated vaults** in `inspect-vault/*` (`36` Waves 1–17 + `42` Wave 18 new & companion vaults across `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, `<skills>` `Navegos/skills`, incl. `opencode`, `cve-*`, `osv-schema`, `skilldoctor`, `sandbox-runtime`, `defenseclaw`, `skill-scanner`) and all **65 / 90+ AI coding agents / runtimes / local inference engines / security scanners**. Every phase prompt starts by reading the global architectural documents — including canonical `<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}` — so parallel or sequential agents maintain identical structural awareness.

## Mandatory Global Pre-Reads (Copy into Every Session)
```text
Read these master architectural files FIRST before editing any code:
- crates/toksi/CONSOLIDATION_PLAN.md (Wave 18 / v5.1, all 78 vaults & 65/90+ ecosystems)
- crates/toksi/PLAN.md (Wave 18 / v5.1 architectural blueprint & 8-port topology)
- crates/toksi/research/RESEARCHPLAN.md (Sections 0–13: Wave 18 blueprints, 3-repo topology, zero-bloat skills, ADR, sandbox, kill-switch, ADC, effort, security)
- crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
- crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
- crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
- crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
- .toksi/projectmem/plan.md
- .toksi/projectmem/PROJECT_MAP.md
- .toksi/projectmem/summary.md
- .agents/brain/PHASE_<N>_<NAME>.md (your active phase specification)
```

---

## Wave 18 / v5.1 Canonical Invariants
- **78 Mandated Vaults & 3-Repository Ecosystem**: `36` Waves 1–17 vaults + `42` Wave 18 additions (`lance`, `openwiki`, `tokview`, `agent-control`, `agent-client-protocol`, `python-genai`, `python-aiplatform`, `langgraph`, `langchain`, `langfuse`, `langflow`, `deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `opencode`, `sandbox-runtime`, `sage`, `codex-security`, `deepsec`, `osv-scanner`, `semgrep`, `semgrep-rules`, `cve-schema`, `cve-validation-library`, `cve-services`, `osv-schema`, `cve-bin-tool`, `cve-lite-cli`, `defenseclaw`, `skill-scanner`, `research-securety-skills`, `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, `agentskills`, `skilldoctor`, plus companion repos `<advisory>` `Navegos/advisory` & `<skills>` `Navegos/skills`).
- **Zero Binary Bloat Externalized Skills/Rules**: Never embed skills, rules, `.md`, `.yaml`, or `.json` files inside `crates/toksi/`. Stage curated assets outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`) synced to `<skills>` (`Navegos/skills`) and optional `~/.toksi/user-skills`, healed/graded by `skilldoctor` + `agentskills` and vetted by `skill-scanner` + `defenseclaw`.
- **Unified 8-Port Daemon & Universal Headroom Effort Pass (`:8787..:8790, :9098, :9099, :9100, :18787`)**: Headroom Proxy (`:8787`), Grok OAuth (`:8788`), Copilot VS Code (`:8789`), Copilot CLI (`:8790`), Protocol Bus MCP/A2A/ACP/SSE (`:9098`), Enterprise AI Gateway (`/metrics`, `/v1/*`, `/v1beta/*`) (`:9099`), Detached Web Cockpit (`:9100`), BYOK & Vertex ADC Gateway (`:18787`). ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`) MUST pass through the shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`), and native `toksi` agent (`src/agent_runtime/`) binds out of the box ("off the back") via `DirectWireBus` + `:8787`/`:9098`/`:9099`/`:18787` to every gateway, Protocol Bus, `src/plugins/`, ADR/Sandbox, and all 78 vaults with zero manual setup.
- **16 Universal Slash Commands (`< 5ms`)**: `/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`.
- **Zero-Plaintext Blind Engine, Native Git/PR Validator, Unified Plugins, ADR, Sandbox & Kill-Switch**: Keyed `BLAKE3` + `sparse::DFA` (`src/security/`, `toksi compile-blind`), `DisclosureGate` + 12-DB `UpstreamDeduplicator` + `src/validator/` (`toksi advisory`), `src/plugins/` (`toksi plugin`, `/plugin`, `plugins_registry`), `src/security/adr.rs` + `src/agent_runtime/sandbox.rs` (3-OS native + `sandbox-runtime` + `sbx` CLI), and `src/commands/guard.rs` (`CommandGuard` + immediate bypass kill-switch shutting down all `toksi` services).
- **Single Repo Memory Root**: Always `<repo>/.toksi/<tool>/` (canonical `.toksi/projectmem/` with `plan_checkpoints/<plan_id>.json`). Zero paid-tier restrictions anywhere.

---

## Complete 35-Phase Execution Catalog (Waves 1–18 — 36 Files Total)

### Wave 1–14: Pure Rust Foundation & Zero-Downloader Hardening
1. `PHASE_01_RTK_FILTER_ENGINE.md` — Port RTK TOML filters, TEE recovery, `toksi gain`, SQLite history.
2. `PHASE_02_PROJECTMEM_PURE_RUST.md` — JSONL append-only ledger + derived `summary.md` + strict `<repo>/.toksi/projectmem/` isolation & legacy `.projectmem/` auto-migration.
3. `PHASE_03_CONTEXTMODE_PURE_RUST.md` — FTS5 BM25 sandbox analyzer & multi-language chunking inside `<repo>/.toksi/context-mode/`.
4. `PHASE_04_ICM_AND_AIMEMORY.md` — Hierarchical SQLite memory + hybrid BM25/vector engine.
5. `PHASE_05_CODEGRAPH_PURE_RUST.md` — Multi-language AST structural indexer, call graphs & blast-radius trace.
6. `PHASE_06_IN_TREE_RULES_AND_SKILLS.md` — Externalized skills/rules staged outside `crates/toksi/` synced to `<skills>` (zero binary bloat) + `<enforce>` security tags + `skilldoctor`/`agentskills` YAML frontmatter healer (`Phase 18.2`).
7. `PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md` — Pure-Rust Headroom AST/TOON proxy (`:8787`), dynamic effort scaling, and free-tier BYOK model routing.
8. `PHASE_08_DOWNLOADER_PURGE.md` — Eliminate third-party npm/pip/curl installers; 100% pure-Rust single binary.
9. `PHASE_09_LEGACY_HEALING_UNHIJACK.md` — Un-hijack corrupted hooks/settings (`~/.claude/`, `~/.codex/`, `~/.gemini/`, `.antigravity/`) & quarantine malicious plugins.
10. `PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md` — Append-only safe agent rule markers (`<enforce>`) & reversible env export shims.
11. `PHASE_11_AGENT_DOCTOR_SELF_HEALING.md` — Self-healing `toksi doctor` CLI & automatic SQLite WAL/FT5/legacy-dotdir recovery.
12. `PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md` — Initial 6-tab Ratatui TUI & Prometheus `/metrics` (`:9099`).
13. `PHASE_13_E2E_VERIFICATION_BENCHMARKS.md` — End-to-end integration tests, RTK benchmarks & Wave 18 doc hygiene verification.
14. `PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md` — Native OS Keychain/DPAPI, SQLite `byok_profiles` (with `auth_mode`, `gcp_project`, `gcp_location`), `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, and Google Cloud ADC bridge.

### Wave 15–16: Enterprise Topology, Vault Connectors, TUI & Cockpit
15.1. `PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md` — `<repo>/.toksi/` consolidation, nested subproject migration, strict `.toksiignore` purge.
15.2. `PHASE_15_2_UNIFIED_5_PORT_DAEMON.md` — Unified multi-port daemon (expanded in Wave 18 to **8 ports** including `:9098` Protocol Bus).
15.3. `PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md` — Universal quota intelligence (`antigravity`, `copilot-chat`, `codex`, `claude`, `zai`, `gemini`) with zero paid-tier restrictions.
15.4. `PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md` — MCP/A2A/ACP/SSE protocol bus (`:9098`), OTLP/OpenTelemetry, headless CI guardrails.
15.5. `PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md` — Token savings attribution (`tokview`), effort-tiered accounting, offline/429 self-healing.
15.6. `PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md` — Zero-download parity across OpenViking, Graphiti, LEANN, PageIndex, Context7, Gitingest, Qdrant/Vector bridges.
16.1. `PHASE_16_1_MOUSE_CAPABLE_TUI.md` — Full mouse-driven 9-tab Ratatui TUI (Tab 1 `tokview`, Tab 4 plan checkpoints, Tab 5 `skill-scanner` CEL/SARIF + `Navegos/advisory` search, Tab 6 `defenseclaw` + ADR/sandbox/kill-switch audit + `sage`, Tab 7 Plugin Manager + External Skills Sync + ADC/effort, Tab 9 `plugins_registry`).
16.2. `PHASE_16_2_WEB_COCKPIT_DASHBOARD.md` — Detached HTMX + Alpine Web Cockpit (`:9100`) mirroring all 9 tabs, `/security` (`defenseclaw` + `skill-scanner`), `/advisory`, `/skills`, `/sandbox`, `/api/plugins`, `/effort`, `/resume`, `plugins_registry` DB export/import, and ADC status.
16.3. `PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md` — Universal 65/90+ agent hooks, `toksi plugin`, **16** slash commands (`/byok`..`/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`), and doc alignment.

### Wave 17: Unified Runtime, TOML Config, 41+ Language Tree-sitter & Direct Wire
17.1. `PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md` — Canonical `~/.toksi/config.toml` (incl. `[repositories]`, `[plugins]`, `[security]`) + `toksi config/byok/auth` CLI + free tier defaults (`AGY_ADC_AUTH` toggle).
17.2. `PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md` — Multi-model DB catalogs + `plugins_registry`/`blind_signatures`/`advisory_embargoes`/`adr_audit_events`, fallback chains, Headroom single-pass AST/TOON pipeline.
17.3. `PHASE_17_3_TREESITTER_41_LANGS_AND_REPO_ISOLATION.md` — Complete 45-language `langs.rs` support (resolving `"dockerfile"` grammar gap + `"vue"`/`"scss"` specialization) + strict repo hash isolation.
17.4. `PHASE_17_4_VAULT_ECOSYSTEM_CONNECTORS_AND_HOOK_CLEANUP.md` — 65/90+ ecosystem adapter generation, all 78 vaults parity, hook cleanup, legacy dotfile migration.
17.5. `PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md` — Native Rust agent runtime (`DirectWireClient`), `src/plugins/` mounts, `< 5ms` **16** universal slash commands (incl. `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`).
17.6. `PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md` — Streaming SSE usage parsing, 429 exponential backoff/failover, gateway telemetry.
17.7. `PHASE_17_7_ASYNC_9TAB_UI_INCREMENTAL_TESTS_AND_DOCS.md` — Async non-blocking 9-tab TUI & `:9100` Cockpit + `.toksi/test-baseline.json` incremental gates.

### Wave 18 (v5.1): 42 New/Companion Vaults, Unified Plugins, Zero-Plaintext Blind Engine, Native Validator, External `<skills>`, ADR/Sandbox/Kill-Switch, ADC, Effort & Checkpoints
18.1. `PHASE_18_1_29_NEW_VAULTS_RESEARCH_AND_SYNTHESIS.md` — Synthesize all 42 new & companion Wave 18 vaults (`lance`, `openwiki`, `tokview`, `agent-control`, `agent-client-protocol`, `python-genai`, `python-aiplatform`, `langgraph`, `langchain`, `langfuse`, `langflow`, `deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `opencode`, `sandbox-runtime`, `sage`, `codex-security`, `deepsec`, `osv-scanner`, `semgrep`, `semgrep-rules`, `cve-schema`, `cve-validation-library`, `cve-services`, `osv-schema`, `cve-bin-tool`, `cve-lite-cli`, `research-securety-skills`, `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, `agentskills`, `skilldoctor`, `defenseclaw`, `skill-scanner`, `<advisory>`, `<skills>`) + `src/plugins/` manifest discovery foundations.
18.2. `PHASE_18_2_ENFORCE_SKILLS_HEALER_AND_41_LANG_SECURITY_ANALYZER.md` — Enforce `<enforce>` across 3 repos (`<neo>`, `<advisory>`, `<skills>` with zero binary bloat), ship Toksi Unified Plugin System (`src/plugins/`, `toksi plugin`, `/plugin`, `plugins_registry`), Zero-Plaintext Blind Signature Engine (`src/security/`, `toksi compile-blind`), Native Git/PR Validator (`src/validator/`, 12-DB `UpstreamDeduplicator`, `toksi advisory`), `defenseclaw` + `skill-scanner`, `skilldoctor` + `agentskills` YAML frontmatter auto-healer, and built-in 41-language AST + OSV lockfile static code security analyzer.
18.3. `PHASE_18_3_GOOGLE_ADC_EFFORT_OPTIMIZATION_AND_PLAN_CHECKPOINT.md` — First-class Google Cloud ADC (`AGY_ADC_AUTH=true` + `"gcp.project"`/`"gcp.location"`) Vertex gateway, Low/Medium/High thinking-effort token optimization (`/effort`), and atomic `.toksi/projectmem/plan_checkpoints/` pause/resume (`/resume`).
18.4. `PHASE_18_4_LEGACY_PROJECTMEM_REPAIR_AND_MASTER_DOC_SYNC.md` — Canonical `<repo>/.toksi/projectmem/` enforcement, 36-file doc hygiene test suite, and final workspace verification (`cargo check`, `cargo clippy -- -D warnings`, `.toksi/test-baseline.json`).
18.5. `PHASE_18_5_AGENT_SOURCE_UNIFICATION_ADR_SANDBOX_AND_KILL_SWITCH.md` — Unify `<neo>/inspect-vault/*<agent>` source commands/skills/rules/MCPs/APIs into `toksi` agent, build ADR Layer (`src/security/adr.rs`), Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs` 3-OS native + `sandbox-runtime` + `sbx` CLI), and `CommandGuard` + Bypass Kill-Switch (`src/commands/guard.rs`).

---

## Global Verification Rules
Every phase agent MUST finish with:
1. `cargo check --workspace` — zero errors.
2. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
3. Focused phase integration test (`cargo test --test <phase_test>`) + incremental update to `.toksi/test-baseline.json`.
4. Strict `<repo>/.toksi/` repo-only isolation check + zero stale `.projectmem/` pre-read refs + zero unrequested background daemons.
