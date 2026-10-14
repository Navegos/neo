// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/targets.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use crate::platform::Platform;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Specifies target processor for compiling and optimizing to specifics of Arm64 micro-architectures.
///
/// This enum should be kept in order, so that toolchains can check whether the requested setting is `>=` values that they support.
/// Note that by enabling this you are changing the minspec for the PC platform, and the resultant executable might cause worse performance or will crash on incompatible processors.
///
/// For more details please see `clang --target=aarch64 --print-enabled-extensions -mcpu='tsv110'`,
/// <https://github.com/llvm/llvm-project/tree/main/clang/test/Driver/print-enabled-extensions>,
/// <https://en.wikipedia.org/wiki/ARM_architecture_family>, and
/// <https://support.arm.com/documentation/109697/2026_06/Feature-descriptions?lang=en>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetCpuArchitectureArm64 {
    /// No CPU selected.
    #[default]
    None,

    /// A generic Arm CPU, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AdvSIMD, FEAT_ETE, FEAT_FP, FEAT_TRBE.
    Generic,

    /// This selects the CPU to generate code for at compilation time by determining the processor type of the compiling machine. Using -march=native enables all instruction subsets supported by the local machine (hence the result might not run on different machines). Using -mtune=native produces code optimized for the local machine under the constraints of the selected instruction set.
    /// In MSVC defaults to Generic.
    /// Unsupported in crosscompile.
    Native,

    // AArch64.v8 A Profile
    // ARMv8-A
    // Cortex_A34,    // X Ultra-low power in-order (Too weak for UE5)
    // Cortex_A35,    // X Ultra-low power in-order (Too weak for UE5)
    // Cortex_A53,    // X Legacy low-tier in-order (Too weak for UE5)
    // Cortex_A57,    // X Legacy, prone to severe thermal throttling (Nintendo Switch era, too old for UE5)

    /// Arm Cortex A72, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_PMUv3, FEAT_SHA1, FEAT_SHA256.
    Cortex_A72,       // ! Borderline (Raspberry Pi 4 class; handles minimal Linux/Android UE5 builds at very low settings)

    /// Arm Cortex A73, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_PMUv3, FEAT_SHA1, FEAT_SHA256.
    Cortex_A73,       // ! Borderline (Legacy mid-tier mobile)

    // Cyclone,       // X Legacy Apple A7 variant (Too old for modern macOS/iOS UE5)
    // Apple_A7,      // X Too old (No modern Vulkan/Metal feature levels)
    // Apple_A8,      // X Too old
    // Apple_A9,      // X Too old

    /// Samsung Exynos M3, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_PMUv3, FEAT_SHA1, FEAT_SHA256.
    Exynos_M3,        //  Capable (Samsung custom OoO mobile core)

    /// Qualcomm's Falkor, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_PMUv3, FEAT_RDM, FEAT_SHA1, FEAT_SHA256.
    Falkor,           //  Capable (Qualcomm server core; great for Linux server/game builds)

    /// Qualcomm's Kryo, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_PMUv3, FEAT_SHA1, FEAT_SHA256.
    Kryo,             //  Capable (Qualcomm Snapdragon core for Android/Windows on ARM)

    // ThunderX,      // X Early Cavium server core (Terrible single-thread IPC for games)
    // ThunderXT81,   // X Legacy server variant
    // ThunderXT83,   // X Legacy server variant
    // ThunderXT88,   // X Legacy server variant

    // ARMv8.1-A

    /// Apple A10, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_LOR, FEAT_PAN, FEAT_PMUv3, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_VHE.
    /// Arch ARMv8.1-A enables additional features: FEAT_LSE.
    /// Requires (Clang >= 19.1.0).
    /// Metal GPU hardware Family: apple3, metal version: metal.
    Apple_A10,        // ! Minimum viable iOS mobile core

    /// Cavium ThunderX2 T99, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_FP, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_VHE.
    ThunderX2T99,     //  Capable (Server builds)

    // ARMv8.2-A
    // Cortex_A55,    // X In-order efficiency core (Will severely bottleneck UE5 physics/game thread)
    // Cortex_A65,    // X Throughput-focused embedded core
    // Cortex_A65AE,  // X Automotive safety embedded core

    /// Arm Cortex A75, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    Cortex_A75,       //  Capable (Mobile/Android baseline)

    /// Arm Cortex A76, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_A76,       //  Capable (Common baseline in modern low-end Android/Windows laptops)

    /// Arm Cortex A76AE, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_A76AE,     //  Capable (Automotive/Embedded variant)

    /// Arm Cortex A77, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_A77,       //  Capable (Solid OoO performance)

    /// Arm Cortex A78, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_A78,       //  Capable (Excellent baseline for modern Android / Windows on ARM devices)

    /// Arm Cortex A78AE, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_A78AE,     //  Capable

    /// Arm Cortex A78C, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_FlagM, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_A78C,      //  Capable (Tablet/Laptop optimized)

    /// Nintendo Switch 2 custom SoC target.
    Switch2,          //  Capable (Tablet/Laptop optimized)

    /// Arm Cortex X1, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_X1,        //  Capable (High-performance flagship mobile core)

    /// Arm Cortex X1C, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_FlagM, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Cortex_X1C,       //  Capable (Laptop variant)

    // Neoverse_E1,   // X Data routing/telecom focus architecture

    /// Arm Neoverse N1, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Neoverse_N1,      //  Capable (AWS Graviton2 instance target for Linux dedicated game servers)

    /// AWS Graviton2, Neoverse N1 ALIAS, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_UAO, FEAT_VHE.
    Graviton2,        //  Capable (Linux cloud game servers)

    /// Apple A11, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    /// Metal GPU hardware Family: apple4, metal version: metal.
    Apple_A11,        //  Capable (Mobile)

    /// Samsung Exynos M4, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    Exynos_M4,        //  Capable

    /// Samsung Exynos M5, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    Exynos_M5,        //  Capable

    /// HiSilicon's TSV110, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_JSCVT, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_UAO, FEAT_VHE.
    TSV110,           //  Capable

    /// Fujitsu A64FX, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_FCMA, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_SVE, FEAT_UAO, FEAT_VHE.
    /// Requires (Clang >= 19.1.0).
    A64FX,            //  Capable (Fujitsu HPC target)

    /// Nvidia Carmel, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_FP, FEAT_FP16, FEAT_LOR, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    Carmel,           //  Capable (Nvidia Jetson AGX Xavier target; excellent for embedded Linux UE5)

    // ARMv8.3-A

    /// Qualcomm's Saphira, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_CCIDX, FEAT_CRC32, FEAT_DIT, FEAT_DPB, FEAT_DotProd, FEAT_FCMA, FEAT_FP, FEAT_FlagM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SPE, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Saphira,          //  Capable

    /// Cavium ThunderX3 T110, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CCIDX, FEAT_CRC32, FEAT_DPB, FEAT_FCMA, FEAT_FP, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    ThunderX3T110,    //  Capable

    /// Apple A12, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DPB, FEAT_FCMA, FEAT_FP, FEAT_FP16, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LSE, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SHA1, FEAT_SHA256, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.3-A enables additional features: FEAT_CCIDX.
    /// Metal GPU hardware Family: apple5, metal version: metal.
    Apple_A12,        //  Capable

    // Apple_S4,      // X Apple Watch chip (Cannot run desktop/mobile UE5 games)
    // Apple_S5,      // X Apple Watch chip

    // ARMv8.4-A

    /// Arm Neoverse V1, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_CCIDX, FEAT_CRC32, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FlagM, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Neoverse_V1,      //  Capable (High-performance server/compute)

    /// Arm Neoverse 512TVB, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_CCIDX, FEAT_CRC32, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FlagM, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Neoverse_512TVB,  //  Capable

    /// AWS Graviton3, Neoverse V1 ALIAS, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_CCIDX, FEAT_CRC32, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FlagM, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Graviton3,        //  Capable (Linux cloud server)

    /// Apple A13, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_CRC32, FEAT_DIT, FEAT_DPB, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FlagM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.4-A enables additional features: FEAT_CCIDX.
    /// Metal GPU hardware Family: apple6, metal version: metal.
    Apple_A13,        //  Capable

    /// Apple A14, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.4-A enables additional features: FEAT_CCIDX.
    /// Metal GPU hardware Family: apple7, metal version: metal 3 & 4.
    Apple_A14,        //  Capable

    /// Apple M1, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.4-A enables additional features: FEAT_CCIDX.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple7, metal version: metal 3 & 4.
    Apple_M1,         //  Capable (Excellent baseline for macOS / Windows on ARM via translation)

    // Apple_S6,      // X Apple Watch chip
    // Apple_S7,      // X Apple Watch chip
    // Apple_S8,      // X Apple Watch chip

    // ARMv8.6-A

    /// Ampere 1, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Requires (Clang >= 19.1.0).
    Ampere1,          //  Capable (Enterprise Cloud Linux servers)

    /// Ampere 1A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Requires (Clang >= 19.1.0).
    Ampere1A,         //  Capable

    /// Apple A15, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.6-A enables additional features: FEAT_CCIDX.
    /// Metal GPU hardware Family: apple8, metal version: metal 3 & 4.
    Apple_A15,        //  Capable (iOS / iPadOS)

    /// Apple A16, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.6-A enables additional features: FEAT_CCIDX.
    /// Metal GPU hardware Family: apple8, metal version: metal 3 & 4.
    Apple_A16,        //  Capable

    /// Apple A17, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.6-A enables additional features: FEAT_CCIDX.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple9, metal version: metal 3 & 4.
    Apple_A17,        //  Capable

    /// Apple M2, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.6-A enables additional features: FEAT_CCIDX.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple8, metal version: metal 3 & 4.
    Apple_M2,         //  Capable (Native macOS target)

    /// Apple M3, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.6-A enables additional features: FEAT_CCIDX.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple9, metal version: metal 3 & 4.
    Apple_M3,         //  Capable (Native macOS target)

    // Apple_S9,      // X Apple Watch chip
    // Apple_S10,     // X Apple Watch chip

    // ARMv8.7-A

    /// Ampere 1B, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSSC, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv8.7-A enables additional features: FEAT_SPEv1p2.
    /// Requires (Clang >= 19.1.0).
    Ampere1B,         //  Capable

    /// Qualcomm's Oryon v1, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    /// Arch ARMv8.7-A enables additional features: FEAT_SPEv1p2, FEAT_WFxT, FEAT_XS.
    Oryon_1,          //  Capable (Snapdragon X Elite; Tier-1 target for Windows on ARM UE5 games)

    /// HiSilicon's HIP12, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BRBE, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NMI, FEAT_GICv3_NMI, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    Hip12,            //  Capable

    // AArch64.v8 R Profile
    // Cortex_R82,    // X Real-time profile, missing necessary MMU support for standard desktop OSs
    // Cortex_R82AE,  // X Automotive Real-time profile

    // AArch64.v9 A Profile
    // ARMv9-A
    // Cortex_A510,   // X In-order efficiency core (Too weak to act as a standalone primary game execution core)

    /// Arm Cortex A710, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Cortex_A710,      //  Capable (Modern Android/Windows flagship cluster core)

    /// Arm Cortex A715, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Cortex_A715,      //  Capable

    /// Arm Cortex X2, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Cortex_X2,        //  Capable

    /// Arm Cortex X3, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Cortex_X3,        //  Capable

    // Neoverse_E2,   // X Low power infrastructure edge core

    /// Arm Neoverse N2, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Neoverse_N2,      //  Capable

    /// Arm Neoverse V2, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Neoverse_V2,      //  Capable

    /// Azure Cobalt 100, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Cobalt_100,       //  Capable (Microsoft Azure custom cloud silicon)

    /// Nvidia Grace, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ETE, FEAT_FCMA, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE.
    Grace,            //  Capable (Nvidia superchip platform)

    // ARMv9.2-A

    /// Ampere 1C, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME, FEAT_SPEv1p2.
    Ampere1c,         //  Capable

    /// Arm Cortex A320, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME, FEAT_SPEv1p2.
    Cortex_A320,      //  Capable

    // Cortex_A520,   // X In-order efficiency core
    // Cortex_A520AE, // X In-order efficiency core

    /// Arm Cortex A720, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Cortex_A720,      //  Capable

    /// Arm Cortex A720AE, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Cortex_A720AE,    //  Capable

    /// Arm Cortex A725, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Cortex_A725,      //  Capable

    /// Arm Cortex X4, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Cortex_X4,        //  Capable

    /// Arm Cortex X925, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_Crypto, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Cortex_X925,      //  Capable

    // Neoverse_E3,   // X Low power networking target

    /// Arm Neoverse N3, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Neoverse_N3,      //  Capable

    /// Arm Neoverse V3, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BRBE, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC.
    Neoverse_V3,      //  Capable

    /// Arm Neoverse V3AE, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BRBE, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC.
    Neoverse_V3AE,    //  Capable

    /// Arm AGI CPU, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BRBE, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC.
    Armagicpu,        //  Capable

    /// Apple A18, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SME, FEAT_SME2, FEAT_SME_F64F64, FEAT_SME_I16I64, FEAT_SPECRES, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_CCIDX, FEAT_MEC, FEAT_RME, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2.
    /// No SVE SVE2 isa extensions.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple9, metal version: metal 3 & 4.
    Apple_A18,        //  Capable

    /// Apple M4, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SME, FEAT_SME2, FEAT_SME_F64F64, FEAT_SME_I16I64, FEAT_SPECRES, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_CCIDX, FEAT_MEC, FEAT_RME, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2.
    /// No SVE SVE2 isa extensions.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple9, metal version: metal 3 & 4.
    Apple_M4,         //  Capable (Native macOS/iPadOS high-end target)

    /// Nvidia Gb10, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_BitPerm, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.2-A enables additional features: FEAT_MEC, FEAT_RME.
    Gb10,             //  Capable (Google Tensor G4 baseline equivalents)

    /// Nvidia Olympus, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BRBE, FEAT_BTI, FEAT_CCIDX, FEAT_CHK, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FP8DOT4, FEAT_FP8FMA, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    Olympus,          //  Capable

    /// Nvidia Rigel, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BRBE, FEAT_BTI, FEAT_CCIDX, FEAT_CHK, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FP8DOT4, FEAT_FP8FMA, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPE, FEAT_SPECRES, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    Rigel,            //  Capable

    // ARMv9.3-A

    /// Fujitsu Monaka, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CLRBHB, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MOPS, FEAT_MPAM, FEAT_NMI, FEAT_GICv3_NMI, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.3-A enables additional features: FEAT_SPEv1p2.
    Fujitsu_Monaka,   //  Capable

    /// Arm C1 Nano, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CLRBHB, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MOPS, FEAT_MPAM, FEAT_NMI, FEAT_GICv3_NMI, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.3-A enables additional features: FEAT_SPEv1p2.
    C1_Nano,          //  Capable

    /// Arm C1 Premium, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CLRBHB, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MOPS, FEAT_MPAM, FEAT_NMI, FEAT_GICv3_NMI, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.3-A enables additional features: FEAT_SPEv1p2.
    C1_Premium,       //  Capable

    /// Arm C1 Pro, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CLRBHB, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MOPS, FEAT_MPAM, FEAT_NMI, FEAT_GICv3_NMI, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.3-A enables additional features: FEAT_SPEv1p2.
    C1_Pro,           //  Capable

    /// Arm C1 Ultra, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CCIDX, FEAT_CLRBHB, FEAT_CRC32, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_ETE, FEAT_FAMINMAX, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FP8, FEAT_FP8DOT2, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LS64, FEAT_LS64_V, FEAT_LS64_ACCDATA, FEAT_LSE, FEAT_LSE2, FEAT_LUT, FEAT_MEC, FEAT_MOPS, FEAT_MPAM, FEAT_NMI, FEAT_GICv3_NMI, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_RME, FEAT_RNG, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SM4, FEAT_SM3, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE_AES, FEAT_SVE_PMULL128, FEAT_SVE_BitPerm, FEAT_SVE_SHA3, FEAT_SVE_SM4, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRBE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.3-A enables additional features: FEAT_SPEv1p2.
    C1_Ultra,         //  Capable

    // ARMv9.4-A

    /// Apple A19, ARMv9.4-A, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSSC, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SME, FEAT_SME2, FEAT_SME2p1, FEAT_SME_B16B16, FEAT_SME_F16F16, FEAT_SME_F64F64, FEAT_SME_I16I64, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SVE_B16B16, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.4-A enables additional features: FEAT_CCIDX, FEAT_MEC, FEAT_MOPS, FEAT_NMI, FEAT_GICv3_NMI, FEAT_PRFMSLC, FEAT_RASv2, FEAT_RME, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE2p1.
    /// No SVE SVE2 isa extensions.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// Metal GPU hardware Family: apple10, metal version: metal 3 & 4.
    Apple_A19,        //  Capable

    /// Apple M5, ARMv9.4-A, ARMv9.3-A, ARMv9.2-A, ARMv9.1-A, ARMv9-A, ARMv8.9-A, ARMv8.8-A, ARMv8.7-A, ARMv8.6-A, ARMv8.5-A, ARMv8.4-A, ARMv8.3-A, ARMv8.2-A, ARMv8.1-A, ARMv8-A isa extensions, AArch64 64-bit.
    /// Enabled features: FEAT_AES, FEAT_PMULL, FEAT_AMUv1, FEAT_AMUv1p1, FEAT_AdvSIMD, FEAT_BF16, FEAT_BTI, FEAT_CRC32, FEAT_CSSC, FEAT_CSV2_2, FEAT_DIT, FEAT_DPB, FEAT_DPB2, FEAT_DotProd, FEAT_ECV, FEAT_FCMA, FEAT_FGT, FEAT_FHM, FEAT_FP, FEAT_FP16, FEAT_FPAC, FEAT_FRINTTS, FEAT_FlagM, FEAT_FlagM2, FEAT_HBC, FEAT_I8MM, FEAT_JSCVT, FEAT_LOR, FEAT_LRCPC, FEAT_LRCPC2, FEAT_LSE, FEAT_LSE2, FEAT_MPAM, FEAT_MTE, FEAT_MTE2, FEAT_NV, FEAT_NV2, FEAT_PAN, FEAT_PAN2, FEAT_PAuth, FEAT_PMUv3, FEAT_RAS, FEAT_RASv1p1, FEAT_RDM, FEAT_SB, FEAT_SEL2, FEAT_SHA1, FEAT_SHA256, FEAT_SHA3, FEAT_SHA512, FEAT_SME, FEAT_SME2, FEAT_SME2p1, FEAT_SME_B16B16, FEAT_SME_F16F16, FEAT_SME_F64F64, FEAT_SME_I16I64, FEAT_SPECRES, FEAT_SPECRES2, FEAT_SVE_B16B16, FEAT_TLBIOS, FEAT_TLBIRANGE, FEAT_TRF, FEAT_UAO, FEAT_VHE, FEAT_WFxT, FEAT_XS.
    /// Arch ARMv9.4-A enables additional features: FEAT_CCIDX, FEAT_MEC, FEAT_MOPS, FEAT_NMI, FEAT_GICv3_NMI, FEAT_PRFMSLC, FEAT_RASv2, FEAT_RME, FEAT_SPEv1p2, FEAT_SSBS, FEAT_SSBS2, FEAT_SVE, FEAT_SVE2, FEAT_SVE2p1.
    /// No SVE SVE2 isa extensions.
    /// This chip feature Apple Silicon unified memory pools and full hardware compatibility with MLX.
    /// (Features hardware-level Neural Accelerators optimized specifically for MLX matrix operations), requires metal version 4.
    /// Metal GPU hardware Family: apple10, metal version: metal 3 & 4.
    Apple_M5,         //  Capable
}

impl TargetCpuArchitectureArm64 {
    /// Returns whether this target CPU is an Apple silicon target (Apple_A10..Apple_A19, Apple_M1..Apple_M5)
    pub fn is_apple(&self) -> bool {
        matches!(
            self,
            TargetCpuArchitectureArm64::Apple_A10
                | TargetCpuArchitectureArm64::Apple_A11
                | TargetCpuArchitectureArm64::Apple_A12
                | TargetCpuArchitectureArm64::Apple_A13
                | TargetCpuArchitectureArm64::Apple_A14
                | TargetCpuArchitectureArm64::Apple_M1
                | TargetCpuArchitectureArm64::Apple_A15
                | TargetCpuArchitectureArm64::Apple_A16
                | TargetCpuArchitectureArm64::Apple_A17
                | TargetCpuArchitectureArm64::Apple_M2
                | TargetCpuArchitectureArm64::Apple_M3
                | TargetCpuArchitectureArm64::Apple_A18
                | TargetCpuArchitectureArm64::Apple_M4
                | TargetCpuArchitectureArm64::Apple_A19
                | TargetCpuArchitectureArm64::Apple_M5
        )
    }

    /// Returns whether this Arm64 target is compatible with the specified platform
    pub fn is_platform_compatible(&self, platform: Platform) -> bool {
        if self.is_apple() {
            platform.is_apple()
        } else {
            true
        }
    }
}

/// Minimum baseline CPU architecture for Arm64 code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MinimumCpuArchitectureArm64 {
    #[default]
    None,
    ARMv8_A,
    ARMv8_1A,
    ARMv8_2A,
    ARMv8_3A,
    ARMv8_4A,
    ARMv8_5A,
    ARMv8_6A,
    ARMv8_7A,
    ARMv8_8A,
    ARMv8_9A,
    ARMv8_R,
    ARMv9_A,
    ARMv9_1A,
    ARMv9_2A,
    ARMv9_3A,
    ARMv9_4A,
    ARMv9_5A,
    ARMv9_6A,
    ARMv9_7A,
}

impl fmt::Display for MinimumCpuArchitectureArm64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinimumCpuArchitectureArm64::None => write!(f, "none"),
            MinimumCpuArchitectureArm64::ARMv8_A => write!(f, "armv8-a"),
            MinimumCpuArchitectureArm64::ARMv8_1A => write!(f, "armv8.1-a"),
            MinimumCpuArchitectureArm64::ARMv8_2A => write!(f, "armv8.2-a"),
            MinimumCpuArchitectureArm64::ARMv8_3A => write!(f, "armv8.3-a"),
            MinimumCpuArchitectureArm64::ARMv8_4A => write!(f, "armv8.4-a"),
            MinimumCpuArchitectureArm64::ARMv8_5A => write!(f, "armv8.5-a"),
            MinimumCpuArchitectureArm64::ARMv8_6A => write!(f, "armv8.6-a"),
            MinimumCpuArchitectureArm64::ARMv8_7A => write!(f, "armv8.7-a"),
            MinimumCpuArchitectureArm64::ARMv8_8A => write!(f, "armv8.8-a"),
            MinimumCpuArchitectureArm64::ARMv8_9A => write!(f, "armv8.9-a"),
            MinimumCpuArchitectureArm64::ARMv8_R => write!(f, "armv8-r"),
            MinimumCpuArchitectureArm64::ARMv9_A => write!(f, "armv9-a"),
            MinimumCpuArchitectureArm64::ARMv9_1A => write!(f, "armv9.1-a"),
            MinimumCpuArchitectureArm64::ARMv9_2A => write!(f, "armv9.2-a"),
            MinimumCpuArchitectureArm64::ARMv9_3A => write!(f, "armv9.3-a"),
            MinimumCpuArchitectureArm64::ARMv9_4A => write!(f, "armv9.4-a"),
            MinimumCpuArchitectureArm64::ARMv9_5A => write!(f, "armv9.5-a"),
            MinimumCpuArchitectureArm64::ARMv9_6A => write!(f, "armv9.6-a"),
            MinimumCpuArchitectureArm64::ARMv9_7A => write!(f, "armv9.7-a"),
        }
    }
}

impl FromStr for MinimumCpuArchitectureArm64 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.trim().to_ascii_lowercase().replace('_', ".");
        match norm.as_str() {
            "none" | "default" => Ok(MinimumCpuArchitectureArm64::None),
            "armv8-a" | "armv8.a" | "armv8" | "armv8.0-a" | "armv8.0" => Ok(MinimumCpuArchitectureArm64::ARMv8_A),
            "armv8.1-a" | "armv8.1a" | "armv8.1" => Ok(MinimumCpuArchitectureArm64::ARMv8_1A),
            "armv8.2-a" | "armv8.2a" | "armv8.2" => Ok(MinimumCpuArchitectureArm64::ARMv8_2A),
            "armv8.3-a" | "armv8.3a" | "armv8.3" => Ok(MinimumCpuArchitectureArm64::ARMv8_3A),
            "armv8.4-a" | "armv8.4a" | "armv8.4" => Ok(MinimumCpuArchitectureArm64::ARMv8_4A),
            "armv8.5-a" | "armv8.5a" | "armv8.5" => Ok(MinimumCpuArchitectureArm64::ARMv8_5A),
            "armv8.6-a" | "armv8.6a" | "armv8.6" => Ok(MinimumCpuArchitectureArm64::ARMv8_6A),
            "armv8.7-a" | "armv8.7a" | "armv8.7" => Ok(MinimumCpuArchitectureArm64::ARMv8_7A),
            "armv8.8-a" | "armv8.8a" | "armv8.8" => Ok(MinimumCpuArchitectureArm64::ARMv8_8A),
            "armv8.9-a" | "armv8.9a" | "armv8.9" => Ok(MinimumCpuArchitectureArm64::ARMv8_9A),
            "armv8-r" | "armv8.r" | "armv8r" => Ok(MinimumCpuArchitectureArm64::ARMv8_R),
            "armv9-a" | "armv9.a" | "armv9" | "armv9.0-a" | "armv9.0" => Ok(MinimumCpuArchitectureArm64::ARMv9_A),
            "armv9.1-a" | "armv9.1a" | "armv9.1" => Ok(MinimumCpuArchitectureArm64::ARMv9_1A),
            "armv9.2-a" | "armv9.2a" | "armv9.2" => Ok(MinimumCpuArchitectureArm64::ARMv9_2A),
            "armv9.3-a" | "armv9.3a" | "armv9.3" => Ok(MinimumCpuArchitectureArm64::ARMv9_3A),
            "armv9.4-a" | "armv9.4a" | "armv9.4" => Ok(MinimumCpuArchitectureArm64::ARMv9_4A),
            "armv9.5-a" | "armv9.5a" | "armv9.5" => Ok(MinimumCpuArchitectureArm64::ARMv9_5A),
            "armv9.6-a" | "armv9.6a" | "armv9.6" => Ok(MinimumCpuArchitectureArm64::ARMv9_6A),
            "armv9.7-a" | "armv9.7a" | "armv9.7" => Ok(MinimumCpuArchitectureArm64::ARMv9_7A),
            other => Err(format!("Unknown MinimumCpuArchitectureArm64: '{}'", other)),
        }
    }
}

/// Mapping between TargetCpuArchitectureArm64 and string representations.
pub struct TargetCpuArchitectureArm64Names;

impl TargetCpuArchitectureArm64Names {
    pub fn name(target: TargetCpuArchitectureArm64) -> &'static str {
        match target {
            TargetCpuArchitectureArm64::None | TargetCpuArchitectureArm64::Generic => "generic",
            TargetCpuArchitectureArm64::Native => "native",
            /* TargetCpuArchitectureArm64::Cortex_A34 => "cortex-a34",
            TargetCpuArchitectureArm64::Cortex_A35 => "cortex-a35",
            TargetCpuArchitectureArm64::Cortex_A53 => "cortex-a53",
            TargetCpuArchitectureArm64::Cortex_A57 => "cortex-a57", */
            TargetCpuArchitectureArm64::Cortex_A72 => "cortex-a72",
            TargetCpuArchitectureArm64::Cortex_A73 => "cortex-a73",
            /* TargetCpuArchitectureArm64::Cyclone => "cyclone",
            TargetCpuArchitectureArm64::Apple_A7 => "apple-a7",
            TargetCpuArchitectureArm64::Apple_A8 => "apple-a8",
            TargetCpuArchitectureArm64::Apple_A9 => "apple-a9", */
            TargetCpuArchitectureArm64::Exynos_M3 => "exynos-m3",
            TargetCpuArchitectureArm64::Falkor => "falkor",
            TargetCpuArchitectureArm64::Kryo => "kryo",
            /* TargetCpuArchitectureArm64::ThunderX => "thunderx",
            TargetCpuArchitectureArm64::ThunderXT81 => "thunderxt81",
            TargetCpuArchitectureArm64::ThunderXT83 => "thunderxt83",
            TargetCpuArchitectureArm64::ThunderXT88 => "thunderxt88", */
            TargetCpuArchitectureArm64::Apple_A10 => "apple-a10",
            TargetCpuArchitectureArm64::ThunderX2T99 => "thunderx2t99",
            /* TargetCpuArchitectureArm64::Cortex_A55 => "cortex-a55",
            TargetCpuArchitectureArm64::Cortex_A65 => "cortex-a65",
            TargetCpuArchitectureArm64::Cortex_A65AE => "cortex-a65ae", */
            TargetCpuArchitectureArm64::Cortex_A75 => "cortex-a75",
            TargetCpuArchitectureArm64::Cortex_A76 => "cortex-a76",
            TargetCpuArchitectureArm64::Cortex_A76AE => "cortex-a76ae",
            TargetCpuArchitectureArm64::Cortex_A77 => "cortex-a77",
            TargetCpuArchitectureArm64::Cortex_A78 => "cortex-a78",
            TargetCpuArchitectureArm64::Cortex_A78AE => "cortex-a78ae",
            TargetCpuArchitectureArm64::Cortex_A78C => "cortex-a78c",
            TargetCpuArchitectureArm64::Switch2 => "cortex-a78c",
            TargetCpuArchitectureArm64::Cortex_X1 => "cortex-x1",
            TargetCpuArchitectureArm64::Cortex_X1C => "cortex-x1c",
            /* TargetCpuArchitectureArm64::Neoverse_E1 => "neoverse-e1", */
            TargetCpuArchitectureArm64::Neoverse_N1 => "neoverse-n1",
            TargetCpuArchitectureArm64::Graviton2 => "neoverse-n1",
            TargetCpuArchitectureArm64::Apple_A11 => "apple-a11",
            TargetCpuArchitectureArm64::Exynos_M4 => "exynos-m4",
            TargetCpuArchitectureArm64::Exynos_M5 => "exynos-m5",
            TargetCpuArchitectureArm64::TSV110 => "tsv110",
            TargetCpuArchitectureArm64::A64FX => "a64fx",
            TargetCpuArchitectureArm64::Carmel => "carmel",
            TargetCpuArchitectureArm64::Saphira => "saphira",
            TargetCpuArchitectureArm64::ThunderX3T110 => "thunderx3t110",
            TargetCpuArchitectureArm64::Apple_A12 => "apple-a12",
            /* TargetCpuArchitectureArm64::Apple_S4 => "apple-s4",
            TargetCpuArchitectureArm64::Apple_S5 => "apple-s5", */
            TargetCpuArchitectureArm64::Neoverse_V1 => "neoverse-v1",
            TargetCpuArchitectureArm64::Neoverse_512TVB => "neoverse-512tvb",
            TargetCpuArchitectureArm64::Graviton3 => "neoverse-v1",
            TargetCpuArchitectureArm64::Apple_A13 => "apple-a13",
            TargetCpuArchitectureArm64::Apple_A14 => "apple-a14",
            TargetCpuArchitectureArm64::Apple_M1 => "apple-m1",
            /* TargetCpuArchitectureArm64::Apple_S6 => "apple-s6",
            TargetCpuArchitectureArm64::Apple_S7 => "apple-s7",
            TargetCpuArchitectureArm64::Apple_S8 => "apple-s8", */
            TargetCpuArchitectureArm64::Ampere1 => "ampere1",
            TargetCpuArchitectureArm64::Ampere1A => "ampere1a",
            TargetCpuArchitectureArm64::Apple_A15 => "apple-a15",
            TargetCpuArchitectureArm64::Apple_A16 => "apple-a16",
            TargetCpuArchitectureArm64::Apple_A17 => "apple-a17",
            TargetCpuArchitectureArm64::Apple_M2 => "apple-m2",
            TargetCpuArchitectureArm64::Apple_M3 => "apple-m3",
            /* TargetCpuArchitectureArm64::Apple_S9 => "apple-s9",
            TargetCpuArchitectureArm64::Apple_S10 => "apple-s10", */
            TargetCpuArchitectureArm64::Ampere1B => "ampere1b",
            TargetCpuArchitectureArm64::Oryon_1 => "oryon-1",
            TargetCpuArchitectureArm64::Hip12 => "hip12",
            /* TargetCpuArchitectureArm64::Cortex_R82 => "cortex-r82",
            TargetCpuArchitectureArm64::Cortex_R82AE => "cortex-r82ae",
            TargetCpuArchitectureArm64::Cortex_A510 => "cortex-a510", */
            TargetCpuArchitectureArm64::Cortex_A710 => "cortex-a710",
            TargetCpuArchitectureArm64::Cortex_A715 => "cortex-a715",
            TargetCpuArchitectureArm64::Cortex_X2 => "cortex-x2",
            TargetCpuArchitectureArm64::Cortex_X3 => "cortex-x3",
            /* TargetCpuArchitectureArm64::Neoverse_E2 => "cortex-a510", */
            TargetCpuArchitectureArm64::Neoverse_N2 => "neoverse-n2",
            TargetCpuArchitectureArm64::Neoverse_V2 => "neoverse-v2",
            TargetCpuArchitectureArm64::Cobalt_100 => "cobalt-100",
            TargetCpuArchitectureArm64::Grace => "grace",
            TargetCpuArchitectureArm64::Ampere1c => "ampere1c",
            TargetCpuArchitectureArm64::Cortex_A320 => "cortex-a320",
            /* TargetCpuArchitectureArm64::Cortex_A520 => "cortex-a520",
            TargetCpuArchitectureArm64::Cortex_A520AE => "cortex-a520ae", */
            TargetCpuArchitectureArm64::Cortex_A720 => "cortex-a720",
            TargetCpuArchitectureArm64::Cortex_A720AE => "cortex-a720ae",
            TargetCpuArchitectureArm64::Cortex_A725 => "cortex-a725",
            TargetCpuArchitectureArm64::Cortex_X4 => "cortex-x4",
            TargetCpuArchitectureArm64::Cortex_X925 => "cortex-x925",
            /* TargetCpuArchitectureArm64::Neoverse_E3 => "cortex-a520", */
            TargetCpuArchitectureArm64::Neoverse_N3 => "neoverse-n3",
            TargetCpuArchitectureArm64::Neoverse_V3 => "neoverse-v3",
            TargetCpuArchitectureArm64::Neoverse_V3AE => "neoverse-v3ae",
            TargetCpuArchitectureArm64::Armagicpu => "armagicpu",
            TargetCpuArchitectureArm64::Apple_A18 => "apple-a18",
            TargetCpuArchitectureArm64::Apple_M4 => "apple-m4",
            TargetCpuArchitectureArm64::Gb10 => "gb10",
            TargetCpuArchitectureArm64::Olympus => "olympus",
            TargetCpuArchitectureArm64::Rigel => "rigel",
            TargetCpuArchitectureArm64::Fujitsu_Monaka => "fujitsu-monaka",
            TargetCpuArchitectureArm64::C1_Nano => "c1-nano",
            TargetCpuArchitectureArm64::C1_Premium => "c1-premium",
            TargetCpuArchitectureArm64::C1_Pro => "c1-pro",
            TargetCpuArchitectureArm64::C1_Ultra => "c1-ultra",
            TargetCpuArchitectureArm64::Apple_A19 => "apple-a19",
            TargetCpuArchitectureArm64::Apple_M5 => "apple-m5",
        }
    }
}

impl FromStr for TargetCpuArchitectureArm64 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.to_ascii_lowercase().replace(['_', '.'], "-");
        match norm.as_str() {
            "generic" | "none" | "default" => Ok(TargetCpuArchitectureArm64::Generic),
            "native" => Ok(TargetCpuArchitectureArm64::Native),
            /* "cortex-a34" => Ok(TargetCpuArchitectureArm64::Cortex_A34),
            "cortex-a35" => Ok(TargetCpuArchitectureArm64::Cortex_A35),
            "cortex-a53" => Ok(TargetCpuArchitectureArm64::Cortex_A53),
            "cortex-a57" => Ok(TargetCpuArchitectureArm64::Cortex_A57), */
            "cortex-a72" => Ok(TargetCpuArchitectureArm64::Cortex_A72),
            "cortex-a73" => Ok(TargetCpuArchitectureArm64::Cortex_A73),
            /* "cyclone" => Ok(TargetCpuArchitectureArm64::Cyclone),
            "apple-a7" => Ok(TargetCpuArchitectureArm64::Apple_A7),
            "apple-a8" => Ok(TargetCpuArchitectureArm64::Apple_A8),
            "apple-a9" => Ok(TargetCpuArchitectureArm64::Apple_A9), */
            "exynos-m3" => Ok(TargetCpuArchitectureArm64::Exynos_M3),
            "falkor" => Ok(TargetCpuArchitectureArm64::Falkor),
            "kryo" => Ok(TargetCpuArchitectureArm64::Kryo),
            /* "thunderx" => Ok(TargetCpuArchitectureArm64::ThunderX),
            "thunderxt81" => Ok(TargetCpuArchitectureArm64::ThunderXT81),
            "thunderxt83" => Ok(TargetCpuArchitectureArm64::ThunderXT83),
            "thunderxt88" => Ok(TargetCpuArchitectureArm64::ThunderXT88), */
            "apple-a10" => Ok(TargetCpuArchitectureArm64::Apple_A10),
            "thunderx2t99" => Ok(TargetCpuArchitectureArm64::ThunderX2T99),
            /* "cortex-a55" => Ok(TargetCpuArchitectureArm64::Cortex_A55),
            "cortex-a65" => Ok(TargetCpuArchitectureArm64::Cortex_A65),
            "cortex-a65ae" => Ok(TargetCpuArchitectureArm64::Cortex_A65AE), */
            "cortex-a75" => Ok(TargetCpuArchitectureArm64::Cortex_A75),
            "cortex-a76" => Ok(TargetCpuArchitectureArm64::Cortex_A76),
            "cortex-a76ae" => Ok(TargetCpuArchitectureArm64::Cortex_A76AE),
            "cortex-a77" => Ok(TargetCpuArchitectureArm64::Cortex_A77),
            "cortex-a78" => Ok(TargetCpuArchitectureArm64::Cortex_A78),
            "cortex-a78ae" => Ok(TargetCpuArchitectureArm64::Cortex_A78AE),
            "cortex-a78c" => Ok(TargetCpuArchitectureArm64::Cortex_A78C),
            "nx2" | "switch2" => Ok(TargetCpuArchitectureArm64::Switch2),
            "cortex-x1" => Ok(TargetCpuArchitectureArm64::Cortex_X1),
            "cortex-x1c" => Ok(TargetCpuArchitectureArm64::Cortex_X1C),
            /* "neoverse-e1" => Ok(TargetCpuArchitectureArm64::Neoverse_E1), */
            "neoverse-n1" => Ok(TargetCpuArchitectureArm64::Neoverse_N1),
            "graviton2" => Ok(TargetCpuArchitectureArm64::Graviton2),
            "apple-a11" => Ok(TargetCpuArchitectureArm64::Apple_A11),
            "exynos-m4" => Ok(TargetCpuArchitectureArm64::Exynos_M4),
            "exynos-m5" => Ok(TargetCpuArchitectureArm64::Exynos_M5),
            "tsv110" => Ok(TargetCpuArchitectureArm64::TSV110),
            "a64fx" => Ok(TargetCpuArchitectureArm64::A64FX),
            "carmel" => Ok(TargetCpuArchitectureArm64::Carmel),
            "saphira" => Ok(TargetCpuArchitectureArm64::Saphira),
            "thunderx3t110" => Ok(TargetCpuArchitectureArm64::ThunderX3T110),
            "apple-a12" => Ok(TargetCpuArchitectureArm64::Apple_A12),
            /* "apple-s4" => Ok(TargetCpuArchitectureArm64::Apple_S4),
            "apple-s5" => Ok(TargetCpuArchitectureArm64::Apple_S5), */
            "neoverse-v1" => Ok(TargetCpuArchitectureArm64::Neoverse_V1),
            "neoverse-512tvb" => Ok(TargetCpuArchitectureArm64::Neoverse_512TVB),
            "graviton3" => Ok(TargetCpuArchitectureArm64::Graviton3),
            "apple-a13" => Ok(TargetCpuArchitectureArm64::Apple_A13),
            "apple-a14" => Ok(TargetCpuArchitectureArm64::Apple_A14),
            "apple-m1" => Ok(TargetCpuArchitectureArm64::Apple_M1),
            /* "apple-s6" => Ok(TargetCpuArchitectureArm64::Apple_S6),
            "apple-s7" => Ok(TargetCpuArchitectureArm64::Apple_S7),
            "apple-s8" => Ok(TargetCpuArchitectureArm64::Apple_S8), */
            "ampere1" => Ok(TargetCpuArchitectureArm64::Ampere1),
            "ampere1a" => Ok(TargetCpuArchitectureArm64::Ampere1A),
            "apple-a15" => Ok(TargetCpuArchitectureArm64::Apple_A15),
            "apple-a16" => Ok(TargetCpuArchitectureArm64::Apple_A16),
            "apple-a17" => Ok(TargetCpuArchitectureArm64::Apple_A17),
            "apple-m2" => Ok(TargetCpuArchitectureArm64::Apple_M2),
            "apple-m3" => Ok(TargetCpuArchitectureArm64::Apple_M3),
            /* "apple-s9" => Ok(TargetCpuArchitectureArm64::Apple_S9),
            "apple-s10" => Ok(TargetCpuArchitectureArm64::Apple_S10), */
            "ampere1b" => Ok(TargetCpuArchitectureArm64::Ampere1B),
            "oryon-1" | "oryon1" => Ok(TargetCpuArchitectureArm64::Oryon_1),
            "hip12" => Ok(TargetCpuArchitectureArm64::Hip12),
            /* "cortex-r82" => Ok(TargetCpuArchitectureArm64::Cortex_R82),
            "cortex-r82ae" => Ok(TargetCpuArchitectureArm64::Cortex_R82AE),
            "cortex-a510" => Ok(TargetCpuArchitectureArm64::Cortex_A510), */
            "cortex-a710" => Ok(TargetCpuArchitectureArm64::Cortex_A710),
            "cortex-a715" => Ok(TargetCpuArchitectureArm64::Cortex_A715),
            "cortex-x2" => Ok(TargetCpuArchitectureArm64::Cortex_X2),
            "cortex-x3" => Ok(TargetCpuArchitectureArm64::Cortex_X3),
            /* "neoverse-e2" => Ok(TargetCpuArchitectureArm64::Neoverse_E2), */
            "neoverse-n2" => Ok(TargetCpuArchitectureArm64::Neoverse_N2),
            "neoverse-v2" => Ok(TargetCpuArchitectureArm64::Neoverse_V2),
            "cobalt-100" => Ok(TargetCpuArchitectureArm64::Cobalt_100),
            "grace" => Ok(TargetCpuArchitectureArm64::Grace),
            "ampere1c" => Ok(TargetCpuArchitectureArm64::Ampere1c),
            "cortex-a320" => Ok(TargetCpuArchitectureArm64::Cortex_A320),
            /* "cortex-a520" => Ok(TargetCpuArchitectureArm64::Cortex_A520),
            "cortex-a520ae" => Ok(TargetCpuArchitectureArm64::Cortex_A520AE), */
            "cortex-a720" => Ok(TargetCpuArchitectureArm64::Cortex_A720),
            "cortex-a720ae" => Ok(TargetCpuArchitectureArm64::Cortex_A720AE),
            "cortex-a725" => Ok(TargetCpuArchitectureArm64::Cortex_A725),
            "cortex-x4" => Ok(TargetCpuArchitectureArm64::Cortex_X4),
            "cortex-x925" => Ok(TargetCpuArchitectureArm64::Cortex_X925),
            /* "neoverse-e3" => Ok(TargetCpuArchitectureArm64::Neoverse_E3), */
            "neoverse-n3" => Ok(TargetCpuArchitectureArm64::Neoverse_N3),
            "neoverse-v3" => Ok(TargetCpuArchitectureArm64::Neoverse_V3),
            "neoverse-v3ae" => Ok(TargetCpuArchitectureArm64::Neoverse_V3AE),
            "armagicpu" => Ok(TargetCpuArchitectureArm64::Armagicpu),
            "apple-a18" => Ok(TargetCpuArchitectureArm64::Apple_A18),
            "apple-m4" => Ok(TargetCpuArchitectureArm64::Apple_M4),
            "gb10" => Ok(TargetCpuArchitectureArm64::Gb10),
            "olympus" => Ok(TargetCpuArchitectureArm64::Olympus),
            "rigel" => Ok(TargetCpuArchitectureArm64::Rigel),
            "fujitsu-monaka" => Ok(TargetCpuArchitectureArm64::Fujitsu_Monaka),
            "c1-nano" => Ok(TargetCpuArchitectureArm64::C1_Nano),
            "c1-premium" => Ok(TargetCpuArchitectureArm64::C1_Premium),
            "c1-pro" => Ok(TargetCpuArchitectureArm64::C1_Pro),
            "c1-ultra" => Ok(TargetCpuArchitectureArm64::C1_Ultra),
            "apple-a19" => Ok(TargetCpuArchitectureArm64::Apple_A19),
            "apple-m5" => Ok(TargetCpuArchitectureArm64::Apple_M5),
            other => Err(format!("Unknown Arm64 target CPU: {}", other)),
        }
    }
}

