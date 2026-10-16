// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/apple.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppleArm64Probe {
    // Cache / Subsystem
    pub external_cache_available: bool,

    // Base Architecture & Vector Extensions
    pub fp_available: bool,
    pub advsimd_available: bool,
    pub advsimd_hpfp_available: bool,
    pub fp16_available: bool,
    pub bf16_available: bool,
    pub dotprod_available: bool,
    pub i8mm_available: bool,
    pub fhm_available: bool,
    pub fcma_available: bool,
    pub rdm_available: bool,
    pub frintts_available: bool,
    pub jscvt_available: bool,

    // Cryptography & Hashing
    pub aes_available: bool,
    pub pmull_available: bool,
    pub crc32_available: bool,
    pub sha1_available: bool,
    pub sha256_available: bool,
    pub sha3_available: bool,
    pub sha512_available: bool,

    // Memory Model, Atomics & System Instructions
    pub lse_available: bool,
    pub lse2_available: bool,
    pub lrcpc_available: bool,
    pub lrcpc2_available: bool,
    pub dpb_available: bool,
    pub dpb2_available: bool,
    pub flagm_available: bool,
    pub flagm2_available: bool,
    pub cssc_available: bool,
    pub mops_available: bool,
    pub hbc_available: bool,
    pub wfxt_available: bool,
    pub lor_available: bool,

    // Security, Virtualization & Speculation Mitigation
    pub bti_available: bool,
    pub pauth_available: bool,
    pub fpac_available: bool,
    pub sb_available: bool,
    pub ssbs_available: bool,
    pub csv2_available: bool,
    pub csv3_available: bool,
    pub dit_available: bool,
    pub ecv_available: bool,
    pub afp_available: bool,
    pub fgt_available: bool,
    pub hcx_available: bool,
    pub ras_available: bool,
    pub specres_available: bool,
    pub pmuv3_available: bool,

    // SME / SME2 (Scalable Matrix Extension)
    pub sme_available: bool,
    pub sme2_available: bool,
    pub sme_f64f64_available: bool,
    pub sme_i16i64_available: bool,
    pub sme_b16b16_available: bool,
}

impl AppleArm64Probe {
    pub fn query() -> Self {
        #[allow(unused_mut)]
        let mut probe = Self::default();

        #[cfg(all(target_vendor = "apple", target_arch = "aarch64"))]
        {
            use std::ffi::CStr;

            #[inline]
            fn check_sysctl(c_name: &CStr) -> bool {
                let mut val: i32 = 0;
                let mut size = std::mem::size_of::<i32>();
                unsafe {
                    let ret = libc::sysctlbyname(
                        c_name.as_ptr(),
                        &mut val as *mut _ as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut::<libc::c_void>(),
                        0,
                    );
                    ret == 0 && val == 1
                }
            }

            pub fn check_mac_cache() -> bool {
                // Rust's type equivalent for C# 'long' in sysctl parameters is i64
                let mut l2_size: i64 = 0;
                let mut l3_size: i64 = 0;

                // In Rust, size_of is a standard compile-time operation
                let mut size_of_l2 = std::mem::size_of::<i64>();
                let mut size_of_l3 = std::mem::size_of::<i64>();

                unsafe {
                    // Call sysctlbyname for L2 cache size (using null-terminated C-strings)
                    let result_l2 = libc::sysctlbyname(
                        c"hw.l2cachesize".as_ptr(),
                        (&mut l2_size as *mut i64).cast::<libc::c_void>(),
                        &mut size_of_l2,
                        std::ptr::null_mut::<libc::c_void>(),
                        0,
                    );

                    // Call sysctlbyname for L3 cache size
                    let result_l3 = libc::sysctlbyname(
                        c"hw.l3cachesize".as_ptr(),
                        (&mut l3_size as *mut i64).cast::<libc::c_void>(),
                        &mut size_of_l3,
                        std::ptr::null_mut::<libc::c_void>(),
                        0,
                    );

                    // Map the condition: sysctl returns 0 on success
                    (result_l2 == 0 && l2_size > 0) || (result_l3 == 0 && l3_size > 0)
                }
            }

            // Cache / Subsystem
            probe.external_cache_available   = check_mac_cache();

            // Handles aliases where older Darwin versions might use a legacy key
            #[inline]
            fn check_sysctl_with_fallback(primary: &CStr, fallback: &CStr) -> bool {
                check_sysctl(primary) || check_sysctl(fallback)
            }

            // Base Architecture & Vector Extensions
            probe.fp_available      = check_sysctl(c"hw.optional.floatingpoint");
            probe.advsimd_available = check_sysctl_with_fallback(
                c"hw.optional.AdvSIMD",
                c"hw.optional.neon",
            );
            probe.advsimd_hpfp_available        = check_sysctl_with_fallback(
                c"hw.optional.AdvSIMD_HPFPCvt",
                c"hw.optional.neon_hpfp",
            );
            probe.fp16_available        = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_FP16",
                c"hw.optional.neon_fp16",
            );
            probe.bf16_available    = check_sysctl(c"hw.optional.arm.FEAT_BF16");
            probe.dotprod_available = check_sysctl(c"hw.optional.arm.FEAT_DotProd");
            probe.i8mm_available    = check_sysctl(c"hw.optional.arm.FEAT_I8MM");
            probe.fhm_available     = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_FHM",
                c"hw.optional.armv8_2_fhm",
            );
            probe.fcma_available        = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_FCMA",
                c"hw.optional.armv8_3_compnum",
            );
            probe.rdm_available     = check_sysctl(c"hw.optional.arm.FEAT_RDM");
            probe.frintts_available = check_sysctl(c"hw.optional.arm.FEAT_FRINTTS");
            probe.jscvt_available   = check_sysctl(c"hw.optional.arm.FEAT_JSCVT");

              // Cryptography & Hashing
            probe.aes_available   = check_sysctl(c"hw.optional.arm.FEAT_AES");
            probe.pmull_available = check_sysctl(c"hw.optional.arm.FEAT_PMULL");
            probe.crc32_available = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_CRC32",
                c"hw.optional.armv8_crc32",
            );
            probe.sha1_available   = check_sysctl(c"hw.optional.arm.FEAT_SHA1");
            probe.sha256_available = check_sysctl(c"hw.optional.arm.FEAT_SHA256");
            probe.sha3_available   = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_SHA3",
                c"hw.optional.armv8_2_sha3",
            );
            probe.sha512_available        = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_SHA512",
                c"hw.optional.armv8_2_sha512",
            );

              // Memory Model, Atomics & System Instructions
            probe.lse_available        = check_sysctl_with_fallback(
                c"hw.optional.arm.FEAT_LSE",
                c"hw.optional.armv8_1_atomics",
            );
            probe.lse2_available   = check_sysctl(c"hw.optional.arm.FEAT_LSE2");
            probe.lrcpc_available  = check_sysctl(c"hw.optional.arm.FEAT_LRCPC");
            probe.lrcpc2_available = check_sysctl(c"hw.optional.arm.FEAT_LRCPC2");
            probe.dpb_available    = check_sysctl(c"hw.optional.arm.FEAT_DPB");
            probe.dpb2_available   = check_sysctl(c"hw.optional.arm.FEAT_DPB2");
            probe.flagm_available  = check_sysctl(c"hw.optional.arm.FEAT_FlagM");
            probe.flagm2_available = check_sysctl(c"hw.optional.arm.FEAT_FlagM2");
            probe.cssc_available   = check_sysctl(c"hw.optional.arm.FEAT_CSSC");
            probe.mops_available   = check_sysctl(c"hw.optional.arm.FEAT_MOPS");
            probe.hbc_available    = check_sysctl(c"hw.optional.arm.FEAT_HBC");
            probe.wfxt_available   = check_sysctl(c"hw.optional.arm.FEAT_WFxT");
            probe.lor_available    = check_sysctl(c"hw.optional.arm.FEAT_LOR");

              // Security, Virtualization & Speculation Mitigation
            probe.bti_available     = check_sysctl(c"hw.optional.arm.FEAT_BTI");
            probe.pauth_available   = check_sysctl(c"hw.optional.arm.FEAT_PAuth");
            probe.fpac_available    = check_sysctl(c"hw.optional.arm.FEAT_FPAC");
            probe.sb_available      = check_sysctl(c"hw.optional.arm.FEAT_SB");
            probe.ssbs_available    = check_sysctl(c"hw.optional.arm.FEAT_SSBS");
            probe.csv2_available    = check_sysctl(c"hw.optional.arm.FEAT_CSV2");
            probe.csv3_available    = check_sysctl(c"hw.optional.arm.FEAT_CSV3");
            probe.dit_available     = check_sysctl(c"hw.optional.arm.FEAT_DIT");
            probe.ecv_available     = check_sysctl(c"hw.optional.arm.FEAT_ECV");
            probe.afp_available     = check_sysctl(c"hw.optional.arm.FEAT_AFP");
            probe.fgt_available     = check_sysctl(c"hw.optional.arm.FEAT_FGT");
            probe.hcx_available     = check_sysctl(c"hw.optional.arm.FEAT_HCX");
            probe.ras_available     = check_sysctl(c"hw.optional.arm.FEAT_RAS");
            probe.specres_available = check_sysctl(c"hw.optional.arm.FEAT_SPECRES");
            probe.pmuv3_available   = check_sysctl(c"hw.optional.arm.FEAT_PMUv3");

              // SME / SME2
            probe.sme_available        = check_sysctl(c"hw.optional.arm.FEAT_SME");
            probe.sme2_available       = check_sysctl(c"hw.optional.arm.FEAT_SME2");
            probe.sme_f64f64_available = check_sysctl(c"hw.optional.arm.FEAT_SME_F64F64");
            probe.sme_i16i64_available = check_sysctl(c"hw.optional.arm.FEAT_SME_I16I64");
            probe.sme_b16b16_available = check_sysctl(c"hw.optional.arm.FEAT_SME_B16B16");
        }

        probe
    }
}

