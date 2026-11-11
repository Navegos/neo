# neo — plan

> Editable **intent** file: ideas + plans — what we *mean to do*.
> This is NOT the event log. `events.jsonl` -> `summary.md` records what
> *happened*; this file records what we *intend*. The AI reads it at
> session start and edits it directly (like `PROJECT_MAP.md`): add ideas
> and plans, check items off, move done work down to Shipped. Plans are
> never logged as events.

## Ideas
_Loose thoughts, not yet committed to._
- Distributed build remote execution using NativeLink REAPI worker daemon.
- Automatic translation of GitHub Actions `.github/workflows/*.yml` directly into local runnable pipelines.
- AI Agent toolchain auto-provisioning: agent queries missing header/tool -> nbuild auto-fetches and injects into environment.

## Disciplinary Coding Standards & Conventions Mandate (For All Agent Workers)
- **Genuine Safe Rust (Edition 2024)**: `#![deny(unsafe_code)]` enforced strictly at `crates/toksi/src/{lib,main}.rs` (eliminate all 23 internal `unsafe` blocks: `set_var`, `LockFileEx`, `pre_exec(setsid)`). Isolate unavoidable Win32 FFI (`advapi32`, `user32`, Job Objects, console) in `crates/toksi-sys` and any legacy C-ABI grammar shims in `crates/toksi-grammars`.
- **Zero Warnings Policy**: Code must compile cleanly under `RUSTFLAGS="-D warnings"` with zero Clippy lints (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Strict Error Handling**: Prohibit `.unwrap()`, `.expect()`, `panic!()`, `unreachable!()` in production. Typed errors via `thiserror` in libraries; `anyhow::Result` with `.context()` in CLI.
- **Continuous Reference Vault Re-Audit (36 Reference Vaults)**: All 36 reference vaults in `neo/inspect-vault/*` (`tokless`, `bifrost`, `headroom`, `rtk`, `context-mode`, `codegraph`, `projectmem`, `ai-memory`, `icm`, `ponytail`, `caveman`, `andrej-karpathy-skills`, `TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `litellm`, `TokenEconomizer`, `agent-opt`, `agents-structures`, plus Wave 17 `hermes-agent`, `openclaw`, `openclaw-rs`, `mofa`, `mofa-engine`, `mofaclaw`, `codex`, `crustly`, `crush`, `goose`, `grok-build`, `gemini-cli`, `crewAI`) MUST be inspected, ported, and verified end-to-end with zero dead stubs.
- **Cross-Platform Portability (Windows, Linux, macOS)**:
  * Universal forward-slash `/` paths standard (`normalize_path(&str)`). Escaped `\\` allowed ONLY in `cmd.exe` commands inside JSON5/TOML configs.
  * PATH validation mandate: verify binaries in `PATH` (`check_executable_in_path`), never encode absolute paths; grey out missing agents cleanly in UI rather than blocking startup.
  * User-home configuration: `~/.toksi/config.toml` + `~/.toksi/toksi.db` (`byok_profiles` `<byok, model, effort>`, `agent_profile_bindings`, `quota_windows`).
  * Strict repo-only intelligence storage: workspace `codegraph`, `projectmem`, `context`, `icm`, `ai-memory`, and `rtk` tracking live strictly inside `<repo>/.toksi/<tool>/` — NEVER in `~/.toksi/<tool>/`, NEVER outside `.git` roots, NEVER inside `.git/` or nested crate subfolders; monorepo `<repo>/crates/*` partitioned via `subproject_id`.
  * Cross-platform file locking via safe `std::fs::File::lock()` / `fs2`; OS Keyring via `crates/toksi-sys` with normalized `gemini:antigravity`; env injection via `Command::env()` + `crates/toksi-sys` (`WM_SETTINGCHANGE`, never `CLOUD_CODE_URL`); RAII `TeardownGuard` using `tokio::signal`/`ctrlc`.
  * Zero Paid-Tier Restrictions: Never reject free/trial/OAuth BYOK accounts; delete `FreeTierDisqualified` / `[PAID]` / `[DISQUALIFIED-FREE]` everywhere.

## Active plans
_What we're working toward now. Use `- [ ]` / `- [x]` checklists._

### toksi — Native Rust Consolidation & Wave 17 Master Architecture
> Master blueprints: `crates/toksi/research/RESEARCHPLAN.md`, `crates/toksi/CONSOLIDATION_PLAN.md`, `crates/toksi/PLAN.md`.
> Modular Brain Execution Index: `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`
- [x] Phases 1–14: Initial in-tree port of RTK, ProjectMem, Context-Mode, ICM, ai-memory, CodeGraph (8 initial langs), Rules, Headroom/Gateway foundation, Downloader Purge, Legacy Healing, Non-Destructive Env Injection, Agent Doctor, Initial TUI/Dashboard, E2E Verification, OS Keyring & SQLite DB.
- [ ] Phase 15.1–16.3 Alignment Upgrades: Complete Wave 17 delta fixes against Phases 15.1–16.3 below:
  > Plans: `.agents/brain/PHASE_15_1_*.md` – `.agents/brain/PHASE_16_3_*.md`
  - [ ] Phase 15.1 Upgrade: Strict `<repo>/.toksi/<tool>` repo-only storage (remove `~/.toksi/{projectmem,icm,ai-memory,context,codegraph}` leakage), fix `find_project_dir()` to stop at `.git` root + monorepo `subproject_id`, purge `toksi-*`/`tokless_*`/`agy-hook codegraph-index`/`rtk hook cursor`, split `antigravity-cli|ide`.
  - [ ] Phase 15.2 Upgrade: Unified 7-port supervisor (`:8787`, `:8788` Grok, `:8789`/`:8790` Copilot, `:18787` BYOK, `:9099` Gateway, `:9100` Cockpit) driven by `~/.toksi/config.toml` + `--agents`; replace Copilot `"copilot proxy ready"` & `"wrap"` stubs.
  - [ ] Phase 15.3 Refactor: Remove `FreeTierDisqualified` / `ACTIVE_PAID_PROVIDERS` completely; open multi-instance `<byok, model, effort>` BYOK quota windows (`hour..year`); configured-only activation; real Bifrost dispatch.
  - [ ] Phase 15.4 Upgrade: Real `A2aTaskStore` v1.0 & `SessionActorQueue` ACP v2.0 + `DirectWireBus` (`direct-wired`); fix `commands/ctx.rs` passthrough `0` stub & `commands/runmcp.rs` `aimemory->icm` misroute; enforce `.git/` exclusion & `.gitignore`/`.ignore` dynamic purge.
  - [ ] Phase 15.5 Upgrade: Ground-truth SSE & Responses usage (`cache_creation_input_tokens`, `0` when idle), rate-limit header quotas, zero-cost `/v1/models`, `ArcSwap<reqwest::Client>` reconnect heal, HTTP/2 ConnectRPC (`#0033`/`#0034`), comprehensive `--help`.
  - [ ] Phase 15.6 Completion: Fix `TokenEconomizer` empty-text stub, `agent-opt` `DefaultHasher` bug, `ast-grep` line heuristic, `scip` stub, `Chroma` ANN + live `proxy.rs` wiring; wire `jcodemunch`/`repomix`/`mem0`; expand `TokenTracker` (47 parsers), `rtk` (`Tracker`/`tee`/`discover`/`learn`), `projectmem` (16 MCP tools), `icm` (28 MCP tools + `temporal`/`fusion`), `caveman` v4, `ponytail` modes.
  - [ ] Phase 16.1–16.3 Alignment: Unified 9-tab async TUI & detached `:9100` Web Cockpit (`Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`), delete all fake telemetry & `[DISQUALIFIED-FREE]`, clickable heatmap modal, `<byok, model, effort>` routing controls, DB export/import, `agents-structures` & `.agents/mcp` parity, and Wave 17 `README.md`.

### Wave 17 Execution Sequence (`PHASE_17_1` – `PHASE_17_7`)
- [ ] **Phase 17.1** (`.agents/brain/PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md`): Create `crates/toksi-sys`, remove all 23 `unsafe` sites & enforce `#![deny(unsafe_code)]`, implement `src/config/mod.rs` (`~/.toksi/config.toml`), add `--agents` & `dashboard start|stop|restart`, delete `FreeTierDisqualified` / `[PAID]`.
- [ ] **Phase 17.2** (`.agents/brain/PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md`): Add `byok_profiles` `<byok, model, effort>` DDL + fix `export_dump`/`import_dump`, normalize `gemini:antigravity` keyring, port pure-Rust `headroom-core` `live_zone`+`CcrStore`, route `/byok/` through Headroom + Chroma ANN, delete Python `headroom` & `cloudcode_patch.rs`.
- [ ] **Phase 17.3** (`.agents/brain/PHASE_17_3_TREESITTER_41_LANGS_AND_REPO_ISOLATION.md`): Create `crates/toksi-grammars`, expand to 41+ Tree-sitter languages (+ Dart/R & 8 `codegraph-kernel` extractors), wire real `ast-grep`/`scip`/`jcodemunch`/`repomix`, fix `<repo>/.toksi/` monorepo partitioning & home-dir cleanup.
- [ ] **Phase 17.4** (`.agents/brain/PHASE_17_4_VAULT_ECOSYSTEM_CONNECTORS_AND_HOOK_CLEANUP.md`): Fix `ctx-hook` & `runmcp` `aimemory` misroute, close deep vault gaps, register 65/90+ Categories 1–5 + 22 local engines, standardize `<tool>-hook <agent>`, split CLI/IDE IDs, reconcile `agents-structures`, purge legacy `toksi-*`/`tokless_*`.
- [ ] **Phase 17.5** (`.agents/brain/PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md`): Build `src/agent_runtime/*` (`toksi chat|run|exec|swarm`) fusing `hermes`/`openclaw`/`mofa`/`mofaclaw`/`crewAI`/`codex`/`grok-build`, 3-OS sandbox, `DirectWireBus`, real `A2aTaskStore` & `SessionActorQueue`, and 10 `< 5ms` slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`).
- [ ] **Phase 17.6** (`.agents/brain/PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md`): Kill gateway/copilot stubs, wire `LoadBalancer` + `mofa-engine` 7-D router + `:18787` daemon supervision, extract SSE/Responses usage & rate limits, zero-cost `/v1/models`, `ArcSwap` reconnect watchdog, HTTP/2 ConnectRPC passthrough (`#0033`/`#0034`).
- [ ] **Phase 17.7** (`.agents/brain/PHASE_17_7_ASYNC_9TAB_UI_INCREMENTAL_TESTS_AND_DOCS.md`): Finalize exact 9-tab TUI + Web Cockpit with zero synthetic stubs, implement `.toksi/test-baseline.json` BLAKE3 incremental test skip harness, add `phase14..17_7` verification suites, and finalize `main.rs` `--help` & `crates/toksi/README.md`.

## Shipped
- [x] Initial toksi proxy, headroom, and command dispatcher
- [x] Phases 1–14 baseline implementation and research/planning synchronization across `RESEARCHPLAN.md`, `CONSOLIDATION_PLAN.md`, `PLAN.md`, `README.md`, and `.agents/brain/`
