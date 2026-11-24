# Phase 19.3 — Live-Safe Sandboxed & Docker Test Isolation

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]`

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
Plus .agents/brain/PHASE_19_3_LIVE_SAFE_SANDBOXED_AND_DOCKER_TEST_ISOLATION.md.

Execute Phase 19.3: Live-Safe Sandboxed & Docker Test Isolation (RESEARCHPLAN.txt:584).
Targets:
  - docker/Dockerfile.test (NEW — clean reproducible Rust + test toolchain container)
  - docker/docker-compose.test.yml (NEW)
  - crates/toksi/src/commands/test_runner.rs (add `--docker`, `--sandbox`, ephemeral env isolation, & live-daemon guard)
  - crates/toksi/src/config/mod.rs (`[testing]` section: `live_safe_isolation`, `sandbox_backend`, `test_image`)
  - .toksi/test-baseline.json (tag each suite with `"isolation": "ephemeral" | "docker_sandbox"`)
  - crates/toksi/tests/*.rs (audit daemon/port/env/keyring suites so zero host ~/.toksi/ or live ports are touched)
  - crates/toksi/tests/phase19_3_live_safe_test_isolation_verification.rs (NEW)

Requirements:
1. Tier 1 — Ephemeral In-Process & Temp-Root Isolation:
   - Guarantee every unit & integration test overrides `TOKSI_HOME` to a unique `tempfile::TempDir`, binds HTTP/IPC endpoints strictly to `127.0.0.1:0` and unique test pipe/socket IDs, disables `HKCU\Environment` & OS Keyring mutations (`--no-env-inject` / mock keyring store), and NEVER stops, kills, or mutates live running host `toksi` services, `~/.toksi/toksi.db`, or live ports `8787..18787`.
2. Tier 2 — Containerized / Sandboxed Daemon Test Runner (`docker/Dockerfile.test` & `toksi test --docker|--sandbox`):
   - Author `docker/Dockerfile.test` with pinned Rust toolchain, `git`, `sqlite3`, `curl`, and optional polyglot test fixtures (zero raw vault bash/CI bloat).
   - Extend `src/commands/test_runner.rs` so `toksi test` checks `docker`/`sbx` availability in `PATH` (following "Test & Inform PATH"), runs `"isolation": "docker_sandbox"` suites (`phase15_2`, `e2e_tests`, `phase19_2`, hook/daemon/env tests) inside isolated containers or `sbx`/3-OS native sandboxes, and streams structured pass/fail results.
3. Preserve `.toksi/test-baseline.json` BLAKE3 incremental skipping (`[SKIP-UNCHANGED]`) without ever relaxing or skipping assertions inside modified suites (`--all` / `--force` runs full suite).
4. Expose test isolation status & trigger controls in Web Cockpit Tab 6 (`Sandbox Inspector`) and TUI.
5. Update `.toksi/test-baseline.json` and all 6 canonical plan destinations; verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p toksi --test phase19_3_live_safe_test_isolation_verification`.
```

---

## Technical Specifications & Details

### 1. Verified Baseline Reality
- `.toksi/test-baseline.json` currently tracks `35` integration suites with `schema_version: 1`, `cargo_lock_blake3`, and per-suite `tree_blake3`.
- `docker/` does not exist on disk yet; `src/commands/test_runner.rs` (401 lines) currently lacks `--docker` / `--sandbox` execution and `"isolation"` dispatch.

### 2. Isolation Invariant Table
| Test Tier | Allowed Operations | Forbidden Operations |
|---|---|---|
| `ephemeral` (`cargo test`) | `tempfile::TempDir` `TOKSI_HOME`, `127.0.0.1:0`, temp Named Pipe/UDS UUID, in-memory/temp SQLite | Binding fixed `8787..18787`, touching `~/.toksi/toksi.db`, killing host PIDs, writing `HKCU\Environment` / real OS Keyring |
| `docker_sandbox` (`toksi test --docker\|--sandbox`) | Full daemon start/stop, fixed ports inside container netns, env injection inside container | Exposing container test ports or mutating host filesystem/registry outside explicit volume mount |

## Verification Criteria
- `cargo test -p toksi --test phase19_3_live_safe_test_isolation_verification` proves host `~/.toksi/toksi.db` mtime/hash and live ports remain completely untouched during test execution.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
