// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/names.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use super::isa::Arm64ISA;
use super::targets::{MinimumCpuArchitectureArm64, TargetCpuArchitectureArm64};

/// Provides a mapping between the MinimumCpuArchitectureArm64 enum values and their Clang target arch string representations.
pub struct MinimumCpuArchitectureArm64ClangNames;

impl MinimumCpuArchitectureArm64ClangNames {
    pub fn name(min_arch: MinimumCpuArchitectureArm64) -> &'static str {
        match min_arch {
            MinimumCpuArchitectureArm64::None => "armv8-a",
            MinimumCpuArchitectureArm64::ARMv8_A => "armv8-a",
            MinimumCpuArchitectureArm64::ARMv8_1A => "armv8.1-a",
            MinimumCpuArchitectureArm64::ARMv8_2A => "armv8.2-a",
            MinimumCpuArchitectureArm64::ARMv8_3A => "armv8.3-a",
            MinimumCpuArchitectureArm64::ARMv8_4A => "armv8.4-a",
            MinimumCpuArchitectureArm64::ARMv8_5A => "armv8.5-a",
            MinimumCpuArchitectureArm64::ARMv8_6A => "armv8.6-a",
            MinimumCpuArchitectureArm64::ARMv8_7A => "armv8.7-a",
            MinimumCpuArchitectureArm64::ARMv8_8A => "armv8.8-a",
            MinimumCpuArchitectureArm64::ARMv8_9A => "armv8.9-a",
            MinimumCpuArchitectureArm64::ARMv8_R => "armv8-r",
            MinimumCpuArchitectureArm64::ARMv9_A => "armv9-a",
            MinimumCpuArchitectureArm64::ARMv9_1A => "armv9.1-a",
            MinimumCpuArchitectureArm64::ARMv9_2A => "armv9.2-a",
            MinimumCpuArchitectureArm64::ARMv9_3A => "armv9.3-a",
            MinimumCpuArchitectureArm64::ARMv9_4A => "armv9.4-a",
            MinimumCpuArchitectureArm64::ARMv9_5A => "armv9.5-a",
            MinimumCpuArchitectureArm64::ARMv9_6A => "armv9.6-a",
            MinimumCpuArchitectureArm64::ARMv9_7A => "armv9.7-a",
        }
    }
}

/// Provides a mapping between the MinimumCpuArchitectureArm64 enum values and their MSVC arch string representations.
pub struct MinimumCpuArchitectureArm64MSVCNames;

impl MinimumCpuArchitectureArm64MSVCNames {
    pub fn name(min_arch: MinimumCpuArchitectureArm64) -> &'static str {
        match min_arch {
            MinimumCpuArchitectureArm64::None | MinimumCpuArchitectureArm64::ARMv8_A => "armv8.0",
            MinimumCpuArchitectureArm64::ARMv8_1A => "armv8.1",
            MinimumCpuArchitectureArm64::ARMv8_2A => "armv8.2",
            MinimumCpuArchitectureArm64::ARMv8_3A => "armv8.3",
            MinimumCpuArchitectureArm64::ARMv8_4A => "armv8.4",
            MinimumCpuArchitectureArm64::ARMv8_5A => "armv8.5",
            MinimumCpuArchitectureArm64::ARMv8_6A => "armv8.6",
            MinimumCpuArchitectureArm64::ARMv8_7A => "armv8.7",
            MinimumCpuArchitectureArm64::ARMv8_8A => "armv8.8",
            MinimumCpuArchitectureArm64::ARMv8_9A => "armv8.9",
            MinimumCpuArchitectureArm64::ARMv8_R => "armv8.4",
            MinimumCpuArchitectureArm64::ARMv9_A => "armv9.0",
            MinimumCpuArchitectureArm64::ARMv9_1A => "armv9.1",
            MinimumCpuArchitectureArm64::ARMv9_2A => "armv9.2",
            MinimumCpuArchitectureArm64::ARMv9_3A => "armv9.3",
            MinimumCpuArchitectureArm64::ARMv9_4A
            | MinimumCpuArchitectureArm64::ARMv9_5A
            | MinimumCpuArchitectureArm64::ARMv9_6A
            | MinimumCpuArchitectureArm64::ARMv9_7A => "armv9.4",
        }
    }
}

/// Provides a mapping between the TargetCpuArchitectureArm64 enum values and their default enabled Arm64ISA enum extensions string representations for Clang.
pub struct ClangTargetCpuArchitectureArm64ISANames;

impl ClangTargetCpuArchitectureArm64ISANames {
    pub fn name(target: TargetCpuArchitectureArm64) -> &'static str {
        match target {
            TargetCpuArchitectureArm64::None | TargetCpuArchitectureArm64::Generic => "crypto+aes+crc+fp+simd",
            TargetCpuArchitectureArm64::Native => "",
            // AArch64.v8 A Profile
            // ARMv8-A
            /* TargetCpuArchitectureArm64::Cortex_A34
            | TargetCpuArchitectureArm64::Cortex_A35
            | TargetCpuArchitectureArm64::Cortex_A53
            | TargetCpuArchitectureArm64::Cortex_A57
            | */ TargetCpuArchitectureArm64::Cortex_A72
            | TargetCpuArchitectureArm64::Cortex_A73 => "crypto+aes+crc+fp+pmuv3+sha2+simd",
            /* TargetCpuArchitectureArm64::Cyclone
            | TargetCpuArchitectureArm64::Apple_A7
            | TargetCpuArchitectureArm64::Apple_A8
            | TargetCpuArchitectureArm64::Apple_A9 => "crypto+aes+fp+pmuv3+sha2+simd", */
            TargetCpuArchitectureArm64::Exynos_M3 => "crypto+aes+crc+fp+pmuv3+sha2+simd",
            TargetCpuArchitectureArm64::Falkor => "crypto+aes+crc+fp+pmuv3+rdm+sha2+simd",
            TargetCpuArchitectureArm64::Kryo
            /* | TargetCpuArchitectureArm64::ThunderX
            | TargetCpuArchitectureArm64::ThunderXT81
            | TargetCpuArchitectureArm64::ThunderXT83
            | TargetCpuArchitectureArm64::ThunderXT88 */ => "crypto+aes+crc+fp+pmuv3+sha2+simd",
            // ARMv8.1-A
            TargetCpuArchitectureArm64::Apple_A10 => "crypto+aes+crc+fp+pmuv3+rdm+sha2+simd",
            TargetCpuArchitectureArm64::ThunderX2T99 => "crypto+aes+crc+fp+lse+rdm+sha2+simd",
            // ARMv8.2-A
            /* TargetCpuArchitectureArm64::Cortex_A55 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+ras+rcpc+rdm+sha2+simd",
            TargetCpuArchitectureArm64::Cortex_A65
            | TargetCpuArchitectureArm64::Cortex_A65AE => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+ras+rcpc+rdm+sha2+simd+ssbs", */
            TargetCpuArchitectureArm64::Cortex_A75 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+ras+rcpc+rdm+sha2+simd",
            TargetCpuArchitectureArm64::Cortex_A76
            | TargetCpuArchitectureArm64::Cortex_A76AE
            | TargetCpuArchitectureArm64::Cortex_A77 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+ras+rcpc+rdm+sha2+simd+ssbs",
            TargetCpuArchitectureArm64::Cortex_A78
            | TargetCpuArchitectureArm64::Cortex_A78AE => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
            TargetCpuArchitectureArm64::Cortex_A78C => "crypto+aes+crc+dotprod+flagm+fp+fp16+lse+pauth+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
            TargetCpuArchitectureArm64::Switch2 => "crypto+aes+crc+dotprod+flagm+fp+fp16+lse+pauth+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
            TargetCpuArchitectureArm64::Cortex_X1 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
            TargetCpuArchitectureArm64::Cortex_X1C => "crypto+aes+crc+dotprod+flagm+fp+fp16+lse+pauth+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
            /* TargetCpuArchitectureArm64::Neoverse_E1 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+ras+rcpc+rdm+sha2+simd+ssbs", */
            TargetCpuArchitectureArm64::Neoverse_N1
            | TargetCpuArchitectureArm64::Graviton2 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+profile+ras+rcpc+rdm+sha2+simd+ssbs",
            TargetCpuArchitectureArm64::Apple_A11 => "crypto+aes+crc+fp+fp16+lse+pmuv3+ras+rdm+sha2+simd",
            TargetCpuArchitectureArm64::Exynos_M4
            | TargetCpuArchitectureArm64::Exynos_M5 => "crypto+aes+crc+dotprod+fp+fp16+lse+pmuv3+ras+rdm+sha2+simd",
            TargetCpuArchitectureArm64::TSV110 => "crypto+aes+crc+dotprod+fcma+fp+fp16+fp16fml+jscvt+lse+pmuv3+profile+ras+rdm+sha2+simd",
            TargetCpuArchitectureArm64::A64FX => "crypto+aes+crc+fcma+fp+fp16+lse+pmuv3+ras+rdm+sha2+simd+sve",
            TargetCpuArchitectureArm64::Carmel => "crypto+aes+crc+fp+fp16+lse+ras+rdm+sha2+simd",
            // ARMv8.3-A
            TargetCpuArchitectureArm64::Saphira => "crypto+aes+crc+dit+dotprod+fcma+flagm+fp+jscvt+lse+pauth+pmuv3+profile+ras+rcpc+rdm+sha2+simd",
            TargetCpuArchitectureArm64::ThunderX3T110 => "crypto+aes+crc+fcma+fp+jscvt+lse+pauth+pmuv3+ras+rcpc+rdm+sha2+simd",
            TargetCpuArchitectureArm64::Apple_A12
            /* | TargetCpuArchitectureArm64::Apple_S4
            | TargetCpuArchitectureArm64::Apple_S5 */ => "crypto+aes+crc+fcma+fp+fp16+jscvt+lse+pauth+pmuv3+ras+rcpc+rdm+sha2+simd",
            // ARMv8.4-A
            TargetCpuArchitectureArm64::Neoverse_V1
            | TargetCpuArchitectureArm64::Neoverse_512TVB
            | TargetCpuArchitectureArm64::Graviton3 => "crypto+aes+bf16+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+profile+ras+rcpc+rdm+rng+sha2+sha3+simd+sm4+ssbs+sve",
            TargetCpuArchitectureArm64::Apple_A13 => "crypto+aes+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+jscvt+lse+pauth+pmuv3+ras+rcpc+rdm+sha2+sha3+simd",
            TargetCpuArchitectureArm64::Apple_A14
            | TargetCpuArchitectureArm64::Apple_M1 => "crypto+aes+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+sb+sha2+sha3+simd+ssbs",
            /* TargetCpuArchitectureArm64::Apple_S6
            | TargetCpuArchitectureArm64::Apple_S7
            | TargetCpuArchitectureArm64::Apple_S8 => "crypto+aes+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+jscvt+lse+pauth+pmuv3+ras+rcpc+rdm+sha2+sha3+simd", */
            // ARMv8.6-A
            TargetCpuArchitectureArm64::Ampere1 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+ssbs",
            TargetCpuArchitectureArm64::Ampere1A => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs",
            TargetCpuArchitectureArm64::Apple_A15
            | TargetCpuArchitectureArm64::Apple_A16
            | TargetCpuArchitectureArm64::Apple_A17
            | TargetCpuArchitectureArm64::Apple_M2
            | TargetCpuArchitectureArm64::Apple_M3
            /* | TargetCpuArchitectureArm64::Apple_S9
            | TargetCpuArchitectureArm64::Apple_S10 */ => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+sb+sha2+sha3+simd+ssbs",
            // ARMv8.7-A
            TargetCpuArchitectureArm64::Ampere1B => "crypto+aes+bf16+bti+crc+cssc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs+wfxt",
            TargetCpuArchitectureArm64::Oryon_1 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+predres+profile+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs",
            TargetCpuArchitectureArm64::Hip12 => "crypto+aes+bf16+brbe+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+hbc+i8mm+jscvt+ls64+lse+pauth+pmuv3+predres+profile+ras+rcpc+rcpc3+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs+sve+sve_aes+sve_bitperm+sve_sha3+sve_sm4+sve2+wfxt",
            // AArch64.v8 R Profile
            // ARMv8-R
            /* TargetCpuArchitectureArm64::Cortex_R82
            | TargetCpuArchitectureArm64::Cortex_R82AE => "crypto+aes+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs", */
            // AArch64.v9 A Profile
            // ARMv9-A
            /* TargetCpuArchitectureArm64::Cortex_A510
            | */ TargetCpuArchitectureArm64::Cortex_A710 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2",
            TargetCpuArchitectureArm64::Cortex_A715 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2",
            TargetCpuArchitectureArm64::Cortex_X2 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2",
            TargetCpuArchitectureArm64::Cortex_X3 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2",
            /* TargetCpuArchitectureArm64::Neoverse_E2
            | */ TargetCpuArchitectureArm64::Neoverse_N2 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2",
            TargetCpuArchitectureArm64::Neoverse_V2 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+rng+sb+simd+ssbs+sve+sve_bitperm+sve2",
            TargetCpuArchitectureArm64::Cobalt_100 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2",
            TargetCpuArchitectureArm64::Grace => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+sb+sha2+sha3+simd+sm4+ssbs+sve+sve_aes+sve_bitperm+sve_sha3+sve_sm4+sve2",
            // ARMv9.2-A
            TargetCpuArchitectureArm64::Ampere1c => "crypto+aes+bf16+bti+crc+cssc+dit+dotprod+faminmax+fcma+flagm+fp+fp16+fp16fml+fp8+fp8fma+i8mm+jscvt+lse+lut+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs+sve+sve_aes+sve_b16b16+sve_sha3+sve_sm4+sve2+wfxt",
            TargetCpuArchitectureArm64::Cortex_A320
            /* | TargetCpuArchitectureArm64::Cortex_A520
            | TargetCpuArchitectureArm64::Cortex_A520AE */ => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2+wfxt",
            TargetCpuArchitectureArm64::Cortex_A720
            | TargetCpuArchitectureArm64::Cortex_A720AE
            | TargetCpuArchitectureArm64::Cortex_A725
            | TargetCpuArchitectureArm64::Cortex_X4
            | TargetCpuArchitectureArm64::Cortex_X925 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2+wfxt",
            /* TargetCpuArchitectureArm64::Neoverse_E3 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+ras+rcpc+rdm+sb+simd+ssbs+sve+sve_bitperm+sve2+wfxt", */
            TargetCpuArchitectureArm64::Neoverse_N3 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+rng+sb+simd+ssbs+sve+sve_bitperm+sve2+wfxt",
            TargetCpuArchitectureArm64::Neoverse_V3
            | TargetCpuArchitectureArm64::Neoverse_V3AE => "crypto+aes+bf16+brbe+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+ls64+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+rng+sb+simd+ssbs+sve+sve_bitperm+sve2+wfxt",
            TargetCpuArchitectureArm64::Armagicpu => "bf16+brbe+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+ls64+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+rng+sb+simd+ssbs+sve+sve_bitperm+sve2+wfxt",
            TargetCpuArchitectureArm64::Apple_A18 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+sb+sha2+sha3+simd+sme+sme_f64f64+sme_i16i64+sme2+wfxt",
            TargetCpuArchitectureArm64::Apple_M4 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+sb+sha2+sha3+simd+sme+sme_f64f64+sme_i16i64+sme2+wfxt",
            TargetCpuArchitectureArm64::Gb10 => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+sb+sha2+sha3+simd+sm4+ssbs+sve+sve_aes+sve_bitperm+sve_sha3+sve_sm4+sve2+wfxt",
            TargetCpuArchitectureArm64::Olympus
            | TargetCpuArchitectureArm64::Rigel => "crypto+aes+bf16+brbe+bti+crc+dit+dotprod+faminmax+fcma+flagm+fp+fp16+fp16fml+fp8+fp8dot2+fp8dot4+fp8fma+i8mm+jscvt+ls64+lse+lut+memtag+pauth+pmuv3+predres+profile+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs+sve+sve_aes+sve_bitperm+sve_sha3+sve_sm4+sve2+wfxt",
            // ARMv9.3-A
            TargetCpuArchitectureArm64::Fujitsu_Monaka => "crypto+aes+bf16+bti+crc+dit+dotprod+faminmax+fcma+flagm+fp+fp16+fp16fml+fp8+fp8dot2+fp8dot4+fp8fma+hbc+i8mm+jscvt+ls64+lse+lut+mops+pauth+pmuv3+predres+predres2+ras+rcpc+rdm+rng+sb+sha2+sha3+simd+sm4+ssbs+sve+sve_aes+sve_bitperm+sve_sha3+sve_sm4+sve2+wfxt",
            TargetCpuArchitectureArm64::C1_Nano => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+hbc+i8mm+jscvt+lse+memtag+mops+pauth+pmuv3+predres+predres2+ras+rcpc+rcpc3+rdm+sb+simd+sme+sme2+ssbs+sve+sve_bitperm+sve2+wfxt",
            TargetCpuArchitectureArm64::C1_Premium
            | TargetCpuArchitectureArm64::C1_Pro
            | TargetCpuArchitectureArm64::C1_Ultra => "crypto+aes+bf16+bti+crc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+hbc+i8mm+jscvt+lse+memtag+mops+pauth+pmuv3+predres+predres2+profile+ras+rcpc+rcpc3+rdm+sb+simd+sme+sme2+ssbs+sve+sve_bitperm+sve2+wfxt",
            TargetCpuArchitectureArm64::Apple_A19 => "crypto+aes+bf16+bti+crc+cssc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+hbc+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+predres2+ras+rcpc+rdm+sb+sha2+sha3+simd+sme+sme_b16b16+sme_f16f16+sme_f64f64+sme_i16i64+sme2+sme2p1+sve_b16b16+wfxt",
            TargetCpuArchitectureArm64::Apple_M5 => "crypto+aes+bf16+bti+crc+cssc+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+hbc+i8mm+jscvt+lse+memtag+pauth+pmuv3+predres+predres2+ras+rcpc+rdm+sb+sha2+sha3+simd+sme+sme_b16b16+sme_f16f16+sme_f64f64+sme_i16i64+sme2+sme2p1+sve_b16b16+wfxt",
        }
    }
}

/// Provides a mapping between the TargetCpuArchitectureArm64 enum values and their default disabled Arm64ISA enum extensions string representations for Clang.
pub struct ClangTargetCpuArchitectureArm64NOISANames;

impl ClangTargetCpuArchitectureArm64NOISANames {
    pub fn name(target: TargetCpuArchitectureArm64) -> &'static str {
        match target {
            TargetCpuArchitectureArm64::Apple_A18
            | TargetCpuArchitectureArm64::Apple_M4 => "sve+sve2",
            | TargetCpuArchitectureArm64::Apple_A19
            | TargetCpuArchitectureArm64::Apple_M5 => "sve+sve2+sve2p1",
            _ => "",
        }
    }
}

/// Provides names for Arm64 ISA instructions to enable for MSVC.
pub struct MSVCTargetCpuArchitectureArm64ISANames;

impl MSVCTargetCpuArchitectureArm64ISANames {
    pub fn name(target: TargetCpuArchitectureArm64) -> &'static str {
        match target {
            TargetCpuArchitectureArm64::None | TargetCpuArchitectureArm64::Generic => "",
            TargetCpuArchitectureArm64::Native => "",
            // AArch64.v8 A Profile
            // ARMv8-A
            /* TargetCpuArchitectureArm64::Cortex_A34
            | TargetCpuArchitectureArm64::Cortex_A35
            | TargetCpuArchitectureArm64::Cortex_A53
            | TargetCpuArchitectureArm64::Cortex_A57
            | */ TargetCpuArchitectureArm64::Cortex_A72
            | TargetCpuArchitectureArm64::Cortex_A73 => "",
            /* TargetCpuArchitectureArm64::Cyclone
            | TargetCpuArchitectureArm64::Apple_A7
            | TargetCpuArchitectureArm64::Apple_A8
            | TargetCpuArchitectureArm64::Apple_A9 => "", */
            TargetCpuArchitectureArm64::Exynos_M3 => "",
            TargetCpuArchitectureArm64::Falkor => "",
            TargetCpuArchitectureArm64::Kryo
            /* | TargetCpuArchitectureArm64::ThunderX
            | TargetCpuArchitectureArm64::ThunderXT81
            | TargetCpuArchitectureArm64::ThunderXT83
            | TargetCpuArchitectureArm64::ThunderXT88 */ => "",
            // ARMv8.1-A
            TargetCpuArchitectureArm64::Apple_A10 => "",
            TargetCpuArchitectureArm64::ThunderX2T99 => "lse",
            // ARMv8.2-A
            /* TargetCpuArchitectureArm64::Cortex_A55 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A65
            | TargetCpuArchitectureArm64::Cortex_A65AE => "lse+rcpc", */
            TargetCpuArchitectureArm64::Cortex_A75 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A76
            | TargetCpuArchitectureArm64::Cortex_A76AE
            | TargetCpuArchitectureArm64::Cortex_A77 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A78
            | TargetCpuArchitectureArm64::Cortex_A78AE => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A78C => "lse+rcpc",
            TargetCpuArchitectureArm64::Switch2 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_X1 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_X1C => "lse+rcpc",
            /* TargetCpuArchitectureArm64::Neoverse_E1 => "lse+rcpc", */
            TargetCpuArchitectureArm64::Neoverse_N1
            | TargetCpuArchitectureArm64::Graviton2 => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_A11 => "lse",
            TargetCpuArchitectureArm64::Exynos_M4
            | TargetCpuArchitectureArm64::Exynos_M5 => "lse",
            TargetCpuArchitectureArm64::TSV110 => "lse",
            TargetCpuArchitectureArm64::A64FX => "lse",
            TargetCpuArchitectureArm64::Carmel => "lse",
            // ARMv8.3-A
            TargetCpuArchitectureArm64::Saphira => "lse+rcpc",
            TargetCpuArchitectureArm64::ThunderX3T110 => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_A12
            /* | TargetCpuArchitectureArm64::Apple_S4
            | TargetCpuArchitectureArm64::Apple_S5 */ => "lse+rcpc",
            // ARMv8.4-A
            TargetCpuArchitectureArm64::Neoverse_V1
            | TargetCpuArchitectureArm64::Neoverse_512TVB
            | TargetCpuArchitectureArm64::Graviton3 => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_A13 => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_A14
            | TargetCpuArchitectureArm64::Apple_M1 => "lse+rcpc",
            /* TargetCpuArchitectureArm64::Apple_S6
            | TargetCpuArchitectureArm64::Apple_S7
            | TargetCpuArchitectureArm64::Apple_S8 => "lse+rcpc", */
            // ARMv8.6-A
            TargetCpuArchitectureArm64::Ampere1 => "lse+rcpc",
            TargetCpuArchitectureArm64::Ampere1A => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_A15
            | TargetCpuArchitectureArm64::Apple_A16
            | TargetCpuArchitectureArm64::Apple_A17
            | TargetCpuArchitectureArm64::Apple_M2
            | TargetCpuArchitectureArm64::Apple_M3
            /* | TargetCpuArchitectureArm64::Apple_S9
            | TargetCpuArchitectureArm64::Apple_S10 */ => "lse+rcpc",
            // ARMv8.7-A
            TargetCpuArchitectureArm64::Ampere1B => "cssc+lse+rcpc",
            TargetCpuArchitectureArm64::Oryon_1 => "lse+rcpc",
            TargetCpuArchitectureArm64::Hip12 => "lse+rcpc+rcpc3",
            // AArch64.v8 R Profile
            // ARMv8-R
            /* TargetCpuArchitectureArm64::Cortex_R82
            | TargetCpuArchitectureArm64::Cortex_R82AE => "lse+rcpc", */
            // AArch64.v9 A Profile
            // ARMv9-A
            /* TargetCpuArchitectureArm64::Cortex_A510
            | */ TargetCpuArchitectureArm64::Cortex_A710 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A715 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_X2 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_X3 => "lse+rcpc",
            /* TargetCpuArchitectureArm64::Neoverse_E2
            | */ TargetCpuArchitectureArm64::Neoverse_N2 => "lse+rcpc",
            TargetCpuArchitectureArm64::Neoverse_V2 => "lse+rcpc",
            TargetCpuArchitectureArm64::Cobalt_100 => "lse+rcpc",
            TargetCpuArchitectureArm64::Grace => "lse+rcpc",
            // ARMv9.2-A
            TargetCpuArchitectureArm64::Ampere1c => "faminmax+cssc+lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A320
            /* | TargetCpuArchitectureArm64::Cortex_A520
            | TargetCpuArchitectureArm64::Cortex_A520AE */ => "lse+rcpc",
            TargetCpuArchitectureArm64::Cortex_A720
            | TargetCpuArchitectureArm64::Cortex_A720AE
            | TargetCpuArchitectureArm64::Cortex_A725
            | TargetCpuArchitectureArm64::Cortex_X4
            | TargetCpuArchitectureArm64::Cortex_X925 => "lse+rcpc",
            /* TargetCpuArchitectureArm64::Neoverse_E3 => "lse+rcpc", */
            TargetCpuArchitectureArm64::Neoverse_N3 => "lse+rcpc",
            TargetCpuArchitectureArm64::Neoverse_V3
            | TargetCpuArchitectureArm64::Neoverse_V3AE => "lse+rcpc",
            TargetCpuArchitectureArm64::Armagicpu => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_A18 => "lse+rcpc",
            TargetCpuArchitectureArm64::Apple_M4 => "lse+rcpc",
            TargetCpuArchitectureArm64::Gb10 => "lse+rcpc",
            TargetCpuArchitectureArm64::Olympus
            | TargetCpuArchitectureArm64::Rigel => "faminmax+lse+rcpc",
            // ARMv9.3-A
            TargetCpuArchitectureArm64::Fujitsu_Monaka => "faminmax+lse+rcpc",
            TargetCpuArchitectureArm64::C1_Nano => "lse+rcpc+rcpc3",
            TargetCpuArchitectureArm64::C1_Premium
            | TargetCpuArchitectureArm64::C1_Pro
            | TargetCpuArchitectureArm64::C1_Ultra => "lse+rcpc+rcpc3",
            TargetCpuArchitectureArm64::Apple_A19 => "cssc+lse+rcpc",
            TargetCpuArchitectureArm64::Apple_M5 => "cssc+lse+rcpc",
        }
    }
}

/// Provides names for Arm64 ISA instructions to enable for Clang.
pub struct ClangArm64ISANames;

impl ClangArm64ISANames {
    pub fn name(isa: Arm64ISA) -> &'static str {
        isa.as_str()
    }
}

/// Provides names for Arm64 ISA instructions to disable for Clang.
pub struct ClangArm64NOISANames;

impl ClangArm64NOISANames {
    pub fn name(isa: Arm64ISA) -> String {
        match isa {
            Arm64ISA::None => "".into(),
            other => format!("no{}", other.as_str().replace('-', "")),
        }
    }
}

/// Provides names for Arm64 ISA instructions to enable for MSVC.
pub struct MSVCArm64ISANames;

impl MSVCArm64ISANames {
    pub fn name(isa: Arm64ISA) -> &'static str {
        match isa {
            Arm64ISA::Cssc => "cssc",
            Arm64ISA::Faminmax => "faminmax",
            Arm64ISA::Lse => "lse",
            Arm64ISA::Rcpc => "rcpc",
            Arm64ISA::Rcpc3 => "rcpc2",
            _ => "",
        }
    }
}

