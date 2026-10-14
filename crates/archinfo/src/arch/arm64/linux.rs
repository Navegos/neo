// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/linux.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinuxArm64Probe {
    pub hwcap: u64,
    pub hwcap2: u64,
    pub hwcap3: u64,

    // Cache / Subsystem
    pub external_cache_available: bool,
    pub evtstrm_available: bool,
    pub cpuid_available: bool,

    // Base Floating-Point & Advanced SIMD (NEON)
    pub fp_available: bool,
    pub asimd_available: bool,
    pub fphp_available: bool,
    pub asimdhp_available: bool,
    pub asimdrdm_available: bool,
    pub asimddp_available: bool,
    pub asimdfhm_available: bool,

    // Cryptography
    pub aes_available: bool,
    pub pmull_available: bool,
    pub sha1_available: bool,
    pub sha2_available: bool,
    pub crc32_available: bool,
    pub sha3_available: bool,
    pub sm3_available: bool,
    pub sm4_available: bool,
    pub sha512_available: bool,

    // Atomics & Memory Model
    pub atomics_available: bool,
    pub uscat_available: bool,
    pub lrcpc_available: bool,
    pub ilrcpc_available: bool,
    pub lrcpc3_available: bool,
    pub lse128_available: bool,
    pub dcpop_available: bool,
    pub dcpodp_available: bool,

    // Floating-Point Conversions & Matrix Arithmetic
    pub jscvt_available: bool,
    pub fcma_available: bool,
    pub frint_available: bool,
    pub i8mm_available: bool,
    pub bf16_available: bool,
    pub ebf16_available: bool,
    pub fpmr_available: bool,
    pub lut_available: bool,
    pub faminmax_available: bool,
    pub f8cvt_available: bool,
    pub f8fma_available: bool,
    pub f8dp4_available: bool,
    pub f8dp2_available: bool,
    pub f8e4m3_available: bool,
    pub f8e5m2_available: bool,
    pub fprcvt_available: bool,
    pub f8mm8_available: bool,
    pub f8mm4_available: bool,
    pub f16mm_available: bool,
    pub f16f32dot_available: bool,
    pub f16f32mm_available: bool,

    // Control Flow, Security & System Instructions
    pub dit_available: bool,
    pub flagm_available: bool,
    pub flagm2_available: bool,
    pub ssbs_available: bool,
    pub sb_available: bool,
    pub paca_available: bool,
    pub pacg_available: bool,
    pub gcs_available: bool,
    pub cmpbr_available: bool,
    pub dgh_available: bool,
    pub rng_available: bool,
    pub bti_available: bool,
    pub mte_available: bool,
    pub mte3_available: bool,
    pub mte_far_available: bool,
    pub mte_store_only_available: bool,
    pub ecv_available: bool,
    pub afp_available: bool,
    pub rpres_available: bool,
    pub wfxt_available: bool,
    pub cssc_available: bool,
    pub rprfm_available: bool,
    pub mops_available: bool,
    pub hbc_available: bool,
    pub poe_available: bool,
    pub lsfe_available: bool,
    pub ls64_available: bool,

    // SVE / SVE2 Extensions
    pub sve_available: bool,
    pub sve2_available: bool,
    pub sve2p1_available: bool,
    pub sve2p2_available: bool,
    pub sve2p3_available: bool,
    pub sve_aes_available: bool,
    pub sve_aes2_available: bool,
    pub sve_pmull_available: bool,
    pub sve_bitperm_available: bool,
    pub sve_sha3_available: bool,
    pub sve_sm4_available: bool,
    pub sve_i8mm_available: bool,
    pub sve_f32mm_available: bool,
    pub sve_f64mm_available: bool,
    pub sve_bf16_available: bool,
    pub sve_ebf16_available: bool,
    pub sve_b16b16_available: bool,
    pub sve_b16mm_available: bool,
    pub sve_f16mm_available: bool,
    pub sve_eltperm_available: bool,
    pub sve_bfscale_available: bool,
    pub sve_lut6_available: bool,

    // SME / SME2 Extensions
    pub sme_available: bool,
    pub sme2_available: bool,
    pub sme2p1_available: bool,
    pub sme2p2_available: bool,
    pub sme2p3_available: bool,
    pub sme_i16i64_available: bool,
    pub sme_f64f64_available: bool,
    pub sme_i8i32_available: bool,
    pub sme_f16f32_available: bool,
    pub sme_b16f32_available: bool,
    pub sme_f32f32_available: bool,
    pub sme_fa64_available: bool,
    pub sme_i16i32_available: bool,
    pub sme_bi32i32_available: bool,
    pub sme_b16b16_available: bool,
    pub sme_f16f16_available: bool,
    pub sme_lutv2_available: bool,
    pub sme_f8f16_available: bool,
    pub sme_f8f32_available: bool,
    pub sme_sf8fma_available: bool,
    pub sme_sf8dp4_available: bool,
    pub sme_sf8dp2_available: bool,
    pub sme_sbitperm_available: bool,
    pub sme_aes_available: bool,
    pub sme_sfexpa_available: bool,
    pub sme_stmop_available: bool,
    pub sme_smop4_available: bool,
    pub sme_lut6_available: bool,
}

impl LinuxArm64Probe {
    pub fn query() -> Self {
        #[allow(unused_mut)]
        let mut probe = Self::default();

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            // AT_HWCAP (Auxiliary Vector Type 16)
            const HWCAP_FP: u64            = 1 << 0;
            const HWCAP_ASIMD: u64         = 1 << 1;
            const HWCAP_EVTSTRM: u64       = 1 << 2;
            const HWCAP_AES: u64           = 1 << 3;
            const HWCAP_PMULL: u64         = 1 << 4;
            const HWCAP_SHA1: u64          = 1 << 5;
            const HWCAP_SHA2: u64          = 1 << 6;
            const HWCAP_CRC32: u64         = 1 << 7;
            const HWCAP_ATOMICS: u64       = 1 << 8;
            const HWCAP_FPHP: u64          = 1 << 9;
            const HWCAP_ASIMDHP: u64       = 1 << 10;
            const HWCAP_CPUID: u64         = 1 << 11;
            const HWCAP_ASIMDRDM: u64      = 1 << 12;
            const HWCAP_JSCVT: u64         = 1 << 13;
            const HWCAP_FCMA: u64          = 1 << 14;
            const HWCAP_LRCPC: u64         = 1 << 15;
            const HWCAP_DCPOP: u64         = 1 << 16;
            const HWCAP_SHA3: u64          = 1 << 17;
            const HWCAP_SM3: u64           = 1 << 18;
            const HWCAP_SM4: u64           = 1 << 19;
            const HWCAP_ASIMDDP: u64       = 1 << 20;
            const HWCAP_SHA512: u64        = 1 << 21;
            const HWCAP_SVE: u64           = 1 << 22;
            const HWCAP_ASIMDFHM: u64      = 1 << 23;
            const HWCAP_DIT: u64           = 1 << 24;
            const HWCAP_USCAT: u64         = 1 << 25;
            const HWCAP_ILRCPC: u64        = 1 << 26;
            const HWCAP_FLAGM: u64         = 1 << 27;
            const HWCAP_SSBS: u64          = 1 << 28;
            const HWCAP_SB: u64            = 1 << 29;
            const HWCAP_PACA: u64          = 1 << 30;
            const HWCAP_PACG: u64          = 1 << 31;
            const HWCAP_GCS: u64           = 1 << 32;
            const HWCAP_CMPBR: u64         = 1 << 33;
            const HWCAP_FPRCVT: u64        = 1 << 34;
            const HWCAP_F8MM8: u64         = 1 << 35;
            const HWCAP_F8MM4: u64         = 1 << 36;
            const HWCAP_SVE_F16MM: u64     = 1 << 37;
            const HWCAP_SVE_ELTPERM: u64   = 1 << 38;
            const HWCAP_SVE_AES2: u64      = 1 << 39;
            const HWCAP_SVE_BFSCALE: u64   = 1 << 40;
            const HWCAP_SVE2P2: u64        = 1 << 41;
            const HWCAP_SME2P2: u64        = 1 << 42;
            const HWCAP_SME_SBITPERM: u64  = 1 << 43;
            const HWCAP_SME_AES: u64       = 1 << 44;
            const HWCAP_SME_SFEXPA: u64    = 1 << 45;
            const HWCAP_SME_STMOP: u64     = 1 << 46;
            const HWCAP_SME_SMOP4: u64     = 1 << 47;

            // AT_HWCAP2 (Auxiliary Vector Type 26)
            const HWCAP2_DCPODP: u64       = 1 << 0;
            const HWCAP2_SVE2: u64         = 1 << 1;
            const HWCAP2_SVEAES: u64       = 1 << 2;
            const HWCAP2_SVEPMULL: u64     = 1 << 3;
            const HWCAP2_SVEBITPERM: u64   = 1 << 4;
            const HWCAP2_SVESHA3: u64      = 1 << 5;
            const HWCAP2_SVESM4: u64       = 1 << 6;
            const HWCAP2_FLAGM2: u64       = 1 << 7;
            const HWCAP2_FRINT: u64        = 1 << 8;
            const HWCAP2_SVEI8MM: u64      = 1 << 9;
            const HWCAP2_SVEF32MM: u64     = 1 << 10;
            const HWCAP2_SVEF64MM: u64     = 1 << 11;
            const HWCAP2_SVEBF16: u64      = 1 << 12;
            const HWCAP2_I8MM: u64         = 1 << 13;
            const HWCAP2_BF16: u64         = 1 << 14;
            const HWCAP2_DGH: u64          = 1 << 15;
            const HWCAP2_RNG: u64          = 1 << 16;
            const HWCAP2_BTI: u64          = 1 << 17;
            const HWCAP2_MTE: u64          = 1 << 18;
            const HWCAP2_ECV: u64          = 1 << 19;
            const HWCAP2_AFP: u64          = 1 << 20;
            const HWCAP2_RPRES: u64        = 1 << 21;
            const HWCAP2_MTE3: u64         = 1 << 22;
            const HWCAP2_SME: u64          = 1 << 23;
            const HWCAP2_SME_I16I64: u64   = 1 << 24;
            const HWCAP2_SME_F64F64: u64   = 1 << 25;
            const HWCAP2_SME_I8I32: u64    = 1 << 26;
            const HWCAP2_SME_F16F32: u64   = 1 << 27;
            const HWCAP2_SME_B16F32: u64   = 1 << 28;
            const HWCAP2_SME_F32F32: u64   = 1 << 29;
            const HWCAP2_SME_FA64: u64     = 1 << 30;
            const HWCAP2_WFXT: u64         = 1 << 31;
            const HWCAP2_EBF16: u64        = 1 << 32;
            const HWCAP2_SVE_EBF16: u64    = 1 << 33;
            const HWCAP2_CSSC: u64         = 1 << 34;
            const HWCAP2_RPRFM: u64        = 1 << 35;
            const HWCAP2_SVE2P1: u64       = 1 << 36;
            const HWCAP2_SME2: u64         = 1 << 37;
            const HWCAP2_SME2P1: u64       = 1 << 38;
            const HWCAP2_SME_I16I32: u64   = 1 << 39;
            const HWCAP2_SME_BI32I32: u64  = 1 << 40;
            const HWCAP2_SME_B16B16: u64   = 1 << 41;
            const HWCAP2_SME_F16F16: u64   = 1 << 42;
            const HWCAP2_MOPS: u64         = 1 << 43;
            const HWCAP2_HBC: u64          = 1 << 44;
            const HWCAP2_SVE_B16B16: u64   = 1 << 45;
            const HWCAP2_LRCPC3: u64       = 1 << 46;
            const HWCAP2_LSE128: u64       = 1 << 47;
            const HWCAP2_FPMR: u64         = 1 << 48;
            const HWCAP2_LUT: u64          = 1 << 49;
            const HWCAP2_FAMINMAX: u64     = 1 << 50;
            const HWCAP2_F8CVT: u64        = 1 << 51;
            const HWCAP2_F8FMA: u64        = 1 << 52;
            const HWCAP2_F8DP4: u64        = 1 << 53;
            const HWCAP2_F8DP2: u64        = 1 << 54;
            const HWCAP2_F8E4M3: u64       = 1 << 55;
            const HWCAP2_F8E5M2: u64       = 1 << 56;
            const HWCAP2_SME_LUTV2: u64    = 1 << 57;
            const HWCAP2_SME_F8F16: u64    = 1 << 58;
            const HWCAP2_SME_F8F32: u64    = 1 << 59;
            const HWCAP2_SME_SF8FMA: u64   = 1 << 60;
            const HWCAP2_SME_SF8DP4: u64   = 1 << 61;
            const HWCAP2_SME_SF8DP2: u64   = 1 << 62;
            const HWCAP2_POE: u64          = 1 << 63;

            // AT_HWCAP3 (Auxiliary Vector Type 29)
            const HWCAP3_MTE_FAR: u64        = 1 << 0;
            const HWCAP3_MTE_STORE_ONLY: u64 = 1 << 1;
            const HWCAP3_LSFE: u64           = 1 << 2;
            const HWCAP3_LS64: u64           = 1 << 3;
            const HWCAP3_SVE_B16MM: u64      = 1 << 4;
            const HWCAP3_SVE2P3: u64         = 1 << 5;
            const HWCAP3_SME_LUT6: u64       = 1 << 6;
            const HWCAP3_SME2P3: u64         = 1 << 7;
            const HWCAP3_F16MM: u64          = 1 << 8;
            const HWCAP3_F16F32DOT: u64      = 1 << 9;
            const HWCAP3_F16F32MM: u64       = 1 << 10;
            const HWCAP3_SVE_LUT6: u64       = 1 << 11;

            const AT_HWCAP: u64  = 16;
            const AT_HWCAP2: u64 = 26;
            const AT_HWCAP3: u64 = 29;

            let hwcap  = unsafe { libc::getauxval(AT_HWCAP as libc::c_ulong) as u64 };
            let hwcap2 = unsafe { libc::getauxval(AT_HWCAP2 as libc::c_ulong) as u64 };
            let hwcap3 = unsafe { libc::getauxval(AT_HWCAP3 as libc::c_ulong) as u64 };

            #[inline(always)]
            fn check(mask: u64, feature: u64) -> bool {
                (mask & feature) != 0
            }

            fn check_linux_cache() -> bool {
                unsafe {
                    const _SC_LEVEL2_CACHE_SIZE: libc::c_int = 190;
                    const _SC_LEVEL3_CACHE_SIZE: libc::c_int = 193;

                    libc::sysconf(_SC_LEVEL2_CACHE_SIZE) > 0
                        || libc::sysconf(_SC_LEVEL3_CACHE_SIZE) > 0
                }
            }

            probe.hwcap = hwcap;
            probe.hwcap2 = hwcap2;
            probe.hwcap3 = hwcap3;

            // Cache / Subsystem
            probe.external_cache_available   = check_linux_cache();
            probe.evtstrm_available          = check(hwcap, HWCAP_EVTSTRM);
            probe.cpuid_available            = check(hwcap, HWCAP_CPUID);

            // Base Floating-Point & NEON
            probe.fp_available               = check(hwcap, HWCAP_FP);
            probe.asimd_available            = check(hwcap, HWCAP_ASIMD);
            probe.fphp_available             = check(hwcap, HWCAP_FPHP);
            probe.asimdhp_available          = check(hwcap, HWCAP_ASIMDHP);
            probe.asimdrdm_available         = check(hwcap, HWCAP_ASIMDRDM);
            probe.asimddp_available          = check(hwcap, HWCAP_ASIMDDP);
            probe.asimdfhm_available         = check(hwcap, HWCAP_ASIMDFHM);

            // Cryptography
            probe.aes_available              = check(hwcap, HWCAP_AES);
            probe.pmull_available            = check(hwcap, HWCAP_PMULL);
            probe.sha1_available             = check(hwcap, HWCAP_SHA1);
            probe.sha2_available             = check(hwcap, HWCAP_SHA2);
            probe.crc32_available            = check(hwcap, HWCAP_CRC32);
            probe.sha3_available             = check(hwcap, HWCAP_SHA3);
            probe.sm3_available              = check(hwcap, HWCAP_SM3);
            probe.sm4_available              = check(hwcap, HWCAP_SM4);
            probe.sha512_available           = check(hwcap, HWCAP_SHA512);

            // Atomics & Memory Model
            probe.atomics_available          = check(hwcap, HWCAP_ATOMICS);
            probe.uscat_available            = check(hwcap, HWCAP_USCAT);
            probe.lrcpc_available            = check(hwcap, HWCAP_LRCPC);
            probe.ilrcpc_available           = check(hwcap, HWCAP_ILRCPC);
            probe.lrcpc3_available           = check(hwcap2, HWCAP2_LRCPC3);
            probe.lse128_available           = check(hwcap2, HWCAP2_LSE128);
            probe.dcpop_available            = check(hwcap, HWCAP_DCPOP);
            probe.dcpodp_available           = check(hwcap2, HWCAP2_DCPODP);

            // Floating-Point Conversions & Matrix Arithmetic
            probe.jscvt_available            = check(hwcap, HWCAP_JSCVT);
            probe.fcma_available             = check(hwcap, HWCAP_FCMA);
            probe.frint_available            = check(hwcap2, HWCAP2_FRINT);
            probe.i8mm_available             = check(hwcap2, HWCAP2_I8MM);
            probe.bf16_available             = check(hwcap2, HWCAP2_BF16);
            probe.ebf16_available            = check(hwcap2, HWCAP2_EBF16);
            probe.fpmr_available             = check(hwcap2, HWCAP2_FPMR);
            probe.lut_available              = check(hwcap2, HWCAP2_LUT);
            probe.faminmax_available         = check(hwcap2, HWCAP2_FAMINMAX);
            probe.f8cvt_available            = check(hwcap2, HWCAP2_F8CVT);
            probe.f8fma_available            = check(hwcap2, HWCAP2_F8FMA);
            probe.f8dp4_available            = check(hwcap2, HWCAP2_F8DP4);
            probe.f8dp2_available            = check(hwcap2, HWCAP2_F8DP2);
            probe.f8e4m3_available           = check(hwcap2, HWCAP2_F8E4M3);
            probe.f8e5m2_available           = check(hwcap2, HWCAP2_F8E5M2);
            probe.fprcvt_available           = check(hwcap, HWCAP_FPRCVT);
            probe.f8mm8_available            = check(hwcap, HWCAP_F8MM8);
            probe.f8mm4_available            = check(hwcap, HWCAP_F8MM4);
            probe.f16mm_available            = check(hwcap3, HWCAP3_F16MM);
            probe.f16f32dot_available        = check(hwcap3, HWCAP3_F16F32DOT);
            probe.f16f32mm_available         = check(hwcap3, HWCAP3_F16F32MM);

            // Control Flow, Security & System Instructions
            probe.dit_available              = check(hwcap, HWCAP_DIT);
            probe.flagm_available            = check(hwcap, HWCAP_FLAGM);
            probe.flagm2_available           = check(hwcap2, HWCAP2_FLAGM2);
            probe.ssbs_available             = check(hwcap, HWCAP_SSBS);
            probe.sb_available               = check(hwcap, HWCAP_SB);
            probe.paca_available             = check(hwcap, HWCAP_PACA);
            probe.pacg_available             = check(hwcap, HWCAP_PACG);
            probe.gcs_available              = check(hwcap, HWCAP_GCS);
            probe.cmpbr_available            = check(hwcap, HWCAP_CMPBR);
            probe.dgh_available              = check(hwcap2, HWCAP2_DGH);
            probe.rng_available              = check(hwcap2, HWCAP2_RNG);
            probe.bti_available              = check(hwcap2, HWCAP2_BTI);
            probe.mte_available              = check(hwcap2, HWCAP2_MTE);
            probe.mte3_available             = check(hwcap2, HWCAP2_MTE3);
            probe.mte_far_available          = check(hwcap3, HWCAP3_MTE_FAR);
            probe.mte_store_only_available   = check(hwcap3, HWCAP3_MTE_STORE_ONLY);
            probe.ecv_available              = check(hwcap2, HWCAP2_ECV);
            probe.afp_available              = check(hwcap2, HWCAP2_AFP);
            probe.rpres_available            = check(hwcap2, HWCAP2_RPRES);
            probe.wfxt_available             = check(hwcap2, HWCAP2_WFXT);
            probe.cssc_available             = check(hwcap2, HWCAP2_CSSC);
            probe.rprfm_available            = check(hwcap2, HWCAP2_RPRFM);
            probe.mops_available             = check(hwcap2, HWCAP2_MOPS);
            probe.hbc_available              = check(hwcap2, HWCAP2_HBC);
            probe.poe_available              = check(hwcap2, HWCAP2_POE);
            probe.lsfe_available             = check(hwcap3, HWCAP3_LSFE);
            probe.ls64_available             = check(hwcap3, HWCAP3_LS64);

            // SVE / SVE2 Extensions
            probe.sve_available              = check(hwcap, HWCAP_SVE);
            probe.sve2_available             = check(hwcap2, HWCAP2_SVE2);
            probe.sve2p1_available           = check(hwcap2, HWCAP2_SVE2P1);
            probe.sve2p2_available           = check(hwcap, HWCAP_SVE2P2);
            probe.sve2p3_available           = check(hwcap3, HWCAP3_SVE2P3);
            probe.sve_aes_available          = check(hwcap2, HWCAP2_SVEAES);
            probe.sve_aes2_available         = check(hwcap, HWCAP_SVE_AES2);
            probe.sve_pmull_available        = check(hwcap2, HWCAP2_SVEPMULL);
            probe.sve_bitperm_available      = check(hwcap2, HWCAP2_SVEBITPERM);
            probe.sve_sha3_available         = check(hwcap2, HWCAP2_SVESHA3);
            probe.sve_sm4_available          = check(hwcap2, HWCAP2_SVESM4);
            probe.sve_i8mm_available         = check(hwcap2, HWCAP2_SVEI8MM);
            probe.sve_f32mm_available        = check(hwcap2, HWCAP2_SVEF32MM);
            probe.sve_f64mm_available        = check(hwcap2, HWCAP2_SVEF64MM);
            probe.sve_bf16_available         = check(hwcap2, HWCAP2_SVEBF16);
            probe.sve_ebf16_available        = check(hwcap2, HWCAP2_SVE_EBF16);
            probe.sve_b16b16_available       = check(hwcap2, HWCAP2_SVE_B16B16);
            probe.sve_b16mm_available        = check(hwcap3, HWCAP3_SVE_B16MM);
            probe.sve_f16mm_available        = check(hwcap, HWCAP_SVE_F16MM);
            probe.sve_eltperm_available      = check(hwcap, HWCAP_SVE_ELTPERM);
            probe.sve_bfscale_available      = check(hwcap, HWCAP_SVE_BFSCALE);
            probe.sve_lut6_available         = check(hwcap3, HWCAP3_SVE_LUT6);

            // SME / SME2 Extensions
            probe.sme_available              = check(hwcap2, HWCAP2_SME);
            probe.sme2_available             = check(hwcap2, HWCAP2_SME2);
            probe.sme2p1_available           = check(hwcap2, HWCAP2_SME2P1);
            probe.sme2p2_available           = check(hwcap, HWCAP_SME2P2);
            probe.sme2p3_available           = check(hwcap3, HWCAP3_SME2P3);
            probe.sme_i16i64_available       = check(hwcap2, HWCAP2_SME_I16I64);
            probe.sme_f64f64_available       = check(hwcap2, HWCAP2_SME_F64F64);
            probe.sme_i8i32_available        = check(hwcap2, HWCAP2_SME_I8I32);
            probe.sme_f16f32_available       = check(hwcap2, HWCAP2_SME_F16F32);
            probe.sme_b16f32_available       = check(hwcap2, HWCAP2_SME_B16F32);
            probe.sme_f32f32_available       = check(hwcap2, HWCAP2_SME_F32F32);
            probe.sme_fa64_available         = check(hwcap2, HWCAP2_SME_FA64);
            probe.sme_i16i32_available       = check(hwcap2, HWCAP2_SME_I16I32);
            probe.sme_bi32i32_available      = check(hwcap2, HWCAP2_SME_BI32I32);
            probe.sme_b16b16_available       = check(hwcap2, HWCAP2_SME_B16B16);
            probe.sme_f16f16_available       = check(hwcap2, HWCAP2_SME_F16F16);
            probe.sme_lutv2_available        = check(hwcap2, HWCAP2_SME_LUTV2);
            probe.sme_f8f16_available        = check(hwcap2, HWCAP2_SME_F8F16);
            probe.sme_f8f32_available        = check(hwcap2, HWCAP2_SME_F8F32);
            probe.sme_sf8fma_available       = check(hwcap2, HWCAP2_SME_SF8FMA);
            probe.sme_sf8dp4_available       = check(hwcap2, HWCAP2_SME_SF8DP4);
            probe.sme_sf8dp2_available       = check(hwcap2, HWCAP2_SME_SF8DP2);
            probe.sme_sbitperm_available     = check(hwcap, HWCAP_SME_SBITPERM);
            probe.sme_aes_available          = check(hwcap, HWCAP_SME_AES);
            probe.sme_sfexpa_available       = check(hwcap, HWCAP_SME_SFEXPA);
            probe.sme_stmop_available        = check(hwcap, HWCAP_SME_STMOP);
            probe.sme_smop4_available        = check(hwcap, HWCAP_SME_SMOP4);
            probe.sme_lut6_available         = check(hwcap3, HWCAP3_SME_LUT6);
        }

        probe
    }
}

