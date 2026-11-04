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

## Active plans
_What we're working toward now. Use `- [ ]` / `- [x]` checklists._

### toksi — Native Rust Consolidation & Single-Daemon Cockpit (v3.2)
- [x] Phase 1: RTK Engine Integration (100+ filters, ANSI stripper, `toksi rtk`)
- [x] Phase 2: ProjectMem Pure Rust Port (schema, JSONL store, summary, precheck, MCP)
- [x] Phase 3: Context-Mode Pure Rust Port (sandbox runner, FTS5 chunker, `ctx_*` tools)
- [x] Phase 4: ICM & ai-memory Integration (SQLite FTS5, multi-agent handoff, tiered briefs)
- [x] Phase 5: CodeGraph Pure Rust Engine (Tree-sitter, petgraph, symbols cache)
- [x] Phase 6: In-Tree Rules & Skills (Karpathy, Caveman, Ponytail embedded via `include_str!`)
- [x] Phase 7: Headroom Deep Integration & Gateway Catalog (36 providers, semantic cache)
- [x] Phase 8: Downloader Purge (zero npm/pip/node/python dependency)
- [x] Phase 9: Legacy Healing & Binary Un-Hijack (`heal.rs`)
- [x] Phase 10: Non-Destructive Environment Injection (`windows.rs`, `WM_SETTINGCHANGE`)
- [x] Phase 11: Agent Doctor & Self-Healing Parity (`doctor --fix`, 17 agent harnesses)
- [x] Phase 12: TUI, Prometheus Telemetry & Web Dashboard (`/metrics`, TUI, dashboard)
- [x] Phase 13: End-to-End System Verification (10/10 e2e tests passing)
- [x] Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration:
  - [x] Implement cross-platform Keyring Manager (`advapi32.dll` on Windows, Keychain on macOS, Secret Service on Linux)
  - [x] Implement base64 OAuth envelope unpacking (`agy` access token & expiration parser)
  - [x] Wire Keyring into `gateway/agent_cli` (`gemini_agy`, `claude_code`, `copilot`, `cursor`)
  - [x] Implement high-performance SQLite config DB (`providers`, `provider_keys`, `provider_models`, `byok_routes`, `agent_configs`, `gateway_configs`) with export/import/backup/restore
  - [x] Enforce BYOK gateway route pre-registration & health check before routing/loadbalancer dispatch
  - [x] Auto-inject authenticated headers into proxy requests when local agents omit credentials
  - [x] Add `toksi auth` and `toksi db` CLI subcommands plus `toksi doctor` probes
- [ ] Phase 15: Unified 5-Port Daemon, Paid Quota Enforcement & Smart CLI Providers:
  - [ ] Upgrade `toksi daemon start` to supervise all 5 dedicated endpoints simultaneously:
    * Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
    * Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
    * Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
    * Port 9099: Enterprise AI Gateway & Drop-in SDKs (`/v1/chat/completions`, `/v1/messages`, `/metrics`)
    * Port 9100: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)
  - [ ] Secure 5 Local Agent CLIs as First-Class BYOK Providers:
    * Register `AntigravityCli` (`agy.exe`), `CopilotCli` (`copilot.exe`), `ClaudeCli` (`claude`), `CursorBridge` (`cursor`), `GrokCli` (`grok.exe`)
    * Auto-check OS Keyring and session files; smart background launch in detached mode; verify chat reachability (`/livez`, ConnectRPC)
  - [ ] Enforce Paid Quota Mandate (No Free Accounts for BYOK Providers):
    * Implement `tier_check.rs`: require active commercial/paid billing tier or subscription quota
    * Disqualify free-tier/trial/unbilled accounts from active gateway rotation pool
    * Graceful inform & zero-fail policy: emit non-blocking notice (`[INFO] Provider 'X' is on Free tier. Paid account required for BYOK rotation; bypassing gracefully`), zero crashes or panics, fallback to active paid provider or direct agent execution
  - [ ] Network Port Performance Isolation: Dedicate port 9099 strictly to LLM inference streaming; move Web Dashboard Cockpit to port 9100 to eliminate browser SSE and static asset socket contention
  - [ ] Expand BYOK Provider catalog: Add `Bifrost` gateway-as-provider, `AWS SageMaker`, `AWS Bedrock Mantle`
  - [ ] Ground-Truth Session Token Accounting: Extract real prompt/completion/cached tokens directly from live SSE response streams and upstream response headers (`x-ratelimit-remaining-tokens`, `anthropic-ratelimit-requests-remaining`) in `telemetry/collector.rs`. Zero virtual/synthetic math
  - [ ] Rules, Skills & Instructions Sync: Implement universal synchronizer updating `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md` across both workspace directories and user config homes for all 17 agents
  - [ ] Implement strict forward-slash `/` normalization (`normalize_path`) across all paths (Windows, Linux, macOS) for Git Bash / MSYS2 compatibility
  - [ ] Relocate default SQLite database from `LOCALAPPDATA/toksi` to `~/.toksi/toksi.db` (`users/.toksi`) with automated startup migration
  - [ ] Update `help_text()` in `crates/toksi/src/main.rs` with comprehensive usage information for all commands, tools, ports, and flags
  - [ ] Ensure 0 compiler warnings and all tests pass
- [ ] Phase 16: Mouse-Capable TUI & Enterprise Web UI Cockpit:
  - [ ] Enable Crossterm mouse capture (`EnableMouseCapture`) in `crates/toksi/src/tui/` with interactive tab switching, row selection, action buttons, and scroll
  - [ ] Upgrade embedded Web Dashboard on port 9100 in `crates/toksi/src/dashboard/assets/`:
    * Non-blocking background execution without monopolizing terminal windows
    * Real-time 17-agent observation grid with live ground-truth token savings, connected provider, model, latency, upstream quota gauges, and paid tier status badges
    * Gateway, routing & load balancer interactive controls (algorithms, provider weights/priorities, complexity tiers)
    * OS Keyring and SQLite DB management interface
  - [ ] Rewrite `crates/toksi/README.md` into comprehensive technical documentation covering 5-port architecture, setup, 17 agents, 39+ providers, smart agent CLI provider bridges, paid quota enforcement, rules sync, and full CLI reference
  - [ ] Verify complete workspace with tests and zero warnings

### nbuild — Toolchain & Workflow Manager
- [x] Initialized `crates/nbuild` crate in root `Cargo.toml`
- [x] Migrated `archinfo` to `crates/archinfo` as workspace dependency
- [x] Directory topology engine ($TOOLCHAINS_DIR/<platform>/<target-triple>/distro/<pkg>)
- [x] GNU Sysroot & Host active (`usr/bin`, `usr/include`, `usr/lib`) symlink engine with collision guards
- [x] `pkg_meta.json` schema and persistence (origins, versions, installed files, hashes, link mappings)
- [x] JSON/YAML Catalog loader (`catalog/*.json`) with multi-path resolution and schema validation
- [x] Local GitHub Actions-style workflow runner (`pipeline::Workflow`)
- [x] Multi-forge remote engine (`core::forge`) supporting GitHub, GitLab, Codeberg/Gitea, Bitbucket, Launchpad, and generic Git
- [x] Dynamic API release discovery (`releases_url`) eliminating hardcoded binary release URLs
- [x] Compression format awareness & platform-specific scoring (`*.tar.gz`, `*.tar.xz`, `*.tar.zst`, `*.zip`, `*.bz2`, `*.rar`, `*.7z`)
- [x] Stem hint resilience (`strip_archive_extension`) making hints immune to upstream archive format changes
- [x] Universal unpacker (`DistroInstaller::unpack_archive`) with bsdtar and 7z fallback, prefix stripping, and wrapper flattening
- [x] Catalog recipes added: `cmake`, `ninja`, `llvm`, `zlib`, `libpng`
- [x] Neo Distro Package & Multi-Stage Release: packaging all crates (`nbuild`, `toksi`, tools) into `$TOOLCHAINS_DIR/<platform>/<triple>/distro/neo/` with synchronized multi-tool updates
- [x] Self-Diagnosis, Doctor & Self-Healing (`nbuild doctor`, `--heal`): verify install topology, warn user if outside structure, auto-install self to distro, heal broken symlinks, repair corrupted package files, run smoke tests
- [x] Environment Topology & PATH Self-Healing: detect `TOOLCHAINS_DIR`, interactively prompt user if missing and set in environment, auto-heal user `PATH` with `$TOOLCHAINS_DIR/usr/bin`, create base folders
- [x] Interactive CLI Menu & TUI System: menu-driven interface (`nbuild menu` / `nbuild -i`) for system doctor, environment setup, package manager, and file operations
- [x] Built-in File & Directory Operations (`core::fs_ops`): CLI and programmatic search, copy, move, rename, delete, and mkdir
- [x] Clean Package Updates & In-Use Self-Update Resilience: delete old package files before update extraction; Windows shadow-swap to bypass binary file locking
- [x] Expand JSON catalog recipes (port remaining `inspect-vault/buildtools/shell` tools & libraries: `openssl`, `zstd`, `lzma`, `brotli`, `cares`, `ccache`, `hwloc`, `jom`, `libdeflate`, `libexpat`, `libjpeg`, `libtiff`, `libuv`, `libwebp`, `libxml2`, `lz4`, `onetbb`, `pkgconf`, `protobuf`, `cuda`, `msmpi`, `nasm`, `python`, `qt`, `git`, `sccache`, `bzip2`)
- [ ] Implement source build engine with configure/build/install cycle for C/C++ libraries (CMake, Ninja, Meson, Make, NMake/Jom)
- [ ] Add remote worker management (`nbuild remote add`, self-bootstrap via SSH/WinRM)
- [ ] Add NativeLink (`inspect-vault/nativelink`) remote cache & execution integration
- [ ] Add Buck2 (`inspect-vault/buck2`) build system adapter
- [ ] Expose MCP server mode (`nbuild mcp`) for direct AI agent integration

## Shipped
_Move completed plans here so the top stays about the future._

### toksi
- [x] Initialize `crates/toksi` crate in root `Cargo.toml`
- [x] Port `internal/core` (types, registries, detection) to Rust
- [x] Port `internal/util` (paths, colors, logger, jsonc, toml, exec, deps, prompt, progress, byok, routing state)
- [x] Port `internal/tools` (rtk, caveman, ponytail, codegraph, contextmode, headroom, projectmem)
- [x] Port `internal/agents` (agent definitions, detection, configs, hooks, proxy routing for all 12 agents)
- [x] Port `internal/headroom` (proxy engine, router, autostart, byok gateway, watchdog, daemons)
- [x] Port `internal/commands` and CLI entrypoint `toksi` binary
- [x] Verify test suite & CLI execution (17/17 tests passing)

### nbuild — Milestone 1 (Foundation, Sysroot, Multi-Forge & Universal Extraction)
- [x] Foundation of `nbuild` and `archinfo` in workspace
- [x] Multi-forge API client with URL security validation
- [x] Dynamic release asset resolution with platform/arch matching
- [x] Stem-based hint resilience for format changes across releases
- [x] Universal unpacking engine (`*.tar.gz`, `*.tar.xz`, `*.tar.zst`, `*.zip`, `*.bz2`, `*.rar`, `*.7z`)
- [x] Workspace verification (72 tests passing across 12 test suites)
