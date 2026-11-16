# Phase 18.1 — 42 New & Companion Wave 18 Vaults Research & Synthesis

## Ready-to-Run Chat Prompt

```text
Implement Phase 18.1 of the Toksi Unified Consolidation Plan (Wave 18 / v5.1: 42 New & Companion Vaults Research & Subsystem Synthesis, Total 78 Mandated Vaults across 3 Repositories: <neo>, <advisory>, <skills>).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 0–3, 5.3, 12–13), crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md, crates/toksi/research/docker_sandboxes_sbx_cli_reference.md, crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_1_29_NEW_VAULTS_RESEARCH_AND_SYNTHESIS.md first.

Your goals in Rust (`crates/toksi`):
1. Lance Vector Store Bridge (`src/aimemory/lance_store.rs`): Implement pure-Rust columnar vector/hybrid storage and IVF/HNSW-compatible segment layout inspired by `inspect-vault/lance` inside `<repo>/.toksi/ai-memory/lance/` with scalar pre-filtering on `subproject_id`, `repo_hash`, and `memory_type`.
2. OpenWiki Architectural Synthesizer (`src/codegraph/openwiki.rs` & `src/projectmem/wiki_sync.rs`): Port `inspect-vault/openwiki` multi-page repository architecture wiki generation over Toksi's Tree-sitter 45-lang CodeGraph, exporting deterministic Mermaid call graphs and `.toksi/projectmem/PROJECT_MAP.md` synchronization.
3. Tokview Telemetry & Observability Bridge (`src/telemetry/tokview.rs` & `src/telemetry/traces.rs`): Port `inspect-vault/tokview`, `inspect-vault/langfuse`, and `inspect-vault/agent-control` session/span tracing, cost/token attribution per model/effort tier, and live prompt/response diff inspection surfaced on `:9098` / `:9100` (local-first SQLite, zero external DB required).
4. ACP Protocol, OpenCode, Off-the-Back `toksi` Agent & Agent Runtime Consolidation (`src/agent_runtime/{acp,codemode,session,modes,graph_checkpoint,deep_planning,fleet,guardrails,zeroclaw_core}.rs`, `src/agents/opencode.rs`):
   - Expose JSON-RPC 2.0 Agent Client Protocol from `inspect-vault/agent-client-protocol` on `:9098/acp` alongside MCP, A2A, and SSE/REST, routing large payloads through the shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`).
   - Synthesize `inspect-vault/opencode` (`codemode` execution, `http-recorder`, Drizzle/SQLite session state, `gui-plugins`, `.opencode/hooks.json` lifecycle hooks) alongside `langgraph` stateful DAG execution, `deepagents` planning-subagent hierarchy, `langchain` + `langflow` runnable/chain adapters, `cline` + `kilocode` Plan/Act/Code/Ask/Debug/Orchestrator modes, `alphacode` TDD verify loops, `bosun` fleet isolation, `Codewhale` structural diff context, and `zeroclaw` zero-overhead trait-driven runtime primitives (`zeroclaw_core.rs`)—wiring `toksi` agent out of the box ("off the back") via `DirectWireBus` and `:8787`/`:9098`/`:9099`/`:18787` to every gateway, Protocol Bus, `src/plugins/`, and all **78** vaults.
5. Gemini / Vertex SDK Adapters (`src/gateway/gemini_wire.rs`): Ensure parity with `inspect-vault/python-genai` (`google-genai` v1beta GenerateContent + thinkingConfig) and `inspect-vault/python-aiplatform` (`aiplatform` regional Vertex endpoints `https://{location}-aiplatform.googleapis.com/...` paired with Phase 18.3 `AGY_ADC_AUTH=true`).
6. CVE/OSV Standards, SkillDoctor Grading, Sandbox-Runtime, & Companion Repo Foundations (`src/security/`, `src/validator/`, `src/rules/skill_validator.rs`, `src/agent_runtime/sandbox.rs`, `src/plugins/{manifest,loader}.rs`):
   - Synthesize `cve-schema` (CVE JSON 5.1), `cve-validation-library` (Diagnostics, ValidationErrors, WarningRegistry), `cve-services` (submission/reservation validation contracts), `osv-schema` (OSV 1.7.4 `database_specific.blind_signatures`), `cve-bin-tool` (458 binary/SBOM/VEX checkers + mismatch FP suppression + multi-DB correlation), `cve-lite-cli` (EPSS scoring + npm/pnpm/yarn/bun transitive lock graphs + PEP 440 ranges), `skilldoctor` (0–100 / A–F grading, byte-exact body-preserving `--fix`, 22 lint/security SARIF rules), `sandbox-runtime` (WFP egress, DACL/ACL, HTTP/SOCKS5 domain proxy, Linux `bwrap`/`seccomp`, macOS `.sbpl`), `defenseclaw`, `skill-scanner`, and companion repos `<advisory>` (`Navegos/advisory`) & `<skills>` (`Navegos/skills`) with externalized staging outside `crates/toksi/` (zero binary bloat).
   - Parse `plugin.toml`, `plugin.yaml`, and `openclaw.plugin.json` manifests across `[plugins].plugin_dirs` (`~/.toksi/plugins`, `<repo>/.toksi/plugins`, `<repo>/.agents/plugins`).
7. Update `.toksi/test-baseline.json` and run `cargo test --test phase18_1_vault_synthesis_verification`. Zero unrequested daemons or home-dir leakage.
```

---

## Technical Specifications & Details

### 1. Inventory of All 42 Wave 18 New & Companion Vaults (Expanding Total from 36 to 78)
1. **Vector, Memory, Observability & Wiki (4 vaults)**:
   - `inspect-vault/lance` -> Pure-Rust columnar vector/hybrid store in `<repo>/.toksi/ai-memory/lance/` with metadata pre-filtering and zero-copy slice scans.
   - `inspect-vault/openwiki` -> Deterministic CodeGraph-backed repository wiki generator writing structured module maps, dependency diagrams, and `PROJECT_MAP.md` summaries.
   - `inspect-vault/tokview` -> High-density local trace, token, savings, and latency telemetry viewer feeding Tab 1 of TUI & `:9100` Cockpit.
   - `inspect-vault/langfuse` -> Local-first LLM span/trace schema (generations, spans, tool calls, cost breakdown per effort level) persisted in `memory.db`.

2. **Agent Protocols, Frameworks, Sandboxes & Runtimes (16 vaults)**:
   - `inspect-vault/agent-control` -> Agent lifecycle control plane, policy enforcement, pause/resume/kill, and real-time runtime telemetry hooks.
   - `inspect-vault/agent-client-protocol` -> Standard IDE-to-agent ACP JSON-RPC protocol (`initialize`, `session/new`, `session/prompt`, `session/update`, permission requests) exposed on `:9098` Protocol Bus.
   - `inspect-vault/python-genai` -> Canonical Gemini Developer API & `google-genai` wire schema (`contents`, `systemInstruction`, `tools`, `generationConfig.thinkingConfig`, usage metadata).
   - `inspect-vault/python-aiplatform` -> Vertex AI enterprise endpoint format (`{location}-aiplatform.googleapis.com`), regional routing (`us-central1`, `europe-west4`, `global`), and ADC bearer exchange.
   - `inspect-vault/langgraph` -> Typed cyclic state-graph executor with super-step checkpointing and resumable interrupt/resume edges.
   - `inspect-vault/langchain` -> Unified runnable/tool-calling/structured-output adapter trait compatible with local gateway endpoints.
   - `inspect-vault/langflow` -> Declarative node-graph flow IR import/export for complex multi-step agent pipelines.
   - `inspect-vault/deepagents` -> Long-horizon planner, subagent delegation, filesystem scratchpad, and deep-research summarization loop.
   - `inspect-vault/bosun` -> Multi-agent fleet coordinator, worktree isolation, PR/merge-queue supervision, and task shard dispatch.
   - `inspect-vault/Codewhale` -> Symbol-aware code comprehension, blast-radius diff minimization, and semantic context ranking.
   - `inspect-vault/cline` -> VS Code / IDE Plan vs. Act mode separation, auto-approve boundaries, browser/terminal safety confirmation, and file checkpointing.
   - `inspect-vault/kilocode` -> Specialized persona modes (`Code`, `Architect`, `Ask`, `Debug`, `Orchestrator`) and subtask handoff protocol.
   - `inspect-vault/alphacode` -> Autonomous hypothesis-edit-test-verify loop with automatic regression isolation.
   - `inspect-vault/zeroclaw` -> Fast cold-start Rust-inspired trait architecture: zero-bloat tool dispatch, strict sandboxing, and minimal resident RAM.
   - `inspect-vault/opencode` -> OpenCode `codemode` sandboxed code execution, `http-recorder`, Drizzle/SQLite session semantics, `gui-plugins`, and `.opencode/hooks.json` (`session_start`, `session_shutdown`, `before_agent_start`, `turn_start`, `turn_end`, `tool_call`, `tool_result`, `shell.env`) unified into `toksi` agent (`PHASE_18_5`).
   - `inspect-vault/sandbox-runtime` -> Anthropic Sandbox Runtime (`src/sandbox/*` + Rust `vendor/srt-win` WFP egress filter, DACL/ACL stamping, MXC BaseContainer, Linux `bwrap`/`seccomp`, macOS `.sbpl`, and HTTP/SOCKS5 domain-filtering proxy) unified into `src/agent_runtime/sandbox.rs` (`PHASE_18_5`).

3. **Security, CVE/OSV Standards, Blind Signatures, & Native Git/PR Validator (14 vaults)**:
   - `inspect-vault/sage`, `inspect-vault/codex-security`, `inspect-vault/deepsec`, `inspect-vault/osv-scanner`, `inspect-vault/semgrep`, `inspect-vault/semgrep-rules`, `inspect-vault/defenseclaw`, `inspect-vault/skill-scanner`, `inspect-vault/cve-schema`, `inspect-vault/cve-validation-library`, `inspect-vault/cve-services`, `inspect-vault/osv-schema`, `inspect-vault/cve-bin-tool`, `inspect-vault/cve-lite-cli`:
     * `cve-schema` & `cve-validation-library` & `cve-services` -> CVE JSON 5.1 schema models (`containers.adp[].x_blind_signatures`), `Diagnostics`/`ValidationErrors`/`WarningRegistry`, version range & state validation (`src/security/schema.rs` & `src/validator/validate.rs`).
     * `osv-schema` -> OSV 1.7.4 canonical models (`database_specific.blind_signatures`, `withdrawn` lifecycle).
     * `cve-bin-tool` -> 458 binary/SBOM/VEX checkers, `mismatch/` false-positive suppression, and NVD/OSV/RedHat/GitLab/cURL correlation.
     * `cve-lite-cli` -> EPSS exploit-prediction scoring, npm/pnpm/yarn/bun transitive dependency lock graphs, PEP 440 range evaluation, license/override policies, and compact remediation output.
     * `defenseclaw` -> Fail-closed runtime governance across agent hooks, plugin gates, tool chains (`tool-chains.json`), HMAC-SHA256 gateway authentication, `codesig` filesystem signature cache, YARA/MCP scan rules, and append-only JSONL audit evidence.
     * `skill-scanner` -> Multi-engine skill & plugin analyzer combining YARA/AST/entropy/link-resolver static checks, pipeline dataflow taint analysis, `.pyc` bytecode mismatch detection, bounded fail-closed CEL policy evaluation (`strict|balanced|low-noise|quiet|permissive`), and SARIF 2.1.0 output.

4. **Agent Skills, SkillDoctor Grading/Healing, & Companion Repositories (8 vaults)**:
   - `inspect-vault/research-securety-skills`, `inspect-vault/agent-skills`, `inspect-vault/openagent-skills`, `inspect-vault/openclaw-agent-skills`, `inspect-vault/agentskills`, `inspect-vault/skilldoctor`, plus companion Git repositories **`inspect-vault/advisory`** (`<advisory>` `Navegos/advisory`) and **`inspect-vault/skills`** (`<skills>` `Navegos/skills`).
   - `skilldoctor`: Deterministic 0–100 / A–F grading (`lint` `{error:12, warning:5, info:1}`, `security` `{error:60, warning:20, info:4}`, batch `0.6*mean + 0.4*worst`), byte-exact body-preserving `--fix` frontmatter healer, and 22 `skill/*`, `tools/*`, `sec/*` SARIF rules paired with `agentskills/skills-ref` (`PHASE_18_2`).
   - Externalized staging: Stage vetted skills/rules outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`) synced to `<skills>`, and push zero-plaintext shards/advisories to `<advisory>` via `toksi advisory publish-pr`.
   - **Exact Vault Partition (`36 + 42 = 78`)**: Data/Wiki/Telemetry (`4`) + Protocols/SDKs/Runtimes/Sandboxes (`16`) + Security/CVE/OSV (`14`) + Skills/Companion Repos (`8`) = **42** Wave 18 additions + **36** prior Waves 1–17 vaults = **78 mandated vaults**.

### 2. Unified Plugin System Manifest Discovery Bridge (`src/plugins/{manifest,loader}.rs`)
- Discover and normalize native `plugin.toml`, YAML `plugin.yaml`, and OpenClaw-style `openclaw.plugin.json` descriptors from `~/.toksi/plugins`, `<repo>/.toksi/plugins`, and `<repo>/.agents/plugins` using forward-slash (`/`) paths and BLAKE3 integrity digests.
- Register capability flags (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `protocol_endpoint`, `agent_to_agent`) and protocol mounts (`mcp`, `a2a`, `acp`, `stdio`, `sse`, `http`) into `plugins_registry` (`security_scan_status = 'unscanned'` until Phase 18.2 `<enforce>` gate clears `passed`).

### 3. Language Grammar Completeness Note (`crates/toksi/src/codegraph/kernel/langs.rs`)
- `langs.rs` currently lists 45 language IDs (`L13-19`), including `rust`, `typescript`, `tsx`, `javascript`, `jsx`, `python`, `go`, `c`, `cpp`, `zig`, `swift`, `java`, `csharp`, `kotlin`, `scala`, `php`, `ruby`, `lua`, `luau`, `julia`, `elixir`, `erlang`, `ocaml`, `haskell`, `html`, `css`, `scss`, `vue`, `svelte`, `graphql`, `bash`, `dockerfile`, `hcl`, `nix`, `make`, `cmake`, `json`, `yaml`, `toml`, `protobuf`, `sql`, `markdown`, `latex`, `dart`, `r`.
- Audit gap to resolve in Phase 17.3 / 18.2: `"dockerfile"` has no arm in `grammar_for()`, and `"vue"`/`"scss"` alias `tree_sitter_html`/`tree_sitter_css`. Security & CodeGraph queries must handle `"dockerfile"` cleanly (dedicated grammar or structured Dockerfile directive parser) so zero listed languages return `None` unexpectedly.
