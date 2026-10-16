# nbuild — System Architecture & Implementation Plan

Universal toolchain manager, local workflow runner, and package engine for AI coding agents.

## 1. Directory & Toolchain Hierarchy

Root directory resolved from `$TOOLCHAINS_DIR` (defaults: Windows `C:\opt\neo-toolchains\` or `C:\opt\toolchains\`; Linux `/opt/neo-toolchains/` or `/opt/toolchains/`).

```
$TOOLCHAINS_DIR/
│
├── usr/                                          # Host active runtime prefix ($TOOLCHAINS_DIR/usr/bin in Host PATH)
│   ├── bin/                                      # Symlinks: host-compatible distro/*/bin/* + DLLs
│   ├── include/                                  # Symlinks: host headers (namespaced if library)
│   ├── lib/                                      # Symlinks: host libraries
│   └── share/                                    # Symlinks: host pkgconfig (.pc), cmake modules
│
└── <platform>/                                   # windows | linux | apple | freebsd | android
    └── <target-triple>/                          # e.g. x86_64-pc-windows-msvc, aarch64-apple-xros
        │
        ├── distro/                               # Isolated immutable package stores (clean folder names, no version in folder)
        │   ├── neo/                                  # Neo unified distro: nbuild, tokless, companion tools
        │   │   ├── bin/                              # nbuild, archinfo, tokless
        │   │   ├── share/                            # bundled catalogs, schemas, templates
        │   │   └── pkg_meta.json                     # Neo package receipt
        │   ├── cmake/
        │   │   ├── bin/, share/, ...
        │   │   └── pkg_meta.json                 # Package receipt
        │   ├── libpng/
        │   │   ├── include/, lib/, share/, ...
        │   │   └── pkg_meta.json
        │   └── openssl/
        │       ├── include/, lib/, ...
        │       └── pkg_meta.json
        │
        ├── # Root Level: GNU System Toolchain (Compilers, OS SDKs, Runtimes)
        ├── bin/                                  # Symlinks: clang, cl.exe, ld + runtime DLLs
        ├── include/                              # Symlinks: OS headers, C/C++ CRT headers
        ├── lib/ (lib64/)                         # Symlinks: libc, libstdc++, msvcrt, libucrt
        ├── libexec/                              # Symlinks: compiler internal drivers
        ├── share/                                # Toolchain specs & base cmake
        │
        └── usr/                                  # GNU Sysroot User Layer (Libraries & Dependencies)
            ├── bin/                              # Symlinks: user tools + dependency DLLs
            ├── include/                          # Symlinks: zlib.h, openssl/*.h, or libpng/*
            ├── lib/ (lib64/)                     # Symlinks: libz.a, libcrypto.lib
            └── share/                            # Symlinks: usr/share/pkgconfig/*.pc, cmake/*
```

## 2. Package Receipt Specification (`pkg_meta.json`)

Saved inside each package: `$TOOLCHAINS_DIR/<platform>/<triple>/distro/<pkg_name>/pkg_meta.json`

```json
{
  "$schema": "https://neo.build/schemas/pkg_meta.v1.json",
  "name": "libpng",
  "version": "1.8.0",
  "rawversion": "1.8.0",
  "abiversion": "18",
  "soversion": "18",
  "type": "source_build",
  "tag_name": "libpng18",
  "commit": "9ec49c2d56cec19107ddc458b648ce224c9697b3",
  "date": "2026-05-03",
  "updated_at": "2026-05-03T21:06:38Z",
  "url": "https://github.com/pnggroup/libpng.git",
  "sha256": "4b98c9f7a...",

  "target": {
    "platform": "windows",
    "triple": "x86_64-pc-windows-msvc",
    "arch": "x86_64",
    "target_level": "user"
  },

  "build": {
    "builder": "cmake",
    "generator": "Ninja",
    "compiler": "clang-cl",
    "compiler_definitions": ["-DPNG_ARM_NEON_OPT=0"],
    "configure_commands": [
      "cmake -B build -G Ninja -DCMAKE_INSTALL_PREFIX=$NEO_DISTRO_DIR -DPNG_SHARED=ON"
    ],
    "build_commands": [
      "cmake --build build --config Release"
    ],
    "install_commands": [
      "cmake --install build --config Release"
    ]
  },

  "dependencies": [
    { "name": "zlib", "version": ">=1.3.0", "dep_type": "link" }
  ],

  "env_exports": {
    "PATH": ["bin"],
    "INCLUDE": ["include"],
    "LIB": ["lib"],
    "PKG_CONFIG_PATH": ["lib/pkgconfig", "share/pkgconfig"],
    "CMAKE_PREFIX_PATH": ["."]
  },

  "symlinks": [
    { "src": "bin/libpng18.dll", "dest": "../usr/bin/libpng18.dll", "symlink_type": "dll" },
    { "src": "include/png.h", "dest": "../usr/include/png.h", "symlink_type": "header" },
    { "src": "lib/libpng18.lib", "dest": "../usr/lib/libpng18.lib", "symlink_type": "lib" },
    { "src": "lib/pkgconfig/libpng.pc", "dest": "../usr/share/pkgconfig/libpng.pc", "symlink_type": "pkgconfig" }
  ],

  "installed_files": [
    { "path": "bin/libpng18.dll", "sha256": "...", "size": 245760 },
    { "path": "include/png.h", "sha256": "...", "size": 128450 },
    { "path": "lib/libpng18.lib", "sha256": "...", "size": 42010 }
  ]
}
```

## 3. Catalog Manifests (`catalog/*.json`)

Pre-bundled with the `nbuild` binary in `catalog/`:
- Fast deserialization.
- Contains download URLs, SHA256 hashes, source build configurations, compiler flags, and install steps.
- Search path priority:
  1. `./catalog`
  2. `crates/nbuild/catalog`
  3. `<exe_dir>/catalog`
  4. `$TOOLCHAINS_DIR/catalog`
  5. Extra user-configured paths (JSON/YAML).

## 4. Multi-Forge Remote Engine & Self-Build Bootstrapping (`core::forge`)

Mirrors the dual-path dependency/build paradigm (`dep-cmake.ps1` vs `build-cmake.ps1`) across all major cloud providers and custom Git hosts:
1. **Multi-Forge API Support (`ForgeType` & `ForgeClient`)**:
   - **GitHub**: `https://api.github.com/repos/{owner}/{repo}/releases/latest` (Auth: `GITHUB_TOKEN`).
   - **GitLab**: `https://gitlab.com/api/v4/projects/{encoded_path}/releases/permalink/latest` (Auth: `GITLAB_TOKEN` / `PRIVATE-TOKEN`).
   - **Codeberg / Gitea / Forgejo**: `https://codeberg.org/api/v1/repos/{owner}/{repo}/releases/latest` (Auth: `CODEBERG_TOKEN`).
   - **Bitbucket**: `https://api.bitbucket.org/2.0/repositories/{workspace}/{repo_slug}/downloads` (Auth: `BITBUCKET_TOKEN`).
   - **Launchpad**: `https://api.launchpad.net/1.0/{project}/releases`.
   - **Generic / Custom Git**: Any HTTPS, SSH (`ssh://` or `git@`) address supported via direct `git clone --depth 1 [--branch <tag>]`.
2. **Security & Validation**:
   - URL protocol validation (`validate_url`) strictly restricts schemes to `http`, `https`, `ssh`, or `git` to block SSRF and local file leaks (`file://`).
   - CLI execution passes Git arguments as distinct tokens without shell interpolation.
3. **Dual-Path Strategy (`ToolLifecycleStrategy`)**:
   - **Path A: Bootstrap Missing Tool**:
     If `<platform>/<triple>/distro/<tool>` is missing, fetch the matching prebuilt release asset from the forge, install to `distro/`, and link to sysroot & host `usr/`.
   - **Path B: Self-Hosted Source Build / Clone**:
     If tool exists locally, inspect `pkg_meta.json` vs latest remote release. If outdated or requested (`nbuild update <tool>` or `nbuild clone <source>`):
     Use the existing local binary as the bootstrap tool to configure and compile the updated release from git/source.
   - **Shadow Swap Pattern**:
     During self-hosting rebuilds (e.g. CMake building CMake), active running binaries are backed up / shadow-swapped into a temporary directory so build processes maintain an intact environment until the new binaries are ready.
4. **Dynamic API Release Discovery & Compression Ranking (`CompressionFormat`)**:
   - Rather than hardcoding static binary URLs per platform, catalogs provide a `releases_url` (e.g. `https://api.github.com/repos/Kitware/CMake/releases/latest`).
   - Automatically queries the forge releases API dynamically, parsing tags, versions, and assets in a single call.
   - Recognizes all major archive compression formats:
     `*.tar.zst` (`.tzst`), `*.tar.xz` (`.txz`), `*.zip`, `*.7z`, `*.tar.gz` (`.tgz`), `*.tar.bz2` (`.tbz2`, `.bz2`), `*.rar`.
   - Ranks candidate assets using OS-platform preference matrices:
     - **Windows**: `.zip` (100) > `.tar.zst` (95) > `.tar.xz` (90) > `.7z` (85) > `.tar.gz` (70) > `.tar.bz2` (60) > `.rar` (40).
     - **Linux / FreeBSD**: `.tar.zst` (100) > `.tar.xz` (95) > `.tar.gz` (90) > `.tar.bz2` (80) > `.7z` (60) > `.zip` (50) > `.rar` (30).
     - **macOS / Apple**: `.tar.zst` (100) > `.tar.xz` (95) > `.tar.gz` (90) > `.zip` (85) > `.7z` (60) > `.tar.bz2` (50) > `.rar` (30).
   - Automatically discards non-payload assets (`.sha256`, `.sha512`, `.asc`, `.sig`, `.md5`, `.txt`, `.json`, `.yaml`, `.log`).
5. **Stem Hint Resilience (`strip_archive_extension`)**:
   - Strips archive extensions from hints (`"windows-x86_64.zip"` -> `"windows-x86_64"`).
   - Catalogs provide extension-agnostic stems in `asset_hints`.
   - If an upstream release migrates compression formats (e.g. from `.zip` or `.tar.gz` to `.tar.zst` or `.tar.xz`), asset discovery continues without catalog edits or failures.
6. **Universal Archive Unpacker (`DistroInstaller::unpack_archive`)**:
   - **In-process**: Pure-Rust streaming decompression for `.zip` (`zip` crate) and `.tar.gz` (`flate2` + `tar` crates).
   - **Subprocess System `tar` (bsdtar)**: Uses native `tar -xf` (Windows 10/11 bsdtar, Linux, macOS) supporting `liblzma` (`.tar.xz`), `libzstd` (`.tar.zst`), and `bz2lib` (`.tar.bz2`).
   - **7-Zip Fallback**: Detects `7z` / `7za` on PATH and Windows installation directories (`C:\Program Files\7-Zip\7z.exe`), with bsdtar fallback for `.7z` and `.rar`.
   - **Directory Flattening & Prefix Stripping**: Automatically strips wrapper directories (`clang+llvm-.../`) and single root wrappers into clean target directories.

## 5. Neo Distro Package & Multi-Stage Release (`distro/neo`)

`nbuild` installs into the canonical distro hierarchy: `$TOOLCHAINS_DIR/<platform>/<triple>/distro/neo/` with all necessary host symlinks into `$TOOLCHAINS_DIR/usr/bin/`, `$TOOLCHAINS_DIR/usr/share/`, and target sysroot directories.

1. **Unified Multi-Crate Monorepo Packaging**:
   - The Neo repository contains multiple crates and tools: `nbuild`, `archinfo`, `tokless`, `archinfo`, and auxiliary agent tools.
   - Forge API endpoint: `https://api.github.com/repos/Navegos/neo/releases/latest`.
   - Multi-stage release asset: a single unified archive containing all compiled crates and tools for each target platform/triple.
   - **Unified Update Semantics**: Updating `nbuild` automatically updates the entire `neo` distro package, updating all binaries (`nbuild`, `archinfo`, `tokless`) in lockstep from the single multi-crate release.
2. **Catalog Manifest (`catalog/neo.json`)**:
   - Defines `name: "neo"` with `releases_url: "https://api.github.com/repos/Navegos/neo/releases/latest"`.
   - Exports all binaries (`nbuild`, `archinfo`, `tokless`) into `bin/` and maps symlinks into `usr/bin/`.
   - Embeds package metadata, version tags, and asset hints (`neo-{triple}`).

## 6. Self-Diagnosis, Doctor & Self-Healing Engine (`nbuild doctor`)

Self-healing diagnostic engine callable via `nbuild doctor`, `--heal`, or automatically upon launching:

1. **Installation Topology & Structure Check**:
   - Inspects the current running executable path against `$TOOLCHAINS_DIR/<platform>/<triple>/distro/neo/bin/nbuild`.
   - **Out-of-Topology Warning**: If executed from outside (e.g. temporary download directory, Desktop, or git checkout), warns user:
     `"Warning: nbuild is not installed in the canonical toolchain hierarchy ($TOOLCHAINS_DIR/.../distro/neo/)"`.
   - **Self-Installation & Provisioning**:
     - Offers interactive prompt or automatic flag to install itself and companion binaries into `$TOOLCHAINS_DIR/<platform>/<triple>/distro/neo/bin/`.
     - Generates canonical `pkg_meta.json` receipt.
     - Provisions all active symlinks into `$TOOLCHAINS_DIR/usr/bin/`.
2. **Broken Link Detection & Auto-Repair**:
   - Scans `$TOOLCHAINS_DIR/usr/bin/`, `$TOOLCHAINS_DIR/usr/include/`, `$TOOLCHAINS_DIR/usr/lib/`, and target triple sysroots for broken, dangling, or missing links.
   - Detects deleted target packages, stale junctions, or broken hardlinks.
   - Self-heals: automatically purges orphaned links and regenerates missing symlinks pointing to valid distro binaries and libraries.
3. **Broken Package & Bundled File Integrity Repair**:
   - Verifies existing installed packages against their `pkg_meta.json` file inventory and SHA256 checksums.
   - If corrupted files, missing headers, or truncated binaries are detected:
     - Automatically repairs the broken package by re-extracting from cache or re-fetching the release archive.
4. **Post-Healing Smoke & Health Test**:
   - Executes immediate brief self-tests after installation or repair:
     - Executes installed binary with `--version` and `--check`.
     - Verifies PATH accessibility and symlink resolution.
     - Reports green health check status to user.

## 7. Environment Topology & PATH Self-Healing Engine

Automates environment setup so tools and packages are immediately operational:

1. **`TOOLCHAINS_DIR` Environment Discovery**:
   - Checks user and system environment for `TOOLCHAINS_DIR`.
   - **Interactive Selection**: If `TOOLCHAINS_DIR` is not found, interactively prompts user to confirm or choose path:
     - Windows default: `C:\opt\neo-toolchains\` (fallback: `C:\opt\toolchains\`).
     - Linux/macOS default: `/opt/neo-toolchains/` (fallback: `/opt/toolchains/`).
   - Persistently sets `TOOLCHAINS_DIR` in user environment:
     - Windows: User registry `HKCU\Environment` + `WM_SETTINGCHANGE` broadcast.
     - Linux/macOS: Shell config files (`~/.bashrc`, `~/.zshrc`, or `/etc/profile.d/neo.sh`).
2. **Host PATH Healing (`$TOOLCHAINS_DIR/usr/bin`)**:
   - Inspects active `PATH` for `$TOOLCHAINS_DIR/usr/bin`.
   - If missing from `PATH`:
     - Self-heals by adding `$TOOLCHAINS_DIR/usr/bin` to the user's permanent `PATH`.
     - Ensures required directory hierarchy exists (`usr/bin`, `usr/include`, `usr/lib`, `usr/share`).
   - Reloads/exports updated environment within the running session.

## 8. Interactive CLI Menu & TUI System

Provides a terminal UI and interactive menu when run without subcommands or via `nbuild menu` / `nbuild -i`:

1. **Dashboard & Status Overview**:
   - Shows active `TOOLCHAINS_DIR`, target triple, and status of host `usr/bin` in `PATH`.
   - Displays installed tools and packages (`neo`, `cmake`, `ninja`, `llvm`, etc.) with versions and health status.
2. **Interactive Menu Options**:
   - `[1] System Doctor & Self-Heal`: Check directory topology, repair broken symlinks, repair corrupted packages, run smoke tests.
   - `[2] Environment & PATH Configuration`: Configure `TOOLCHAINS_DIR`, heal user `PATH`, inspect active environment.
   - `[3] Package & Toolchain Manager`: Search, install, update, clone, build, or remove packages from catalogs.
   - `[4] File & Directory Operations`: Built-in search, copy, move, rename, delete, mkdir.
   - `[5] Local Workflow Runner`: Inspect and trigger local GitHub Actions-style workflows.
   - `[6] Exit`.
3. **Keyboard & Terminal Support**:
   - Arrow-key navigation, quick number selection, search filter, and instant execution.

## 9. File & Directory Management Engine (`core::fs_ops`)

Comprehensive cross-platform filesystem management subsystem supporting CLI commands and programmatic APIs:

1. **Capabilities**:
   - **Search**: Glob patterns, regex, recursive search for files and directories with size and type filters.
   - **Create**: Directory trees (`mkdir -p`), touch empty files, write atomic files.
   - **Copy**: Recursive directory copy and single-file copy with collision guards and attribute preservation.
   - **Move & Rename**: Atomic rename, cross-filesystem directory and file movement.
   - **Delete**: Safe file deletion and recursive directory removal with read-only permission clearing (Windows read-only file deletion resilience).
2. **CLI Commands**:
   - `nbuild fs search <pattern> [--dir <path>] [--type file|dir]`
   - `nbuild fs copy <src> <dest> [--recursive]`
   - `nbuild fs move <src> <dest>`
   - `nbuild fs rename <src> <new_name>`
   - `nbuild fs delete <path> [--recursive] [--force]`
   - `nbuild fs mkdir <dir>`

## 10. Clean Package Updates & In-Use Self-Update Resilience

Guarantees clean directory state and prevents Windows executable locking failures during updates:

1. **Clean Directory Updates**:
   - Before extracting an updated package or tool, old package files recorded in `pkg_meta.json` are purged from disk first.
   - Avoids orphaned legacy files, obsolete headers, or stale libraries polluting the new installation.
2. **Windows In-Use Binary Locking Mitigation (Shadow Swap / Self-Replace)**:
   - On Windows, running binaries cannot be directly overwritten (`ERROR_SHARING_VIOLATION` / `ERROR_ACCESS_DENIED`).
   - However, the Windows NTFS filesystem **permits renaming running executables**.
   - **Self-Update & Active Tool Replacement Procedure**:
     - Step 1: When updating `nbuild` or any tool in active use, rename the active running executable (e.g. `nbuild.exe` -> `nbuild.exe.old` or temporary swap name).
     - Step 2: Unpack or compile the new binary directly into the canonical location (`distro/neo/bin/nbuild.exe`).
     - Step 3: Spawn the newly installed binary to complete post-install symlinking, smoke tests, and environment healing.
     - Step 4: Schedule deletion of the renamed legacy `.old` binary on exit or during the next run.

## 11. Symlink Engine & Collision Safety

- **Windows directory links**: NTFS Junctions (`junction`), no administrator rights required.
- **Windows file links**: File symlink when Developer Mode is enabled; automatic fallback to NTFS Hard Link (`std::fs::hard_link`).
- **Windows runtime DLLs**: Automatically placed into `bin/` or `usr/bin/` so they resolve on `PATH`.
- **Collision guards**: Checks destination before link creation. If different SHA256 detected, reports collision warning and refuses destructive overwrite.
- **Header isolation**: System headers go to `include/`; library dependencies go to `usr/include/` (or namespaced directories e.g. `usr/include/libpng/`).

## 12. Build Engine Adapters

Trait `BuildEngine` in `src/builders/mod.rs` with adapters for:
- **C/C++ Native**: CMake, Ninja, Meson, Autotools (Autoconf/Automake), Make, NMake/Jom, MSBuild, Xcodebuild.
- **Distributed & Monorepo**:
  - Buck2 (`inspect-vault/buck2`)
  - Bazel
  - NativeLink (`inspect-vault/nativelink`) remote cache & execution engine (REAPI v2).
- **Language Runtimes**: Cargo, Go build, Python build (flit/hatch/maturin), Node.js (npm/yarn/pnpm/npx), JVM (Gradle/Maven/Ant).

## 13. Remote Worker Execution

- `nbuild remote add <name> <connection-uri>`
- `nbuild remote install <name>`: Deploys self-contained `nbuild` binary to remote node via SSH/WinRM.
- `nbuild remote exec <name> ...`: Triggers builds or installs on remote nodes for master/worker distributed setups.

## 14. AI Agent Integrations

- `--json`: Structured agent-readable JSON schema responses.
- `--quiet`: Terse output, tail-only error reports on compiler failures (saves LLM context tokens).
- `nbuild env --shell json`: Emits active environment variables map (`PATH`, `INCLUDE`, `LIB`, `CMAKE_PREFIX_PATH`).
- MCP server mode (`nbuild mcp`): Exposes native tool calls (`nbuild_get`, `nbuild_build`, `nbuild_env`, `nbuild_list`) for autonomous agents.

## 15. Current Status & Implemented Packages

- **Subsystems Implemented**:
  - `core::config`: Multi-root discovery, unversioned distro store paths.
  - `core::arch`: Target triple resolution, CPU feature detection (`archinfo`).
  - `core::meta`: `pkg_meta.json` schema, file integrity tracking, symlink manifests.
  - `core::symlink`: NTFS junctions, hardlinks, symlinks, collision detection.
  - `core::sysroot`: GNU root and `usr/` sysroot builder, host runtime prefix.
  - `core::forge`: Multi-forge API client, dynamic release resolution, compression ranking, stem hint matching.
  - `core::distro`: Universal archive extraction (`.tar.gz`, `.tar.xz`, `.tar.zst`, `.zip`, `.tar.bz2`, `.7z`, `.rar`).
  - `core::catalog`: JSON/YAML catalog registry with multi-path loader.
  - `cli`: Subcommands `init`, `get`, `check`, `update`, `clone`, `build`, `list`, `env`, `link`, `run`.
- **Active Catalog Manifests (`catalog/*.json`)**:
  - `cmake.json`: Kitware/CMake dynamic releases API (`v4.4.4` live verified).
  - `ninja.json`: ninja-build/ninja dynamic releases API (`v1.13.2` live verified).
  - `llvm.json`: llvm/llvm-project dynamic releases API (`v23.1.3` live verified).
  - `zlib.json`: madler/zlib dynamic releases API.
  - `libpng.json`: pnggroup/libpng dynamic releases API + zlib dependency.
- **Verification**:
  - 72/72 tests passing across all workspace crates.

