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
- [x] JSON/YAML Catalog loader (`catalog/*.json`) with multi-path resolution and schema validation
- [x] Local GitHub Actions-style workflow runner (`pipeline::Workflow`)
- [x] Multi-forge remote engine (`core::forge`) supporting GitHub, GitLab, Codeberg/Gitea, Bitbucket, Launchpad, and generic Git
- [x] Dynamic API release discovery (`releases_url`) eliminating hardcoded binary release URLs
- [x] Compression format awareness & platform-specific scoring (`*.tar.gz`, `*.tar.xz`, `*.tar.zst`, `*.zip`, `*.bz2`, `*.rar`, `*.7z`)
- [x] Stem hint resilience (`strip_archive_extension`) making hints immune to upstream archive format changes
- [x] Universal unpacker (`DistroInstaller::unpack_archive`) with bsdtar and 7z fallback, prefix stripping, and wrapper flattening
- [x] Catalog recipes added: `cmake`, `ninja`, `llvm`, `zlib`, `libpng`
- [x] Neo Distro Package & Multi-Stage Release: `https://api.github.com/repos/Navegos/neo/releases/latest` packaging all crates (`nbuild`, `tokless`, tools) into `$TOOLCHAINS_DIR/<platform>/<triple>/distro/neo/` with synchronized multi-tool updates
- [x] Self-Diagnosis, Doctor & Self-Healing (`nbuild doctor`, `--heal`): verify install topology, warn user if outside structure, auto-install self to distro, heal broken symlinks, repair corrupted package files, run smoke tests
- [x] Environment Topology & PATH Self-Healing: detect `TOOLCHAINS_DIR`, interactively prompt user if missing and set in environment, auto-heal user `PATH` with `$TOOLCHAINS_DIR/usr/bin`, create base folders
- [x] Interactive CLI Menu & TUI System: menu-driven interface (`nbuild menu` / `nbuild -i`) for system doctor, environment setup, package manager, and file operations
- [x] Built-in File & Directory Operations (`core::fs_ops`): CLI and programmatic search, copy, move, rename, delete, and mkdir
- [x] Clean Package Updates & In-Use Self-Update Resilience: delete old package files before update extraction; Windows shadow-swap (rename running `.exe` -> `.old`, install new, self-spawn) to bypass binary file locking
- [ ] Expand JSON catalog recipes (port remaining `inspect-vault/buildtools/shell` tools & libraries: `openssl`, `zstd`, `lzma`, `brotli`, `cares`, `ccache`, `hwloc`, `jom`, `libdeflate`, `libexpat`, `libjpeg`, `libtiff`, `libuv`, `libwebp`, `libxml2`, `lz4`, `onetbb`, `pkgconf`, `protobuf`, `cuda`, `msmpi`, `nasm`, `python`, `qt`)
- [ ] Implement source build engine with configure/build/install cycle for C/C++ libraries (CMake, Ninja, Meson, Make, NMake/Jom)
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

### nbuild — Milestone 1 (Foundation, Sysroot, Multi-Forge & Universal Extraction)
- [x] Foundation of `nbuild` and `archinfo` in workspace
- [x] Multi-forge API client with URL security validation
- [x] Dynamic release asset resolution with platform/arch matching
- [x] Stem-based hint resilience for format changes across releases
- [x] Universal unpacking engine (`*.tar.gz`, `*.tar.xz`, `*.tar.zst`, `*.zip`, `*.bz2`, `*.rar`, `*.7z`)
- [x] Workspace verification (72 tests passing across 12 test suites)
