# Phase 06: In-Tree Rules & Skills Integration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_06_IN_TREE_RULES_AND_SKILLS.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/andrej-karpathy-skills, neo/inspect-vault/caveman, neo/inspect-vault/ponytail is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new guidelines, rule updates, and skill definitions into consolidated toksi.

Execute Phase 06: In-Tree Rules & Skills Integration.
Sources:
  - neo/inspect-vault/andrej-karpathy-skills
  - neo/inspect-vault/caveman
  - neo/inspect-vault/ponytail
Target: crates/toksi/src/rules/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Zero External Downloads: Embed markdown rules and skill definitions at compile-time via include_str!(). Zero network/git fetches at runtime.
4. Universal Forward-Slash (/) Standard: Use normalize_path() for all rule destinations and file targets.
5. Directory Placement Policy:
   - User-Level configs (~/.<agent>/ or ~/.agents/) are auto-installed if selected.
   - Workspace repository install is strictly optional and user-selected.
   - Workspace rules are placed strictly in the first workspace root directory.

Requirements:
1. Ingest guidelines directly into `crates/toksi/src/rules/embedded/`:
   - `principles.rs` (Karpathy engineering principles: think before coding, simplicity first, surgical changes, goal-driven execution)
   - `caveman.rs` (ASD-STE100 primitive English compression + Wave 17 v4 modes: `/ultracave`, `/megacave`, `caveman-commit`, `caveman-review`, `caveman-compress`, `shrink`, `browse`, `.blocks`)
   - `ponytail.rs` (lazy senior developer build discipline ladder + mode variants, companion skills, and statusline telemetry)
2. Implement universal synchronizer in `crates/toksi/src/rules/sync.rs` strictly for **user-configured/selected** agents across `18+` built-in harnesses and open conventions (`agents-structures/` parity) with **granular component toggles** (`hooks`, `mcp`, `skills`, `rules`, `instructions`, `configs`):
   - `.agents/` (OpenAgents: `agents.md`, `system-prompt.md`, `skills/`, `.agents/mcp`)
   - `.gemini/` (Antigravity: `GEMINI.md`, `settings.json`, `commands/*.toml`, `rules/`, `skills/`)
   - `.copilot/` & `.github/` (Copilot: `copilot-instructions.md`, `hooks/`, `skills/`)
   - `.claude/` (Claude Code: `CLAUDE.md`, `.mcp.json`, `rules/`, `skills/`)
   - `.codex/` (Codex: `AGENTS.md`, `rules/`)
   - `.cursor/` (Cursor: `.cursorrules`, `rules/*.mdc`, `skills/`)
   - `.grok/` (Grok: `hooks/*.json`, `skills/`, `rules/`)
   Plus auto-merge nested subfolder `.toksi/` / `.<agent>/` dirs into root `<repo>/` cleanly.
3. Support skill wiring from MCP settings: map `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp`, `.agents/mcp` to `skills/<tool>`.
4. Ensure 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Compile-Time Embedded Rules** (`src/rules/embedded/`): Embedded via `include_str!()` to guarantee complete standalone binary portability without external files.
- **Universal Synchronizer** (`src/rules/sync.rs`): Injects or updates rule files across all agent configuration directories in the workspace root or user home.
- **Rules Formatting & Header Preserver**: Merges toksi-managed rules cleanly while preserving existing user-defined rules.

### 2. Disciplinary Standards & Invariants
- Embedded markdown text must be clean UTF-8 with normalized Unix `\n` line endings.
- Do not overwrite user-customized sections; append or update within designated delimiter blocks (`<!-- TOKSI-RULES-START -->`).

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- rules` passes 100%.
- Compiled `toksi` binary contains all rules in `.rodata` with 0 disk reads required.
