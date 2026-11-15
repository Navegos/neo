# Project Memory Summary (Wave 18 / v5.0)

_Synchronized 2026-11-15: Wave 18 architecture consolidation across 67 audited vaults and 90+ ecosystems._

## Current Status
- Consolidated master architecture in `crates/toksi/CONSOLIDATION_PLAN.md`, `crates/toksi/PLAN.md`, `crates/toksi/README.md`, and `d:/AI/neo/.agents/brain/` upgraded to **Wave 18 / v5.0**.
- Total audited research vaults expanded from **36** to **67** (**31** new Wave 18 vaults adding `defenseclaw`, `skill-scanner`, vector stores, wikis, observability, ACP/agent runtimes, SAST/OSV security, and `agentskills` ecosystems).
- Legacy `.projectmem/` pre-read references across all existing phase prompts and master plans repaired to canonical `<repo>/.toksi/projectmem/`, while preserving runtime auto-migration logic for legacy folders.

## Active Architectural Decisions
- **Canonical Workspace Root**: `<repo>/.toksi/<tool>/` is the sole repo-local storage path (`<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/}`), preventing home-dir leakage and loose root dot-directories.
- **8-Port Unified Daemon**: `:8787` (Headroom Proxy), `:8788` (Grok OAuth), `:8789` (Copilot VS Code), `:8790` (Copilot CLI), `:9098` (Protocol Bus: MCP/A2A/ACP/SSE), `:9099` (Gateway & Prometheus Metrics), `:9100` (Web Cockpit), `:18787` (BYOK Gateway).
- **Zero Paid-Tier Restrictions**: Every Toksi feature (41-lang static security analyzer, `agentskills` YAML healer, Unified Plugin System, Headroom effort scaling, Google Cloud ADC BYOK gateway, plan checkpoint/resume, `:9100` Cockpit) operates offline-first with zero subscription gates.
- **Toksi Unified Plugin System (`src/plugins/`)**: Single host loading 7 mount surfaces (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP` on `:9098`, `agent_to_agent`) under `<enforce>` pre-load scan, tracked in SQLite `plugins_registry` and managed via `toksi plugin` + `/plugin` (13th instant slash command).
- **`<enforce>`, `defenseclaw`, `skill-scanner` & Built-in 41-Language Security**: All rules/skills/plugins/scripts are guarded by `<enforce>` boundary tags, `defenseclaw` fail-closed hookguard & audit evidence (`codesig`, `tool-chains.json`), and `skill-scanner` multi-engine taint/bytecode/bounded CEL policy alongside `sage`, `deepsec`, `codex-security`, `semgrep-rules`, and `osv-scanner` lockfile parsers.
- **Google Cloud ADC (`AGY_ADC_AUTH=true`)**: Native pure-Rust OAuth2/service-account ADC exchange honoring VS Code `"gcp.project"` / `"gcp.location"` (`us-central1` default) with zero subprocess overhead.
- **Thinking-Effort Optimization (`Low`/`Medium`/`High`)**: Single `/effort` interface dynamically translates upstream reasoning budgets and scales Headroom compression aggressiveness.
- **Safe Plan Checkpointing**: Atomic `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` (incl. active plugin snapshot) + `events.jsonl` snapshots enable zero-loss `/resume` after interruption, rate-limit `429`, or crash.

## Key Files
- `crates/toksi/Cargo.toml` — Consolidated workspace crate manifest
- `crates/toksi/CONSOLIDATION_PLAN.md` — Wave 18 / v5.0 Master Plan (67 vaults, 34 phases, ready prompts)
- `crates/toksi/PLAN.md` — Wave 18 / v5.0 Architecture & 8-port topology
- `crates/toksi/research/RESEARCHPLAN.md` — Detailed Wave 18 engineering blueprint
- `crates/toksi/src/codegraph/kernel/langs.rs` — 45-language Tree-sitter registry (`dockerfile`/`vue`/`scss` audit target)
- `crates/toksi/src/rules/` — In-tree skills (`caveman`, `ponytail`, `principles`) + Wave 18.2 `skill_validator.rs` / `skills_catalog.rs`
- `crates/toksi/src/keyring/` — Encrypted OS Keychain/DPAPI + Wave 18.3 Google Cloud ADC (`adc.rs`)
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` — 34-phase execution index (`PHASE_01`..`PHASE_18_4`)
