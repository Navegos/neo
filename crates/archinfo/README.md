# ArchInfo

[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20Consoles%20%7C%20Mobile-brightgreen.svg)]()

**ArchInfo** is a high-performance CPU feature detection, hardware architecture mapping, and compiler Instruction Set Architecture (ISA) configuration engine written in Rust.

Designed for game engines (such as Unreal Engine 5), native toolchains, and high-performance computing runtimes, ArchInfo probes runtime hardware capabilities, maps microarchitecture specifications, resolves target vector lengths, and synthesizes optimal compiler flags for **Clang/LLVM** and **MSVC**.

---

## Features

- **Live Host CPU Probing**:
  - **x86_64**: Complete leaf and sub-leaf CPUID feature detection and XCR0 OS-save state validation (AVX, AVX2, AVX-512, AVX10.1, AVX10.2, APX-F, AMX, SHA, AES, and more).
  - **AArch64 (ARM64)**: Hardware capability probing via OS APIs (`getauxval`/ELF HWCAP on Linux/Android, `sysctlbyname` on macOS/iOS, `IsProcessorFeaturePresent` on Windows).
  - **RISC-V 64**: Extension discovery (`i`, `m`, `a`, `f`, `d`, `c`, `v`, `zba`, `zbb`, `zbc`, `zbs`, and experimental extensions).
- **Target Platform & Console Presets**:
  - Preconfigured hardware profiles for consoles: **PlayStation 5** (AMD Zen 2 custom APU), **PlayStation 4**, **Xbox Series X/S**, **Xbox One**, **Nintendo Switch 2** (Tegra T239 / Cortex-A78C), **Steam Deck**, and **Steam Machine**.
  - Desktop and mobile platforms: **Windows**, **Linux**, **macOS**, **iOS**, **tvOS**, **visionOS (xrOS)**, **Android**, and **FreeBSD**.
- **Compiler Flag Synthesis**:
  - Direct translation of CPU features into Clang/LLVM arguments (`-mcpu=...`, `-march=...`, `-target-feature +<isa>`, `-mprefer-vector-width=...`).
  - Direct translation into MSVC arguments (`/arch:AVX2`, `/arch:AVX512`, `/arch:armv8.x`, etc.).
- **SIMD Vector Length Resolution**:
  - Explicit and profile-aware SIMD width configuration (`vl128`, `vl256`, `vl512`).
  - Enforces console-specific microarchitectural optimizations (e.g. PS5 Zen 2 Oberon APU defaulting to 128-bit vector width).
- **Baseline Architectures & Custom Extensions**:
  - Define minimum CPU architectural baselines: `x86-64-v2` through `x86-64-v4`, AVX, AVX2, AVX512, AVX10.1, AVX10.2 for x86_64; `armv8.0-a` through `armv9.7-a` for ARM64.
  - Enable or disable specific ISA extensions on generic targets (`--enable-extensions "aes+sha2"`, `--disable-extensions "sve+sve2"`).
- **Compatibility Matrix Generation**:
  - Generate full cross-platform, cross-architecture configuration matrices.
  - Save standardized canonical JSON profiles (`{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}.json`).
- **Rust Library & C-FFI ABI**:
  - First-class Rust API.
  - Exported C ABI symbols for seamless integration into C/C++ build systems, UnrealBuildTool, CMake, or Meson.

---

## Supported Architectures & Platforms

### Architectures
- `x86_64` (AMD64 / Intel 64)
- `arm64` (`aarch64`, `arm64ec`, `arm64e`)
- `riscv64` (RISC-V 64-bit Application Profiles)

### Platforms & Environments
| Category | Supported Platforms |
|---|---|
| **Consoles** | `ps5`, `ps4`, `xboxxs` (Series X/S), `xboxone`, `switch2` (NX2), `steamdeck`, `steammachine` |
| **Desktop / Server** | `windows`, `linux`, `macosx`, `freebsd` |
| **Mobile / XR** | `android`, `ios`, `tvos`, `xros` (visionOS) |

---

## Installation & Building

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) 1.85+ (Edition 2024)
- Cargo

### Build from Source
```bash
# Clone the repository
git clone https://github.com/Navegos/ArchInfo.git
cd ArchInfo

# Build in release mode
cargo build --release

# The binary will be available at:
# ./target/release/archinfo (Linux/macOS)
# .\target\release\archinfo.exe (Windows)
```

### Run Tests
```bash
cargo test
```

---

## CLI Usage

```text
High-performance CPU features detection, architecture mapping, and compiler ISA configuration tool

Usage: archinfo [OPTIONS]

Options:
  -p, --platform <PLATFORM>                    Target OS platform (native, windows, linux, mac, ios, android, ps5, switch, xboxxs, etc.)
  -a, --arch <ARCH>                            Target hardware architecture (native, x86_64, arm64, riscv64)
  -t, --target <TARGET>                        Target microarchitecture (native, generic, znver3, alderlake, apple-m4, cortex-a78c, etc.)
  -u, --target-tune <TARGET_TUNE_CPU>          Target tune CPU microarchitecture
  -m, --min-cpu-arch <MIN_CPU_ARCH>            Minimum CPU architecture baseline (AVX, AVX2, AVX512, AVX10.1, AVX10.2; ARMv8-A..ARMv9.7-A)
  -e, --enable-extensions <ENABLE_EXTENSIONS>  Enabled ISA extensions to include (generic target only, e.g. "aes+sha2")
  -d, --disable-extensions <DISABLE_EXTENSIONS>Disabled ISA extensions to exclude (generic target only, e.g. "sse4.1")
  -v, --vector-length <VECTOR_LENGTH>          Preferred vector length for SIMD code generation (128, 256, 512, vl128, vl256, vl512)
  -o, --output <OUTPUT>                        Path to output JSON file or directory
  -s, --save                                   Save result using canonical filename in default or selected output folder
  -M, --matrix                                 Generates full compatibility matrix across all platforms and architectures
  -D, --detect                                 Force host CPU detection
  -f, --format <FORMAT>                        Output format (json, extensions, clang, msvc, filename, dir) [default: json]
  -h, --help                                   Print help
  -V, --version                                Print version
```

### Examples

#### 1. Probing Host CPU
Detect the running host CPU instruction set and return a full JSON report:
```bash
archinfo --detect
```

Print Clang compiler flags for the current host:
```bash
archinfo --detect --format clang
```

#### 2. Querying Console Profiles
Inspect target settings for Nintendo Switch 2 (Tegra T239 Cortex-A78C):
```bash
archinfo -p switch2 --format json
```

Generate Clang compiler flags for PlayStation 5:
```bash
archinfo -p ps5 --format clang
```

#### 3. Targeting Specific Microarchitectures
Generate Clang flags for Snapdragon X Elite (Oryon-1) on Windows Arm64:
```bash
archinfo -p windows -a arm64 -t oryon-1 --format clang
```

Generate MSVC flags for Intel Alder Lake on Windows:
```bash
archinfo -p windows -a x86_64 -t alderlake --format msvc
```

#### 4. Baseline & Vector Width Configuration
Target generic x86_64 with AVX-512 baseline and 256-bit preferred vector length:
```bash
archinfo -a x86_64 -m avx512 -v 256 --format clang
```

Target generic x86_64 with AVX2 baseline, enabling SHA and AES extensions:
```bash
archinfo -a x86_64 -m avx2 -e "aes+sha" --format json
```

#### 5. Compatibility Matrix Export
Generate and save all platform/architecture profiles to the standard directory:
```bash
archinfo --matrix --save
```

Output all profiles to a custom folder:
```bash
archinfo --matrix -o ./profiles/
```

---

## Canonical JSON Schema

ArchInfo outputs structured JSON profiles adhering to the following schema:

```json
{
  "platform": "switch2",
  "arch": "aarch64",
  "extensions": "crypto+aes+crc+dotprod+flagm+fp+fp16+lse+pauth+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
  "target_cpu": "cortex-a78c",
  "target_tune_cpu": "cortex-a78c",
  "min_cpu_arch": "armv8.4-a",
  "vector_length": "vl128",
  "target_msvc_arch": "",
  "target_msvc_vlen": "",
  "target_clan_arch": "-m'arch=armv8.4-a+aes+crc+crypto+dotprod+flagm+fp+fp16+lse+pauth+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs'",
  "target_clang_isaarch": "",
  "target_clang_cpu": "-m'cpu=cortex-a78c'",
  "target_clang_tune_cpu": "-m'tune=cortex-a78c'",
  "target_clang_vlen": "-m'prefer-vector-width=128'",
  "target_clang_extraargs": "",
  "features": {
    "aes": true,
    "crc": true,
    "dotprod": true,
    "fp16": true,
    "lse": true,
    "pauth": true,
    "sha2": true,
    "simd": true,
    "...": false
  }
}
```

### Canonical Filename Pattern
When saved, files follow the standard convention:
- **Android**: `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}-ndk-{target_os_level.Major.Minor}.json`
- **FreeBSD**: `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}-freebsd-{target_os_level.Major.Minor}.json`
- **Linux | Steamdeck | Steammachine**: `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}-glibc-{target_runtime_level.Major.Minor}.json`
- **Windows | Xboxone | Xboxxs**: `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}-msvc-{target_runtime_level.Major.Minor}.json`
- **Macosx**: `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}-macosx-{target_os_level.Major.Minor}.json`
- **IOS / TVOS / XrOS**: `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}-{os}-{target_os_level.Major.Minor}[-simulator].json`
- **Ps4 | Ps5 | Switch2** (or unset/empty levels): `{platform}-{arch}-{targetcpu}-{mincpuarch}-{vectorlength}.json`

For example:
- `switch2-aarch64-cortex-a78c-armv8.4-a-vl128.json`
- `ps5-x86_64-ps5-avx2-vl128.json`
- `windows-x86_64-generic-x86-64-v3-avx2-vl256-msvc-19.51.json`
- `linux-x86_64-znver3-avx2-vl256-glibc-2.17.json`
- `macosx-aarch64-apple-m4-armv9.2-a-vl128-macosx-15.0.json`
- `ios-aarch64-apple-m4-armv9.2-a-vl128-ios-26.0-simulator.json`

### Default Storage Paths
- **Windows**: `%APPDATA%\ArchInfo` or `%USERPROFILE%\Documents\ArchInfo`
- **macOS**: `~/.config/ArchInfo`
- **Linux / FreeBSD**: `$XDG_CONFIG_HOME/ArchInfo` or `~/.config/ArchInfo`

---

## Library Integration

### Rust API
Add ArchInfo to your `Cargo.toml`:
```toml
[dependencies]
archinfo = { path = "../path/to/ArchInfo" }
```

Use in code:
```rust
use archinfo::{Arch, Platform, ArchFeaturesReport, CPUFeatures};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Live host CPU detection
    let host_features = CPUFeatures::detect_host();
    let host_report = ArchFeaturesReport::from_host(&host_features);
    println!("Host extensions: {}", host_report.extensions);

    // 2. Query target profile (e.g. Nintendo Switch 2)
    let switch_report = ArchFeaturesReport::from_target(Platform::Switch2, Arch::Arm64)?;
    println!("Switch 2 Clang CPU: {:?}", switch_report.target_clang_cpu);

    // 3. Custom evaluation with microarchitecture tuning and vector length
    let custom_report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        Some("znver3"),
        Some("avx2"),
        Some("aes+sha"),
        None,
        None,
    )?;
    println!("JSON output:\n{}", custom_report.to_json()?);

    Ok(())
}
```

### C-FFI API
ArchInfo exports an `extern "C"` ABI for C/C++ projects, UnrealBuildTool, or Python bindings:

```c
#ifdef __cplusplus
extern "C" {
#endif

// Returns the OS default output folder (Caller must free using archinfo_free_string)
char* archinfo_get_default_output_dir(void);

// Runs live host detection and returns a JSON string (Caller must free)
char* archinfo_detect_host_json(void);

// Queries target platform + architecture features JSON
char* archinfo_query_target_json(const char* platform_str, const char* arch_str);

// Evaluates platform, architecture, and microarchitecture target
char* archinfo_evaluate_json(const char* platform_str, const char* arch_str, const char* target_cpu_str);

// Frees any string allocated by the ArchInfo library
void archinfo_free_string(char* s);

#ifdef __cplusplus
}
#endif
```

---

## Contributing

Contributions, issues, and feature requests are welcome!

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/new-target-cpu`).
3. Ensure all tests pass:
   ```bash
   cargo test
   ```
4. Commit your changes (`git commit -m "feat: add support for cortex-x925"`).
5. Push to the branch (`git push origin feature/new-target-cpu`).
6. Open a Pull Request.

---

## License

Copyright (c) 2026 Navegos. [@DevelVitorF](https://github.com/DevelVitorF). All Rights Reserved.

Licensed under the [Apache License, Version 2.0](LICENSE).
