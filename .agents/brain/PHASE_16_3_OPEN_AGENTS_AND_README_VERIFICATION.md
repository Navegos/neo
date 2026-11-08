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
3. Universal Forward-Slash (/) Standard: Use normalize_path() for all directory joins and configuration paths.
4. User-Level Auto-Install & Selective Workspace Installation:
   - Agent hooks, MCPs, instructions, rules, skills, and configs are auto-installed to the user-level configuration directory (`~/.<agent>/` or `~/.agents/`) if selected for install.
   - Workspace repository installation is NEVER automatic; it is only installed into `<root>/` if the user explicitly triggers it.
5. Comprehensive Documentation: Rewrite `crates/toksi/README.md` with full technical specifications, configuration options, protocols, and agent preparation guides.

Requirements:
1. Open Agents & Vendor Agent Directory Support:
   - Open standard directory convention: `.agents/` (`mcp.json`, `models.json`, `system-prompt.md`, `agents.md`, `agents/`, `skills/`, `tasks/`, `knowledge/`, `memories/`).
   - Support all vendor protocols: `.gemini/` (Antigravity), `.copilot/` & `.github/` (Copilot), `.claude/` (Claude), `.codex/` (Codex), `.cursor/` (Cursor), `.grok/` (Grok).
   - Lifecycle management operations via CLI (`toksi agent ...`), TUI, and Web Dashboard: `install`, `health`, `mismatch`, `upgrade`, `fix`, `restore`, `remove`, `edit`.
2. Comprehensive Technical README (`crates/toksi/README.md`):
   - 5-port network architecture explanation.
   - Single-daemon launch (`toksi daemon start`).
   - 17 supported agent configurations and open protocols.
   - 39+ BYOK providers and smart local agent CLI bridges.
   - Paid quota enforcement and graceful free-tier disqualification.
   - In-tree rules (Karpathy, Caveman, Ponytail) and dynamic ignore purging.
   - Complete CLI command reference and environment variables.
3. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and 100% tests pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **OpenAgents Directory Manager** (`src/agents/open_agents.rs`): Reads and writes vendor-agnostic `.agents/` folder structures.
- **Agent Lifecycle Operations** (`src/agents/manager.rs`): Handles installation, health checks, version upgrades, and clean removals across all 17 agent directories.
- **Technical Documentation** (`README.md`): Provides complete, professional reference documentation for developers and automated coding agents.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi` passes 100%.
- `crates/toksi/README.md` covers all technical features, commands, and conventions.
