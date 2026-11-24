# Phase 19.5 — Wave 19 SI Agentic Stack, Workforce, `ANP` & Security/Systems Vaults Consolidation (91 Vaults)

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read ALL 10 canonical workspace files FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. .toksi/projectmem/plan.md
5. .toksi/projectmem/PROJECT_MAP.md
6. .toksi/projectmem/summary.md
7. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
8. crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
9. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
10. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
Plus .agents/brain/PHASE_19_5_SI_AGENTIC_STACK_WORKFORCE_ANP_AND_SECURITY_VAULTS.md.

Execute Phase 19.5: Wave 19 SI Agentic Stack, Workforce, `ANP` DID-WBA & ATR/Guardrails Consolidation across all 91 Deep-Researched Group A (56), Group B (28), and Group D (7) Vaults.
Sources (all verified `[DEEP-RESEARCH-DONE]` in RESEARCHPLAN.md Sections 2.11, 2.12, 2.14):
  - Group A (56 Agent/Swarm/Spec/Voice Vaults): `adk-rust` (`ws@3.0.0`, 51 members), `rig` (`0.46.0`), `swarms-rs`, `swarms`, `maki` (`0.7.2`, 15 crates), `steer` (`ws@0.19.0`), `chevalier` (`chevalier-durable`, `chevalier-vfs-hash`), `funes` (`1.3.1+dev`), `agentrix-os` (`spatial-os@0.1.0` w/ `awp_gate`), `anp` (`1.0.7` DID-WBA), `OpenSpec` (`2.1.0`), `deer-flow` (`2.3.0-dev`), `openworker` (`0.3.4`), `dsh-superagent-prototype` (`0.1.0`), `ChatDev` (`DevAll@0.1.0`), `multica` (`0.2.0`), `orca` (`1.4.214`), `nemo-platform`, `tura` (`0.1.0`, 17 members `#![deny(unsafe_code)]`), `smolagents`, `SillyTavern`, `sim`, `livekit*`, `adk-python`, `agents-cli`, `bridge-mcp` (`8.0.0`, 476 SSH tools/77 groups), `openrag`, `pochi`, `tabby` (`ws@0.33.0-dev.0`), `refact`, `native-cli-ai` (`nca@0.66.0`), `ironwire` (`0.1.0`), `anything-llm`, `cc-router`, `continue`, `claurst`, `tidev` (`0.9.0`, 16 crates), `ironclaw` (`0.1.0`, 68 crates Reborn Architecture), `xa` (`0.1.9`), `octos` (`2.0.3-rc.11`, 38 crates), `rust-code` (`sgr-agent-core`), `ringlet` (`0.1.0`), `ironcode`, `brat` (`libbrat-{engine,worktree,workflow}`), `arc-cli` (`0.5.0`, 31 crates), `aitk`, `moly-ai` (`moly@0.2.5-rc1`), `agent-code`, `agent-manager`, `langgraph-swarm-py`, `openai-agents-python`, `openai-cli`, `openai-python`, `open-multi-agent`
  - Group B (28 Rust Systems/IPC/Storage/Cloud SDK Vaults): `depyler` (`4.1.2`, `depyler-verify`), `paiml-mcp-agent-toolkit` (`pmat@3.45.1`), `tiktoken` (`0.14.0`), `BLAKE3` (`1.8.7`), `rpmalloc*`, `pingora` (24 crates), `quinn` (7 crates), `grpc`, `protobuf`, `vertex_rust_sdk` (`threatflux-vertex-rust-sdk@1.0.0`), `higress` (`v2`), `lancedb` (`0.42.0-beta.20`), `arrow-rs` (`60.2.0`, 28 crates), `milvus-storage`, `rocksdb`, `coredb`, `coach`, `databend`, `iggy` (55 crates), `mongo`, `cockroach`, `redis`, `elasticsearch`, `gloo`, `robius` (`0.3.1`), `robrix` (`1.0.0-beta.7`), `hf-hub`
  - Group D (7 Security/Guardrails/Skill Vaults): `agent-threat-rules` (`SPEC.md` v1.0.1, 19,699 YAMLs), `openai-guardrails-python` (`0.3.6`), `model_spec`, `upskill` (`0.2.1`), `qdrant-skills` (35 `SKILL.md`), `crewAIInc-skills` (11 `SKILL.md`), `Essential-Hermes-Skills` (91 `SKILL.md` w/ strict `<enforce>` audit)
Targets:
  - crates/toksi/src/agent_runtime/{swarm/mod.rs,openspec.rs,slash.rs,orchestrator.rs,durable_journal.rs,reflection.rs}
  - crates/toksi/src/tools/multi_protocol.rs (`ANP` DID-WBA + `QUIC` transport)
  - crates/toksi/src/aimemory/{funes_recall.rs,lance_store.rs}, crates/toksi/src/codegraph/pmat.rs
  - crates/toksi/src/headroom/tokenizer.rs (`tiktoken`), crates/toksi/src/gateway/{providers/frontier/vertex.rs,proxy.rs,routing.rs}
  - crates/toksi/src/security/{atr_engine.rs,openai_guardrails.rs,model_spec_verifier.rs,ssh_bridge.rs}
  - crates/toksi/src/rules/{skill_validator.rs,skills_repo.rs} (`upskill` harness integration)
  - crates/toksi/tests/phase19_5_siagent_workforce_anp_security_verification.rs (NEW)

Requirements:
1. `<toksi-siagent>` Trait, Workforce, & Spec-Driven Unification (Group A):
   - Harmonize `adk-rust` (`adk-core`/`adk-agent`/`adk-acp`/`adk-session`), `rig-core`/`rig-agent`, `rust-code` (`Tool`/`FileBackend`/`AgentContext`), `maki-acp`, `tidev`, `tura`, `ironclaw`, `octos`, and `arc-cli` contracts cleanly into `src/agent_runtime/`.
   - Build `/workforce` (spawning git-worktree swarms from `swarms-rs` + `langgraph-swarm-py` + `brat` + `ringlet` + `deer-flow` + `ChatDev` + `orca`) and `/spec` (`OpenSpec` `propose -> spec -> tasks -> apply -> verify -> archive` integrated with `.toksi/projectmem/`).
   - Port `chevalier-durable`/`chevalier-vfs-hash` content-addressed step verification, `xa` sub-5ms single-shot `toksi exec` filtering, and `ironwire`/`cc-router` subscription pool balancing.
2. Protocol, Systems, Storage & Cloud SDK Hardening (Group A/B):
   - Add **Agent Network Protocol (`ANP`)** (`DID-WBA` `ANP_MESSAGE_SERVICE_TYPE`/`VM_KEY_AUTH`/`VM_KEY_E2EE_AGREEMENT`) + `quinn` QUIC transport on `:9098`.
   - Integrate `vertex_rust_sdk` pure-Rust Vertex AI client (`auth`, `cache`, `claude`, `media`), `pingora-pool` connection pooling, `lancedb` + `arrow-rs` columnar vector tables, `funes` high-precision agent recall, `pmat` TDG complexity scoring, `tiktoken` BPE exact counting, `hf-hub` pure-Rust model fetch, `bridge-mcp` SSH whitelist/blacklist + secret redaction, and `coach` vector drill chaos checks + `depyler` contract verification invariants.
3. ATR, OpenAI Guardrails, Model Spec & Skill Distillation (`<enforce>` — Group D):
   - Implement `atr_engine.rs` supporting **ATR `SPEC.md` v1.0.1** rules compiled strictly zero-plaintext via `blind_compiler.rs` (zero plaintext patterns on disk/RAM).
   - Port `openai-guardrails-python` tripwires (`competitors`, `hallucination_detection`, PII, moderation, jailbreak, URL/tool policy) + `model_spec` instruction hierarchy (`System > Developer > User > Tool`) into `reflection.rs` / `guardrails.rs`.
   - Integrate `upskill` skill evaluation benchmarks into `toksi skills eval` before pushing audited `qdrant-skills`/`crewAIInc-skills`/`Essential-Hermes-Skills` (minus unsafe `reverse-skill` parts) to `Navegos/skills`.
4. Ensure all 35 `<toksi-siagent>` components bind out of the box across all 7 Headroom-compressed entrypoints (`:8787..:8790, :9098, :9099, :18787`) and Direct-Wire IPC with zero paid-tier restrictions.
5. Update `.toksi/test-baseline.json` and all 6 canonical plan destinations; verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p toksi --test phase19_5_siagent_workforce_anp_security_verification`.
```

---

## Technical Specifications & Details

### 1. Verified Rust Crate Anchors in `inspect-vault/`
- **`adk-rust` (`ws@3.0.0`):** 51 members (`adk-core`, `adk-agent`, `adk-acp`, `adk-action`, `adk-artifact`, `adk-runner`, `adk-session`, `adk-tool`, `adk-model`, `adk-server`).
- **`rig` (`0.46.0`):** `rig-core`, `rig-agent`, `rig-candle`, `rig-fastembed`, `rig-gemini-grpc`, `rig-cli`.
- **`brat` (`0.1.0`):** `libbrat-{engine,config,grite,worktree,session,workflow,kb,swimlane}`, `brat-mcp`, `brat-skills`.
- **`vertex_rust_sdk` (`threatflux-vertex-rust-sdk@1.0.0`):** `api`, `auth`, `builders`, `cache`, `chat_core`, `claude`, `client`, `config`, `error`, `media`.
- **`agent-threat-rules`:** `SPEC.md` v1.0.1 (19,699 YAML rule/fixture files).

## Verification Criteria
- `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `phase19_5_siagent_workforce_anp_security_verification` pass 100%.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
