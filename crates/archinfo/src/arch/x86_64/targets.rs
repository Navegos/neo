// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/x86_64/targets.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use crate::vector_length::CpuArchitectureVectorLength;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Specifies target processor for compiling and optimizing to specifics of micro-architectures.
///
/// This enum should be kept in order, so that toolchains can check whether the requested setting is `>=` values that they support.
/// Note that by enabling this you are changing the minspec for the PC platform, and the resultant executable might cause worse performance or will crash on incompatible processors.
/// Also note that we don't support x86 CPUs with no SSE4.2 support.
///
/// For more details please see <https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/TargetParser/X86TargetParser.def>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetCpuArchitectureX64 {
    /// No CPU selected.
    #[default]
    None,

    /// A generic CPU with 64-bit extensions.
    Generic,

    /// This selects the CPU to generate code for at compilation time by determining the processor type of the compiling machine.
    /// Using `-march=native` enables all instruction subsets supported by the local machine (hence the result might not run on different machines).
    /// Using `-mtune=native` produces code optimized for the local machine under the constraints of the selected instruction set.
    /// In MSVC defaults to Generic.
    /// Unsupported in cross-compilation.
    Native,

    // SSE4.2 Support CPUs

    /// Generic x86_64 CPU with 64-bit extensions, CX16, LAHF-SAHF, POPCNT, SSE3, SSE4.1, SSE4.2, SSSE3.
    X86_64_v2,

    /// Intel Core i7 generic CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF and FXSR instruction set support.
    Corei7,

    /// Intel Nehalem CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF and FXSR instruction set support.
    Nehalem,

    /// Intel Westmere CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR and PCLMUL instruction set support.
    Westmere,

    /// Intel Silvermont CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, PCLMUL, PREFETCHW and RDRND instruction set support.
    Slm,

    /// Intel Silvermont CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, PCLMUL, PREFETCHW and RDRND instruction set support.
    Silvermont,

    /// Intel Goldmont CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, PCLMUL, PREFETCHW, RDRND, AES, SHA, RDSEED, XSAVE, XSAVEC, XSAVES, XSAVEOPT, CLFLUSHOPT and FSGSBASE instruction set support.
    Goldmont,

    /// Intel Goldmont Plus CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, PCLMUL, PREFETCHW, RDRND, AES, SHA, RDSEED, XSAVE, XSAVEC, XSAVES, XSAVEOPT, CLFLUSHOPT, FSGSBASE, PTWRITE, RDPID and SGX instruction set support.
    Goldmont_plus,

    /// Intel Tremont CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, PCLMUL, PREFETCHW, RDRND, AES, SHA, RDSEED, XSAVE, XSAVEC, XSAVES, XSAVEOPT, CLFLUSHOPT, FSGSBASE, PTWRITE, RDPID, SGX, CLWB, GFNI-SSE, MOVDIRI, MOVDIR64B, CLDEMOTE and WAITPKG instruction set support.
    Tremont,

    // AVX Support CPUs

    /// Intel Core i7 CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE and PCLMUL instruction set support.
    Corei7_avx,

    /// Intel Core i AVX CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND and F16C instruction set support.
    Core_avx_i,

    /// Intel Sandy Bridge CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE and PCLMUL instruction set support.
    Sandybridge,

    /// Intel Ivy Bridge CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND and F16C instruction set support.
    Ivybridge,

    /// CPUs based on AMD Family 15h cores with x86-64 instruction set support. This supersets FMA4, AVX, XOP, LWP, AES, PCLMUL, CX16, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM and 64-bit instruction set extensions.
    Bdver1,

    /// AMD Family 15h core based CPUs with x86-64 instruction set support. This supersets BMI, TBM, F16C, FMA, FMA4, AVX, XOP, LWP, AES, PCLMUL, CX16, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM and 64-bit instruction set extensions.
    Bdver2,

    /// AMD Family 15h core based CPUs with x86-64 instruction set support. This supersets BMI, TBM, F16C, FMA, FMA4, FSGSBASE, AVX, XOP, LWP, AES, PCLMUL, CX16, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM and 64-bit instruction set extensions.
    Bdver3,

    /// CPUs based on AMD Family 16h cores with x86-64 instruction set support. This includes MOVBE, F16C, BMI, AVX, PCLMUL, AES, SSE4.2, SSE4.1, CX16, ABM, SSE4A, SSSE3, SSE3, SSE2, SSE, MMX and 64-bit instruction set extensions.
    Btver2,

    /// Microsoft Xbox One CPU target.
    Xboxone,

    /// Sony PlayStation 4 CPU target.
    Ps4,

    // AVX2 Support CPUs

    /// Generic x86_64 CPU with 64-bit extensions, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, MOVBE, OSXSAVE.
    X86_64_v3,

    /// Intel Core AVX2 CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE and HLE instruction set support.
    Core_AVX2,

    /// Intel Haswell CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE and HLE instruction set support.
    Haswell,

    /// Intel Broadwell CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX and PREFETCHW instruction set support.
    Broadwell,

    /// Intel Skylake CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES and SGX instruction set support.
    Skylake,

    /// Intel Alder Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL and AVX-VNNI instruction set support.
    Alderlake,

    /// Intel Raptor Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL and AVX-VNNI instruction set support.
    Raptorlake,

    /// Intel Meteor Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL and AVX-VNNI instruction set support.
    Meteorlake,

    /// Intel Gracemont CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL and AVX-VNNI instruction set support.
    Gracemont,

    /// Intel Sierra Forest CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, CLDEMOTE, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL, AVX-VNNI, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, ENQCMD and UINTR instruction set support.
    Sierraforest,

    /// Intel Grand Ridge CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, CLDEMOTE, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL, AVX-VNNI, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, ENQCMD and UINTR instruction set support.
    Grandridge,

    /// Intel Arrow Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL, AVX-VNNI, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT and CMPCCXADD instruction set support.
    Arrowlake,

    /// Intel Arrow Lake S CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL, AVX-VNNI, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, AVXVNNIINT16, SHA512, SM3 and SM4 instruction set support.
    Arrowlake_s,

    /// Intel Lunar Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, KL, WIDEKL, AVX-VNNI, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, AVXVNNIINT16, SHA512, SM3 and SM4 instruction set support.
    Lunarlake,

    /// Intel Panther Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, AVX-VNNI, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, AVXVNNIINT16, SHA512, SM3 and SM4 instruction set support.
    Pantherlake,

    /// Intel Wildcat Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, AVX-VNNI, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, AVXVNNIINT16, SHA512, SM3 and SM4 instruction set support.
    Wildcatlake,

    /// Intel Clearwater Forest CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, CLDEMOTE, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, AVX-VNNI, ENQCMD, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, AVXVNNIINT16, SHA512, SM3, SM4, USER_MSR and PREFETCHI instruction set support.
    Clearwaterforest,

    /// AMD Family 15h core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, TBM, F16C, FMA, FMA4, FSGSBASE, AVX, AVX2, XOP, LWP, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM and 64-bit instruction set extensions.
    Bdver4,

    /// AMD Family 17h core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, F16C, FMA, FSGSBASE, AVX, AVX2, ADCX, RDSEED, MWAITX, SHA, CLZERO, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM, XSAVEC, XSAVES, CLFLUSHOPT, POPCNT, and 64-bit instruction set extensions.
    Znver1,

    /// AMD Family 17h core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, CLWB, F16C, FMA, FSGSBASE, AVX, AVX2, ADCX, RDSEED, MWAITX, SHA, CLZERO, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM, XSAVEC, XSAVES, CLFLUSHOPT, POPCNT, RDPID, WBNOINVD, and 64-bit instruction set extensions.
    Znver2,

    /// Microsoft Xbox Series X/S CPU target.
    Xboxxs,

    /// Sony PlayStation 5 CPU target.
    Ps5,

    /// Valve Steam Deck CPU target.
    Steamdeck,

    /// AMD Family 19h core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, CLWB, F16C, FMA, FSGSBASE, AVX, AVX2, ADCX, RDSEED, MWAITX, SHA, CLZERO, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM, XSAVEC, XSAVES, CLFLUSHOPT, POPCNT, RDPID, WBNOINVD, PKU, VPCLMULQDQ, VAES, and 64-bit instruction set extensions.
    Znver3,

    // AVX512 Support CPUs

    /// Generic x86_64 CPU with 64-bit extensions, AVX512F, AVX512BW, AVX512CD, AVX512DQ, AVX512VL.
    X86_64_v4,

    /// Intel Skylake Server CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, CLWB, AVX512VL, AVX512BW, AVX512DQ and AVX512CD instruction set support.
    Skx,

    /// Intel Skylake Server CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, CLWB, AVX512VL, AVX512BW, AVX512DQ and AVX512CD instruction set support.
    Skylake_avx512,

    /// Intel Cannon Lake Server CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA and SHA instruction set support.
    Cannonlake,

    /// Intel Ice Lake Client CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2 , VPCLMULQDQ, AVX512BITALG, RDPID and AVX512VPOPCNTDQ instruction set support.
    Icelake_client,

    /// Intel Ice Lake Server CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2 , VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, PCONFIG, WBNOINVD and CLWB instruction set support.
    Icelake_server,

    /// Intel Cascade Lake CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, CLWB, AVX512VL, AVX512BW, AVX512DQ, AVX512CD and AVX512VNNI instruction set support.
    Cascadelake,

    /// Intel Cooper Lake CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, CLWB, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, AVX512VNNI and AVX512BF16 instruction set support.
    Cooperlake,

    /// Intel Rocket Lake CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID and AVX512VPOPCNTDQ instruction set support.
    Rocketlake,

    /// Intel Tiger Lake CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, MOVDIRI, MOVDIR64B, CLWB, AVX512VP2INTERSECT and KEYLOCKER instruction set support.
    Tigerlake,

    /// Intel Sapphire Rapids CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, PCONFIG, WBNOINVD, CLWB, MOVDIRI, MOVDIR64B, ENQCMD, CLDEMOTE, PTWRITE, WAITPKG, SERIALIZE, TSXLDTRK, UINTR, AMX-BF16, AMX-TILE, AMX-INT8, AVX-VNNI, AVX512-FP16 and AVX512BF16 instruction set support.
    Sapphirerapids,

    /// Intel Emerald Rapids CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, PCONFIG, WBNOINVD, CLWB, MOVDIRI, MOVDIR64B, ENQCMD, CLDEMOTE, PTWRITE, WAITPKG, SERIALIZE, TSXLDTRK, UINTR, AMX-BF16, AMX-TILE, AMX-INT8, AVX-VNNI, AVX512-FP16 and AVX512BF16 instruction set support.
    Emeraldrapids,

    /// Intel Granite Rapids CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, PCONFIG, WBNOINVD, CLWB, MOVDIRI, MOVDIR64B, ENQCMD, CLDEMOTE, PTWRITE, WAITPKG, SERIALIZE, TSXLDTRK, UINTR, AMX-BF16, AMX-TILE, AMX-INT8, AVX-VNNI, AVX512-FP16, AVX512BF16, AMX-FP16 and PREFETCHI instruction set support.
    Graniterapids,

    /// Intel Granite Rapids D CPU with 64-bit extensions, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, PCONFIG, WBNOINVD, CLWB, MOVDIRI, MOVDIR64B, ENQCMD, CLDEMOTE, PTWRITE, WAITPKG, SERIALIZE, TSXLDTRK, UINTR, AMX-BF16, AMX-TILE, AMX-INT8, AVX-VNNI, AVX512FP16, AVX512BF16, AMX-FP16, PREFETCHI and AMX-COMPLEX instruction set support.
    Graniterapids_d,

    /// AMD Family 19h core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, CLWB, F16C, FMA, FSGSBASE, AVX, AVX2, ADCX, RDSEED, MWAITX, SHA, CLZERO, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM, XSAVEC, XSAVES, CLFLUSHOPT, POPCNT, RDPID, WBNOINVD, PKU, VPCLMULQDQ, VAES, AVX512F, AVX512DQ, AVX512IFMA, AVX512CD, AVX512BW, AVX512VL, AVX512BF16, AVX512VBMI, AVX512VBMI2, AVX512VNNI, AVX512BITALG, AVX512VPOPCNTDQ, GFNI and 64-bit instruction set extensions.
    Znver4,

    /// Valve Steam Machine / custom target.
    Steammachine,

    /// AMD Family 1ah core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, CLWB, F16C, FMA, FSGSBASE, AVX, AVX2, ADCX, RDSEED, MWAITX, SHA, CLZERO, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM, XSAVEC, XSAVES, CLFLUSHOPT, POPCNT, RDPID, WBNOINVD, PKU, VPCLMULQDQ, VAES, AVX512F, AVX512DQ, AVX512IFMA, AVX512CD, AVX512BW, AVX512VL, AVX512BF16, AVX512VBMI, AVX512VBMI2, AVX512VNNI, AVX512BITALG, AVX512VPOPCNTDQ, GFNI, AVXVNNI, MOVDIRI, MOVDIR64B, AVX512VP2INTERSECT, PREFETCHI and 64-bit instruction set extensions.
    Znver5,

    /// AMD Family 1ah core based CPUs with x86-64 instruction set support. This supersets BMI, BMI2, CLWB, F16C, FMA, FSGSBASE, AVX, AVX2, ADCX, RDSEED, MWAITX, SHA, CLZERO, AES, PCLMUL, CX16, MOVBE, MMX, SSE, SSE2, SSE3, SSE4A, SSSE3, SSE4.1, SSE4.2, ABM, XSAVEC, XSAVES, CLFLUSHOPT, POPCNT, RDPID, WBNOINVD, PKU, VPCLMULQDQ, VAES, AVX512F, AVX512DQ, AVX512IFMA, AVX512CD, AVX512BW, AVX512VL, AVX512BF16, AVX512VBMI, AVX512VBMI2, AVX512VNNI, AVX512BITALG, AVX512VPOPCNTDQ, GFNI, AVXVNNI, MOVDIRI, MOVDIR64B, AVX512VP2INTERSECT, PREFETCHI, AVXVNNIINT8, AVXIFMA, AVX512FP16, AVXNECONVERT, AVX512BMM and 64-bit instruction set extensions.
    Znver6,

    // AVX10.2 Support CPUs

    /// Intel Diamond Rapids CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, CX16, SAHF, FXSR, AVX, XSAVE, PCLMUL, FSGSBASE, RDRND, F16C, AVX2, BMI, BMI2, LZCNT, FMA, MOVBE, HLE, RDSEED, ADCX, PREFETCHW, AES, CLFLUSHOPT, XSAVEC, XSAVES, SGX, AVX512F, AVX512VL, AVX512BW, AVX512DQ, AVX512CD, PKU, AVX512VBMI, AVX512IFMA, SHA, AVX512VNNI, GFNI, VAES, AVX512VBMI2, VPCLMULQDQ, AVX512BITALG, RDPID, AVX512VPOPCNTDQ, PCONFIG, WBNOINVD, CLWB, MOVDIRI, MOVDIR64B, ENQCMD, CLDEMOTE, PTWRITE, WAITPKG, SERIALIZE, TSXLDTRK, UINTR, AMX-BF16, AMX-TILE, AMX-INT8, AVX-VNNI, AVX512FP16, AVX512BF16, AMX-FP16, PREFETCHI, AMX-COMPLEX, AVX10.1-512, AVX-IFMA, AVX-NE-CONVERT, AVX-VNNI-INT16, AVX-VNNI-INT8, CMPccXADD, SHA512, SM3, SM4, AVX10.2-512, APX_F, AMX-AVX512, AMX-FP8, AMX-TF32, MOVRS and AMX-MOVRS instruction set support.
    Diamondrapids,

    /// Intel Nova Lake CPU with 64-bit extensions, MOVBE, MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT, AES, PREFETCHW, PCLMUL, RDRND, XSAVE, XSAVEC, XSAVES, XSAVEOPT, FSGSBASE, PTWRITE, RDPID, SGX, GFNI-SSE, CLWB, MOVDIRI, MOVDIR64B, WAITPKG, ADCX, AVX, AVX2, BMI, BMI2, F16C, FMA, LZCNT, PCONFIG, PKU, VAES, VPCLMULQDQ, SERIALIZE, HRESET, AVX-VNNI, UINTR, AVXIFMA, AVXVNNIINT8, AVXNECONVERT, CMPCCXADD, AVXVNNIINT16, SHA512, SM3, SM4, PREFETCHI, APX_F, AVX10.1, AVX10.2 and MOVRS instruction set support.
    Novalake,
}

/// Minimum baseline CPU architecture for x86_64 code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MinimumCpuArchitectureX64 {
    #[default]
    None,
    AVX,
    AVX2,
    AVX512,
    AVX10_1,
    AVX10_2,
}

impl fmt::Display for MinimumCpuArchitectureX64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinimumCpuArchitectureX64::None => write!(f, "sse4.2"),
            MinimumCpuArchitectureX64::AVX => write!(f, "avx"),
            MinimumCpuArchitectureX64::AVX2 => write!(f, "avx2"),
            MinimumCpuArchitectureX64::AVX512 => write!(f, "avx512"),
            MinimumCpuArchitectureX64::AVX10_1 => write!(f, "avx10.1"),
            MinimumCpuArchitectureX64::AVX10_2 => write!(f, "avx10.2"),
        }
    }
}

impl FromStr for MinimumCpuArchitectureX64 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.trim().to_ascii_lowercase().replace('_', ".");
        match norm.as_str() {
            "none" | "sse4.2" | "sse42" | "default" => Ok(MinimumCpuArchitectureX64::None),
            "avx" => Ok(MinimumCpuArchitectureX64::AVX),
            "avx2" => Ok(MinimumCpuArchitectureX64::AVX2),
            "avx512" | "avx-512" => Ok(MinimumCpuArchitectureX64::AVX512),
            "avx10.1" | "avx10-1" => Ok(MinimumCpuArchitectureX64::AVX10_1),
            "avx10.2" | "avx10-2" => Ok(MinimumCpuArchitectureX64::AVX10_2),
            other => Err(format!("Unknown MinimumCpuArchitectureX64: '{}'", other)),
        }
    }
}

impl MinimumCpuArchitectureX64 {
    /// Evaluates optimal vector length (normal default)
    pub fn vector_length(&self) -> CpuArchitectureVectorLength {
        self.resolve_vector_length(None)
    }

    /// Evaluates vector length given an optional user requested vector length:
    /// - SSE4.2 and AVX: fixed vl128, ignores user input in vector_length.
    /// - AVX2: vl256 or vl128, user can select; if not specified or higher, defaults to vl256.
    /// - AVX512: vl512 or vl256 or vl128, user can select; if not specified, defaults to vl512.
    /// - AVX10.1 and AVX10.2: vl512 or vl256 or vl128, user can select; if not specified, defaults to vl256.
    pub fn resolve_vector_length(&self, requested: Option<CpuArchitectureVectorLength>) -> CpuArchitectureVectorLength {
        match self {
            MinimumCpuArchitectureX64::None | MinimumCpuArchitectureX64::AVX => {
                CpuArchitectureVectorLength::VL128
            }
            MinimumCpuArchitectureX64::AVX2 => {
                match requested {
                    Some(CpuArchitectureVectorLength::VL128) => CpuArchitectureVectorLength::VL128,
                    _ => CpuArchitectureVectorLength::VL256,
                }
            }
            MinimumCpuArchitectureX64::AVX512 => {
                match requested {
                    Some(CpuArchitectureVectorLength::VL128) => CpuArchitectureVectorLength::VL128,
                    Some(CpuArchitectureVectorLength::VL256) => CpuArchitectureVectorLength::VL256,
                    Some(CpuArchitectureVectorLength::VL512) => CpuArchitectureVectorLength::VL512,
                    _ => CpuArchitectureVectorLength::VL512,
                }
            }
            MinimumCpuArchitectureX64::AVX10_1 | MinimumCpuArchitectureX64::AVX10_2 => {
                match requested {
                    Some(CpuArchitectureVectorLength::VL128) => CpuArchitectureVectorLength::VL128,
                    Some(CpuArchitectureVectorLength::VL512) => CpuArchitectureVectorLength::VL512,
                    Some(CpuArchitectureVectorLength::VL256) => CpuArchitectureVectorLength::VL256,
                    _ => CpuArchitectureVectorLength::VL256,
                }
            }
        }
    }
}

/// Provides a mapping between the TargetCpuArchitectureX64 enum values and their corresponding string representations.
pub struct TargetCpuArchitectureX64Names;

impl TargetCpuArchitectureX64Names {
    pub fn name(target: TargetCpuArchitectureX64) -> &'static str {
        match target {
            TargetCpuArchitectureX64::None | TargetCpuArchitectureX64::Generic => "x86-64-v2",
            TargetCpuArchitectureX64::Native => "native",
            TargetCpuArchitectureX64::X86_64_v2 => "x86-64-v2",
            TargetCpuArchitectureX64::Corei7 => "corei7",
            TargetCpuArchitectureX64::Nehalem => "nehalem",
            TargetCpuArchitectureX64::Westmere => "westmere",
            TargetCpuArchitectureX64::Slm => "slm",
            TargetCpuArchitectureX64::Silvermont => "silvermont",
            TargetCpuArchitectureX64::Goldmont => "goldmont",
            TargetCpuArchitectureX64::Goldmont_plus => "goldmont-plus",
            TargetCpuArchitectureX64::Tremont => "tremont",
            TargetCpuArchitectureX64::Corei7_avx => "corei7-avx",
            TargetCpuArchitectureX64::Core_avx_i => "core-avx-i",
            TargetCpuArchitectureX64::Sandybridge => "sandybridge",
            TargetCpuArchitectureX64::Ivybridge => "ivybridge",
            TargetCpuArchitectureX64::Bdver1 => "bdver1",
            TargetCpuArchitectureX64::Bdver2 => "bdver2",
            TargetCpuArchitectureX64::Bdver3 => "bdver3",
            TargetCpuArchitectureX64::Btver2 => "btver2",
            TargetCpuArchitectureX64::Xboxone => "btver2",
            TargetCpuArchitectureX64::Ps4 => "btver2",
            TargetCpuArchitectureX64::X86_64_v3 => "x86-64-v3",
            TargetCpuArchitectureX64::Core_AVX2 => "core-avx2",
            TargetCpuArchitectureX64::Haswell => "haswell",
            TargetCpuArchitectureX64::Broadwell => "broadwell",
            TargetCpuArchitectureX64::Skylake => "skylake",
            TargetCpuArchitectureX64::Alderlake => "alderlake",
            TargetCpuArchitectureX64::Raptorlake => "raptorlake",
            TargetCpuArchitectureX64::Meteorlake => "meteorlake",
            TargetCpuArchitectureX64::Gracemont => "gracemont",
            TargetCpuArchitectureX64::Sierraforest => "sierraforest",
            TargetCpuArchitectureX64::Grandridge => "grandridge",
            TargetCpuArchitectureX64::Arrowlake => "arrowlake",
            TargetCpuArchitectureX64::Arrowlake_s => "arrowlake-s",
            TargetCpuArchitectureX64::Lunarlake => "lunarlake",
            TargetCpuArchitectureX64::Pantherlake => "pantherlake",
            TargetCpuArchitectureX64::Wildcatlake => "wildcatlake",
            TargetCpuArchitectureX64::Clearwaterforest => "clearwaterforest",
            TargetCpuArchitectureX64::Bdver4 => "bdver4",
            TargetCpuArchitectureX64::Znver1 => "znver1",
            TargetCpuArchitectureX64::Znver2 => "znver2",
            TargetCpuArchitectureX64::Xboxxs => "znver2",
            TargetCpuArchitectureX64::Ps5 => "znver2",
            TargetCpuArchitectureX64::Steamdeck => "znver2",
            TargetCpuArchitectureX64::Znver3 => "znver3",
            TargetCpuArchitectureX64::X86_64_v4 => "x86-64-v4",
            TargetCpuArchitectureX64::Skx => "skx",
            TargetCpuArchitectureX64::Skylake_avx512 => "skylake-avx512",
            TargetCpuArchitectureX64::Cannonlake => "cannonlake",
            TargetCpuArchitectureX64::Icelake_client => "icelake-client",
            TargetCpuArchitectureX64::Icelake_server => "icelake-server",
            TargetCpuArchitectureX64::Cascadelake => "cascadelake",
            TargetCpuArchitectureX64::Cooperlake => "cooperlake",
            TargetCpuArchitectureX64::Rocketlake => "rocketlake",
            TargetCpuArchitectureX64::Tigerlake => "tigerlake",
            TargetCpuArchitectureX64::Sapphirerapids => "sapphirerapids",
            TargetCpuArchitectureX64::Emeraldrapids => "emeraldrapids",
            TargetCpuArchitectureX64::Graniterapids => "graniterapids",
            TargetCpuArchitectureX64::Graniterapids_d => "graniterapids-d",
            TargetCpuArchitectureX64::Znver4 => "znver4",
            TargetCpuArchitectureX64::Steammachine => "znver4",
            TargetCpuArchitectureX64::Znver5 => "znver5",
            TargetCpuArchitectureX64::Znver6 => "znver6",
            TargetCpuArchitectureX64::Diamondrapids => "diamondrapids",
            TargetCpuArchitectureX64::Novalake => "novalake",
        }
    }
}

impl FromStr for TargetCpuArchitectureX64 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.to_ascii_lowercase().replace(['_', '.'], "-");
        match norm.as_str() {
            "x86-64" | "generic" | "none" | "default" => Ok(TargetCpuArchitectureX64::Generic),
            "native" => Ok(TargetCpuArchitectureX64::Native),
            "x86-64-v2" => Ok(TargetCpuArchitectureX64::X86_64_v2),
            "corei7" => Ok(TargetCpuArchitectureX64::Corei7),
            "nehalem" => Ok(TargetCpuArchitectureX64::Nehalem),
            "westmere" => Ok(TargetCpuArchitectureX64::Westmere),
            "slm" => Ok(TargetCpuArchitectureX64::Slm),
            "silvermont" => Ok(TargetCpuArchitectureX64::Silvermont),
            "goldmont" => Ok(TargetCpuArchitectureX64::Goldmont),
            "goldmont-plus" => Ok(TargetCpuArchitectureX64::Goldmont_plus),
            "tremont" => Ok(TargetCpuArchitectureX64::Tremont),
            "corei7-avx" => Ok(TargetCpuArchitectureX64::Corei7_avx),
            "core-avx-i" => Ok(TargetCpuArchitectureX64::Core_avx_i),
            "sandybridge" => Ok(TargetCpuArchitectureX64::Sandybridge),
            "ivybridge" => Ok(TargetCpuArchitectureX64::Ivybridge),
            "bdver1" => Ok(TargetCpuArchitectureX64::Bdver1),
            "bdver2" => Ok(TargetCpuArchitectureX64::Bdver2),
            "bdver3" => Ok(TargetCpuArchitectureX64::Bdver3),
            "btver2" => Ok(TargetCpuArchitectureX64::Btver2),
            "xboxone" => Ok(TargetCpuArchitectureX64::Xboxone),
            "ps4" => Ok(TargetCpuArchitectureX64::Ps4),
            "x86-64-v3" => Ok(TargetCpuArchitectureX64::X86_64_v3),
            "core-avx2" => Ok(TargetCpuArchitectureX64::Core_AVX2),
            "haswell" => Ok(TargetCpuArchitectureX64::Haswell),
            "broadwell" => Ok(TargetCpuArchitectureX64::Broadwell),
            "skylake" => Ok(TargetCpuArchitectureX64::Skylake),
            "alderlake" => Ok(TargetCpuArchitectureX64::Alderlake),
            "raptorlake" => Ok(TargetCpuArchitectureX64::Raptorlake),
            "meteorlake" => Ok(TargetCpuArchitectureX64::Meteorlake),
            "gracemont" => Ok(TargetCpuArchitectureX64::Gracemont),
            "sierraforest" => Ok(TargetCpuArchitectureX64::Sierraforest),
            "grandridge" => Ok(TargetCpuArchitectureX64::Grandridge),
            "arrowlake" => Ok(TargetCpuArchitectureX64::Arrowlake),
            "arrowlake-s" => Ok(TargetCpuArchitectureX64::Arrowlake_s),
            "lunarlake" => Ok(TargetCpuArchitectureX64::Lunarlake),
            "pantherlake" => Ok(TargetCpuArchitectureX64::Pantherlake),
            "wildcatlake" => Ok(TargetCpuArchitectureX64::Wildcatlake),
            "clearwaterforest" => Ok(TargetCpuArchitectureX64::Clearwaterforest),
            "bdver4" => Ok(TargetCpuArchitectureX64::Bdver4),
            "znver1" => Ok(TargetCpuArchitectureX64::Znver1),
            "znver2" => Ok(TargetCpuArchitectureX64::Znver2),
            "xboxxs" => Ok(TargetCpuArchitectureX64::Xboxxs),
            "ps5" => Ok(TargetCpuArchitectureX64::Ps5),
            "steamdeck" => Ok(TargetCpuArchitectureX64::Steamdeck),
            "znver3" => Ok(TargetCpuArchitectureX64::Znver3),
            "x86-64-v4" => Ok(TargetCpuArchitectureX64::X86_64_v4),
            "skx" => Ok(TargetCpuArchitectureX64::Skx),
            "skylake-avx512" => Ok(TargetCpuArchitectureX64::Skylake_avx512),
            "cannonlake" => Ok(TargetCpuArchitectureX64::Cannonlake),
            "icelake-client" => Ok(TargetCpuArchitectureX64::Icelake_client),
            "icelake-server" => Ok(TargetCpuArchitectureX64::Icelake_server),
            "cascadelake" => Ok(TargetCpuArchitectureX64::Cascadelake),
            "cooperlake" => Ok(TargetCpuArchitectureX64::Cooperlake),
            "rocketlake" => Ok(TargetCpuArchitectureX64::Rocketlake),
            "tigerlake" => Ok(TargetCpuArchitectureX64::Tigerlake),
            "sapphirerapids" => Ok(TargetCpuArchitectureX64::Sapphirerapids),
            "emeraldrapids" => Ok(TargetCpuArchitectureX64::Emeraldrapids),
            "graniterapids" => Ok(TargetCpuArchitectureX64::Graniterapids),
            "graniterapids-d" => Ok(TargetCpuArchitectureX64::Graniterapids_d),
            "znver4" => Ok(TargetCpuArchitectureX64::Znver4),
            "steammachine" => Ok(TargetCpuArchitectureX64::Steammachine),
            "znver5" => Ok(TargetCpuArchitectureX64::Znver5),
            "znver6" => Ok(TargetCpuArchitectureX64::Znver6),
            "diamondrapids" => Ok(TargetCpuArchitectureX64::Diamondrapids),
            "novalake" => Ok(TargetCpuArchitectureX64::Novalake),
            other => Err(format!("Unknown x64 target CPU: {}", other)),
        }
    }
}

