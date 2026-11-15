# Phase 18.2 — `<enforce>` Security, Unified Plugin System, `defenseclaw`, `skill-scanner`, `agentskills` Healer & 41-Lang SAST

## Ready-to-Run Chat Prompt

```text
Implement Phase 18.2 of the Toksi Unified Consolidation Plan (`<enforce>` Skill/Plugin/Script Guard, Toksi Unified Plugin System `src/plugins/`, `defenseclaw` Fail-Closed Governance, `skill-scanner` Multi-Engine/CEL Scanning, `agentskills` Frontmatter Auto-Healer, and 41-Language Static Code Security Analyzer).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 1–3, 5.3, 12–13), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_2_ENFORCE_SKILLS_HEALER_AND_41_LANG_SECURITY_ANALYZER.md first.

Your goals in Rust (`crates/toksi`):
1. Strict `<enforce>` Security Layer, `defenseclaw` Fail-Closed Governance & `skill-scanner` Multi-Engine/CEL Engine (`src/security/` & `src/rules/sync.rs`):
   - Wrap system-critical instruction blocks, loaded plugins, and generated rule payloads in canonical `<enforce>...</enforce>` compliance delimiters preventing prompt-injection override of tool safety boundaries.
   - Integrate `inspect-vault/defenseclaw` (`defenseclaw.rs`, `hookguard.rs`, `audit.rs`, `yara_mcp.rs`): fail-closed JSON-RPC / tool-call / plugin hookguard (`defenseclaw_fail_closed = true`), input/output inspection, ordered tool-chain policies (`tool-chains.json`), HMAC-SHA256 gateway auth, filesystem code-signature verification (`codesig`), YARA/MCP scan rules, and append-only JSONL audit evidence logs.
   - Integrate `inspect-vault/skill-scanner` (`skill_scanner.rs`, `cel_decision.rs`, `pipeline_taint.rs`, `bytecode_audit.rs`, `sarif.rs`): multi-engine skill/plugin orchestrator combining static YARA/AST/entropy/markdown-link resolution (`research-securety-skills`, `sage`), pipeline taint flow analysis, `.pyc`/compiled artifact mismatch detection, bounded fail-closed CEL policy decisions, taxonomy normalization, and SARIF 2.1.0 / JSON / Markdown / Table / HTML output across 5 presets (`strict|balanced|low-noise|quiet|permissive`).
   - Provide safe quarantined override audit flags (`--allow-risky` NEVER bypasses `<enforce>` core secret-exfiltration protection or fail-closed hookguard without explicit interactive audit log).
2. Toksi Unified Plugin System (`src/plugins/{mod,manifest,loader,hooks,slash_router,cli_bridge,analyzer_plugin}.rs`, `src/commands/plugin.rs`, SQLite `plugins_registry`):
   - Support 7 unified mount surfaces: `agent_feature`, `cli_cmd` (`toksi plugin run <cmd>`), `tool`, `chat_cmd` (`/plugin:*`), `wire_hook` (Categories 1–3 via `DirectWireBus`), `protocol_endpoint` (`MCP`, `A2A`, `ACP`, `stdio`, `SSE`, `HTTP` on `:9098`), and `agent_to_agent` messaging (`[plugins].allow_agent_to_agent`).
   - Enforce mandatory `<enforce>` pre-load verification (`[plugins].enforce_security_scan = true` running `skill-scanner` + `defenseclaw` before activation), persist state in `plugins_registry` (`plugin_id`, `version`, normalized forward-slash `manifest_path`, `blake3_hash`, `capabilities_json`, `protocols_json`, `security_scan_status`, `is_enabled`, `updated_at`), and wire `toksi plugin list|install|remove|enable|disable|reload|scan|info|run|create` + **`/plugin` (13th instant `< 5ms` slash command)** + TUI Tab 7 & Web Cockpit `/api/plugins`.
3. `agentskills` YAML Frontmatter Validator & Auto-Healer + Skill Deduplicator (`src/rules/skill_validator.rs` & `src/rules/skills_catalog.rs`):
   - Port the exact validation rules from `inspect-vault/agentskills/skills-ref/src/skills_ref/{validator.py,parser.py,models.py}` (`ALLOWED_FIELDS`, NFKC `name <= 64` matching dir name, `description <= 1024`, `compatibility <= 500`), build non-destructive auto-healer (unclosed `---`, unquoted colons, moving unknown keys to `metadata:`, synthesizing description), and deduplicate colliding skill triggers across `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, and `.agents/skills/`.
4. Built-in 41-Language Static Code Security Analyzer (`src/security/{mod.rs,analyzer.rs,sage_rules.rs,osv.rs,semgrep_engine.rs,external_analyzers.rs,rules/builtin/*}`):
   - Compile multi-language AST pattern matching across Toksi's 41+ Tree-sitter languages (fixing `"dockerfile"` in `src/codegraph/kernel/langs.rs` `grammar_for()` + `"vue"`/`"scss"` specialization) porting high-signal rules from `semgrep`, `semgrep-rules`, `codex-security`, `deepsec`, and `sage`.
   - Port pure-Rust lockfile parser (`src/security/osv.rs`) from `inspect-vault/osv-scanner` + optional `PATH` external analyzers (`rust-analyzer`, `gopls`, `clang-tidy`, `cargo-audit`).
   - Wire `toksi security [scan|scan-skills|guardrails|audit-evidence|heal-skills|analyzers]` (`--format json|sarif|markdown|table|html`, `--policy strict|balanced|low-noise|quiet|permissive`), `/security [scan [path]|scan-skills [path]|guardrails|audit-evidence|rules|audit-skills]` & `/plugin` slash commands, `:9099/v1/security/scan`, TUI Tabs 5–7 & 9, and Web Cockpit `:9100`.
5. Update `.toksi/test-baseline.json` and run `cargo test --test phase18_2_security_and_skills_verification`.
```

---

## Technical Specifications & Details

### 1. Exact `agentskills` Specification Parity (`inspect-vault/agentskills/skills-ref`)
- `ALLOWED_FIELDS`: `{"name", "description", "license", "allowed-tools", "metadata", "compatibility"}`.
- `name` constraints: NFKC normalized, lowercase ASCII/Unicode alphanumerics + `-` (`a-z0-9-`), length `1..=64`, cannot start/end with `-`, cannot contain `--`, must equal parent skill directory name.
- `description` constraints: non-empty after trim, length `1..=1024`.
- `compatibility` constraints: optional, if present length `1..=500`.
- Auto-healer behavior:
  1. Parse leading YAML block; if missing closing `---`, detect end of key-value block before first `#` Markdown heading or blank line and insert `---`.
  2. Escape/quote scalar strings with unquoted colons (`key: val: extra`) or leading special YAML indicators (`@`, `*`, `&`, `[`, `{`).
  3. Normalize `name` to NFKC lowercase slug matching directory name, logging rename only in diff summary.
  4. Move unknown keys (e.g. `author`, `version`, `tags`, `globs`) under `metadata:` so strict `skills-ref` validators pass without data loss.

### 2. Threat Signature Taxonomy (`src/security/`)
| Domain | Source Vault(s) | Engine Mechanism | Outputs |
|---|---|---|---|
| Runtime Governance & Guardrails | `defenseclaw` | Fail-closed hookguard, `tool-chains.json`, HMAC-SHA256 gateway auth, `codesig` hash cache, YARA/MCP | Allow / Block (`fail_closed`) + JSONL audit evidence |
| Multi-Engine Skill & Plugin Scan | `skill-scanner`, `research-securety-skills` | Static YARA/AST/entropy, pipeline taint, `.pyc` bytecode audit, bounded CEL decision (`strict..permissive`) | Verdict + CEL reason + SARIF 2.1.0 / Tab 5 & 7 |
| 41-Lang SAST Code Rules | `semgrep`, `semgrep-rules`, `deepsec`, `codex-security`, `sage` | Tree-sitter S-expression queries + taint sink trace | SARIF 2.1.0, JSON, Markdown, Table, HTML, TUI Tab 5/6 |
| Lockfile / SCA Dependencies | `osv-scanner` (`pkg/lockfile`) | Pure-Rust lockfile parsers + SQLite OSV cache | CVE/GHSA severity + fix version |
| Native Compiler Diagnostics | `PATH` (`rust-analyzer`, `gopls`, `clang-tidy`) | Optional subprocess wrapper with RTK output compaction | Unified `SecurityFinding` stream |

### 3. Toksi Unified Plugin Mounts & Pre-Load `<enforce>` Contract
- Every injected rule file (`AGENTS.md`, `GEMINI.md`, `CLAUDE.md`, `.cursor/rules/*`, `~/.codex/AGENTS.md`), skill, and Unified Plugin payload (`src/plugins/`) encloses immutable security invariants inside `<enforce>...</enforce>` tags.
- `src/security/skill_audit.rs` + `skill_scanner.rs` assign trust tiers (`BuiltinVerified`, `WorkspaceLocal`, `ImportedAudited`, `Quarantined`); when `[plugins].enforce_security_scan = true` or `defenseclaw_fail_closed = true`, plugins with `blocked_fail_closed` or `unscanned` status cannot activate hooks, CLI bridges, tools, `/plugin:*` chat commands, `:9098` protocols (`MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP`), or agent-to-agent messaging.
