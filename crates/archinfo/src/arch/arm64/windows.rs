// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/windows.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
use windows_sys::Win32::System::Threading::IsProcessorFeaturePresent;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsArm64Probe {
    // Legacy / 32-bit & Base Core
    pub vfp_32_registers_available: bool,
    pub neon_available: bool,
    pub divide_available: bool,
    pub loadstore_atomic_64bit_available: bool,
    pub external_cache_available: bool,
    pub fmac_available: bool,
    pub v8_available: bool,

    // Armv8.x Extensions
    pub crypto_available: bool,
    pub crc32_available: bool,
    pub v81_atomic_available: bool,
    pub v82_dp_available: bool,
    pub v82_i8mm_available: bool,
    pub v82_fp16_available: bool,
    pub v83_jscvt_available: bool,
    pub v83_lrcpc_available: bool,
    pub v86_bf16_available: bool,
    pub v86_ebf16_available: bool,
    pub lse2_available: bool,
    pub sha3_available: bool,
    pub sha512_available: bool,

    // SVE / SVE2 Extensions
    pub sve_available: bool,
    pub sve2_available: bool,
    pub sve2_1_available: bool,
    pub sve_aes_available: bool,
    pub sve_pmull128_available: bool,
    pub sve_bitperm_available: bool,
    pub sve_bf16_available: bool,
    pub sve_ebf16_available: bool,
    pub sve_b16b16_available: bool,
    pub sve_sha3_available: bool,
    pub sve_sm4_available: bool,
    pub sve_i8mm_available: bool,
    pub sve_f32mm_available: bool,
    pub sve_f64mm_available: bool,

    // SME / SME2 Extensions
    pub sme_available: bool,
    pub sme2_available: bool,
    pub sme2_1_available: bool,
    pub sme2_2_available: bool,
    pub sme_aes_available: bool,
    pub sme_sbitperm_available: bool,
    pub sme_sf8mm4_available: bool,
    pub sme_sf8mm8_available: bool,
    pub sme_sf8dp2_available: bool,
    pub sme_sf8dp4_available: bool,
    pub sme_sf8fma_available: bool,
    pub sme_f8f32_available: bool,
    pub sme_f8f16_available: bool,
    pub sme_f16f16_available: bool,
    pub sme_b16b16_available: bool,
    pub sme_f64f64_available: bool,
    pub sme_i16i64_available: bool,
    pub sme_lutv2_available: bool,
    pub sme_fa64_available: bool,
}

impl WindowsArm64Probe {
    pub fn query() -> Self {
        #[allow(unused_mut)]
        let mut probe = Self::default();

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            // Base & Microarchitecture
            const PF_ARM_VFP_32_REGISTERS_AVAILABLE: u32          = 18;
            const PF_ARM_NEON_INSTRUCTIONS_AVAILABLE: u32         = 19;
            const PF_ARM_DIVIDE_INSTRUCTION_AVAILABLE: u32        = 24;
            const PF_ARM_64BIT_LOADSTORE_ATOMIC: u32              = 25;
            const PF_ARM_EXTERNAL_CACHE_AVAILABLE: u32            = 26;
            const PF_ARM_FMAC_INSTRUCTIONS_AVAILABLE: u32         = 27;
            const PF_ARM_V8_INSTRUCTIONS_AVAILABLE: u32           = 29;

            // Cryptography & Armv8 Features
            const PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE: u32    = 30;
            const PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE: u32     = 31;
            const PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE: u32   = 34;
            const PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE: u32       = 43;
            const PF_ARM_V83_JSCVT_INSTRUCTIONS_AVAILABLE: u32    = 44;
            const PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE: u32    = 45;
            const PF_ARM_LSE2_AVAILABLE: u32                      = 62;
            const PF_ARM_SHA3_INSTRUCTIONS_AVAILABLE: u32         = 64;
            const PF_ARM_SHA512_INSTRUCTIONS_AVAILABLE: u32       = 65;
            const PF_ARM_V82_I8MM_INSTRUCTIONS_AVAILABLE: u32     = 66;
            const PF_ARM_V82_FP16_INSTRUCTIONS_AVAILABLE: u32     = 67;
            const PF_ARM_V86_BF16_INSTRUCTIONS_AVAILABLE: u32     = 68;
            const PF_ARM_V86_EBF16_INSTRUCTIONS_AVAILABLE: u32    = 69;

            // SVE / SVE2
            const PF_ARM_SVE_INSTRUCTIONS_AVAILABLE: u32          = 46;
            const PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE: u32         = 47;
            const PF_ARM_SVE2_1_INSTRUCTIONS_AVAILABLE: u32       = 48;
            const PF_ARM_SVE_AES_INSTRUCTIONS_AVAILABLE: u32      = 49;
            const PF_ARM_SVE_PMULL128_INSTRUCTIONS_AVAILABLE: u32 = 50;
            const PF_ARM_SVE_BITPERM_INSTRUCTIONS_AVAILABLE: u32  = 51;
            const PF_ARM_SVE_BF16_INSTRUCTIONS_AVAILABLE: u32     = 52;
            const PF_ARM_SVE_EBF16_INSTRUCTIONS_AVAILABLE: u32    = 53;
            const PF_ARM_SVE_B16B16_INSTRUCTIONS_AVAILABLE: u32   = 54;
            const PF_ARM_SVE_SHA3_INSTRUCTIONS_AVAILABLE: u32     = 55;
            const PF_ARM_SVE_SM4_INSTRUCTIONS_AVAILABLE: u32      = 56;
            const PF_ARM_SVE_I8MM_INSTRUCTIONS_AVAILABLE: u32     = 57;
            const PF_ARM_SVE_F32MM_INSTRUCTIONS_AVAILABLE: u32    = 58;
            const PF_ARM_SVE_F64MM_INSTRUCTIONS_AVAILABLE: u32    = 59;

            // SME / SME2
            const PF_ARM_SME_INSTRUCTIONS_AVAILABLE: u32          = 70;
            const PF_ARM_SME2_INSTRUCTIONS_AVAILABLE: u32         = 71;
            const PF_ARM_SME2_1_INSTRUCTIONS_AVAILABLE: u32       = 72;
            const PF_ARM_SME2_2_INSTRUCTIONS_AVAILABLE: u32       = 73;
            const PF_ARM_SME_AES_INSTRUCTIONS_AVAILABLE: u32      = 74;
            const PF_ARM_SME_SBITPERM_INSTRUCTIONS_AVAILABLE: u32 = 75;
            const PF_ARM_SME_SF8MM4_INSTRUCTIONS_AVAILABLE: u32   = 76;
            const PF_ARM_SME_SF8MM8_INSTRUCTIONS_AVAILABLE: u32   = 77;
            const PF_ARM_SME_SF8DP2_INSTRUCTIONS_AVAILABLE: u32   = 78;
            const PF_ARM_SME_SF8DP4_INSTRUCTIONS_AVAILABLE: u32   = 79;
            const PF_ARM_SME_SF8FMA_INSTRUCTIONS_AVAILABLE: u32   = 80;
            const PF_ARM_SME_F8F32_INSTRUCTIONS_AVAILABLE: u32    = 81;
            const PF_ARM_SME_F8F16_INSTRUCTIONS_AVAILABLE: u32    = 82;
            const PF_ARM_SME_F16F16_INSTRUCTIONS_AVAILABLE: u32   = 83;
            const PF_ARM_SME_B16B16_INSTRUCTIONS_AVAILABLE: u32   = 84;
            const PF_ARM_SME_F64F64_INSTRUCTIONS_AVAILABLE: u32   = 85;
            const PF_ARM_SME_I16I64_INSTRUCTIONS_AVAILABLE: u32   = 86;
            const PF_ARM_SME_LUTV2_INSTRUCTIONS_AVAILABLE: u32    = 87;
            const PF_ARM_SME_FA64_INSTRUCTIONS_AVAILABLE: u32     = 88;

            #[inline(always)]
            fn check(feature: u32) -> bool {
                unsafe { IsProcessorFeaturePresent(feature) != 0 }
            }

            // Legacy & Base Core
            probe.vfp_32_registers_available = check(PF_ARM_VFP_32_REGISTERS_AVAILABLE);
            probe.neon_available = check(PF_ARM_NEON_INSTRUCTIONS_AVAILABLE);
            probe.divide_available = check(PF_ARM_DIVIDE_INSTRUCTION_AVAILABLE);
            probe.loadstore_atomic_64bit_available = check(PF_ARM_64BIT_LOADSTORE_ATOMIC);
            probe.external_cache_available = check(PF_ARM_EXTERNAL_CACHE_AVAILABLE);
            probe.fmac_available = check(PF_ARM_FMAC_INSTRUCTIONS_AVAILABLE);
            probe.v8_available = check(PF_ARM_V8_INSTRUCTIONS_AVAILABLE);

            // Armv8.x Extensions
            probe.crypto_available = check(PF_ARM_V8_CRYPTO_INSTRUCTIONS_AVAILABLE);
            probe.crc32_available = check(PF_ARM_V8_CRC32_INSTRUCTIONS_AVAILABLE);
            probe.v81_atomic_available = check(PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE);
            probe.v82_dp_available = check(PF_ARM_V82_DP_INSTRUCTIONS_AVAILABLE);
            probe.v82_i8mm_available = check(PF_ARM_V82_I8MM_INSTRUCTIONS_AVAILABLE);
            probe.v82_fp16_available = check(PF_ARM_V82_FP16_INSTRUCTIONS_AVAILABLE);
            probe.v83_jscvt_available = check(PF_ARM_V83_JSCVT_INSTRUCTIONS_AVAILABLE);
            probe.v83_lrcpc_available = check(PF_ARM_V83_LRCPC_INSTRUCTIONS_AVAILABLE);
            probe.v86_bf16_available = check(PF_ARM_V86_BF16_INSTRUCTIONS_AVAILABLE);
            probe.v86_ebf16_available = check(PF_ARM_V86_EBF16_INSTRUCTIONS_AVAILABLE);
            probe.lse2_available = check(PF_ARM_LSE2_AVAILABLE);
            probe.sha3_available = check(PF_ARM_SHA3_INSTRUCTIONS_AVAILABLE);
            probe.sha512_available = check(PF_ARM_SHA512_INSTRUCTIONS_AVAILABLE);

            // SVE / SVE2 Extensions
            probe.sve_available = check(PF_ARM_SVE_INSTRUCTIONS_AVAILABLE);
            probe.sve2_available = check(PF_ARM_SVE2_INSTRUCTIONS_AVAILABLE);
            probe.sve2_1_available = check(PF_ARM_SVE2_1_INSTRUCTIONS_AVAILABLE);
            probe.sve_aes_available = check(PF_ARM_SVE_AES_INSTRUCTIONS_AVAILABLE);
            probe.sve_pmull128_available = check(PF_ARM_SVE_PMULL128_INSTRUCTIONS_AVAILABLE);
            probe.sve_bitperm_available = check(PF_ARM_SVE_BITPERM_INSTRUCTIONS_AVAILABLE);
            probe.sve_bf16_available = check(PF_ARM_SVE_BF16_INSTRUCTIONS_AVAILABLE);
            probe.sve_ebf16_available = check(PF_ARM_SVE_EBF16_INSTRUCTIONS_AVAILABLE);
            probe.sve_b16b16_available = check(PF_ARM_SVE_B16B16_INSTRUCTIONS_AVAILABLE);
            probe.sve_sha3_available = check(PF_ARM_SVE_SHA3_INSTRUCTIONS_AVAILABLE);
            probe.sve_sm4_available = check(PF_ARM_SVE_SM4_INSTRUCTIONS_AVAILABLE);
            probe.sve_i8mm_available = check(PF_ARM_SVE_I8MM_INSTRUCTIONS_AVAILABLE);
            probe.sve_f32mm_available = check(PF_ARM_SVE_F32MM_INSTRUCTIONS_AVAILABLE);
            probe.sve_f64mm_available = check(PF_ARM_SVE_F64MM_INSTRUCTIONS_AVAILABLE);

            // SME / SME2 Extensions
            probe.sme_available = check(PF_ARM_SME_INSTRUCTIONS_AVAILABLE);
            probe.sme2_available = check(PF_ARM_SME2_INSTRUCTIONS_AVAILABLE);
            probe.sme2_1_available = check(PF_ARM_SME2_1_INSTRUCTIONS_AVAILABLE);
            probe.sme2_2_available = check(PF_ARM_SME2_2_INSTRUCTIONS_AVAILABLE);
            probe.sme_aes_available = check(PF_ARM_SME_AES_INSTRUCTIONS_AVAILABLE);
            probe.sme_sbitperm_available = check(PF_ARM_SME_SBITPERM_INSTRUCTIONS_AVAILABLE);
            probe.sme_sf8mm4_available = check(PF_ARM_SME_SF8MM4_INSTRUCTIONS_AVAILABLE);
            probe.sme_sf8mm8_available = check(PF_ARM_SME_SF8MM8_INSTRUCTIONS_AVAILABLE);
            probe.sme_sf8dp2_available = check(PF_ARM_SME_SF8DP2_INSTRUCTIONS_AVAILABLE);
            probe.sme_sf8dp4_available = check(PF_ARM_SME_SF8DP4_INSTRUCTIONS_AVAILABLE);
            probe.sme_sf8fma_available = check(PF_ARM_SME_SF8FMA_INSTRUCTIONS_AVAILABLE);
            probe.sme_f8f32_available = check(PF_ARM_SME_F8F32_INSTRUCTIONS_AVAILABLE);
            probe.sme_f8f16_available = check(PF_ARM_SME_F8F16_INSTRUCTIONS_AVAILABLE);
            probe.sme_f16f16_available = check(PF_ARM_SME_F16F16_INSTRUCTIONS_AVAILABLE);
            probe.sme_b16b16_available = check(PF_ARM_SME_B16B16_INSTRUCTIONS_AVAILABLE);
            probe.sme_f64f64_available = check(PF_ARM_SME_F64F64_INSTRUCTIONS_AVAILABLE);
            probe.sme_i16i64_available = check(PF_ARM_SME_I16I64_INSTRUCTIONS_AVAILABLE);
            probe.sme_lutv2_available = check(PF_ARM_SME_LUTV2_INSTRUCTIONS_AVAILABLE);
            probe.sme_fa64_available = check(PF_ARM_SME_FA64_INSTRUCTIONS_AVAILABLE);
        }

        probe
    }
}

