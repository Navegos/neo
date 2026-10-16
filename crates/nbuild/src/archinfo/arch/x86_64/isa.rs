// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/x86_64/isa.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Enum representing Instruction Set Architecture (ISA) extensions for the x86_64 architecture.
///
/// This enum is used to specify the supported or required ISA extensions for compilation.
/// Each variant corresponds to a specific feature or instruction set extension, enabling
/// fine-grained control over the compilation process for x64 targets.
///
/// For more details, see <https://clang.llvm.org/docs/ClangCommandLineReference.html#x86>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum X64ISA {
    /// No specific ISA extension.
    None,

    /// ADX (Multi-Precision Add-Carry Instruction Extensions).
    /// Provides instructions for multi-precision arithmetic, such as ADCX and ADOX.
    Adx,

    /// AES (Advanced Encryption Standard) instructions.
    /// Provides hardware acceleration for AES encryption and decryption.
    Aes,

    /// AMX (Advanced Matrix Extensions) for AVX-512.
    /// Provides support for matrix operations, such as tile configuration and data movement.
    AmxAvx512,

    /// AMX BF16 (Bfloat16) instructions.
    /// Enables support for bfloat16 matrix operations, commonly used in machine learning workloads.
    AmxBf16,

    /// AMX Complex Number instructions.
    /// Provides support for complex number matrix operations.
    AmxComplex,

    /// AMX FP16 (16-bit floating-point) instructions.
    /// Enables support for 16-bit floating-point matrix operations.
    AmxFp16,

    /// AMX FP8 (8-bit floating-point) instructions.
    /// Provides support for 8-bit floating-point matrix operations.
    AmxFp8,

    /// AMX INT8 (8-bit integer) instructions.
    /// Enables support for 8-bit integer matrix operations.
    AmxInt8,

    /// AMX MOVRS (Move Rows) instructions.
    /// Provides instructions for moving rows in matrix operations.
    AmxMovrs,

    /// AMX TF32 (TensorFloat-32) instructions.
    /// Enables support for TensorFloat-32 matrix operations.
    AmxTf32,

    /// AMX Tile instructions.
    /// Provides support for configuring and manipulating tiles in matrix operations.
    AmxTile,

    /// APX-F (Advanced Performance Extensions).
    /// Provides general-purpose register doubling (EGPR), 3-operand forms, conditional instructions, and NF flags.
    Apxf,

    /// AVX (Advanced Vector Extensions).
    /// Provides 256-bit SIMD instructions for floating-point and integer operations.
    Avx,

    /// AVX10.1 instructions.
    /// Provides support for AVX10.1 operations.
    Avx10_1,

    /// AVX10.2 instructions.
    /// Enables support for AVX10.2 operations.
    Avx10_2,

    /// AVX2 (Advanced Vector Extensions 2).
    /// Extends AVX with support for integer operations and additional instructions.
    Avx2,

    /// AVX-512 BF16 (Bfloat16) instructions.
    /// Provides support for bfloat16 operations in AVX-512.
    Avx512bf16,

    /// AVX-512 Bit Algorithms instructions.
    /// Enables support for bit manipulation operations in AVX-512.
    Avx512bitalg,

    /// AVX-512 Byte and Word instructions.
    /// Provides support for byte and word operations in AVX-512.
    Avx512bw,

    /// AVX-512 Bit Manipulation Matrix.
    /// Provides support for bit-level matrix operations and bit reversals in AVX-512.
    Avx512bmm,

    /// AVX-512 Conflict Detection instructions.
    /// Enables conflict detection operations in AVX-512.
    Avx512cd,

    /// AVX-512 Doubleword and Quadword instructions.
    /// Provides support for doubleword and quadword operations in AVX-512.
    Avx512dq,

    /// AVX-512 Foundation instructions.
    /// The base set of AVX-512 instructions, providing foundational SIMD operations.
    Avx512f,

    /// AVX-512 FP16 (16-bit floating-point) instructions.
    /// Enables support for 16-bit floating-point operations in AVX-512.
    Avx512fp16,

    /// AVX-512 Integer Fused Multiply-Add instructions.
    /// Provides support for integer fused multiply-add operations in AVX-512.
    Avx512ifma,

    /// AVX-512 Vector Byte Manipulation instructions.
    /// Enables vector byte manipulation operations in AVX-512.
    Avx512vbmi,

    /// AVX-512 Vector Byte Manipulation 2 instructions.
    /// Extends vector byte manipulation capabilities in AVX-512.
    Avx512vbmi2,

    /// AVX-512 Vector Length extensions.
    /// Provides support for variable vector lengths in AVX-512.
    Avx512vl,

    /// AVX-512 Vector Neural Network instructions.
    /// Enables neural network operations in AVX-512.
    Avx512vnni,

    /// AVX-512 Vector Pair Intersection instructions.
    /// Provides support for vector pair intersection operations in AVX-512.
    Avx512vp2intersect,

    /// AVX-512 Vector Population Count Doubleword and Quadword instructions.
    /// Enables population count operations for doubleword and quadword in AVX-512.
    Avx512vpopcntdq,

    /// AVX Integer Fused Multiply-Add instructions.
    /// Provides support for integer fused multiply-add operations in AVX.
    Avxifma,

    /// AVX Neural Network Convert instructions.
    /// Enables neural network conversion operations in AVX.
    Avxneconvert,

    /// AVX Vector Neural Network instructions.
    /// Provides support for neural network operations in AVX.
    Avxvnni,

    /// AVX Vector Neural Network INT16 instructions.
    /// Enables support for INT16 neural network operations in AVX.
    Avxvnniint16,

    /// AVX Vector Neural Network INT8 instructions.
    /// Provides support for INT8 neural network operations in AVX.
    Avxvnniint8,

    /// BMI (Bit Manipulation Instructions).
    /// Provides instructions for bit manipulation, such as ANDN, BEXTR, and BLSI.
    Bmi,

    /// BMI2 (Bit Manipulation Instructions 2).
    /// Extends BMI with additional instructions, such as MULX and RORX.
    Bmi2,

    /// Cache Line Demote instructions.
    /// Provides instructions for demoting cache lines to a lower cache level.
    Cldemote,

    /// Cache Line Flush Optimized instructions.
    /// Enables optimized cache line flush operations.
    Clflushopt,

    /// Cache Line Write Back instructions.
    /// Provides instructions for writing back cache lines.
    Clwb,

    /// Cache Line Zero instructions.
    /// Enables instructions for zeroing cache lines.
    Clzero,

    /// Compare and Exchange Add instructions.
    /// Provides instructions for atomic compare-and-exchange operations.
    Cmpccxadd,

    /// CRC32 (Cyclic Redundancy Check) instructions.
    /// Enables hardware acceleration for CRC32 checksum calculations.
    Crc32,

    /// Compare and Exchange 16B instructions.
    /// Provides support for 16-byte atomic compare-and-exchange (`CMPXCHG16B`) operations.
    Cx16,

    /// Enqueue Command instructions.
    /// Enables instructions for enqueuing commands in hardware queues.
    Enqcmd,

    /// F16C (16-bit floating-point conversion) instructions.
    /// Enables hardware acceleration for 16-bit floating-point conversions.
    F16c,

    /// FMA (Fused Multiply-Add) instructions.
    /// Provides support for fused multiply-add operations.
    Fma,

    /// FMA4 (Fused Multiply-Add 4) instructions.
    /// Extends FMA with additional fused multiply-add operations.
    Fma4,

    /// FS/GS Base instructions.
    /// Provides instructions for accessing FS and GS segment bases.
    Fsgsbase,

    /// FXSR (Floating Point Extended Save and Restore) instructions.
    /// Enables support for saving and restoring floating-point state.
    Fxsr,

    /// Galois Field New Instructions.
    /// Provides support for Galois field arithmetic operations.
    Gfni,

    /// Key Locker instructions.
    /// Enables support for key locker operations.
    Kl,

    /// Lightweight Profiling instructions.
    /// Provides support for lightweight profiling operations.
    Lwp,

    /// Leading Zero Count instructions.
    /// Enables hardware acceleration for counting leading zeros.
    Lzcnt,

    /// Move Big Endian instructions.
    /// Enables support for moving data in big-endian format.
    Movbe,

    /// Move Directory 64-bit instructions.
    /// Provides support for moving 64-bit directory entries.
    Movdir64b,

    /// Move Directory instructions.
    /// Enables support for moving directory entries.
    Movdiri,

    /// Move Rows instructions.
    /// Provides support for moving rows in matrix operations.
    Movrs,

    /// Monitor Wait Extended instructions.
    /// Enables support for extended monitor wait operations.
    Mwaitx,

    /// PCLMULQDQ (Carry-Less Multiplication Quadword) instructions.
    /// Provides support for carry-less multiplication operations.
    Pclmul,

    /// Protection Key instructions.
    /// Provides support for memory protection key operations.
    Pku,

    /// Population Count instructions.
    /// Enables hardware acceleration for population count operations.
    Popcnt,

    /// Prefetch Instructions.
    /// Provides support for prefetching data into cache.
    Prefetchi,

    /// Prefetch Write instructions.
    /// Enables support for prefetching data for write operations.
    Prfchw,

    /// PT Write instructions.
    /// Provides support for writing to processor trace buffers.
    Ptwrite,

    /// RAOINT (Remote Atomic Operations on Integers) instructions.
    /// Enables support for non-blocking atomic operations on integer operands across memory fabrics.
    Raoint,

    /// Read Processor ID instructions.
    /// Provides support for reading processor IDs.
    Rdpid,

    /// Read Processor Unique instructions.
    /// Enables support for reading unique processor identifiers.
    Rdpru,

    /// Random Number Generator instructions.
    /// Provides support for generating random numbers.
    Rdrnd,

    /// Random Seed instructions.
    /// Enables support for generating random seeds.
    Rdseed,

    /// Restricted Transactional Memory instructions.
    /// Enables support for transactional memory operations.
    Rtm,

    /// Save AH Register instructions.
    /// Provides support for saving AH register values.
    Sahf,

    /// Serialize instructions.
    /// Enables support for instruction execution serialization without side effects.
    Serialize,

    /// Software Guard Extensions.
    /// Provides support for secure enclaves using SGX.
    Sgx,

    /// Secure Hash Algorithm instructions.
    /// Enables support for SHA cryptographic hash operations.
    Sha,

    /// Secure Hash Algorithm 512 instructions.
    /// Provides support for SHA-512 cryptographic hash operations.
    Sha512,

    /// SM4 instructions.
    /// Enables support for SM4 block cipher cryptographic operations.
    Sm4,

    /// SSE (Streaming SIMD Extensions) instructions.
    /// Provides support for SIMD operations using SSE.
    Sse,

    /// SSE2 (Streaming SIMD Extensions 2) instructions.
    /// Extends SSE with support for additional SIMD operations.
    Sse2,

    /// SSE3 (Streaming SIMD Extensions 3) instructions.
    /// Provides support for additional SIMD operations in SSE3.
    Sse3,

    /// SSE4.1 (Streaming SIMD Extensions 4.1) instructions.
    /// Provides support for additional SIMD operations in SSE4.1.
    Sse4_1,

    /// SSE4.2 (Streaming SIMD Extensions 4.2) instructions.
    /// Extends SSE4.1 with support for additional SIMD operations.
    Sse4_2,

    /// AMD SSE4A (Streaming SIMD Extensions 4A) instructions.
    /// Provides support for AMD-specific SIMD operations.
    Sse4a,

    /// SSSE3 (Supplemental Streaming SIMD Extensions 3) instructions.
    /// Extends SSE3 with support for additional SIMD operations.
    Ssse3,

    /// AMD TBM (Trailing Bit Manipulation) instructions.
    /// Provides support for AMD-specific bit manipulation operations.
    Tbm,

    /// TSXLDTRK (Transactional Synchronization Extensions Load Tracking) instructions.
    /// Enables support for load tracking in transactional synchronization.
    Tsxldtrk,

    /// User Interrupt instructions.
    /// Provides support for user interrupt operations.
    Uintr,

    /// User Mode MSR instructions.
    /// Enables support for user mode MSR access operations.
    Usermsr,

    /// VAES (Vector AES) instructions.
    /// Provides support for vectorized AES operations.
    Vaes,

    /// VPCLMULQDQ (Vector Carry-Less Multiplication Quadword) instructions.
    /// Enables support for vectorized carry-less multiplication operations.
    Vpclmulqdq,

    /// VZEROUPPER (Zero Upper Bits of YMM and ZMM Registers) instructions.
    /// Instructs the compiler to emit a `vzeroupper` instruction before a transfer of control flow out of the function to minimize the AVX to SSE transition penalty as well as remove unnecessary zeroupper intrinsics.
    Vzeroupper,

    /// Wait Package instructions.
    /// Enables support for timed monitor wait package operations (`UMONITOR`, `UMWAIT`, `TPAUSE`).
    Waitpkg,

    /// AES Key Locker instructions.
    /// Enables support for AES key locker operations.
    Aeskl,

    /// Wide Key Locker instructions.
    /// Enables support for wide key locker operations.
    Widekl,

    /// AMD XOP (eXtended Operations) instructions.
    /// Provides support for AMD-specific extended operations.
    Xop,

    /// XSAVE (Save Processor State) instructions.
    /// Enables support for saving processor state.
    Xsave,

    /// XSAVEC (Save Processor State with Compaction) instructions.
    /// Provides support for saving processor state with compaction.
    Xsavec,

    /// XSAVEOPT (Optimized Save Processor State) instructions.
    /// Enables support for optimized saving of processor state.
    Xsaveopt,

    /// XSAVES (Save Processor State Supervisor) instructions.
    /// Provides support for saving processor state for supervisor mode.
    Xsaves,
}

impl fmt::Display for X64ISA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl X64ISA {
    pub fn as_str(&self) -> &'static str {
        match self {
            X64ISA::None => "none",
            X64ISA::Adx => "adx",
            X64ISA::Aes => "aes",
            X64ISA::AmxAvx512 => "amx-avx512",
            X64ISA::AmxBf16 => "amx-bf16",
            X64ISA::AmxComplex => "amx-complex",
            X64ISA::AmxFp16 => "amx-fp16",
            X64ISA::AmxFp8 => "amx-fp8",
            X64ISA::AmxInt8 => "amx-int8",
            X64ISA::AmxMovrs => "amx-movrs",
            X64ISA::AmxTf32 => "amx-tf32",
            X64ISA::AmxTile => "amx-tile",
            X64ISA::Apxf => "apxf",
            X64ISA::Avx => "avx",
            X64ISA::Avx10_1 => "avx10.1",
            X64ISA::Avx10_2 => "avx10.2",
            X64ISA::Avx2 => "avx2",
            X64ISA::Avx512bf16 => "avx512bf16",
            X64ISA::Avx512bitalg => "avx512bitalg",
            X64ISA::Avx512bw => "avx512bw",
            X64ISA::Avx512bmm => "avx512bmm",
            X64ISA::Avx512cd => "avx512cd",
            X64ISA::Avx512dq => "avx512dq",
            X64ISA::Avx512f => "avx512f",
            X64ISA::Avx512fp16 => "avx512fp16",
            X64ISA::Avx512ifma => "avx512ifma",
            X64ISA::Avx512vbmi => "avx512vbmi",
            X64ISA::Avx512vbmi2 => "avx512vbmi2",
            X64ISA::Avx512vl => "avx512vl",
            X64ISA::Avx512vnni => "avx512vnni",
            X64ISA::Avx512vp2intersect => "avx512vp2intersect",
            X64ISA::Avx512vpopcntdq => "avx512vpopcntdq",
            X64ISA::Avxifma => "avxifma",
            X64ISA::Avxneconvert => "avxneconvert",
            X64ISA::Avxvnni => "avxvnni",
            X64ISA::Avxvnniint16 => "avxvnniint16",
            X64ISA::Avxvnniint8 => "avxvnniint8",
            X64ISA::Bmi => "bmi",
            X64ISA::Bmi2 => "bmi2",
            X64ISA::Cldemote => "cldemote",
            X64ISA::Clflushopt => "clflushopt",
            X64ISA::Clwb => "clwb",
            X64ISA::Clzero => "clzero",
            X64ISA::Cmpccxadd => "cmpccxadd",
            X64ISA::Crc32 => "crc32",
            X64ISA::Cx16 => "cx16",
            X64ISA::Enqcmd => "enqcmd",
            X64ISA::F16c => "f16c",
            X64ISA::Fma => "fma",
            X64ISA::Fma4 => "fma4",
            X64ISA::Fsgsbase => "fsgsbase",
            X64ISA::Fxsr => "fxsr",
            X64ISA::Gfni => "gfni",
            X64ISA::Kl => "kl",
            X64ISA::Lwp => "lwp",
            X64ISA::Lzcnt => "lzcnt",
            X64ISA::Movbe => "movbe",
            X64ISA::Movdir64b => "movdir64b",
            X64ISA::Movdiri => "movdiri",
            X64ISA::Movrs => "movrs",
            X64ISA::Mwaitx => "mwaitx",
            X64ISA::Pclmul => "pclmul",
            X64ISA::Pku => "pku",
            X64ISA::Popcnt => "popcnt",
            X64ISA::Prefetchi => "prefetchi",
            X64ISA::Prfchw => "prfchw",
            X64ISA::Ptwrite => "ptwrite",
            X64ISA::Raoint => "raoint",
            X64ISA::Rdpid => "rdpid",
            X64ISA::Rdpru => "rdpru",
            X64ISA::Rdrnd => "rdrnd",
            X64ISA::Rdseed => "rdseed",
            X64ISA::Rtm => "rtm",
            X64ISA::Sahf => "sahf",
            X64ISA::Serialize => "serialize",
            X64ISA::Sgx => "sgx",
            X64ISA::Sha => "sha",
            X64ISA::Sha512 => "sha512",
            X64ISA::Sm4 => "sm4",
            X64ISA::Sse => "sse",
            X64ISA::Sse2 => "sse2",
            X64ISA::Sse3 => "sse3",
            X64ISA::Sse4_1 => "sse4.1",
            X64ISA::Sse4_2 => "sse4.2",
            X64ISA::Sse4a => "sse4a",
            X64ISA::Ssse3 => "ssse3",
            X64ISA::Tbm => "tbm",
            X64ISA::Tsxldtrk => "tsxldtrk",
            X64ISA::Uintr => "uintr",
            X64ISA::Usermsr => "usermsr",
            X64ISA::Vaes => "vaes",
            X64ISA::Vpclmulqdq => "vpclmulqdq",
            X64ISA::Vzeroupper => "vzeroupper",
            X64ISA::Waitpkg => "waitpkg",
            X64ISA::Aeskl => "",
            X64ISA::Widekl => "widekl",
            X64ISA::Xop => "xop",
            X64ISA::Xsave => "xsave",
            X64ISA::Xsavec => "xsavec",
            X64ISA::Xsaveopt => "xsaveopt",
            X64ISA::Xsaves => "xsaves",
        }
    }

    pub fn all() -> &'static [X64ISA] {
        &[
            X64ISA::Adx,
            X64ISA::Aes,
            X64ISA::AmxAvx512,
            X64ISA::AmxBf16,
            X64ISA::AmxComplex,
            X64ISA::AmxFp16,
            X64ISA::AmxFp8,
            X64ISA::AmxInt8,
            X64ISA::AmxMovrs,
            X64ISA::AmxTf32,
            X64ISA::AmxTile,
            X64ISA::Apxf,
            X64ISA::Avx,
            X64ISA::Avx10_1,
            X64ISA::Avx10_2,
            X64ISA::Avx2,
            X64ISA::Avx512bf16,
            X64ISA::Avx512bitalg,
            X64ISA::Avx512bw,
            X64ISA::Avx512bmm,
            X64ISA::Avx512cd,
            X64ISA::Avx512dq,
            X64ISA::Avx512f,
            X64ISA::Avx512fp16,
            X64ISA::Avx512ifma,
            X64ISA::Avx512vbmi,
            X64ISA::Avx512vbmi2,
            X64ISA::Avx512vl,
            X64ISA::Avx512vnni,
            X64ISA::Avx512vp2intersect,
            X64ISA::Avx512vpopcntdq,
            X64ISA::Avxifma,
            X64ISA::Avxneconvert,
            X64ISA::Avxvnni,
            X64ISA::Avxvnniint16,
            X64ISA::Avxvnniint8,
            X64ISA::Bmi,
            X64ISA::Bmi2,
            X64ISA::Cldemote,
            X64ISA::Clflushopt,
            X64ISA::Clwb,
            X64ISA::Clzero,
            X64ISA::Cmpccxadd,
            X64ISA::Crc32,
            X64ISA::Cx16,
            X64ISA::Enqcmd,
            X64ISA::F16c,
            X64ISA::Fma,
            X64ISA::Fma4,
            X64ISA::Fsgsbase,
            X64ISA::Fxsr,
            X64ISA::Gfni,
            X64ISA::Kl,
            X64ISA::Lwp,
            X64ISA::Lzcnt,
            X64ISA::Movbe,
            X64ISA::Movdir64b,
            X64ISA::Movdiri,
            X64ISA::Movrs,
            X64ISA::Mwaitx,
            X64ISA::Pclmul,
            X64ISA::Pku,
            X64ISA::Popcnt,
            X64ISA::Prefetchi,
            X64ISA::Prfchw,
            X64ISA::Ptwrite,
            X64ISA::Raoint,
            X64ISA::Rdpid,
            X64ISA::Rdpru,
            X64ISA::Rdrnd,
            X64ISA::Rdseed,
            X64ISA::Rtm,
            X64ISA::Sahf,
            X64ISA::Serialize,
            X64ISA::Sgx,
            X64ISA::Sha,
            X64ISA::Sha512,
            X64ISA::Sm4,
            X64ISA::Sse,
            X64ISA::Sse2,
            X64ISA::Sse3,
            X64ISA::Sse4_1,
            X64ISA::Sse4_2,
            X64ISA::Sse4a,
            X64ISA::Ssse3,
            X64ISA::Tbm,
            X64ISA::Tsxldtrk,
            X64ISA::Uintr,
            X64ISA::Usermsr,
            X64ISA::Vaes,
            X64ISA::Vpclmulqdq,
            X64ISA::Vzeroupper,
            X64ISA::Waitpkg,
            X64ISA::Aeskl,
            X64ISA::Widekl,
            X64ISA::Xop,
            X64ISA::Xsave,
            X64ISA::Xsavec,
            X64ISA::Xsaveopt,
            X64ISA::Xsaves,
        ]
    }
}

impl FromStr for X64ISA {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.to_ascii_lowercase().replace(['-', '.'], "_");
        match norm.as_str() {
            "none" => Ok(X64ISA::None),
            "adx" => Ok(X64ISA::Adx),
            "aes" => Ok(X64ISA::Aes),
            "amx_avx512" => Ok(X64ISA::AmxAvx512),
            "amx_bf16" => Ok(X64ISA::AmxBf16),
            "amx_complex" => Ok(X64ISA::AmxComplex),
            "amx_fp16" => Ok(X64ISA::AmxFp16),
            "amx_fp8" => Ok(X64ISA::AmxFp8),
            "amx_int8" => Ok(X64ISA::AmxInt8),
            "amx_movrs" => Ok(X64ISA::AmxMovrs),
            "amx_tf32" => Ok(X64ISA::AmxTf32),
            "amx_tile" => Ok(X64ISA::AmxTile),
            "apxf" | "apx_f" => Ok(X64ISA::Apxf),
            "avx" => Ok(X64ISA::Avx),
            "avx10_1" | "avx10_1_512" | "avx10_1_256" => Ok(X64ISA::Avx10_1),
            "avx10_2" | "avx10_2_512" | "avx10_2_256" => Ok(X64ISA::Avx10_2),
            "avx2" => Ok(X64ISA::Avx2),
            "avx512bf16" => Ok(X64ISA::Avx512bf16),
            "avx512bitalg" => Ok(X64ISA::Avx512bitalg),
            "avx512bw" => Ok(X64ISA::Avx512bw),
            "avx512bmm" => Ok(X64ISA::Avx512bmm),
            "avx512cd" => Ok(X64ISA::Avx512cd),
            "avx512dq" => Ok(X64ISA::Avx512dq),
            "avx512f" => Ok(X64ISA::Avx512f),
            "avx512fp16" => Ok(X64ISA::Avx512fp16),
            "avx512ifma" => Ok(X64ISA::Avx512ifma),
            "avx512vbmi" => Ok(X64ISA::Avx512vbmi),
            "avx512vbmi2" => Ok(X64ISA::Avx512vbmi2),
            "avx512vl" => Ok(X64ISA::Avx512vl),
            "avx512vnni" => Ok(X64ISA::Avx512vnni),
            "avx512vp2intersect" => Ok(X64ISA::Avx512vp2intersect),
            "avx512vpopcntdq" => Ok(X64ISA::Avx512vpopcntdq),
            "avxifma" => Ok(X64ISA::Avxifma),
            "avxneconvert" | "avx_ne_convert" => Ok(X64ISA::Avxneconvert),
            "avxvnni" | "avx_vnni" => Ok(X64ISA::Avxvnni),
            "avxvnniint16" | "avx_vnni_int16" => Ok(X64ISA::Avxvnniint16),
            "avxvnniint8" | "avx_vnni_int8" => Ok(X64ISA::Avxvnniint8),
            "bmi" | "bmi1" => Ok(X64ISA::Bmi),
            "bmi2" => Ok(X64ISA::Bmi2),
            "cldemote" => Ok(X64ISA::Cldemote),
            "clflushopt" => Ok(X64ISA::Clflushopt),
            "clwb" => Ok(X64ISA::Clwb),
            "clzero" => Ok(X64ISA::Clzero),
            "cmpccxadd" => Ok(X64ISA::Cmpccxadd),
            "crc32" => Ok(X64ISA::Crc32),
            "cx16" => Ok(X64ISA::Cx16),
            "enqcmd" => Ok(X64ISA::Enqcmd),
            "f16c" => Ok(X64ISA::F16c),
            "fma" | "fma3" => Ok(X64ISA::Fma),
            "fma4" => Ok(X64ISA::Fma4),
            "fsgsbase" => Ok(X64ISA::Fsgsbase),
            "fxsr" => Ok(X64ISA::Fxsr),
            "gfni" => Ok(X64ISA::Gfni),
            "kl" => Ok(X64ISA::Kl),
            "lwp" => Ok(X64ISA::Lwp),
            "lzcnt" | "abm" => Ok(X64ISA::Lzcnt),
            "movbe" => Ok(X64ISA::Movbe),
            "movdir64b" => Ok(X64ISA::Movdir64b),
            "movdiri" => Ok(X64ISA::Movdiri),
            "movrs" => Ok(X64ISA::Movrs),
            "mwaitx" => Ok(X64ISA::Mwaitx),
            "pclmul" | "pclmulqdq" => Ok(X64ISA::Pclmul),
            "pku" => Ok(X64ISA::Pku),
            "popcnt" => Ok(X64ISA::Popcnt),
            "prefetchi" => Ok(X64ISA::Prefetchi),
            "prfchw" | "prefetchw" => Ok(X64ISA::Prfchw),
            "ptwrite" => Ok(X64ISA::Ptwrite),
            "raoint" => Ok(X64ISA::Raoint),
            "rdpid" => Ok(X64ISA::Rdpid),
            "rdpru" => Ok(X64ISA::Rdpru),
            "rdrnd" | "rdrand" => Ok(X64ISA::Rdrnd),
            "rdseed" => Ok(X64ISA::Rdseed),
            "rtm" => Ok(X64ISA::Rtm),
            "sahf" => Ok(X64ISA::Sahf),
            "serialize" => Ok(X64ISA::Serialize),
            "sgx" => Ok(X64ISA::Sgx),
            "sha" => Ok(X64ISA::Sha),
            "sha512" => Ok(X64ISA::Sha512),
            "sm4" => Ok(X64ISA::Sm4),
            "sse" => Ok(X64ISA::Sse),
            "sse2" => Ok(X64ISA::Sse2),
            "sse3" => Ok(X64ISA::Sse3),
            "sse4_1" | "sse41" => Ok(X64ISA::Sse4_1),
            "sse4_2" | "sse42" => Ok(X64ISA::Sse4_2),
            "sse4a" => Ok(X64ISA::Sse4a),
            "ssse3" => Ok(X64ISA::Ssse3),
            "tbm" => Ok(X64ISA::Tbm),
            "tsxldtrk" => Ok(X64ISA::Tsxldtrk),
            "uintr" => Ok(X64ISA::Uintr),
            "usermsr" | "user_msr" => Ok(X64ISA::Usermsr),
            "vaes" => Ok(X64ISA::Vaes),
            "vpclmulqdq" => Ok(X64ISA::Vpclmulqdq),
            "vzeroupper" => Ok(X64ISA::Vzeroupper),
            "waitpkg" => Ok(X64ISA::Waitpkg),
            "aeskl" => Ok(X64ISA::Aeskl),
            "widekl" => Ok(X64ISA::Widekl),
            "xop" => Ok(X64ISA::Xop),
            "xsave" => Ok(X64ISA::Xsave),
            "xsavec" => Ok(X64ISA::Xsavec),
            "xsaveopt" => Ok(X64ISA::Xsaveopt),
            "xsaves" => Ok(X64ISA::Xsaves),
            other => Err(format!("Unknown x64 ISA extension: {}", other)),
        }
    }
}

