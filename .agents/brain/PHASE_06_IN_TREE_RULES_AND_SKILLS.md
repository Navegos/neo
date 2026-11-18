# Phase 06: Externalized Rules, `<skills>` Staging, `<enforce>` Guard & Skill Healing

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 0.1, 1.1.4, 3, 9, 12.1), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_06_IN_TREE_RULES_AND_SKILLS.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/andrej-karpathy-skills, neo/inspect-vault/caveman, neo/inspect-vault/ponytail, neo/inspect-vault/skilldoctor, neo/inspect-vault/agentskills, neo/inspect-vault/skills is updated; treat as newly added/updated code that must be audited, healed, staged externally outside crates/toksi/, and synchronized to configured agents.

Execute Phase 06: Externalized Rules & Skills Integration (Zero Binary Bloat).
Sources:
  - neo/inspect-vault/andrej-karpathy-skills
  - neo/inspect-vault/caveman
  - neo/inspect-vault/ponytail
  - neo/inspect-vault/skilldoctor
  - neo/inspect-vault/agentskills
  - neo/inspect-vault/skills (<skills> Navegos/skills)
Targets:
  - <neo>/agents/ and <neo>/rules/ (external staged skills/rules outside crates/toksi/)
  - crates/toksi/src/rules/{mod.rs,sync.rs,skill_validator.rs,skills_catalog.rs,skills_repo.rs}

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Zero Binary Bloat Policy (`RESEARCHPLAN.md` Section 1.1.4): NEVER embed skills, rules, `.md`, `.yaml`, or `.json` files inside the `toksi` binary (`crates/toksi/`). Stage curated assets outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`) synced to `<skills>` (`Navegos/skills`); installed `toksi` fetches vetted skills online from `<skills>` (or `[repositories].skills_dir`) + user custom skills dir (`[repositories].user_skills_dir`), heals/grades them via `skilldoctor` + `agentskills`, verifies via `skill-scanner` + `defenseclaw`, and installs cleanly.
4. Universal Forward-Slash (/) Standard: Use normalize_path() for all rule destinations and file targets.
5. Directory Placement Policy:
   - User-Level configs (~/.<agent>/ or ~/.agents/) are auto-installed if selected.
   - Workspace repository install is strictly optional and user-selected.
   - Workspace rules are placed strictly in the first workspace root directory.

Requirements:
1. Stage & maintain externalized rule/skill definitions outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`, synced with `<skills>` `Navegos/skills`) with `<enforce>...</enforce>` wrappers:
   - `principles` (Karpathy engineering principles: think before coding, simplicity first, surgical changes, goal-driven execution)
   - `caveman` (ASD-STE100 primitive English compression + Wave 17 v4 modes: `/ultracave`, `/megacave`, `caveman-commit`, `caveman-review`, `caveman-compress`, `shrink`, `browse`, `.blocks`)
   - `ponytail` (lazy senior developer build discipline ladder + mode variants, companion skills, and statusline telemetry)
2. Implement `src/rules/skills_repo.rs` + `src/rules/skill_validator.rs` + `src/rules/skills_catalog.rs`:
   - Fetch/sync official vetted skills from `Navegos/skills` (`[repositories].skills_dir` / `skills_pages_url`) + optional `[repositories].user_skills_dir`.
   - Run `skilldoctor` deterministic 0–100 / A–F grading + byte-exact body-preserving `--fix` frontmatter repair + `agentskills/skills-ref` YAML validation & unknown-key migration to `metadata:` + trigger collision deduplication + `skill-scanner`/`defenseclaw` `<enforce>` vetting before installation.
3. Implement universal synchronizer in `crates/toksi/src/rules/sync.rs` strictly for **user-configured/selected** agents across `18+` built-in harnesses and open conventions (`agents-structures/` parity) with **granular component toggles** (`hooks`, `mcp`, `skills`, `rules`, `instructions`, `configs`):
   - `.agents/` (OpenAgents: `agents.md`, `system-prompt.md`, `skills/`, `.agents/mcp`)
   - `.gemini/` (Antigravity: `GEMINI.md`, `settings.json`, `commands/*.toml`, `rules/`, `skills/`)
   - `.copilot/` & `.github/` (Copilot: `copilot-instructions.md`, `hooks/`, `skills/`)
   - `.claude/` (Claude Code: `CLAUDE.md`, `.mcp.json`, `rules/`, `skills/`)
   - `.codex/` (Codex: `AGENTS.md`, `rules/`)
   - `.cursor/` (Cursor: `.cursorrules`, `rules/*.mdc`, `skills/`)
   - `.grok/` (Grok: `hooks/*.json`, `skills/`, `rules/`)
   Plus auto-merge nested subfolder `.toksi/` / `.<agent>/` dirs into root `<repo>/` cleanly.
4. Support skill wiring from MCP settings: map `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp`, `.agents/mcp` to `skills/<tool>`.
5. Ensure 0 compiler warnings and targeted `cargo test` pass across the **295-vault** externalized `<skills>` / `<advisory>` 3-repo architecture, **35-component `<toksi-siagent>`** (`DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/` across **37** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_18_6`).
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Zero-Bloat Externalized `<skills>` Distribution** (`<neo>/agents/`, `<neo>/rules/`, `src/rules/skills_repo.rs`): Keeps `toksi` binary lean by storing vetted markdown/YAML/JSON skills and rules outside `crates/toksi/` synced to `Navegos/skills` (`<skills>`), while `skilldoctor` + `agentskills` heal frontmatter and `<enforce>` guards block unsafe payloads on install.
- **Universal Synchronizer** (`src/rules/sync.rs`): Injects or updates vetted rule files across all agent configuration directories in the workspace root or user home.
- **Rules Formatting & Header Preserver**: Merges toksi-managed rules cleanly while preserving existing user-defined rules.

### 2. Disciplinary Standards & Invariants
- Synced markdown text must be clean UTF-8 with normalized Unix `\n` line endings and `<enforce>...</enforce>` guardrails.
- Do not overwrite user-customized sections; append or update within designated delimiter blocks (`<!-- TOKSI-RULES-START -->`).

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- rules` passes 100%.
- Zero `.md`/`.yaml`/`.json` skill/rule payloads embedded inside `crates/toksi/` (`src/rules/embedded/` retired cleanly).

---

## Wave 18 (v5.2) Synchronization Addendum
- Enforce `<enforce>...</enforce>` wrapping on all staged rule blocks, script hooks, externalized `<skills>` assets (`huggingface-skills`, `openai-skills`, `langchain-skills`, `mofa-skills`, `NeMo-Skills`, `NVIDIA-skills`, `agent-skills`, `openagent-skills`, `openclaw-agent-skills` — zero binary bloat inside `crates/toksi/`), and **Toksi Unified Plugin System (`src/plugins/`)** mounts across all **295 mandated vaults**, **35-component `<toksi-siagent>`**, **37** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, and canonical `<repo>/.toksi/projectmem/` (`PHASE_18_2`).
- Integrate pure-Rust `skilldoctor` grading/fixing + `SkillSpector` + `SkillEvaluator` + `agentskills` YAML frontmatter validator & auto-healer (`src/rules/skill_validator.rs`) plus semantic trigger deduplication (`src/rules/skills_catalog.rs`).
- Audit every synced skill, plugin, and script against **`defenseclaw`** fail-closed hookguard/audit + **`skill-scanner`** multi-engine/CEL policy + **`opa`/`cedar`** Policy-as-Code + `research-securety-skills` and `sage` threat rules before activation (`toksi skills heal`, `toksi security scan-skills`, `toksi plugin scan`).
