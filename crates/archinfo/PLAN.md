# ArchInfo — System Architecture & Implementation Plan

High-performance CPU feature detection, microarchitecture mapping, and compiler Instruction Set Architecture (ISA) configuration tool in Rust.

## 1. System Overview

`archinfo` probes runtime hardware capabilities, maps microarchitecture specifications, resolves target vector lengths, and synthesizes optimal compiler flags for Clang/LLVM, GCC, and MSVC. It functions both as an autonomous CLI utility and as an embedded Rust / C-FFI library for native build engines (e.g. `nbuild`, UnrealBuildTool, CMake, Meson).

## 2. Microarchitecture & ISA Detection Engine

### x86_64 Hardware Probing
- **CPUID Leaf & Subleaf Decoders**:
  - Leaf `0x00000001`: Basic features (SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, AVX, FMA3, AES, PCLMULQDQ).
  - Leaf `0x00000007, Subleaf 0`: Extended features (AVX2, BMI1, BMI2, SHA-NI, AVX-512 Foundation, CD, BW, DQ, VL, IFMA, VBMI, VPOPCNTDQ, BFLOAT16).
  - Leaf `0x00000007, Subleaf 1`: Cutting-edge extensions (AVX10.1, AVX10.2, APX-F, AMX-TILE, AMX-INT8, AMX-BF16, AVX-VNNI).
  - Leaf `0x80000001`: AMD extended features (SSE4A, FMA4, XOP, 3DNow!).
- **OS-Save State Validation (XCR0)**:
  - Validates `XGETBV` register bitmask (`XSTATE_FP`, `XSTATE_SSE`, `XSTATE_YMM`, `XSTATE_OPMASK`, `XSTATE_ZMM_Hi256`, `XSTATE_Hi16_ZMM`) before enabling AVX/AVX-512 execution to guarantee OS context switching support.

### AArch64 (ARM64) Probing
- **Linux & Android**: Probes ELF auxiliary vector capabilities via `getauxval(AT_HWCAP)` and `getauxval(AT_HWCAP2)`.
- **macOS, iOS, visionOS, tvOS**: Probes CPU features dynamically via `sysctlbyname("hw.optional.*")`.
- **Windows on ARM**: Probes processor capabilities via `IsProcessorFeaturePresent()`.
- **Supported ISAs**: NEON, FP16, DotProd, SVE, SVE2, I8MM, BF16, AES, SHA2, SHA3, SM4, PAC/BTI, LSE, CRC32.

### RISC-V 64 Probing
- **Standard Extension Discovery**: Base `rv64gc` (`i`, `m`, `a`, `f`, `d`, `c`) + Vector (`v`, `zve32x`, `zve32f`, `zve64x`, `zve64f`, `zve64d`).
- **Bitmanip Extensions**: `zba`, `zbb`, `zbc`, `zbs`.

## 3. Platform & Console Hardware Profiles

Preconfigured hardware architectures tuned for specific compute and gaming targets:

| Category | Profile Name | Microarchitecture & Vector Constraints |
|---|---|---|
| **Consoles** | `ps5` | AMD Zen 2 (Oberon APU), 8c/16t, vector width clamped to 128-bit (`vl128`) for optimal Zen 2 throughput |
| | `ps4` | AMD Jaguar, 8c/8t, AVX / F16C baseline |
| | `xboxxs` | AMD Zen 2 (Scarlett APU), 8c/16t, 256-bit SIMD (`vl256`) |
| | `xboxone` | AMD Jaguar, 8c/8t |
| | `switch2` | NVIDIA Tegra T239 (Drake), Cortex-A78C, ARMv8.2-A + DotProd + FP16 |
| | `steamdeck` | AMD Zen 2 (Van Gogh APU), 4c/8t, RDNA 2 |
| **Desktop / Server** | `windows` | Native host probing or explicit target triple (`x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`) |
| | `linux` | Native host probing or explicit target triple (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`) |
| | `macosx` | Apple Silicon (`aarch64-apple-darwin`) or Intel x86_64 |
| | `freebsd` | FreeBSD x86_64 / AArch64 |
| **Mobile / XR** | `android` | ARM64 Android NDK ABI baselines |
| | `ios` / `tvos` | Apple A-series / M-series ARM64 baselines |
| | `xros` | Apple Vision Pro (xrOS / visionOS) M2 + R1 architecture |

## 4. Compiler Flag Synthesis Engine

Direct, deterministic translation of target feature sets into compiler flags:

- **Clang / LLVM**:
  - Architecture & CPU: `-march=<arch>`, `-mcpu=<cpu>`
  - Target features: `-target-feature +<ext>` or `-m<ext>` (e.g. `-mavx2`, `-mfma`, `-msve2`)
  - Vector width: `-mprefer-vector-width=<128|256|512>`
- **MSVC (Microsoft Visual C++)**:
  - Flags: `/arch:AVX`, `/arch:AVX2`, `/arch:AVX512`, `/arch:armv8.0` through `/arch:armv8.8`
- **GCC**:
  - Flags: `-march=<arch>`, `-mtune=<tune>`, `-m<ext>`

## 5. Library & C-FFI ABI Specification

Exported symbols declared in `src/c_abi.rs` for integration into non-Rust ecosystems:
- `archinfo_detect_host()`: Probes the current hardware host.
- `archinfo_has_feature(const char* feature)`: Boolean query for specific ISA feature.
- `archinfo_synthesize_cflags(const char* compiler, char* out_buf, size_t buf_len)`: Emits ready-to-use compiler arguments.

## 6. Crates.io Publishing & Release Lifecycle

1. **Autonomous Entity on Crates.io**:
   - Package name: `archinfo`.
   - Independent semver releases (`v1.0.5`, `v1.0.6`, ...).
   - Serves as the foundation dependency for `nbuild` and other ecosystem crates.
2. **Release Pipeline**:
   - Tagged release triggers GitHub Actions (`v*`).
   - Cross-compiles for Linux, Windows, macOS (x86_64, aarch64, riscv64).
   - Generates `.tar.zst` distribution bundles with SHA256 hashes.
   - Publishes to crates.io via `cargo publish -p archinfo` as Step 1 of the release pipeline.

