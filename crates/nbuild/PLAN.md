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

## 5. Symlink Engine & Collision Safety

- **Windows directory links**: NTFS Junctions (`junction`), no administrator rights required.
- **Windows file links**: File symlink when Developer Mode is enabled; automatic fallback to NTFS Hard Link (`std::fs::hard_link`).
- **Windows runtime DLLs**: Automatically placed into `bin/` or `usr/bin/` so they resolve on `PATH`.
- **Collision guards**: Checks destination before link creation. If different SHA256 detected, reports collision warning and refuses destructive overwrite.
- **Header isolation**: System headers go to `include/`; library dependencies go to `usr/include/` (or namespaced directories e.g. `usr/include/libpng/`).

## 6. Build Engine Adapters

Trait `BuildEngine` in `src/builders/mod.rs` with adapters for:
- **C/C++ Native**: CMake, Ninja, Meson, Autotools (Autoconf/Automake), Make, NMake/Jom, MSBuild, Xcodebuild.
- **Distributed & Monorepo**:
  - Buck2 (`inspect-vault/buck2`)
  - Bazel
  - NativeLink (`inspect-vault/nativelink`) remote cache & execution engine (REAPI v2).
- **Language Runtimes**: Cargo, Go build, Python build (flit/hatch/maturin), Node.js (npm/yarn/pnpm/npx), JVM (Gradle/Maven/Ant).

## 7. Remote Worker Execution

- `nbuild remote add <name> <connection-uri>`
- `nbuild remote install <name>`: Deploys self-contained `nbuild` binary to remote node via SSH/WinRM.
- `nbuild remote exec <name> ...`: Triggers builds or installs on remote nodes for master/worker distributed setups.

## 8. AI Agent Integrations

- `--json`: Structured agent-readable JSON schema responses.
- `--quiet`: Terse output, tail-only error reports on compiler failures (saves LLM context tokens).
- `nbuild env --shell json`: Emits active environment variables map (`PATH`, `INCLUDE`, `LIB`, `CMAKE_PREFIX_PATH`).
- MCP server mode (`nbuild mcp`): Exposes native tool calls (`nbuild_get`, `nbuild_build`, `nbuild_env`, `nbuild_list`) for autonomous agents.

