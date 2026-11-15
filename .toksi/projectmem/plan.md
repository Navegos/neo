# Toksi Consolidated Master Plan (Wave 18 / v5.0)

- **Target Crate**: `crates/toksi` (`d:/AI/neo`)
- **Master Architecture**: `crates/toksi/CONSOLIDATION_PLAN.md`, `crates/toksi/PLAN.md`, `crates/toksi/research/RESEARCHPLAN.md`
- **Audited Vaults**: **67** `inspect-vault/*` repositories (`36` Waves 1–17 + `31` Wave 18) across **90+ AI coding agents, runtimes, local inference engines & security scanners**
- **Daemon Topology**: **8-Port Unified Local Daemon** (`:8787` Headroom Proxy, `:8788` Grok OAuth, `:8789` Copilot VS Code, `:8790` Copilot CLI, `:9098` Protocol Bus MCP/A2A/ACP/SSE, `:9099` Enterprise Gateway & Metrics, `:9100` Detached Web Cockpit, `:18787` BYOK Gateway w/ Google Cloud ADC `AGY_ADC_AUTH=true`)
- **Universal Slash Commands (`< 5ms`)**: **13 commands**: `/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/plugin`, `/resume`

---

## Implementation Phases Checklist

### Waves 1–14: Core Pure-Rust Consolidation (`PHASE_01`..`PHASE_14`)
- [x] **Architecture & Research**: Audited all **67** vaults (`inspect-vault/*`) and 90+ ecosystem tools; created Wave 18 blueprints in `crates/toksi/research/RESEARCHPLAN.md`.
- [ ] **Phase 1–5**: Pure-Rust `rtk` filters, `projectmem` (strict `<repo>/.toksi/projectmem/`), `context-mode` FTS5 BM25 sandbox, `icm` + `ai-memory` hybrid engine, and `codegraph` 45-lang AST engine.
- [ ] **Phase 6–10**: In-tree skills (`caveman`, `ponytail`, `principles`) + `<enforce>` wrapper, Headroom AST/TOON proxy (`:8787`), downloader purge, legacy healing, non-destructive rule injection.
- [ ] **Phase 11–14**: Self-healing `toksi doctor`, 6->9 tab Ratatui TUI & `:9099` metrics, E2E benchmarks, native OS keyring + SQLite (`byok_profiles` w/ `auth_mode`, `gcp_project`, `gcp_location`, `plugins_registry`).

### Waves 15–16: Enterprise Topology, Protocols & UI (`PHASE_15_1`..`PHASE_16_3`)
- [ ] **Phase 15.1–15.6**: `<repo>/.toksi/` workspace isolation, unified 8-port daemon (`:9098` Protocol Bus), smart CLI providers (zero paid-tier blocks), MCP/A2A/ACP/SSE, `tokview` token attribution, vault intelligence bridges across all **67** vaults.
- [ ] **Phase 16.1–16.3**: Mouse-capable 9-tab Ratatui TUI, HTMX/Alpine Web Cockpit (`:9100`) with `skill-scanner` / `defenseclaw` / Plugin Manager, universal 90+ agent hooks & **13** slash commands.

### Wave 17: TOML Config, Direct Wire & Incremental Baselines (`PHASE_17_1`..`PHASE_17_7`)
- [ ] **Phase 17.1–17.4**: Canonical `~/.toksi/config.toml` (incl. `[plugins]`, `defenseclaw_fail_closed`, `skill_scanner_policy`), multi-model SQLite catalogs (`plugins_registry`), complete `langs.rs` grammar coverage (`dockerfile`, `vue`, `scss`), vault ecosystem connectors.
- [ ] **Phase 17.5–17.7**: Native Rust `DirectWireClient`, **13** `< 5ms` slash commands (`/plugin`), streaming SSE usage accounting + 429 self-heal, async 9-tab UI & `.toksi/test-baseline.json` incremental gates.

### Wave 18 (v5.0): 31 New Vaults, Unified Plugins, `defenseclaw`, `skill-scanner`, ADC, Effort & Checkpoints (`PHASE_18_1`..`PHASE_18_4`)
- [ ] **Phase 18.1**: Synthesize **31** new Wave 18 vaults (`lance`, `openwiki`, `tokview`, `agent-control`, `agent-client-protocol`, `python-genai`, `python-aiplatform`, `langgraph`, `langchain`, `langfuse`, `langflow`, `deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `sage`, `codex-security`, `deepsec`, `osv-scanner`, `semgrep`, `semgrep-rules`, `research-securety-skills`, `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, `agentskills`, `defenseclaw`, `skill-scanner`) + `src/plugins/` manifest/discovery foundations.
- [ ] **Phase 18.2**: Enforce `<enforce>` across all skills/scripts/rules/plugins, ship Toksi Unified Plugin System (`src/plugins/*`, `toksi plugin`, `/plugin`, SQLite `plugins_registry`), integrate `defenseclaw` fail-closed hookguard/audit + `skill-scanner` multi-engine/CEL policy, implement `agentskills` YAML frontmatter auto-healer + skill deduplicator, and build built-in 41-language static code security analyzer + OSV lockfile scanner (`src/security/`).
- [ ] **Phase 18.3**: Implement Google Cloud ADC (`AGY_ADC_AUTH=true` + `"gcp.project"`/`"gcp.location"`) Vertex gateway, Low/Medium/High thinking-effort optimization (`/effort`), and atomic `.toksi/projectmem/plan_checkpoints/` (w/ active plugin snapshot) pause/resume (`/resume`).
- [x] **Phase 18.4 (Docs Sync)**: Repair all stale `.projectmem/` pre-read references to `.toksi/projectmem/`, synchronize all 35 `.agents/brain/*.md` files, `CONSOLIDATION_PLAN.md`, `PLAN.md`, `README.md`, and `.toksi/projectmem/*`, and verify clean workspace compile/clippy baseline.
