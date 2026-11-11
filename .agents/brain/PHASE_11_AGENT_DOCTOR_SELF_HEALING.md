# Phase 11: Agent Doctor & Self-Healing Parity

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_11_AGENT_DOCTOR_SELF_HEALING.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/tokless, neo/inspect-vault/agents-structures is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and agent harnesses into consolidated toksi.

Execute Phase 11: Agent Doctor & Self-Healing Parity.
Sources:
  - neo/inspect-vault/tokless
  - neo/inspect-vault/agents-structures
Target: crates/toksi/src/commands/doctor.rs, crates/toksi/src/agents/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors and `anyhow::Result` with `.context()` in CLI.
3. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all discovered agent directories.
4. PATH Validation Mandate: Validate agent CLI executables (`agy`, `copilot`, `claude`, `codex`, `cursor`, `grok`, etc.) in system `PATH` cleanly via `check_executable_in_path` without blocking errors.
5. Workspace Topology: Enforce single root `<repo>/.toksi/<tool>` (never `~/.toksi/<tool>`) and root-only workspace agent directories (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.gemini`, `.github`, `.agents`).
6. Standardized Naming: Hook syntax is `<tool>-hook <agent>`, separated `antigravity-cli` / `antigravity-ide` & `copilot-cli` / `copilot-vscode`, `agents-structures` parity, `skills/<tool>` MCP dir, and `DirectWireBus` (`direct-wired`) support. Normalized MCP identifiers: `projectmem`, `context`, `codegraph`, `icm`, `aimemory`, `rtk` (purge legacy `toksi-*` & `tokless_*`).

Requirements:
1. Implement full self-healing audit in `crates/toksi/src/commands/doctor.rs` & `agents/manager.rs`:
   - Audit and verify configured agent configurations (`18+` built-in adapters including `"openagents"` + Category 1–5 layouts) across user homes and workspace roots.
   - Detect and fix broken legacy MCP configurations (purge stale `node`, `npx`, `python`, `uvx`, `toksi-*`, `tokless_*`, `toksi-codegraph-init.js` commands, replacing with native `toksi <tool>` / `DirectWireBus`).
   - Standardize hook configurations across all agents to `<tool>-hook <agent>` (e.g. `codegraph-hook agy`, `projectmem-hook agy`; purge `agy-hook codegraph-index` and `rtk hook cursor`).
   - Validate presence of required in-tree rules (`Karpathy`, `Caveman` v4, `Ponytail`), skills, and instructions (`~/.toksi/config.toml` compliant).
2. Implement `toksi doctor --fix`:
   - Automatically repairs misconfigured MCP configs, repairs legacy hooks, removes stale shims, and synchronizes rules strictly for configured agents (never touching workspace repo automatically without explicit opt-in).
   - Merges any nested subfolder `.toksi` / `.<agent>` directories into root `<repo>/.toksi/` / `<repo>/.<agent>/` and cleans them up.
3. Ensure 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Diagnostic Engine** (`src/commands/doctor.rs`): Audits agent harnesses, ports, PATH executables, OS Keyring status, and configuration files.
- **Auto-Healing Pipeline** (`doctor --fix`): Safe in-place modification of agent JSON, JSON5, TOML, and YAML config files without breaking custom user settings.
- **Rule & Skill Integrity Verifier**: Confirms that Karpathy principles, Caveman compression, and Ponytail build discipline are present and up to date in the target agent directories.

### 2. Disciplinary Standards & Invariants
- Never overwrite custom user MCP tools or instructions; only repair and update `toksi`-managed entries.
- Use atomic temporary file writes when updating configuration files to prevent partial file corruption.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- doctor` passes 100%.
- `toksi doctor` reports granular diagnostic health across all installed agents.
