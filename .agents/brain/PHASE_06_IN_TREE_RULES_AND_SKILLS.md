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
1. Ingest guidelines directly into crates/toksi/src/rules/embedded/:
   - principles.rs (Karpathy engineering principles: think before coding, simplicity first, surgical changes, goal-driven execution)
   - caveman.rs (ASD-STE100 primitive English compression: terseness, zero fluff, full technical accuracy)
   - ponytail.rs (lazy senior developer build discipline ladder: reuse first, minimal runnable checks, YAGNI)
2. Implement universal synchronizer in `crates/toksi/src/rules/sync.rs` targeting all 17 supported agents and open conventions:
   - `.agents/` (OpenAgents: `agents.md`, `system-prompt.md`, `skills/`)
   - `.gemini/` (Antigravity: `GEMINI.md`, `rules/`, `skills/`)
   - `.copilot/` & `.github/` (Copilot: `copilot-instructions.md`, `skills/`)
   - `.claude/` (Claude Code: `CLAUDE.md`, `rules/`, `skills/`)
   - `.codex/` (Codex: `AGENTS.md`, `rules/`)
   - `.cursor/` (Cursor: `.cursorrules`, `rules/*.mdc`, `skills/`)
   - `.grok/` (Grok: `skills/`, `rules/`)
3. Support skill wiring from MCP settings: map `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp` to `skills/<tool>`.
4. Ensure 0 compiler warnings and passing unit tests.
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
