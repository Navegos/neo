# Phase 18.2 — `<enforce>` Security, Unified Plugin System, Zero-Plaintext Engine, Validator, `<skills>` + `skilldoctor`/`agentskills`, & 41-Lang SAST

## Ready-to-Run Chat Prompt

```text
Implement Phase 18.2 of the Toksi Unified Consolidation Plan (`<enforce>` Skill/Plugin/Script Guard, Externalized `<skills>` Distribution with Zero Binary Bloat, Toksi Unified Plugin System `src/plugins/`, Zero-Plaintext Blind Signature Engine `src/security/`, Native Git/PR Validator `src/validator/`, `defenseclaw` + `skill-scanner` Governance, `skilldoctor` + `agentskills` Frontmatter Auto-Healer, and 41-Language Static Security Analyzer).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 0–3, 5.3, 12–13), crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_2_ENFORCE_SKILLS_HEALER_AND_41_LANG_SECURITY_ANALYZER.md first.

Your goals in Rust (`crates/toksi`) & Externalized Repos (`<neo>`, `<advisory>`, `<skills>`):
1. Strict `<enforce>` Security Layer, `defenseclaw` Fail-Closed Governance & `skill-scanner` Multi-Engine/CEL Engine (`src/security/` & `src/rules/sync.rs`):
   - Wrap system-critical instruction blocks, loaded plugins, and generated rule payloads in canonical `<enforce>...</enforce>` compliance delimiters preventing prompt-injection override of tool safety boundaries.
   - Integrate `inspect-vault/defenseclaw` (`defenseclaw.rs`, `hookguard.rs`, `audit.rs`, `yara_mcp.rs`): fail-closed JSON-RPC / tool-call / plugin hookguard (`defenseclaw_fail_closed = true`), input/output inspection, ordered tool-chain policies (`tool-chains.json`), HMAC-SHA256 gateway auth, filesystem code-signature verification (`codesig`), YARA/MCP scan rules, and append-only JSONL audit evidence logs.
   - Integrate `inspect-vault/skill-scanner` (`skill_scanner.rs`, `cel_decision.rs`, `pipeline_taint.rs`, `bytecode_audit.rs`, `sarif.rs`): multi-engine skill/plugin orchestrator combining static YARA/AST/entropy/markdown-link resolution (`research-securety-skills`, `sage`), pipeline taint flow analysis, `.pyc`/compiled artifact mismatch detection, bounded fail-closed CEL policy decisions, taxonomy normalization, and SARIF 2.1.0 / JSON / Markdown / Table / HTML output across 5 presets (`strict|balanced|low-noise|quiet|permissive`).
2. Zero-Plaintext Blind Signature Engine (`src/security/{schema,polyglot,blind_compiler,scanner,advisory_client,secure_export}.rs`) & Native Git/PR Validator (`src/validator/{mod,validate,fp_pruner,shard_builder,git_pr_driver}.rs`):
   - **Zero-Plaintext Rule**: NEVER persist raw exploit strings, PoC code, malicious payloads, SQLite malware blobs, or plaintext regexes on disk (`~/.toksi/toksi.db`, `Navegos/advisory`), in Git commits, SARIF reports, or RAM during scanning.
   - `toksi compile-blind`: Normalize commands and AST sliding n-grams (`3, 4, 5`) into keyed `BLAKE3` digests (`blake3::keyed_hash`) and compile regex rules offline via `regex_automata::dfa::dense::Builder` -> `to_sparse()` -> `to_bytes_little_endian()` -> Base64, executing zero-copy runtime matching via `sparse::DFA::from_bytes`. Embed into OSV 1.7.4 (`database_specific.blind_signatures`), CVE JSON 5.1 (`containers.adp[].x_blind_signatures`), and STIX 2.1 (`x_blind_signatures`).
   - `advisory_client.rs` & `secure_export.rs`: O(1) prefix search against `<advisory>` (`https://navegos.github.io/advisory/`, cached in `~/.toksi/feed/`), `DisclosureGate` embargo enforcement (stored in `advisory_embargoes`), **12-DB `UpstreamDeduplicator`** (`Navegos/advisory`, NVD, Red Hat Security Data, GitLab Advisory DB, OSV, RustSec, cURL `vuln.json`, GitHub Advisory DB, FriendsOfPHP, Go Vulnerability DB, PyPA Advisory DB, RubySec), and hardened TLS 1.3 (`rustls` + `secrecy` + OS Keyring) exporter (GitHub SARIF, DefectDojo, TAXII 2.1/OpenCTI, MISP).
   - `validator/`: Port `cve-schema`, `cve-validation-library`, `cve-services`, `osv-schema`, `cve-bin-tool`, `cve-lite-cli` rules into `validate.rs`; scrub false positives against `corpus/benign/` (`fp_pruner.rs`); build symmetrical 3-tree GitHub Pages shards (`advisories/`, `domains/` `<= 500` rows/page, `lookup/` prefix maps + paired `index.json`/`index.html` in `shard_builder.rs`); and drive local clone updates (`[repositories].advisory_dir`), branching, Git push, and REST PRs (`POST /repos/Navegos/advisory/pulls`) via `git_pr_driver.rs` (`toksi advisory` CLI & `/advisory` slash command).
3. Toksi Unified Plugin System (`src/plugins/{mod,manifest,loader,hooks,slash_router,cli_bridge,analyzer_plugin}.rs`, `src/commands/plugin.rs`, SQLite `plugins_registry`):
   - Support 7 unified mount surfaces: `agent_feature`, `cli_cmd` (`toksi plugin run <cmd>`), `tool`, `chat_cmd` (`/plugin:*`), `wire_hook` (Categories 1–3 via `DirectWireBus`), `protocol_endpoint` (`MCP`, `A2A`, `ACP`, `stdio`, `SSE`, `HTTP` on `:9098`), and `agent_to_agent` messaging (`[plugins].allow_agent_to_agent`).
   - Enforce mandatory `<enforce>` pre-load verification (`[plugins].enforce_security_scan = true` running `skill-scanner` + `defenseclaw` before activation), persist state in `plugins_registry`, and wire `toksi plugin` CLI + `/plugin` slash command + TUI Tab 7 & Web Cockpit `/api/plugins`.
4. Zero-Bloat Externalized `<skills>` Distribution (`Navegos/skills`), `skilldoctor` Grading/Fixing, & `agentskills` YAML Frontmatter Auto-Healer (`src/rules/{skill_validator,skills_catalog,skills_repo,sync}.rs`):
   - **Never embed skills, rules, `.md`, `.yaml`, or `.json` files inside the `toksi` binary (`crates/toksi/`).** Stage curated files outside `crates/toksi/` (`<neo>/agents/` and `<neo>/rules/`) synced to `<skills>` (`Navegos/skills`); installed `toksi` fetches vetted skills online from `<skills>` (or `[repositories].skills_dir`) + user custom skills dir (`[repositories].user_skills_dir`).
   - Port `skilldoctor` (0–100 / A–F grading `lint {error:12,warning:5,info:1}` + `security {error:60,warning:20,info:4}` + `0.6*mean + 0.4*worst` batch aggregation, byte-exact body-preserving `--fix`, 22 `skill/*`, `tools/*`, `sec/*` SARIF rules) AND `agentskills/skills-ref` (`ALLOWED_FIELDS`, NFKC `name <= 64` matching dir name, `description <= 1024`, `compatibility <= 500`, moving unknown keys to `metadata:`), deduplicating colliding triggers across `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, and `.agents/skills/` via `toksi skills` & `/skills`.
5. Built-in 41-Language Static Code Security Analyzer (`src/security/{mod.rs,analyzer.rs,sage_rules.rs,osv.rs,epss.rs,semgrep_engine.rs,external_analyzers.rs}`, `<neo>/rules/polyglot-code/*`):
   - Compile multi-language AST pattern matching across Toksi's 41+ Tree-sitter languages (fixing `"dockerfile"` in `src/codegraph/kernel/langs.rs` `grammar_for()` + `"vue"`/`"scss"` specialization) porting high-signal rules from `semgrep`, `semgrep-rules`, `codex-security`, `deepsec`, and `sage`.
   - Port pure-Rust lockfile parser (`src/security/osv.rs`) + EPSS/remediation scoring (`src/security/epss.rs`) from `osv-scanner` + `cve-lite-cli` + `cve-bin-tool` + optional `PATH` external analyzers (`rust-analyzer`, `gopls`, `clang-tidy`, `cargo-audit`).
   - Wire `toksi security [scan|scan-skills|guardrails|audit-evidence|heal-skills|analyzers]` (`--format json|sarif|markdown|table|html`), `toksi compile-blind`, `toksi advisory`, `toksi skills`, `/security`, `/advisory`, `/skills`, `/plugin`, `:9099/v1/security/scan`, TUI Tabs 5–7 & 9, and Web Cockpit `:9100`.
6. Update `.toksi/test-baseline.json` and run `cargo test --test phase18_2_security_and_skills_verification`.
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

### 2. Threat Signature Taxonomy & Zero-Plaintext Engines (`src/security/` & `src/validator/`)
| Domain | Source Vault(s) | Engine Mechanism | Outputs |
|---|---|---|---|
| Zero-Plaintext Blind Engine | `Navegos/advisory`, `cve-schema`, `osv-schema`, `cve-validation-library`, `cve-services` | Keyed `BLAKE3` n-grams (`3,4,5`) + serialized little-endian `sparse::DFA` (`toksi compile-blind`), `DisclosureGate`, 12-DB `UpstreamDeduplicator` | OSV 1.7.4 (`blind_signatures`), CVE 5.1 (`x_blind_signatures`), STIX 2.1, TLS 1.3 export |
| Native Git/PR Validator | `advisory`, `corpus/benign/`, `cve-bin-tool`, `cve-lite-cli` | Structural validation, benign corpus `fp_pruner`, 3-tree `shard_builder` (`advisories/`, `domains/`, `lookup/`), REST `git_pr_driver` | Static GitHub Pages shards + `Navegos/advisory` PRs (`toksi advisory`) |
| Runtime Governance & Guardrails | `defenseclaw` | Fail-closed hookguard, `tool-chains.json`, HMAC-SHA256 gateway auth, `codesig` hash cache, YARA/MCP | Allow / Block (`fail_closed`) + JSONL audit evidence |
| Multi-Engine Skill & Plugin Scan | `skill-scanner`, `skilldoctor`, `research-securety-skills` | Static YARA/AST/entropy, pipeline taint, `.pyc` bytecode audit, bounded CEL decision (`strict..permissive`), 0–100 A–F grading | Verdict + CEL reason + SARIF 2.1.0 / Tab 5 & 7 |
| 41-Lang SAST Code Rules | `semgrep`, `semgrep-rules`, `deepsec`, `codex-security`, `sage` | Tree-sitter S-expression queries + taint sink trace (`<neo>/rules/polyglot-code/`) | SARIF 2.1.0, JSON, Markdown, Table, HTML, TUI Tab 5/6 |
| Lockfile / SCA Dependencies | `osv-scanner`, `cve-lite-cli`, `cve-bin-tool` | Pure-Rust lockfile parsers + EPSS scoring + offline OSV/SBOM correlation | CVE/GHSA severity + fix version |
| Native Compiler Diagnostics | `PATH` (`rust-analyzer`, `gopls`, `clang-tidy`) | Optional subprocess wrapper with RTK output compaction | Unified `SecurityFinding` stream |

### 3. Externalized `<skills>` Distribution, Unified Plugins & `<enforce>` Contract
- Stage all curated rules, skills, ATR packs, and SAST `.scm` rules outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`) synced to `Navegos/skills` (`<skills>`) — **zero binary bloat** inside `crates/toksi/`.
- Enclose immutable security invariants inside `<enforce>...</enforce>` tags across synced agent configs (`AGENTS.md`, `GEMINI.md`, `CLAUDE.md`, `.cursor/rules/*`, `~/.codex/AGENTS.md`), skills, and Unified Plugins (`src/plugins/`).
- `src/security/skill_audit.rs` + `skill_scanner.rs` assign trust tiers (`BuiltinVerified`, `WorkspaceLocal`, `ImportedAudited`, `Quarantined`); when `[plugins].enforce_security_scan = true` or `defenseclaw_fail_closed = true`, plugins with `blocked_fail_closed` or `unscanned` status cannot activate hooks, CLI bridges, tools, `/plugin:*` chat commands, `:9098` protocols (`MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP`), or agent-to-agent messaging.
