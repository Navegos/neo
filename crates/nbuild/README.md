# nbuild

[![Crates.io](https://img.shields.io/crates/v/nbuild.svg)](https://crates.io/crates/nbuild)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**nbuild** is a universal toolchain manager, GNU sysroot builder, local GitHub Actions workflow runner, and package engine for AI coding agents and native developers.

Written in modern Rust (Edition 2024), `nbuild` provisions clean, immutable toolchain directories, automatically maps sysroot symlinks with collision guards, dynamically resolves prebuilt release assets across multiple Git forges, and heals broken environment paths and packages.

---

## Features

- **Standardized Directory & Toolchain Topology**:
  - Root managed under `$TOOLCHAINS_DIR` (defaults to `C:\opt\neo-toolchains\` on Windows, `/opt/neo-toolchains/` on Linux/macOS).
  - Isolated immutable distro store: `$TOOLCHAINS_DIR/<platform>/<target-triple>/distro/<pkg_name>/` (clean, unversioned folders).
  - Complete GNU Sysroot: Root level (compilers, OS SDKs, runtimes) and User layer (`usr/bin`, `usr/include`, `usr/lib`, `usr/share`).
  - Active Host Runtime Prefix (`$TOOLCHAINS_DIR/usr/bin`) automatically exposed on `PATH`.
- **Dynamic Multi-Forge Release Discovery**:
  - Queries forge releases APIs (GitHub, GitLab, Codeberg/Gitea, Bitbucket) dynamically at runtime.
  - No fragile hardcoded download URLs; resolves tags, versions, and archive assets on demand.
  - Archive format ranking with OS-preference scoring: `.tar.zst` > `.tar.xz` > `.zip` > `.7z` > `.tar.gz` > `.tar.bz2` > `.rar`.
  - Extension-agnostic stem hint matching (`strip_archive_extension`) makes catalogs resilient against upstream format migrations.
- **Universal Archive Extraction Engine**:
  - In-process pure-Rust decompression for `.zip` and `.tar.gz`.
  - Subprocess system `tar` (bsdtar) for `.tar.xz`, `.tar.zst`, `.tar.bz2`.
  - 7-Zip fallback for `.7z` and `.rar` archives.
  - Automatic directory flattening and wrapper directory prefix stripping.
- **Self-Diagnosis & Self-Healing (`nbuild doctor`)**:
  - Validates installation topology; warns when executed outside canonical toolchain structure.
  - Auto-installs self and companion tools (`tokless`, `archinfo`) into `distro/neo/bin/`.
  - Automatically detects and repairs broken, dangling, or stale symlinks and junctions.
  - Verifies package integrity against `pkg_meta.json` checksums; repairs missing or broken files.
  - Executes immediate smoke tests (`--version`, link resolution) to ensure operational readiness.
- **Environment & PATH Self-Healing (`nbuild env-setup`)**:
  - Automatically detects or prompts for `TOOLCHAINS_DIR` and persists it permanently into the user environment.
  - Inspects user `PATH` for `$TOOLCHAINS_DIR/usr/bin`; appends or prepends it if missing and provisions base folders.
- **Interactive CLI Menu & TUI (`nbuild menu`)**:
  - Terminal-based dashboard for toolchain health, package updates, doctor self-healing, and workflow execution.
- **Built-in File & Directory Operations (`nbuild fs`)**:
  - Cross-platform `search` (glob/recursive), `copy`, `move`, `rename`, `delete` (with Windows read-only attribute clearing), `mkdir`, and `touch`.
- **Clean Updates & Windows In-Use Binary Resilience**:
  - Purges old package files prior to extraction to prevent orphaned legacy files.
  - Overcomes Windows executable file locking (`ERROR_ACCESS_DENIED`) by renaming active running binaries before replacement.
- **Local GitHub Actions Workflow Runner**:
  - Parses `.github/workflows/*.yml` files and executes workflow jobs and steps locally without Docker dependencies.

---

## Directory Hierarchy

```
$TOOLCHAINS_DIR/
│
├── usr/                                          # Host active runtime prefix ($TOOLCHAINS_DIR/usr/bin in Host PATH)
│   ├── bin/                                      # Symlinks: host-compatible distro/*/bin/* + DLLs
│   ├── include/                                  # Symlinks: host headers
│   ├── lib/                                      # Symlinks: host libraries
│   └── share/                                    # Symlinks: host pkgconfig (.pc), cmake modules
│
└── <platform>/                                   # windows | linux | apple | freebsd | android
    └── <target-triple>/                          # e.g. x86_64-pc-windows-msvc, aarch64-unknown-linux-gnu
        │
        ├── distro/                               # Isolated immutable package stores (clean folder names)
        │   ├── neo/                              # Neo unified distro: nbuild, tokless, companion tools
        │   ├── cmake/
        │   ├── ninja/
        │   ├── llvm/
        │   └── zlib/
        │
        ├── bin/, include/, lib/, share/          # Target Triple Root GNU Sysroot (Compilers, SDKs)
        └── usr/                                  # Target Triple User Sysroot Layer (Libraries, Headers)
            ├── bin/
            ├── include/
            ├── lib/
            └── share/
```

---

## Installation

### From Crates.io
```bash
cargo install nbuild
```

### From Source
```bash
git clone https://github.com/Navegos/neo.git
cd neo
cargo build --release -p nbuild
```

---

## CLI Usage

### System Doctor & Self-Healing
```bash
# Run diagnostic check on directory structure, symlinks, and environment
nbuild doctor

# Run self-diagnosis and automatically heal broken links and install self to distro/
nbuild doctor --heal --brief-test
```

### Environment Setup
```bash
# Interactively configure TOOLCHAINS_DIR and add $TOOLCHAINS_DIR/usr/bin to PATH
nbuild env-setup

# Non-interactive configuration with default directory
nbuild env-setup --non-interactive
```

### Interactive Menu
```bash
# Launch interactive terminal UI
nbuild menu

# Or simply run nbuild with no arguments
nbuild
```

### Package Management
```bash
# Install a prebuilt tool from catalog (e.g. cmake, ninja, llvm, neo)
nbuild get cmake

# Check package status against latest remote forge release
nbuild check cmake

# Update package to latest version (cleanly deleting old files)
nbuild update cmake

# List installed packages and available catalog recipes
nbuild list
```

### File Operations (`nbuild fs`)
```bash
# Recursive search with pattern or wildcard
nbuild fs search "*.dll" --dir C:\opt\neo-toolchains

# Recursive directory copy
nbuild fs copy src_dir dest_dir --recursive

# Move or rename
nbuild fs move old_path new_path
nbuild fs rename file.txt file_renamed.txt

# Delete file or directory (clears read-only attributes on Windows)
nbuild fs delete dir_to_remove --recursive --force

# Create directory tree (mkdir -p)
nbuild fs mkdir path/to/dir
```

### Run Local Workflow
```bash
nbuild run .github/workflows/build.yml
```

### Export Environment
```bash
# Output PowerShell environment exports
nbuild env

# Output Bash environment exports
nbuild env --shell bash

# Output JSON map for AI coding agents
nbuild env --shell json
```

---

## Package Receipt (`pkg_meta.json`)

Each installed package contains an immutable JSON receipt:

```json
{
  "$schema": "https://neo.build/schemas/pkg_meta.v1.json",
  "name": "ninja",
  "version": "1.13.2",
  "type": "prebuilt_binary",
  "target": {
    "platform": "windows",
    "triple": "x86_64-pc-windows-msvc",
    "arch": "x86_64",
    "target_level": "system"
  },
  "installed_files": [
    { "path": "bin/ninja.exe", "sha256": "...", "size": 654336 }
  ],
  "symlinks": [
    { "src": "bin/ninja.exe", "dest": "C:/opt/neo-toolchains/usr/bin/ninja.exe", "symlink_type": "exec" }
  ]
}
```

---

## License

Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

