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

### nbuild — Toolchain & Workflow Manager
- [x] Initialized `crates/nbuild` crate in root `Cargo.toml`
- [x] Migrated `archinfo` to `crates/archinfo` as workspace dependency
- [x] Directory topology engine ($TOOLCHAINS_DIR/<platform>/<target-triple>/distro/<pkg>)
- [x] GNU Sysroot & Host active (`usr/bin`, `usr/include`, `usr/lib`) symlink engine with collision guards
- [x] `pkg_meta.json` schema and persistence (origins, versions, installed files, hashes, link mappings)
- [x] JSON Catalog loader (`catalog/*.json`) with multi-path resolution
- [x] Local GitHub Actions-style workflow runner (`pipeline::Workflow`)
- [ ] Expand JSON catalog recipes (port remaining `inspect-vault/buildtools/shell` tools & recipes)
- [ ] Implement source build engine with configure/build/install cycle for C/C++ libraries (CMake, Ninja, Meson, Make)
- [ ] Add remote worker management (`nbuild remote add`, self-bootstrap via SSH/WinRM)
- [ ] Add NativeLink (`inspect-vault/nativelink`) remote cache & execution integration
- [ ] Add Buck2 (`inspect-vault/buck2`) build system adapter
- [ ] Expose MCP server mode (`nbuild mcp`) for direct AI agent integration

## Shipped
_Move completed plans here so the top stays about the future._

### tokless
- [x] Initialize `crates/tokless` crate in root `Cargo.toml`
- [x] Port `internal/core` (types, registries, detection) to Rust
- [x] Port `internal/util` (paths, colors, logger, jsonc, toml, exec, deps, prompt, progress, byok, routing state)
- [x] Port `internal/tools` (rtk, caveman, ponytail, codegraph, contextmode, headroom, projectmem)
- [x] Port `internal/agents` (agent definitions, detection, configs, hooks, proxy routing for all 12 agents)
- [x] Port `internal/headroom` (proxy engine, router, autostart, byok gateway, watchdog, daemons)
- [x] Port `internal/commands` and CLI entrypoint `tokless` binary
- [x] Verify test suite & CLI execution (17/17 tests passing)

### nbuild
- [x] Foundation of `nbuild` and `archinfo` in workspace with 4 unit tests passing
