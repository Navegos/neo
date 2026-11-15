# Phase 18.1 — 31 New Wave 18 Vaults Research & Subsystem Synthesis

## Ready-to-Run Chat Prompt

```text
Implement Phase 18.1 of the Toksi Unified Consolidation Plan (Wave 18 / v5.0: 31 New Vaults Synthesis & Subsystem Bridges, Total 67 Audited Vaults).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 1–3, 5.3, 12–13), crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_1_29_NEW_VAULTS_RESEARCH_AND_SYNTHESIS.md first.

Your goals in Rust (`crates/toksi`):
1. Lance Vector Store Bridge (`src/aimemory/lance_store.rs`): Implement pure-Rust columnar vector/hybrid storage and IVF/HNSW-compatible segment layout inspired by `inspect-vault/lance` inside `<repo>/.toksi/ai-memory/lance/` with scalar pre-filtering on `subproject_id`, `repo_hash`, and `memory_type`.
2. OpenWiki Architectural Synthesizer (`src/codegraph/openwiki.rs` & `src/projectmem/wiki_sync.rs`): Port `inspect-vault/openwiki` multi-page repository architecture wiki generation over Toksi's Tree-sitter 45-lang CodeGraph, exporting deterministic Mermaid call graphs and `.toksi/projectmem/PROJECT_MAP.md` synchronization.
3. Tokview Telemetry & Observability Bridge (`src/telemetry/tokview.rs` & `src/telemetry/traces.rs`): Port `inspect-vault/tokview`, `inspect-vault/langfuse`, and `inspect-vault/agent-control` session/span tracing, cost/token attribution per model/effort tier, and live prompt/response diff inspection surfaced on `:9098` / `:9100` (local-first SQLite, zero external DB required).
4. ACP Protocol, Off-the-Back `toksi` Agent & Agent Runtime Consolidation (`src/agent_runtime/acp.rs`, `modes.rs`, `graph_checkpoint.rs`, `deep_planning.rs`, `fleet.rs`, `guardrails.rs`, `zeroclaw_core.rs`):
   - Expose JSON-RPC 2.0 Agent Client Protocol from `inspect-vault/agent-client-protocol` on `:9098/acp` alongside MCP, A2A, and SSE/REST, routing large payloads through the shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`).
   - Wire `toksi` agent out of the box ("off the back") via `DirectWireBus` and `:8787`/`:9098`/`:9099`/`:18787` to Enterprise AI Gateway, BYOK & Vertex ADC Gateway, Protocol Bus, `HeadroomEffortCompressor`, `src/plugins/`, and all 67 vaults with zero manual setup; synthesize `inspect-vault/langgraph` stateful DAG execution, `inspect-vault/deepagents` planning-subagent hierarchy, `inspect-vault/langchain` + `langflow` runnable/chain adapters, `inspect-vault/cline` + `kilocode` Plan/Act/Code/Ask/Debug/Orchestrator mode contracts, `inspect-vault/alphacode` TDD verify loops, `inspect-vault/bosun` multi-agent fleet isolation, `inspect-vault/Codewhale` structural diff context, and `inspect-vault/zeroclaw` zero-overhead trait-driven runtime primitives (`zeroclaw_core.rs`).
5. Gemini / Vertex SDK Adapters (`src/gateway/gemini_wire.rs`): Ensure parity with `inspect-vault/python-genai` (`google-genai` v1beta GenerateContent + thinkingConfig) and `inspect-vault/python-aiplatform` (`aiplatform` regional Vertex endpoints `https://{location}-aiplatform.googleapis.com/...` paired with Phase 18.3 `AGY_ADC_AUTH=true`).
6. Unified Plugin Manifest Discovery & Fail-Closed Security Architecture Bridge (`src/plugins/{manifest,loader}.rs` + `src/security/` foundations): Parse `plugin.toml`, `plugin.yaml`, `openclaw.plugin.json`, and native plugin manifests across `[plugins].plugin_dirs` (`~/.toksi/plugins`, `<repo>/.toksi/plugins`, `<repo>/.agents/plugins`), and synthesize upstream models from `inspect-vault/defenseclaw` (fail-closed hookguard, `tool-chains.json`, `codesig`, JSONL audit evidence, YARA/MCP) and `inspect-vault/skill-scanner` (multi-engine taint, `.pyc` bytecode audit, bounded CEL policy evaluator, SARIF 2.1.0) ready for full enforcement in Phase 18.2.
7. Update `.toksi/test-baseline.json` and run `cargo test --test phase18_1_vault_synthesis_verification`. Zero unrequested daemons or home-dir leakage.
```

---

## Technical Specifications & Details

### 1. Inventory of All 31 Wave 18 Vaults (Expanding Total from 36 to 67)
1. **Vector, Memory, Observability & Wiki (4 vaults)**:
   - `inspect-vault/lance` -> Pure-Rust columnar vector/hybrid store in `<repo>/.toksi/ai-memory/lance/` with metadata pre-filtering and zero-copy slice scans.
   - `inspect-vault/openwiki` -> Deterministic CodeGraph-backed repository wiki generator writing structured module maps, dependency diagrams, and `PROJECT_MAP.md` summaries.
   - `inspect-vault/tokview` -> High-density local trace, token, savings, and latency telemetry viewer feeding Tab 1 of TUI & `:9100` Cockpit.
   - `inspect-vault/langfuse` -> Local-first LLM span/trace schema (generations, spans, tool calls, cost breakdown per effort level) persisted in `memory.db`.

2. **Agent Protocols, Frameworks & Runtimes (14 vaults)**:
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

3. **Security, Runtime Governance & Multi-Engine Scanning (8 vaults -> deep-dived in Phase 18.2)**:
   - `inspect-vault/sage`, `inspect-vault/codex-security`, `inspect-vault/deepsec`, `inspect-vault/osv-scanner`, `inspect-vault/semgrep`, `inspect-vault/semgrep-rules`, `inspect-vault/defenseclaw`, `inspect-vault/skill-scanner`:
     * `defenseclaw`: Fail-closed runtime governance across agent hooks, plugin gates, tool chains (`tool-chains.json`), HMAC-SHA256 gateway authentication, `codesig` filesystem signature cache, YARA/MCP scan rules, and append-only JSONL audit evidence.
     * `skill-scanner`: Multi-engine skill & plugin analyzer combining YARA/AST/entropy/link-resolver static checks, pipeline dataflow taint analysis, `.pyc` bytecode mismatch detection, bounded fail-closed CEL policy evaluation (`strict|balanced|low-noise|quiet|permissive`), and SARIF 2.1.0 output.
   - Clarified exact partition of all **31** Wave 18 vaults:
     - Data/Observability/Wiki (4): `lance`, `openwiki`, `tokview`, `langfuse`
     - Protocols/SDKs/Runtimes (14): `agent-control`, `agent-client-protocol`, `python-genai`, `python-aiplatform`, `langgraph`, `langchain`, `langflow`, `deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`
     - Security & Vulnerability Scanners (8): `sage`, `codex-security`, `deepsec`, `osv-scanner`, `semgrep`, `semgrep-rules`, `defenseclaw`, `skill-scanner`
     - Agent Skills & Skill Security (5): `research-securety-skills`, `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, `agentskills`
     - Total new Wave 18 vaults: `4 + 14 + 8 + 5 = 31`. Prior Waves 1–17 vaults: `36`. Combined total: `67`.

### 2. Unified Plugin System Manifest Discovery Bridge (`src/plugins/{manifest,loader}.rs`)
- Discover and normalize native `plugin.toml`, YAML `plugin.yaml`, and OpenClaw-style `openclaw.plugin.json` descriptors from `~/.toksi/plugins`, `<repo>/.toksi/plugins`, and `<repo>/.agents/plugins` using forward-slash (`/`) paths and BLAKE3 integrity digests.
- Register capability flags (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `protocol_endpoint`, `agent_to_agent`) and protocol mounts (`mcp`, `a2a`, `acp`, `stdio`, `sse`, `http`) into `plugins_registry` (`security_scan_status = 'unscanned'` until Phase 18.2 `<enforce>` gate clears `passed`).

### 3. Language Grammar Completeness Note (`crates/toksi/src/codegraph/kernel/langs.rs`)
- `langs.rs` currently lists 45 language IDs (`L13-19`), including `rust`, `typescript`, `tsx`, `javascript`, `jsx`, `python`, `go`, `c`, `cpp`, `zig`, `swift`, `java`, `csharp`, `kotlin`, `scala`, `php`, `ruby`, `lua`, `luau`, `julia`, `elixir`, `erlang`, `ocaml`, `haskell`, `html`, `css`, `scss`, `vue`, `svelte`, `graphql`, `bash`, `dockerfile`, `hcl`, `nix`, `make`, `cmake`, `json`, `yaml`, `toml`, `protobuf`, `sql`, `markdown`, `latex`, `dart`, `r`.
- Audit gap to resolve in Phase 17.3 / 18.2: `"dockerfile"` has no arm in `grammar_for()`, and `"vue"`/`"scss"` alias `tree_sitter_html`/`tree_sitter_css`. Security & CodeGraph queries must handle `"dockerfile"` cleanly (dedicated grammar or structured Dockerfile directive parser) so zero listed languages return `None` unexpectedly.
