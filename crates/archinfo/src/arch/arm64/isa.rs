// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/isa.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Enum representing various Arm64 ISA (Instruction Set Architecture) extensions and architecture profiles.
///
/// These extensions define specific hardware capabilities or features that can be utilized during compilation.
/// Each variant corresponds to a specific feature or instruction set extension, enabling fine-grained control
/// over the compilation process for Arm64 targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm64ISA {
    /// No specific ISA extension.
    None,

    /// Advanced Encryption Standard (AES) instructions.
    /// Enables support for `FEAT_AES` and `FEAT_PMULL`, providing cryptographic capabilities.
    Aes,

    /// Bfloat16 (Brain Floating Point) support for machine learning workloads.
    /// Enables the `FEAT_BF16` extension.
    Bf16,

    /// Branch Record Buffer Extensions (BRBE) for branch recording and profiling.
    /// Enables the `FEAT_BRBE` extension.
    Brbe,

    /// Branch Target Identification (BTI) for control-flow integrity and security enhancements.
    /// Enables the `FEAT_BTI` extension.
    Bti,

    /// Branch Target Identification Exception support (`FEAT_BTIE`).
    Btie,

    /// Compare and Branch instructions.
    /// Enables the `FEAT_CMPBR` extension, part of the Armv9.6-A base profile.
    Cmpbr,

    /// Cache Protection and Allocation (CPA) instructions.
    /// Enables the `FEAT_CPA` extension, introduced in Armv9.5-A.
    Cpa,

    /// Cyclic Redundancy Check (CRC) instructions.
    /// Enables the `FEAT_CRC32` extension, introduced in Armv8.0-A.
    Crc,

    /// Cryptographic instructions for secure data processing (AES, PMULL, SHA-1, SHA-256).
    /// Enables the legacy `FEAT_Crypto` architecture extension group.
    Crypto,

    /// Common Short Sequence Compression instructions.
    /// Enables the `FEAT_CSSC` extension.
    Cssc,

    /// 128-bit system architecture and memory processing instructions.
    /// Enables `FEAT_D128`, `FEAT_LVA3`, `FEAT_SYSREG128`, and `FEAT_SYSINSTR128` extensions.
    D128,

    /// Data Independent Timing (DIT) instructions for constant-time cryptographic operations and timing attack mitigation.
    /// Enables the `FEAT_DIT` extension, introduced in Armv8.4-A.
    Dit,

    /// Dot Product instructions for Advanced SIMD vector operations.
    /// Enables the `FEAT_DotProd` extension.
    Dotprod,

    /// Half-precision floating-point to single-precision floating-point dot product operations (`FEAT_F16F32DOT`).
    F16f32dot,

    /// Half-precision to single-precision floating-point matrix multiplication instructions (`FEAT_F16F32MM`).
    F16f32mm,

    /// Half-precision floating-point matrix multiplication instructions (`FEAT_F16MM`).
    F16mm,

    /// 32-bit floating-point matrix multiplication instructions.
    /// Enables the `FEAT_F32MM` extension.
    F32mm,

    /// 64-bit floating-point matrix multiplication instructions.
    /// Enables the `FEAT_F64MM` extension.
    F64mm,

    /// 8-bit to 16-bit floating-point matrix multiplication instructions.
    /// Enables the `FEAT_F8F16MM` extension, introduced in Armv9.6-A.
    F8f16mm,

    /// 8-bit to 32-bit floating-point matrix multiplication instructions.
    /// Enables the `FEAT_F8F32MM` extension, introduced in Armv9.6-A.
    F8f32mm,

    /// Floating-point minimum and maximum operations with IEEE 754-2019 semantics.
    /// Enables the `FEAT_FAMINMAX` extension.
    Faminmax,

    /// Floating-point Complex Number Arithmetic (FCMA) instructions.
    /// Enables the `FEAT_FCMA` extension, introduced in Armv8.3-A.
    Fcma,

    /// Flag Manipulation instructions.
    /// Enables the `FEAT_FlagM` extension, introduced in Armv8.4-A.
    Flagm,

    /// Floating-point scalar and vector instructions.
    /// Enables the `FEAT_FP` extension, introduced in Armv8.0-A.
    Fp,

    /// Half-precision (16-bit) floating-point data processing support.
    /// Enables the `FEAT_FP16` extension.
    Fp16,

    /// Half-precision floating-point fused multiply-accumulate long instructions.
    /// Enables the `FEAT_FHM` extension.
    Fp16fml,

    /// 8-bit floating-point format support (E4M3 and E5M2).
    /// Enables the `FEAT_FP8` extension.
    Fp8,

    /// 8-bit floating-point dot product with 2-way accumulators.
    /// Enables the `FEAT_FP8DOT2` extension.
    Fp8dot2,

    /// 8-bit floating-point dot product with 4-way accumulators.
    /// Enables the `FEAT_FP8DOT4` extension.
    Fp8dot4,

    /// 8-bit floating-point fused multiply-add instructions.
    /// Enables the `FEAT_FP8FMA` extension, introduced in Armv9.5-A.
    Fp8fma,

    /// Floating-point rounding and conversion instructions to convert between various FP formats.
    /// Enables the `FEAT_FPRCVT` extension, introduced in Armv9.6-A.
    Fprcvt,

    /// Guarded Control Stack Instruction Execution support (`FEAT_GCIE`).
    Gcie,

    /// Guarded Control Stack (GCS) instructions for hardware-enforced shadow stacks.
    /// Enables the `FEAT_GCS` extension, introduced in Armv9.4-A.
    Gcs,

    /// Hinted Branch Condition (HBC) instructions.
    /// Enables the `FEAT_HBC` extension, introduced in Armv8.8-A.
    Hbc,

    /// Extended A64 hint instruction space (`FEAT_HINT`), introduced under the Armv9.0-A profile.
    Hinte,

    /// 8-bit integer matrix multiplication instructions.
    /// Enables the `FEAT_I8MM` extension.
    I8mm,

    /// Instructions for If-Then-Else (ITE) conditional select operations.
    /// Enables the `FEAT_ITE` extension, introduced in Armv9.4-A.
    Ite,

    /// JavaScript conversion instructions for double-precision float to signed 32-bit integer rounding (`FJCVTZS`).
    /// Enables the `FEAT_JSCVT` extension, introduced in Armv8.3-A.
    Jscvt,

    /// 64-byte atomic Load-Store instructions.
    /// Enables the `FEAT_LS64`, `FEAT_LS64_V`, and `FEAT_LS64_ACCDATA` extensions.
    Ls64,

    /// Load-Store Check Pointers instructions (`FEAT_LSCP`).
    Lscp,

    /// Large System Extensions (LSE) for hardware atomic memory instructions.
    /// Enables the `FEAT_LSE` extension, introduced in Armv8.1-A.
    Lse,

    /// 128-bit Large System Extensions (LSE) for 128-bit atomic operations.
    /// Enables the `FEAT_LSE128` extension, introduced in Armv9.4-A.
    Lse128,

    /// Load-Store Fast Exception handling instructions.
    /// Enables the `FEAT_LSFE` extension, introduced in Armv9.6-A.
    Lsfe,

    /// Load-Store Unprivileged Instructions.
    /// Enables the `FEAT_LSUI` extension, introduced in Armv9.6-A.
    Lsui,

    /// Lookup Table (LUT) instructions for byte and nibble lookups across registers.
    /// Enables the `FEAT_LUT` extension.
    Lut,

    /// Memory Tagging Extension (MTE) for detecting spatial and temporal memory safety bugs.
    /// Enables the `FEAT_MTE` and `FEAT_MTE2` extensions.
    Memtag,

    /// Standardized Memory Operations (MOPS) for high-performance `memcpy`, `memmove`, and `memset`.
    /// Enables the `FEAT_MOPS` extension, introduced in Armv8.8-A.
    Mops,

    /// Memory Operations Go (MOPS_GO) variant for garbage-collected runtime copy optimizations.
    MopsGo,

    /// Memory Tagging Extension Tag Check Fault enhancements (`FEAT_MTETC`).
    Mtetc,

    /// Ordered Cache Coherency Memory Operations (OCCMO).
    /// Enables the `FEAT_OCCMO` extension, introduced in Armv9.6-A.
    Occmo,

    /// Pointer Authentication (PAuth) instructions to mitigate ROP and JOP exploits.
    /// Enables the `FEAT_PAuth` extension, introduced in Armv8.3-A.
    Pauth,

    /// Pointer Authentication with Link Register (PAUTH_LR).
    /// Enables the `FEAT_PAuth_LR` extension, introduced in Armv9.5-A.
    PauthLr,

    /// Performance Monitoring Unit Version 3 (PMUv3) architectural counters.
    /// Enables the `FEAT_PMUv3` extension, introduced in Armv8.0-A.
    Pmuv3,

    /// Permission Overlay Extension version 2 (`FEAT_POE2`).
    Poe2,

    /// Population Count to Pair of Registers instructions.
    /// Enables the `FEAT_PoPS` extension, introduced in Armv9.6-A.
    Pops,

    /// Predicate Result instructions to control speculative execution barriers.
    /// Enables the `FEAT_SPECRES` extension, introduced in Armv8.5-A.
    Predres,

    /// Second version of Predicate Result instructions.
    /// Enables the `FEAT_SPECRES2` extension.
    Predres2,

    /// Statistical Profiling Extension (SPE) for detailed instruction execution telemetry.
    /// Enables the `FEAT_SPE` extension.
    Profile,

    /// Reliability, Availability, and Serviceability (RAS) architecture extensions.
    /// Enables the `FEAT_RAS` and `FEAT_RASv1p1` extensions, introduced in Armv8.0-A.
    Ras,

    /// Version 2 of Reliability, Availability, and Serviceability (RAS) instructions.
    /// Enables the `FEAT_RASv2` extension, introduced in Armv8.9-A.
    Rasv2,

    /// Release Consistent Processor Consistency (RCPC) instructions providing acquire semantics.
    /// Enables the `FEAT_LRCPC` extension.
    Rcpc,

    /// Third version of RCPC instructions adding load-acquire PRFM and store-release instructions.
    /// Enables the `FEAT_LRCPC3` extension, introduced in Armv8.9-A.
    Rcpc3,

    /// Rounding Double Multiply Accumulate/Subtract instructions.
    /// Enables the `FEAT_RDM` extension, introduced in Armv8.1-A.
    Rdm,

    /// True Random Number Generator (TRNG) instructions.
    /// Enables the `FEAT_RNG` extension.
    Rng,

    /// Speculation Barrier (SB) instruction to restrict speculation execution.
    /// Enables the `FEAT_SB` extension, introduced in Armv8.5-A.
    Sb,

    /// SHA-1 and SHA-256 cryptographic hash acceleration instructions.
    /// Enables the `FEAT_SHA1` and `FEAT_SHA256` extensions.
    Sha2,

    /// SHA-3 and SHA-512 cryptographic hash acceleration instructions.
    /// Enables the `FEAT_SHA3` and `FEAT_SHA512` extensions.
    Sha3,

    /// Advanced SIMD (NEON) vector instruction set.
    /// Enables the `FEAT_AdvSIMD` extension.
    Simd,

    /// SM3 and SM4 cryptographic cipher acceleration instructions.
    /// Enables the `FEAT_SM3` and `FEAT_SM4` extensions.
    Sm4,

    /// Scalable Matrix Extension (SME) foundation architecture.
    /// Enables the `FEAT_SME` extension.
    Sme,

    /// SME B16B16 matrix multiplication instructions.
    /// Enables the `FEAT_SME_B16B16` extension.
    SmeB16b16,

    /// SME F16F16 matrix multiplication instructions.
    /// Enables the `FEAT_SME_F16F16` extension.
    SmeF16f16,

    /// SME F64F64 matrix multiplication instructions.
    /// Enables the `FEAT_SME_F64F64` extension.
    SmeF64f64,

    /// SME F8F16 matrix multiplication instructions.
    /// Enables the `FEAT_SME_F8F16` extension.
    SmeF8f16,

    /// SME F8F32 matrix multiplication instructions.
    /// Enables the `FEAT_SME_F8F32` extension.
    SmeF8f32,

    /// Full 64-bit SVE execution mode enablement in Streaming SVE state (`FEAT_SME_FA64`).
    /// Enables the `FEAT_SME_FA64` extension.
    SmeFa64,

    /// SME I16I64 matrix multiplication instructions.
    /// Enables the `FEAT_SME_I16I64` extension.
    SmeI16i64,

    /// SME Lookup Table Version 2 instructions.
    /// Enables the `FEAT_SME_LUTv2` extension.
    SmeLutv2,

    /// SME 4-way Outer Product Matrix Operations instructions.
    /// Enables the `FEAT_SME_MOP4` extension.
    SmeMop4,

    /// SME Transposed Matrix Operation instructions.
    /// Enables the `FEAT_SME_TMOP` extension.
    SmeTmop,

    /// Second version of the Scalable Matrix Extension (SME2).
    /// Enables the `FEAT_SME2` extension.
    Sme2,

    /// SME 2.1 instructions and vector multi-register support.
    /// Enables the `FEAT_SME2p1` extension.
    Sme2p1,

    /// SME 2.2 instructions.
    /// Enables the `FEAT_SME2p2` extension, introduced in Armv9.6-A.
    Sme2p2,

    /// SME 2.3 instruction extensions (`FEAT_SME2p3`).
    Sme2p3,

    /// Speculative Store Bypass Safe (SSBS) control instructions.
    /// Enables the `FEAT_SSBS` and `FEAT_SSBS2` extensions.
    Ssbs,

    /// Streaming Scalable Vector Extension (SSVE) AES instructions.
    /// Enables the `FEAT_SSVE_AES` extension, introduced in Armv9.6-A.
    SsveAes,

    /// Streaming Scalable Vector Extension (SSVE) Bit Permutation instructions.
    /// Enables the `FEAT_SSVE_BitPerm` extension, introduced in Armv9.6-A.
    SsveBitperm,

    /// Streaming Scalable Vector Extension (SSVE) Floating-Point Exponential Approximation instructions.
    /// Enables the `FEAT_SSVE_FEXPA` extension, introduced in Armv9.6-A.
    SsveFexpa,

    /// Streaming Scalable Vector Extension (SSVE) FP8 Dot Product with 2 accumulators.
    /// Enables the `FEAT_SSVE_FP8DOT2` extension.
    SsveFp8dot2,

    /// Streaming Scalable Vector Extension (SSVE) FP8 Dot Product with 4 accumulators.
    /// Enables the `FEAT_SSVE_FP8DOT4` extension.
    SsveFp8dot4,

    /// Streaming Scalable Vector Extension (SSVE) FP8 Fused Multiply-Add instructions.
    /// Enables the `FEAT_SSVE_FP8FMA` extension.
    SsveFp8fma,

    /// Scalable Vector Extension (SVE) foundational architecture.
    /// Enables the `FEAT_SVE` extension.
    Sve,

    /// Scalable Vector Extension (SVE) AES cryptographic instructions.
    /// Enables the `FEAT_SVE_AES` and `FEAT_SVE_PMULL128` extensions.
    SveAes,

    /// Version 2 of SVE AES instructions.
    /// Enables the `FEAT_SVE_AES2` extension, introduced in Armv9.6-A.
    SveAes2,

    /// Scalable Vector Extension (SVE) B16B16 matrix multiplication instructions.
    /// Enables the `FEAT_SVE_B16B16` extension.
    SveB16b16,

    /// SVE Bfloat16 matrix multiplication instructions (`FEAT_SVE_B16MM`).
    SveB16mm,

    /// Scalable Vector Extension (SVE) BFScale instructions.
    /// Enables the `FEAT_SVE_BFSCALE` extension, introduced in Armv9.6-A.
    SveBfscale,

    /// Scalable Vector Extension (SVE) Bit Permutation instructions.
    /// Enables the `FEAT_SVE_BitPerm` extension.
    SveBitperm,

    /// Scalable Vector Extension (SVE) F16F32 matrix multiplication instructions.
    /// Enables the `FEAT_SVE_F16F32MM` extension, introduced in Armv9.6-A.
    SveF16f32mm,

    /// Scalable Vector Extension (SVE) SHA-3 instructions.
    /// Enables the `FEAT_SVE_SHA3` extension, introduced in Armv9.6-A.
    SveSha3,

    /// Scalable Vector Extension (SVE) SM4 instructions.
    /// Enables the `FEAT_SVE_SM4` extension, introduced in Armv9.6-A.
    SveSm4,

    /// Scalable Vector Extension 2 (SVE2) base instruction set.
    /// Enables the `FEAT_SVE2` extension.
    Sve2,

    /// Scalable Vector Extension 2 (SVE2) AES cryptographic instructions.
    /// Enables the `FEAT_SVE_AES` extension for SVE2.
    Sve2Aes,

    /// Scalable Vector Extension 2 (SVE2) Bit Permutation instructions.
    /// Enables the `FEAT_SVE_BitPerm` extension for SVE2.
    Sve2Bitperm,

    /// Scalable Vector Extension 2 (SVE2) SHA-3 cryptographic instructions.
    /// Enables the `FEAT_SVE_SHA3` extension for SVE2.
    Sve2Sha3,

    /// Scalable Vector Extension 2 (SVE2) SM4 cryptographic instructions.
    /// Enables the `FEAT_SVE_SM4` extension for SVE2.
    Sve2Sm4,

    /// SVE2 version 2.1 instructions and vector multi-register support.
    /// Enables the `FEAT_SVE2p1` extension.
    Sve2p1,

    /// SVE2 version 2.2 instructions.
    /// Enables the `FEAT_SVE2p2` extension, introduced in Armv9.6-A.
    Sve2p2,

    /// SVE2 version 2.3 instruction extensions (`FEAT_SVE2p3`).
    Sve2p3,

    /// Transaction Evaluation and Verification support (`FEAT_TEV`).
    Tev,

    /// Translation Hardening Extension (THE) for translation table protection.
    /// Enables the `FEAT_THE` extension, introduced in Armv8.9-A.
    The,

    /// Translation Lookaside Buffer Invalidate by Demand instructions (`FEAT_TLBID`).
    Tlbid,

    /// Translation Lookaside Buffer Invalidate by Walk (TLBIW) instructions.
    /// Enables the `FEAT_TLBIW` extension, introduced in Armv9.5-A.
    Tlbiw,

    /// Wait for Event with Timeout (WFxT) instructions.
    /// Enables the `FEAT_WFxT` extension, introduced in Armv8.7-A.
    Wfxt,
}

impl fmt::Display for Arm64ISA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Arm64ISA {
    pub fn as_str(&self) -> &'static str {
        match self {
            Arm64ISA::None => "none",
            Arm64ISA::Aes => "aes",
            Arm64ISA::Bf16 => "bf16",
            Arm64ISA::Brbe => "brbe",
            Arm64ISA::Bti => "bti",
            Arm64ISA::Btie => "btie",
            Arm64ISA::Cmpbr => "cmpbr",
            Arm64ISA::Cpa => "cpa",
            Arm64ISA::Crc => "crc",
            Arm64ISA::Crypto => "crypto",
            Arm64ISA::Cssc => "cssc",
            Arm64ISA::D128 => "d128",
            Arm64ISA::Dit => "dit",
            Arm64ISA::Dotprod => "dotprod",
            Arm64ISA::F16f32dot => "f16f32dot",
            Arm64ISA::F16f32mm => "f16f32mm",
            Arm64ISA::F16mm => "f16mm",
            Arm64ISA::F32mm => "f32mm",
            Arm64ISA::F64mm => "f64mm",
            Arm64ISA::F8f16mm => "f8f16mm",
            Arm64ISA::F8f32mm => "f8f32mm",
            Arm64ISA::Faminmax => "faminmax",
            Arm64ISA::Fcma => "fcma",
            Arm64ISA::Flagm => "flagm",
            Arm64ISA::Fp => "fp",
            Arm64ISA::Fp16 => "fp16",
            Arm64ISA::Fp16fml => "fp16fml",
            Arm64ISA::Fp8 => "fp8",
            Arm64ISA::Fp8dot2 => "fp8dot2",
            Arm64ISA::Fp8dot4 => "fp8dot4",
            Arm64ISA::Fp8fma => "fp8fma",
            Arm64ISA::Fprcvt => "fprcvt",
            Arm64ISA::Gcie => "gcie",
            Arm64ISA::Gcs => "gcs",
            Arm64ISA::Hbc => "hbc",
            Arm64ISA::Hinte => "hinte",
            Arm64ISA::I8mm => "i8mm",
            Arm64ISA::Ite => "ite",
            Arm64ISA::Jscvt => "jscvt",
            Arm64ISA::Ls64 => "ls64",
            Arm64ISA::Lscp => "lscp",
            Arm64ISA::Lse => "lse",
            Arm64ISA::Lse128 => "lse128",
            Arm64ISA::Lsfe => "lsfe",
            Arm64ISA::Lsui => "lsui",
            Arm64ISA::Lut => "lut",
            Arm64ISA::Memtag => "memtag",
            Arm64ISA::Mops => "mops",
            Arm64ISA::MopsGo => "mops-go",
            Arm64ISA::Mtetc => "mtetc",
            Arm64ISA::Occmo => "occmo",
            Arm64ISA::Pauth => "pauth",
            Arm64ISA::PauthLr => "pauth-lr",
            Arm64ISA::Pmuv3 => "pmuv3",
            Arm64ISA::Poe2 => "poe2",
            Arm64ISA::Pops => "pops",
            Arm64ISA::Predres => "predres",
            Arm64ISA::Predres2 => "predres2",
            Arm64ISA::Profile => "profile",
            Arm64ISA::Ras => "ras",
            Arm64ISA::Rasv2 => "rasv2",
            Arm64ISA::Rcpc => "rcpc",
            Arm64ISA::Rcpc3 => "rcpc3",
            Arm64ISA::Rdm => "rdm",
            Arm64ISA::Rng => "rng",
            Arm64ISA::Sb => "sb",
            Arm64ISA::Sha2 => "sha2",
            Arm64ISA::Sha3 => "sha3",
            Arm64ISA::Simd => "simd",
            Arm64ISA::Sm4 => "sm4",
            Arm64ISA::Sme => "sme",
            Arm64ISA::SmeB16b16 => "sme-b16b16",
            Arm64ISA::SmeF16f16 => "sme-f16f16",
            Arm64ISA::SmeF64f64 => "sme-f64f64",
            Arm64ISA::SmeF8f16 => "sme-f8f16",
            Arm64ISA::SmeF8f32 => "sme-f8f32",
            Arm64ISA::SmeFa64 => "sme-fa64",
            Arm64ISA::SmeI16i64 => "sme-i16i64",
            Arm64ISA::SmeLutv2 => "sme-lutv2",
            Arm64ISA::SmeMop4 => "sme-mop4",
            Arm64ISA::SmeTmop => "sme-tmop",
            Arm64ISA::Sme2 => "sme2",
            Arm64ISA::Sme2p1 => "sme2p1",
            Arm64ISA::Sme2p2 => "sme2p2",
            Arm64ISA::Sme2p3 => "sme2p3",
            Arm64ISA::Ssbs => "ssbs",
            Arm64ISA::SsveAes => "ssve-aes",
            Arm64ISA::SsveBitperm => "ssve-bitperm",
            Arm64ISA::SsveFexpa => "ssve-fexpa",
            Arm64ISA::SsveFp8dot2 => "ssve-fp8dot2",
            Arm64ISA::SsveFp8dot4 => "ssve-fp8dot4",
            Arm64ISA::SsveFp8fma => "ssve-fp8fma",
            Arm64ISA::Sve => "sve",
            Arm64ISA::SveAes => "sve-aes",
            Arm64ISA::SveAes2 => "sve-aes2",
            Arm64ISA::SveB16b16 => "sve-b16b16",
            Arm64ISA::SveB16mm => "sve-b16mm",
            Arm64ISA::SveBfscale => "sve-bfscale",
            Arm64ISA::SveBitperm => "sve-bitperm",
            Arm64ISA::SveF16f32mm => "sve-f16f32mm",
            Arm64ISA::SveSha3 => "sve-sha3",
            Arm64ISA::SveSm4 => "sve-sm4",
            Arm64ISA::Sve2 => "sve2",
            Arm64ISA::Sve2Aes => "sve2-aes",
            Arm64ISA::Sve2Bitperm => "sve2-bitperm",
            Arm64ISA::Sve2Sha3 => "sve2-sha3",
            Arm64ISA::Sve2Sm4 => "sve2-sm4",
            Arm64ISA::Sve2p1 => "sve2p1",
            Arm64ISA::Sve2p2 => "sve2p2",
            Arm64ISA::Sve2p3 => "sve2p3",
            Arm64ISA::Tev => "tev",
            Arm64ISA::The => "the",
            Arm64ISA::Tlbid => "tlbid",
            Arm64ISA::Tlbiw => "tlbiw",
            Arm64ISA::Wfxt => "wfxt",
        }
    }

    pub fn all() -> &'static [Arm64ISA] {
        &[
            Arm64ISA::Aes,
            Arm64ISA::Bf16,
            Arm64ISA::Brbe,
            Arm64ISA::Bti,
            Arm64ISA::Btie,
            Arm64ISA::Cmpbr,
            Arm64ISA::Cpa,
            Arm64ISA::Crc,
            Arm64ISA::Crypto,
            Arm64ISA::Cssc,
            Arm64ISA::D128,
            Arm64ISA::Dit,
            Arm64ISA::Dotprod,
            Arm64ISA::F16f32dot,
            Arm64ISA::F16f32mm,
            Arm64ISA::F16mm,
            Arm64ISA::F32mm,
            Arm64ISA::F64mm,
            Arm64ISA::F8f16mm,
            Arm64ISA::F8f32mm,
            Arm64ISA::Faminmax,
            Arm64ISA::Fcma,
            Arm64ISA::Flagm,
            Arm64ISA::Fp,
            Arm64ISA::Fp16,
            Arm64ISA::Fp16fml,
            Arm64ISA::Fp8,
            Arm64ISA::Fp8dot2,
            Arm64ISA::Fp8dot4,
            Arm64ISA::Fp8fma,
            Arm64ISA::Fprcvt,
            Arm64ISA::Gcie,
            Arm64ISA::Gcs,
            Arm64ISA::Hbc,
            Arm64ISA::Hinte,
            Arm64ISA::I8mm,
            Arm64ISA::Ite,
            Arm64ISA::Jscvt,
            Arm64ISA::Ls64,
            Arm64ISA::Lscp,
            Arm64ISA::Lse,
            Arm64ISA::Lse128,
            Arm64ISA::Lsfe,
            Arm64ISA::Lsui,
            Arm64ISA::Lut,
            Arm64ISA::Memtag,
            Arm64ISA::Mops,
            Arm64ISA::MopsGo,
            Arm64ISA::Mtetc,
            Arm64ISA::Occmo,
            Arm64ISA::Pauth,
            Arm64ISA::PauthLr,
            Arm64ISA::Pmuv3,
            Arm64ISA::Poe2,
            Arm64ISA::Pops,
            Arm64ISA::Predres,
            Arm64ISA::Predres2,
            Arm64ISA::Profile,
            Arm64ISA::Ras,
            Arm64ISA::Rasv2,
            Arm64ISA::Rcpc,
            Arm64ISA::Rcpc3,
            Arm64ISA::Rdm,
            Arm64ISA::Rng,
            Arm64ISA::Sb,
            Arm64ISA::Sha2,
            Arm64ISA::Sha3,
            Arm64ISA::Simd,
            Arm64ISA::Sm4,
            Arm64ISA::Sme,
            Arm64ISA::SmeB16b16,
            Arm64ISA::SmeF16f16,
            Arm64ISA::SmeF64f64,
            Arm64ISA::SmeF8f16,
            Arm64ISA::SmeF8f32,
            Arm64ISA::SmeFa64,
            Arm64ISA::SmeI16i64,
            Arm64ISA::SmeLutv2,
            Arm64ISA::SmeMop4,
            Arm64ISA::SmeTmop,
            Arm64ISA::Sme2,
            Arm64ISA::Sme2p1,
            Arm64ISA::Sme2p2,
            Arm64ISA::Sme2p3,
            Arm64ISA::Ssbs,
            Arm64ISA::SsveAes,
            Arm64ISA::SsveBitperm,
            Arm64ISA::SsveFexpa,
            Arm64ISA::SsveFp8dot2,
            Arm64ISA::SsveFp8dot4,
            Arm64ISA::SsveFp8fma,
            Arm64ISA::Sve,
            Arm64ISA::SveAes,
            Arm64ISA::SveAes2,
            Arm64ISA::SveB16b16,
            Arm64ISA::SveB16mm,
            Arm64ISA::SveBfscale,
            Arm64ISA::SveBitperm,
            Arm64ISA::SveF16f32mm,
            Arm64ISA::SveSha3,
            Arm64ISA::SveSm4,
            Arm64ISA::Sve2,
            Arm64ISA::Sve2Aes,
            Arm64ISA::Sve2Bitperm,
            Arm64ISA::Sve2Sha3,
            Arm64ISA::Sve2Sm4,
            Arm64ISA::Sve2p1,
            Arm64ISA::Sve2p2,
            Arm64ISA::Sve2p3,
            Arm64ISA::Tev,
            Arm64ISA::The,
            Arm64ISA::Tlbid,
            Arm64ISA::Tlbiw,
            Arm64ISA::Wfxt,
        ]
    }
}

impl FromStr for Arm64ISA {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.to_ascii_lowercase().replace(['_', '.'], "-");
        match norm.as_str() {
            "none" => Ok(Arm64ISA::None),
            "aes" => Ok(Arm64ISA::Aes),
            "bf16" => Ok(Arm64ISA::Bf16),
            "brbe" => Ok(Arm64ISA::Brbe),
            "bti" => Ok(Arm64ISA::Bti),
            "btie" => Ok(Arm64ISA::Btie),
            "cmpbr" => Ok(Arm64ISA::Cmpbr),
            "cpa" => Ok(Arm64ISA::Cpa),
            "crc" | "crc32" => Ok(Arm64ISA::Crc),
            "crypto" => Ok(Arm64ISA::Crypto),
            "cssc" => Ok(Arm64ISA::Cssc),
            "d128" => Ok(Arm64ISA::D128),
            "dit" => Ok(Arm64ISA::Dit),
            "dotprod" => Ok(Arm64ISA::Dotprod),
            "f16f32dot" => Ok(Arm64ISA::F16f32dot),
            "f16f32mm" => Ok(Arm64ISA::F16f32mm),
            "f16mm" => Ok(Arm64ISA::F16mm),
            "f32mm" => Ok(Arm64ISA::F32mm),
            "f64mm" => Ok(Arm64ISA::F64mm),
            "f8f16mm" => Ok(Arm64ISA::F8f16mm),
            "f8f32mm" => Ok(Arm64ISA::F8f32mm),
            "faminmax" => Ok(Arm64ISA::Faminmax),
            "fcma" => Ok(Arm64ISA::Fcma),
            "flagm" => Ok(Arm64ISA::Flagm),
            "fp" => Ok(Arm64ISA::Fp),
            "fp16" => Ok(Arm64ISA::Fp16),
            "fp16fml" | "fhm" => Ok(Arm64ISA::Fp16fml),
            "fp8" => Ok(Arm64ISA::Fp8),
            "fp8dot2" => Ok(Arm64ISA::Fp8dot2),
            "fp8dot4" => Ok(Arm64ISA::Fp8dot4),
            "fp8fma" => Ok(Arm64ISA::Fp8fma),
            "fprcvt" => Ok(Arm64ISA::Fprcvt),
            "gcie" => Ok(Arm64ISA::Gcie),
            "gcs" => Ok(Arm64ISA::Gcs),
            "hbc" => Ok(Arm64ISA::Hbc),
            "hinte" => Ok(Arm64ISA::Hinte),
            "i8mm" => Ok(Arm64ISA::I8mm),
            "ite" => Ok(Arm64ISA::Ite),
            "jscvt" => Ok(Arm64ISA::Jscvt),
            "ls64" => Ok(Arm64ISA::Ls64),
            "lscp" => Ok(Arm64ISA::Lscp),
            "lse" => Ok(Arm64ISA::Lse),
            "lse128" => Ok(Arm64ISA::Lse128),
            "lsfe" => Ok(Arm64ISA::Lsfe),
            "lsui" => Ok(Arm64ISA::Lsui),
            "lut" => Ok(Arm64ISA::Lut),
            "memtag" | "mte" => Ok(Arm64ISA::Memtag),
            "mops" => Ok(Arm64ISA::Mops),
            "mops-go" => Ok(Arm64ISA::MopsGo),
            "mtetc" => Ok(Arm64ISA::Mtetc),
            "occmo" => Ok(Arm64ISA::Occmo),
            "pauth" => Ok(Arm64ISA::Pauth),
            "pauth-lr" => Ok(Arm64ISA::PauthLr),
            "pmuv3" => Ok(Arm64ISA::Pmuv3),
            "poe2" => Ok(Arm64ISA::Poe2),
            "pops" => Ok(Arm64ISA::Pops),
            "predres" | "specres" => Ok(Arm64ISA::Predres),
            "predres2" | "specres2" => Ok(Arm64ISA::Predres2),
            "profile" | "spe" => Ok(Arm64ISA::Profile),
            "ras" => Ok(Arm64ISA::Ras),
            "rasv2" => Ok(Arm64ISA::Rasv2),
            "rcpc" | "lrcpc" => Ok(Arm64ISA::Rcpc),
            "rcpc3" | "lrcpc3" => Ok(Arm64ISA::Rcpc3),
            "rdm" => Ok(Arm64ISA::Rdm),
            "rng" => Ok(Arm64ISA::Rng),
            "sb" => Ok(Arm64ISA::Sb),
            "sha2" | "sha1" | "sha256" => Ok(Arm64ISA::Sha2),
            "sha3" | "sha512" => Ok(Arm64ISA::Sha3),
            "simd" | "advsimd" | "neon" => Ok(Arm64ISA::Simd),
            "sm4" => Ok(Arm64ISA::Sm4),
            "sme" => Ok(Arm64ISA::Sme),
            "sme-b16b16" => Ok(Arm64ISA::SmeB16b16),
            "sme-f16f16" => Ok(Arm64ISA::SmeF16f16),
            "sme-f64f64" => Ok(Arm64ISA::SmeF64f64),
            "sme-f8f16" => Ok(Arm64ISA::SmeF8f16),
            "sme-f8f32" => Ok(Arm64ISA::SmeF8f32),
            "sme-fa64" => Ok(Arm64ISA::SmeFa64),
            "sme-i16i64" => Ok(Arm64ISA::SmeI16i64),
            "sme-lutv2" => Ok(Arm64ISA::SmeLutv2),
            "sme-mop4" => Ok(Arm64ISA::SmeMop4),
            "sme-tmop" => Ok(Arm64ISA::SmeTmop),
            "sme2" => Ok(Arm64ISA::Sme2),
            "sme2p1" => Ok(Arm64ISA::Sme2p1),
            "sme2p2" => Ok(Arm64ISA::Sme2p2),
            "sme2p3" => Ok(Arm64ISA::Sme2p3),
            "ssbs" => Ok(Arm64ISA::Ssbs),
            "ssve-aes" => Ok(Arm64ISA::SsveAes),
            "ssve-bitperm" => Ok(Arm64ISA::SsveBitperm),
            "ssve-fexpa" => Ok(Arm64ISA::SsveFexpa),
            "ssve-fp8dot2" => Ok(Arm64ISA::SsveFp8dot2),
            "ssve-fp8dot4" => Ok(Arm64ISA::SsveFp8dot4),
            "ssve-fp8fma" => Ok(Arm64ISA::SsveFp8fma),
            "sve" => Ok(Arm64ISA::Sve),
            "sve-aes" => Ok(Arm64ISA::SveAes),
            "sve-aes2" => Ok(Arm64ISA::SveAes2),
            "sve-b16b16" => Ok(Arm64ISA::SveB16b16),
            "sve-b16mm" => Ok(Arm64ISA::SveB16mm),
            "sve-bfscale" => Ok(Arm64ISA::SveBfscale),
            "sve-bitperm" => Ok(Arm64ISA::SveBitperm),
            "sve-f16f32mm" => Ok(Arm64ISA::SveF16f32mm),
            "sve-sha3" => Ok(Arm64ISA::SveSha3),
            "sve-sm4" => Ok(Arm64ISA::SveSm4),
            "sve2" => Ok(Arm64ISA::Sve2),
            "sve2-aes" => Ok(Arm64ISA::Sve2Aes),
            "sve2-bitperm" => Ok(Arm64ISA::Sve2Bitperm),
            "sve2-sha3" => Ok(Arm64ISA::Sve2Sha3),
            "sve2-sm4" => Ok(Arm64ISA::Sve2Sm4),
            "sve2p1" => Ok(Arm64ISA::Sve2p1),
            "sve2p2" => Ok(Arm64ISA::Sve2p2),
            "sve2p3" => Ok(Arm64ISA::Sve2p3),
            "tev" => Ok(Arm64ISA::Tev),
            "the" => Ok(Arm64ISA::The),
            "tlbid" => Ok(Arm64ISA::Tlbid),
            "tlbiw" => Ok(Arm64ISA::Tlbiw),
            "wfxt" => Ok(Arm64ISA::Wfxt),
            other => Err(format!("Unknown Arm64 ISA extension: {}", other)),
        }
    }
}

