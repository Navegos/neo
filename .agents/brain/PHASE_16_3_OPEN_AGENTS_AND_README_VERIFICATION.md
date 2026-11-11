# Phase 16.3: Open Agents Directory Protocols & README Verification

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/agents-structures, neo/inspect-vault/tokless is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new directory layouts, agent schemas, and protocol compliance into consolidated toksi.

Execute Phase 16.3: Open Agents Directory Protocols & README Verification.
Sources:
  - neo/inspect-vault/agents-structures (directory layouts and schemas)
  - neo/inspect-vault/tokless
Targets:
  - crates/toksi/src/agents/mod.rs
  - crates/toksi/src/agents/manager.rs
  - crates/toksi/src/agents/open_agents.rs
  - crates/toksi/src/commands/agent.rs
  - crates/toksi/README.md

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors and anyhow::Result with .context() in CLI.
3. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all directory joins and configuration paths.
4. Configured-Only & User-Level Auto-Install vs Explicit Repo Opt-In:
   - Agent hooks, MCPs, instructions, rules, skills, and configs auto-install ONLY to user-level directories (`~/.<agent>/` or `~/.agents/`) ONLY for user-configured/selected agents, with granular component toggles (`hooks`, `mcp`, `skills`, `rules`, `instructions`, `configs`).
   - Workspace repository installation (`<repo>/.<agent>/`) is NEVER automatic; it only installs into root `<repo>/` when explicitly selected by the user (and never into nested crate subfolders).
5. Comprehensive Documentation: Rewrite `crates/toksi/README.md` with full Wave 17 technical specifications, configuration options, protocols, and agent preparation guides.

Requirements:
1. Open Agents & Vendor Agent Directory Support (`agents-structures/` parity):
   - Open standard directory convention: `.agents/` (`mcp.json`, `models.json`, `system-prompt.md`, `agents.md`, `agents/`, `skills/`, `.agents/mcp -> skills/<tool>`, `tasks/`, `knowledge/`, `memories/`, plus working `"openagents"` adapter in `mcp_adapters.rs`).
   - Vendor protocol alignment: `.gemini/` (`settings.json`, `commands/*.toml`, split `antigravity-cli` vs `antigravity-ide`), `.copilot/` (`hooks/`) & `.github/copilot/` (split `copilot-cli` vs `copilot-vscode`), `.claude/` (`.mcp.json`), `.codex/`, `.cursor/`, `.grok/` (`hooks/*.json`).
   - Purge legacy `toksi-*` / `tokless_*` / `agy-hook codegraph-index` / `rtk hook cursor` references and auto-merge nested subfolder `.<agent>/` / `.toksi/` dirs.
   - Lifecycle management operations via CLI (`toksi agent ...`), TUI, and Web Dashboard: `install`, `health`, `mismatch`, `upgrade`, `fix`, `restore`, `remove`, `edit`.
2. Comprehensive Technical README (`crates/toksi/README.md`):
   - Unified 7-port network architecture (`:8787`, `:8788` Grok, `:8789`/`:8790` Copilot, `:18787` BYOK, `:9099` Gateway, `:9100` Cockpit).
   - `~/.toksi/config.toml` reference & `toksi daemon|proxy|dashboard start|stop|restart --agents` CLI control.
   - Native `toksi chat|run|exec|swarm` coding CLI (`src/agent_runtime/`), `DirectWireBus` (`direct-wired`), and 10 in-chat slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`).
   - 65/90+ Categories 1–5 + 22 local engines, multi-instance `<byok, model, effort>` routing, **zero paid-tier restrictions**, read-only quota windows, and mandatory Headroom BYOK optimization.
   - 41+ Tree-sitter languages, strict `<repo>/.toksi/<tool>` repo-only storage, monorepo subproject partitioning, `.gitignore`/`.ignore` dynamic purge, and 9-tab TUI/Web parity.
   - Complete CLI command reference (`--help` matched 1:1) and `.toksi/test-baseline.json` incremental verification guide.
3. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and incremental `.toksi/test-baseline.json` `cargo test` ensuring 0 warnings and 100% pass rate.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **OpenAgents Directory Manager** (`src/agents/open_agents.rs`): Reads and writes vendor-agnostic `.agents/` folder structures.
- **Agent Lifecycle Operations** (`src/agents/manager.rs`): Handles installation, health checks, version upgrades, and clean removals across configured agents (`18+` built-in harnesses + `.agents/` + Category 1–5 layouts).
- **Technical Documentation** (`README.md`): Provides complete, professional reference documentation for developers and automated coding agents.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi` passes 100%.
- `crates/toksi/README.md` covers all technical features, commands, and conventions.
