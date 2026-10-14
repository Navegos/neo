// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/arm64/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-14

pub mod apple;
pub mod freebsd;
pub mod isa;
pub mod linux;
pub mod names;
pub mod targets;
pub mod windows;

use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};

pub use isa::Arm64ISA;
pub use names::*;
pub use targets::*;

/// Represents the detected or configured CPU features for Arm64 architecture.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Arm64CPUFeatures {
    pub aes: bool,
    pub bf16: bool,
    pub brbe: bool,
    pub bti: bool,
    pub btie: bool,
    pub cmpbr: bool,
    pub cpa: bool,
    pub crc: bool,
    pub crypto: bool,
    pub cssc: bool,
    pub d128: bool,
    pub dit: bool,
    pub dotprod: bool,
    pub f16f32dot: bool,
    pub f16f32mm: bool,
    pub f16mm: bool,
    pub f32mm: bool,
    pub f64mm: bool,
    pub f8f16mm: bool,
    pub f8f32mm: bool,
    pub faminmax: bool,
    pub fcma: bool,
    pub flagm: bool,
    pub fp: bool,
    pub fp16: bool,
    pub fp16fml: bool,
    pub fp8: bool,
    pub fp8dot2: bool,
    pub fp8dot4: bool,
    pub fp8fma: bool,
    pub fprcvt: bool,
    pub gcie: bool,
    pub gcs: bool,
    pub hbc: bool,
    pub hinte: bool,
    pub i8mm: bool,
    pub ite: bool,
    pub jscvt: bool,
    pub ls64: bool,
    pub lscp: bool,
    pub lse: bool,
    pub lse128: bool,
    pub lsfe: bool,
    pub lsui: bool,
    pub lut: bool,
    pub memtag: bool,
    pub mops: bool,
    pub mops_go: bool,
    pub mtetc: bool,
    pub occmo: bool,
    pub pauth: bool,
    pub pauth_lr: bool,
    pub pmuv3: bool,
    pub poe2: bool,
    pub pops: bool,
    pub predres: bool,
    pub predres2: bool,
    pub profile: bool,
    pub ras: bool,
    pub rasv2: bool,
    pub rcpc: bool,
    pub rcpc3: bool,
    pub rdm: bool,
    pub rng: bool,
    pub sb: bool,
    pub sha2: bool,
    pub sha3: bool,
    pub simd: bool,
    pub sm4: bool,
    pub sme: bool,
    pub sme_b16b16: bool,
    pub sme_f16f16: bool,
    pub sme_f64f64: bool,
    pub sme_f8f16: bool,
    pub sme_f8f32: bool,
    pub sme_fa64: bool,
    pub sme_i16i64: bool,
    pub sme_lutv2: bool,
    pub sme_mop4: bool,
    pub sme_tmop: bool,
    pub sme2: bool,
    pub sme2p1: bool,
    pub sme2p2: bool,
    pub sme2p3: bool,
    pub ssbs: bool,
    pub ssve_aes: bool,
    pub ssve_bitperm: bool,
    pub ssve_fexpa: bool,
    pub ssve_fp8dot2: bool,
    pub ssve_fp8dot4: bool,
    pub ssve_fp8fma: bool,
    pub sve: bool,
    pub sve_aes: bool,
    pub sve_aes2: bool,
    pub sve_b16b16: bool,
    pub sve_b16mm: bool,
    pub sve_bfscale: bool,
    pub sve_bitperm: bool,
    pub sve_f16f32mm: bool,
    pub sve_sha3: bool,
    pub sve_sm4: bool,
    pub sve2: bool,
    pub sve2_aes: bool,
    pub sve2_bitperm: bool,
    pub sve2_sha3: bool,
    pub sve2_sm4: bool,
    pub sve2p1: bool,
    pub sve2p2: bool,
    pub sve2p3: bool,
    pub tev: bool,
    pub the_feat: bool,
    pub tlbid: bool,
    pub tlbiw: bool,
    pub wfxt: bool,
}

impl Arm64CPUFeatures {
    /// Detects features on the host machine using OS-specific facilities
    pub fn detect_host() -> Self {
        #[allow(unused_mut)]
        let mut f = Self::default();

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            let win = windows::WindowsArm64Probe::query();
            f.simd = win.neon_available || win.v8_available;
            f.fp = win.v8_available;
            f.crypto = win.crypto_available;
            f.aes = win.crypto_available;
            f.sha2 = win.crypto_available;
            f.crc = win.crc32_available;
            f.lse = win.v81_atomic_available;
            f.dotprod = win.v82_dp_available;
            f.i8mm = win.v82_i8mm_available;
            f.fp16 = win.v82_fp16_available;
            f.jscvt = win.v83_jscvt_available;
            f.rcpc = win.v83_lrcpc_available;
            f.bf16 = win.v86_bf16_available || win.v86_ebf16_available;
            f.sha3 = win.sha3_available || win.sha512_available;

            f.sve = win.sve_available;
            f.sve2 = win.sve2_available;
            f.sve2p1 = win.sve2_1_available;
            f.sve_aes = win.sve_aes_available;
            f.sve_bitperm = win.sve_bitperm_available;
            f.sve_b16b16 = win.sve_b16b16_available;
            f.sve_sha3 = win.sve_sha3_available;
            f.sve_sm4 = win.sve_sm4_available;
            f.sve2_aes = win.sve_aes_available;
            f.sve2_bitperm = win.sve_bitperm_available;
            f.sve2_sha3 = win.sve_sha3_available;
            f.sve2_sm4 = win.sve_sm4_available;
            f.f32mm = win.sve_f32mm_available;
            f.f64mm = win.sve_f64mm_available;
            if win.sve_bf16_available || win.sve_ebf16_available {
                f.bf16 = true;
                f.sve = true;
            }
            if win.sve_i8mm_available {
                f.i8mm = true;
                f.sve = true;
            }

            f.sme = win.sme_available;
            f.sme2 = win.sme2_available;
            f.sme2p1 = win.sme2_1_available;
            f.sme2p2 = win.sme2_2_available;
            f.ssve_aes = win.sme_aes_available;
            f.ssve_bitperm = win.sme_sbitperm_available;
            f.ssve_fp8dot2 = win.sme_sf8dp2_available;
            f.ssve_fp8dot4 = win.sme_sf8dp4_available;
            f.ssve_fp8fma = win.sme_sf8fma_available;
            f.f8f32mm = win.sme_sf8mm4_available;
            f.f8f16mm = win.sme_sf8mm8_available;
            f.sme_f8f32 = win.sme_f8f32_available;
            f.sme_f8f16 = win.sme_f8f16_available;
            f.sme_f16f16 = win.sme_f16f16_available;
            f.sme_b16b16 = win.sme_b16b16_available;
            f.sme_f64f64 = win.sme_f64f64_available;
            f.sme_i16i64 = win.sme_i16i64_available;
            f.sme_lutv2 = win.sme_lutv2_available;
            f.sme_fa64 = win.sme_fa64_available;
        }

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            let lnx = linux::LinuxArm64Probe::query();
            f.fp = lnx.fp_available;
            f.simd = lnx.asimd_available;
            f.fp16 = lnx.fphp_available || lnx.asimdhp_available;
            f.rdm = lnx.asimdrdm_available;
            f.dotprod = lnx.asimddp_available;
            f.fp16fml = lnx.asimdfhm_available;

            f.aes = lnx.aes_available;
            f.sha2 = lnx.sha1_available || lnx.sha2_available;
            f.crc = lnx.crc32_available;
            f.sha3 = lnx.sha3_available || lnx.sha512_available;
            f.sm4 = lnx.sm4_available;
            f.crypto = f.aes && f.sha2;

            f.lse = lnx.atomics_available;
            f.rcpc = lnx.lrcpc_available || lnx.ilrcpc_available;
            f.rcpc3 = lnx.lrcpc3_available;
            if f.rcpc3 {
                f.rcpc = true;
            }
            f.lse128 = lnx.lse128_available;

            f.jscvt = lnx.jscvt_available;
            f.fcma = lnx.fcma_available;
            f.i8mm = lnx.i8mm_available;
            f.bf16 = lnx.bf16_available || lnx.ebf16_available;
            f.lut = lnx.lut_available;
            f.faminmax = lnx.faminmax_available;
            f.fp8 = lnx.f8cvt_available || lnx.f8e4m3_available || lnx.f8e5m2_available;
            f.fp8fma = lnx.f8fma_available;
            f.fp8dot4 = lnx.f8dp4_available;
            f.fp8dot2 = lnx.f8dp2_available;
            f.fprcvt = lnx.fprcvt_available;
            f.f8f16mm = lnx.f8mm8_available;
            f.f8f32mm = lnx.f8mm4_available;
            f.f16mm = lnx.f16mm_available;
            f.f16f32dot = lnx.f16f32dot_available;
            f.f16f32mm = lnx.f16f32mm_available;

            f.dit = lnx.dit_available;
            f.flagm = lnx.flagm_available || lnx.flagm2_available;
            f.ssbs = lnx.ssbs_available;
            f.sb = lnx.sb_available;
            f.pauth = lnx.paca_available || lnx.pacg_available;
            f.gcs = lnx.gcs_available;
            f.cmpbr = lnx.cmpbr_available;
            f.rng = lnx.rng_available;
            f.bti = lnx.bti_available;
            f.memtag = lnx.mte_available || lnx.mte3_available || lnx.mte_far_available || lnx.mte_store_only_available;
            f.predres = lnx.rpres_available;
            f.wfxt = lnx.wfxt_available;
            f.cssc = lnx.cssc_available;
            f.mops = lnx.mops_available;
            f.hbc = lnx.hbc_available;
            f.poe2 = lnx.poe_available;
            f.lsfe = lnx.lsfe_available;
            f.ls64 = lnx.ls64_available;

            f.sve = lnx.sve_available;
            f.sve2 = lnx.sve2_available;
            f.sve2p1 = lnx.sve2p1_available;
            f.sve2p2 = lnx.sve2p2_available;
            f.sve2p3 = lnx.sve2p3_available;
            f.sve_aes = lnx.sve_aes_available;
            f.sve_aes2 = lnx.sve_aes2_available;
            f.sve_bitperm = lnx.sve_bitperm_available;
            f.sve_sha3 = lnx.sve_sha3_available;
            f.sve_sm4 = lnx.sve_sm4_available;
            f.sve2_aes = lnx.sve_aes_available;
            f.sve2_bitperm = lnx.sve_bitperm_available;
            f.sve2_sha3 = lnx.sve_sha3_available;
            f.sve2_sm4 = lnx.sve_sm4_available;
            f.sve_b16b16 = lnx.sve_b16b16_available;
            f.sve_b16mm = lnx.sve_b16mm_available;
            f.sve_bfscale = lnx.sve_bfscale_available;
            if lnx.sve_i8mm_available {
                f.i8mm = true;
                f.sve = true;
            }
            if lnx.sve_f32mm_available {
                f.f32mm = true;
                f.sve = true;
            }
            if lnx.sve_f64mm_available {
                f.f64mm = true;
                f.sve = true;
            }
            if lnx.sve_bf16_available || lnx.sve_ebf16_available {
                f.bf16 = true;
                f.sve = true;
            }
            if lnx.sve_f16mm_available {
                f.f16mm = true;
                f.sve = true;
            }

            f.sme = lnx.sme_available;
            f.sme2 = lnx.sme2_available;
            f.sme2p1 = lnx.sme2p1_available;
            f.sme2p2 = lnx.sme2p2_available;
            f.sme2p3 = lnx.sme2p3_available;
            f.sme_i16i64 = lnx.sme_i16i64_available;
            f.sme_f64f64 = lnx.sme_f64f64_available;
            f.sme_fa64 = lnx.sme_fa64_available;
            f.sme_b16b16 = lnx.sme_b16b16_available;
            f.sme_f16f16 = lnx.sme_f16f16_available;
            f.sme_lutv2 = lnx.sme_lutv2_available;
            f.sme_f8f16 = lnx.sme_f8f16_available;
            f.sme_f8f32 = lnx.sme_f8f32_available;
            f.ssve_fp8fma = lnx.sme_sf8fma_available;
            f.ssve_fp8dot4 = lnx.sme_sf8dp4_available;
            f.ssve_fp8dot2 = lnx.sme_sf8dp2_available;
            f.ssve_bitperm = lnx.sme_sbitperm_available;
            f.ssve_aes = lnx.sme_aes_available;
            f.ssve_fexpa = lnx.sme_sfexpa_available;
            f.sme_tmop = lnx.sme_stmop_available;
            f.sme_mop4 = lnx.sme_smop4_available;
        }

        #[cfg(all(target_os = "freebsd", target_arch = "aarch64"))]
        {
            let fbsd = freebsd::FreeBsdArm64Probe::query();
            f.fp = fbsd.fp_available;
            f.simd = fbsd.asimd_available;
            f.fp16 = fbsd.fphp_available || fbsd.asimdhp_available;
            f.rdm = fbsd.asimdrdm_available;
            f.dotprod = fbsd.asimddp_available;
            f.fp16fml = fbsd.asimdfhm_available;

            f.aes = fbsd.aes_available;
            f.sha2 = fbsd.sha1_available || fbsd.sha2_available;
            f.crc = fbsd.crc32_available;
            f.sha3 = fbsd.sha3_available || fbsd.sha512_available;
            f.sm4 = fbsd.sm4_available;
            f.crypto = f.aes && f.sha2;

            f.lse = fbsd.atomics_available;
            f.rcpc = fbsd.lrcpc_available || fbsd.ilrcpc_available;
            f.rcpc3 = fbsd.lrcpc3_available;
            if f.rcpc3 {
                f.rcpc = true;
            }
            f.lse128 = fbsd.lse128_available;

            f.jscvt = fbsd.jscvt_available;
            f.fcma = fbsd.fcma_available;
            f.i8mm = fbsd.i8mm_available;
            f.bf16 = fbsd.bf16_available || fbsd.ebf16_available;
            f.lut = fbsd.lut_available;
            f.faminmax = fbsd.faminmax_available;
            f.fp8 = fbsd.f8cvt_available || fbsd.f8e4m3_available || fbsd.f8e5m2_available;
            f.fp8fma = fbsd.f8fma_available;
            f.fp8dot4 = fbsd.f8dp4_available;
            f.fp8dot2 = fbsd.f8dp2_available;
            f.fprcvt = fbsd.fprcvt_available;
            f.f8f16mm = fbsd.f8mm8_available;
            f.f8f32mm = fbsd.f8mm4_available;
            f.f16mm = fbsd.f16mm_available;
            f.f16f32dot = fbsd.f16f32dot_available;
            f.f16f32mm = fbsd.f16f32mm_available;

            f.dit = fbsd.dit_available;
            f.flagm = fbsd.flagm_available || fbsd.flagm2_available;
            f.ssbs = fbsd.ssbs_available;
            f.sb = fbsd.sb_available;
            f.pauth = fbsd.paca_available || fbsd.pacg_available;
            f.gcs = fbsd.gcs_available;
            f.cmpbr = fbsd.cmpbr_available;
            f.rng = fbsd.rng_available;
            f.bti = fbsd.bti_available;
            f.memtag = fbsd.mte_available || fbsd.mte3_available || fbsd.mte_far_available || fbsd.mte_store_only_available;
            f.predres = fbsd.rpres_available;
            f.wfxt = fbsd.wfxt_available;
            f.cssc = fbsd.cssc_available;
            f.mops = fbsd.mops_available;
            f.hbc = fbsd.hbc_available;
            f.poe2 = fbsd.poe_available;
            f.lsfe = fbsd.lsfe_available;
            f.ls64 = fbsd.ls64_available;

            f.sve = fbsd.sve_available;
            f.sve2 = fbsd.sve2_available;
            f.sve2p1 = fbsd.sve2p1_available;
            f.sve2p2 = fbsd.sve2p2_available;
            f.sve2p3 = fbsd.sve2p3_available;
            f.sve_aes = fbsd.sve_aes_available;
            f.sve_aes2 = fbsd.sve_aes2_available;
            f.sve_bitperm = fbsd.sve_bitperm_available;
            f.sve_sha3 = fbsd.sve_sha3_available;
            f.sve_sm4 = fbsd.sve_sm4_available;
            f.sve2_aes = fbsd.sve_aes_available;
            f.sve2_bitperm = fbsd.sve_bitperm_available;
            f.sve2_sha3 = fbsd.sve_sha3_available;
            f.sve2_sm4 = fbsd.sve_sm4_available;
            f.sve_b16b16 = fbsd.sve_b16b16_available;
            f.sve_b16mm = fbsd.sve_b16mm_available;
            f.sve_bfscale = fbsd.sve_bfscale_available;
            if fbsd.sve_i8mm_available {
                f.i8mm = true;
                f.sve = true;
            }
            if fbsd.sve_f32mm_available {
                f.f32mm = true;
                f.sve = true;
            }
            if fbsd.sve_f64mm_available {
                f.f64mm = true;
                f.sve = true;
            }
            if fbsd.sve_bf16_available || fbsd.sve_ebf16_available {
                f.bf16 = true;
                f.sve = true;
            }
            if fbsd.sve_f16mm_available {
                f.f16mm = true;
                f.sve = true;
            }

            f.sme = fbsd.sme_available;
            f.sme2 = fbsd.sme2_available;
            f.sme2p1 = fbsd.sme2p1_available;
            f.sme2p2 = fbsd.sme2p2_available;
            f.sme2p3 = fbsd.sme2p3_available;
            f.sme_i16i64 = fbsd.sme_i16i64_available;
            f.sme_f64f64 = fbsd.sme_f64f64_available;
            f.sme_fa64 = fbsd.sme_fa64_available;
            f.sme_b16b16 = fbsd.sme_b16b16_available;
            f.sme_f16f16 = fbsd.sme_f16f16_available;
            f.sme_lutv2 = fbsd.sme_lutv2_available;
            f.sme_f8f16 = fbsd.sme_f8f16_available;
            f.sme_f8f32 = fbsd.sme_f8f32_available;
            f.ssve_fp8fma = fbsd.sme_sf8fma_available;
            f.ssve_fp8dot4 = fbsd.sme_sf8dp4_available;
            f.ssve_fp8dot2 = fbsd.sme_sf8dp2_available;
            f.ssve_bitperm = fbsd.sme_sbitperm_available;
            f.ssve_aes = fbsd.sme_aes_available;
            f.ssve_fexpa = fbsd.sme_sfexpa_available;
            f.sme_tmop = fbsd.sme_stmop_available;
            f.sme_mop4 = fbsd.sme_smop4_available;
        }

        #[cfg(all(target_vendor = "apple", target_arch = "aarch64"))]
        {
            let apple = apple::AppleArm64Probe::query();
            f.fp = apple.fp_available;
            f.simd = apple.advsimd_available;
            f.fp16 = apple.fp16_available;
            f.bf16 = apple.bf16_available;
            f.dotprod = apple.dotprod_available;
            f.i8mm = apple.i8mm_available;
            f.fp16fml = apple.fhm_available;
            f.fcma = apple.fcma_available;
            f.rdm = apple.rdm_available;
            f.jscvt = apple.jscvt_available;

            f.aes = apple.aes_available;
            f.crc = apple.crc32_available;
            f.sha2 = apple.sha1_available || apple.sha256_available;
            f.sha3 = apple.sha3_available || apple.sha512_available;
            f.crypto = f.aes && f.sha2;

            f.lse = apple.lse_available || apple.lse2_available;
            f.rcpc = apple.lrcpc_available || apple.lrcpc2_available;
            f.flagm = apple.flagm_available || apple.flagm2_available;
            f.cssc = apple.cssc_available;
            f.mops = apple.mops_available;
            f.hbc = apple.hbc_available;
            f.wfxt = apple.wfxt_available;

            f.bti = apple.bti_available;
            f.pauth = apple.pauth_available;
            f.sb = apple.sb_available;
            f.ssbs = apple.ssbs_available;
            f.dit = apple.dit_available;
            f.ras = apple.ras_available;
            f.predres = apple.specres_available;
            f.pmuv3 = apple.pmuv3_available;

            f.sme = apple.sme_available;
            f.sme2 = apple.sme2_available;
            f.sme_f64f64 = apple.sme_f64f64_available;
            f.sme_i16i64 = apple.sme_i16i64_available;
            f.sme_b16b16 = apple.sme_b16b16_available;
        }

        f
    }

    /// Constructs features from a '+' delimited extension string
    pub fn from_extensions_str(ext_str: &str) -> Self {
        let tokens: HashSet<&str> = ext_str.split('+').filter(|s| !s.is_empty()).collect();
        let mut f = Self::default();
        f.aes = tokens.contains("aes");
        f.bf16 = tokens.contains("bf16");
        f.brbe = tokens.contains("brbe");
        f.bti = tokens.contains("bti");
        f.btie = tokens.contains("btie");
        f.cmpbr = tokens.contains("cmpbr");
        f.cpa = tokens.contains("cpa");
        f.crc = tokens.contains("crc");
        f.crypto = tokens.contains("crypto");
        f.cssc = tokens.contains("cssc");
        f.d128 = tokens.contains("d128");
        f.dit = tokens.contains("dit");
        f.dotprod = tokens.contains("dotprod");
        f.f16f32dot = tokens.contains("f16f32dot");
        f.f16f32mm = tokens.contains("f16f32mm");
        f.f16mm = tokens.contains("f16mm");
        f.f32mm = tokens.contains("f32mm");
        f.f64mm = tokens.contains("f64mm");
        f.f8f16mm = tokens.contains("f8f16mm");
        f.f8f32mm = tokens.contains("f8f32mm");
        f.faminmax = tokens.contains("faminmax");
        f.fcma = tokens.contains("fcma");
        f.flagm = tokens.contains("flagm");
        f.fp = tokens.contains("fp");
        f.fp16 = tokens.contains("fp16");
        f.fp16fml = tokens.contains("fp16fml");
        f.fp8 = tokens.contains("fp8");
        f.fp8dot2 = tokens.contains("fp8dot2");
        f.fp8dot4 = tokens.contains("fp8dot4");
        f.fp8fma = tokens.contains("fp8fma");
        f.fprcvt = tokens.contains("fprcvt");
        f.gcie = tokens.contains("gcie");
        f.gcs = tokens.contains("gcs");
        f.hbc = tokens.contains("hbc");
        f.hinte = tokens.contains("hinte");
        f.i8mm = tokens.contains("i8mm");
        f.ite = tokens.contains("ite");
        f.jscvt = tokens.contains("jscvt");
        f.ls64 = tokens.contains("ls64");
        f.lscp = tokens.contains("lscp");
        f.lse = tokens.contains("lse");
        f.lse128 = tokens.contains("lse128");
        f.lsfe = tokens.contains("lsfe");
        f.lsui = tokens.contains("lsui");
        f.lut = tokens.contains("lut");
        f.memtag = tokens.contains("memtag");
        f.mops = tokens.contains("mops");
        f.mops_go = tokens.contains("mops-go") || tokens.contains("mops_go");
        f.mtetc = tokens.contains("mtetc");
        f.occmo = tokens.contains("occmo");
        f.pauth = tokens.contains("pauth");
        f.pauth_lr = tokens.contains("pauth-lr") || tokens.contains("pauth_lr");
        f.pmuv3 = tokens.contains("pmuv3");
        f.poe2 = tokens.contains("poe2");
        f.pops = tokens.contains("pops");
        f.predres = tokens.contains("predres");
        f.predres2 = tokens.contains("predres2");
        f.profile = tokens.contains("profile");
        f.ras = tokens.contains("ras");
        f.rasv2 = tokens.contains("rasv2");
        f.rcpc = tokens.contains("rcpc");
        f.rcpc3 = tokens.contains("rcpc3");
        f.rdm = tokens.contains("rdm");
        f.rng = tokens.contains("rng");
        f.sb = tokens.contains("sb");
        f.sha2 = tokens.contains("sha2");
        f.sha3 = tokens.contains("sha3");
        f.simd = tokens.contains("simd");
        f.sm4 = tokens.contains("sm4");
        f.sme = tokens.contains("sme");
        f.sme_b16b16 = tokens.contains("sme-b16b16") || tokens.contains("sme_b16b16");
        f.sme_f16f16 = tokens.contains("sme-f16f16") || tokens.contains("sme_f16f16");
        f.sme_f64f64 = tokens.contains("sme-f64f64") || tokens.contains("sme_f64f64");
        f.sme_f8f16 = tokens.contains("sme-f8f16") || tokens.contains("sme_f8f16");
        f.sme_f8f32 = tokens.contains("sme-f8f32") || tokens.contains("sme_f8f32");
        f.sme_fa64 = tokens.contains("sme-fa64") || tokens.contains("sme_fa64");
        f.sme_i16i64 = tokens.contains("sme-i16i64") || tokens.contains("sme_i16i64");
        f.sme_lutv2 = tokens.contains("sme-lutv2") || tokens.contains("sme_lutv2");
        f.sme_mop4 = tokens.contains("sme-mop4") || tokens.contains("sme_mop4") || tokens.contains("sme-smop4") || tokens.contains("sme_smop4");
        f.sme_tmop = tokens.contains("sme-tmop") || tokens.contains("sme_tmop") || tokens.contains("sme-stmop") || tokens.contains("sme_stmop");
        f.sme2 = tokens.contains("sme2");
        f.sme2p1 = tokens.contains("sme2p1");
        f.sme2p2 = tokens.contains("sme2p2");
        f.sme2p3 = tokens.contains("sme2p3");
        f.ssbs = tokens.contains("ssbs");
        f.ssve_aes = tokens.contains("ssve-aes") || tokens.contains("ssve_aes") || tokens.contains("sme-aes") || tokens.contains("sme_aes");
        f.ssve_bitperm = tokens.contains("ssve-bitperm") || tokens.contains("ssve_bitperm") || tokens.contains("sme-bitperm") || tokens.contains("sme_bitperm");
        f.ssve_fexpa = tokens.contains("ssve-fexpa") || tokens.contains("ssve_fexpa");
        f.ssve_fp8dot2 = tokens.contains("ssve-fp8dot2") || tokens.contains("ssve_fp8dot2");
        f.ssve_fp8dot4 = tokens.contains("ssve-fp8dot4") || tokens.contains("ssve_fp8dot4");
        f.ssve_fp8fma = tokens.contains("ssve-fp8fma") || tokens.contains("ssve_fp8fma");
        f.sve = tokens.contains("sve");
        f.sve_aes = tokens.contains("sve-aes") || tokens.contains("sve_aes");
        f.sve_aes2 = tokens.contains("sve-aes2") || tokens.contains("sve_aes2");
        f.sve_b16b16 = tokens.contains("sve-b16b16") || tokens.contains("sve_b16b16");
        f.sve_b16mm = tokens.contains("sve-b16mm") || tokens.contains("sve_b16mm");
        f.sve_bfscale = tokens.contains("sve-bfscale") || tokens.contains("sve_bfscale");
        f.sve_bitperm = tokens.contains("sve-bitperm") || tokens.contains("sve_bitperm");
        f.sve_f16f32mm = tokens.contains("sve-f16f32mm") || tokens.contains("sve_f16f32mm");
        f.sve_sha3 = tokens.contains("sve-sha3") || tokens.contains("sve_sha3");
        f.sve_sm4 = tokens.contains("sve-sm4") || tokens.contains("sve_sm4");
        f.sve2 = tokens.contains("sve2");
        f.sve2_aes = tokens.contains("sve2-aes") || tokens.contains("sve2_aes");
        f.sve2_bitperm = tokens.contains("sve2-bitperm") || tokens.contains("sve2_bitperm");
        f.sve2_sha3 = tokens.contains("sve2-sha3") || tokens.contains("sve2_sha3");
        f.sve2_sm4 = tokens.contains("sve2-sm4") || tokens.contains("sve2_sm4");
        f.sve2p1 = tokens.contains("sve2p1");
        f.sve2p2 = tokens.contains("sve2p2");
        f.sve2p3 = tokens.contains("sve2p3");
        f.tev = tokens.contains("tev");
        f.the_feat = tokens.contains("the");
        f.tlbid = tokens.contains("tlbid");
        f.tlbiw = tokens.contains("tlbiw");
        f.wfxt = tokens.contains("wfxt");
        f
    }

    /// Constructs features from a known TargetCpuArchitectureArm64
    pub fn from_target(target: TargetCpuArchitectureArm64) -> Self {
        match target {
            TargetCpuArchitectureArm64::Native => Self::detect_host(),
            other => {
                let ext = ClangTargetCpuArchitectureArm64ISANames::name(other);
                Self::from_extensions_str(ext)
            }
        }
    }

    /// Constructs features for Generic target given a requested MinimumCpuArchitectureArm64
    pub fn from_min_arch(min_arch: MinimumCpuArchitectureArm64) -> Self {
        match min_arch {
            MinimumCpuArchitectureArm64::None => {
                Self::from_target(TargetCpuArchitectureArm64::Generic)
            }
            MinimumCpuArchitectureArm64::ARMv8_A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2")
            }
            MinimumCpuArchitectureArm64::ARMv8_1A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm")
            }
            MinimumCpuArchitectureArm64::ARMv8_2A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras")
            }
            MinimumCpuArchitectureArm64::ARMv8_3A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma")
            }
            MinimumCpuArchitectureArm64::ARMv8_4A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs")
            }
            MinimumCpuArchitectureArm64::ARMv8_5A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti")
            }
            MinimumCpuArchitectureArm64::ARMv8_6A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm")
            }
            MinimumCpuArchitectureArm64::ARMv8_7A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt")
            }
            MinimumCpuArchitectureArm64::ARMv8_8A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops")
            }
            MinimumCpuArchitectureArm64::ARMv8_9A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops+cssc")
            }
            MinimumCpuArchitectureArm64::ARMv8_R => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+pauth")
            }
            MinimumCpuArchitectureArm64::ARMv9_A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+sve+sve2")
            }
            MinimumCpuArchitectureArm64::ARMv9_1A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+sve+sve2")
            }
            MinimumCpuArchitectureArm64::ARMv9_2A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+sve+sve2+sme+sme2")
            }
            MinimumCpuArchitectureArm64::ARMv9_3A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops+hbc+sve+sve2+sme+sme2")
            }
            MinimumCpuArchitectureArm64::ARMv9_4A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops+hbc+cssc+gcs+lse128+sve2p1+sve+sve2+sme+sme2")
            }
            MinimumCpuArchitectureArm64::ARMv9_5A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops+hbc+cssc+gcs+lse128+sve2p1+sve+sve2+sme+sme2+cpa+faminmax+fp8")
            }
            MinimumCpuArchitectureArm64::ARMv9_6A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops+hbc+cssc+gcs+lse128+sve2p1+sve+sve2+sme+sme2+cpa+faminmax+fp8+poe2+pops+cmpbr+sve2p2")
            }
            MinimumCpuArchitectureArm64::ARMv9_7A => {
                Self::from_extensions_str("crypto+aes+simd+crc+fp+pmuv3+sha2+lse+rdm+rcpc+fp16+dotprod+ras+pauth+jscvt+fcma+dit+flagm+ssbs+sb+predres+bti+bf16+i8mm+wfxt+mops+hbc+cssc+gcs+lse128+sve2p1+sve+sve2+sme+sme2+cpa+faminmax+fp8+poe2+pops+cmpbr+sve2p2+d128")
            }
        }
    }

    pub fn has_feature(&self, isa: Arm64ISA) -> bool {
        match isa {
            Arm64ISA::None => false,
            Arm64ISA::Aes => self.aes,
            Arm64ISA::Bf16 => self.bf16,
            Arm64ISA::Brbe => self.brbe,
            Arm64ISA::Bti => self.bti,
            Arm64ISA::Btie => self.btie,
            Arm64ISA::Cmpbr => self.cmpbr,
            Arm64ISA::Cpa => self.cpa,
            Arm64ISA::Crc => self.crc,
            Arm64ISA::Crypto => self.crypto,
            Arm64ISA::Cssc => self.cssc,
            Arm64ISA::D128 => self.d128,
            Arm64ISA::Dit => self.dit,
            Arm64ISA::Dotprod => self.dotprod,
            Arm64ISA::F16f32dot => self.f16f32dot,
            Arm64ISA::F16f32mm => self.f16f32mm,
            Arm64ISA::F16mm => self.f16mm,
            Arm64ISA::F32mm => self.f32mm,
            Arm64ISA::F64mm => self.f64mm,
            Arm64ISA::F8f16mm => self.f8f16mm,
            Arm64ISA::F8f32mm => self.f8f32mm,
            Arm64ISA::Faminmax => self.faminmax,
            Arm64ISA::Fcma => self.fcma,
            Arm64ISA::Flagm => self.flagm,
            Arm64ISA::Fp => self.fp,
            Arm64ISA::Fp16 => self.fp16,
            Arm64ISA::Fp16fml => self.fp16fml,
            Arm64ISA::Fp8 => self.fp8,
            Arm64ISA::Fp8dot2 => self.fp8dot2,
            Arm64ISA::Fp8dot4 => self.fp8dot4,
            Arm64ISA::Fp8fma => self.fp8fma,
            Arm64ISA::Fprcvt => self.fprcvt,
            Arm64ISA::Gcie => self.gcie,
            Arm64ISA::Gcs => self.gcs,
            Arm64ISA::Hbc => self.hbc,
            Arm64ISA::Hinte => self.hinte,
            Arm64ISA::I8mm => self.i8mm,
            Arm64ISA::Ite => self.ite,
            Arm64ISA::Jscvt => self.jscvt,
            Arm64ISA::Ls64 => self.ls64,
            Arm64ISA::Lscp => self.lscp,
            Arm64ISA::Lse => self.lse,
            Arm64ISA::Lse128 => self.lse128,
            Arm64ISA::Lsfe => self.lsfe,
            Arm64ISA::Lsui => self.lsui,
            Arm64ISA::Lut => self.lut,
            Arm64ISA::Memtag => self.memtag,
            Arm64ISA::Mops => self.mops,
            Arm64ISA::MopsGo => self.mops_go,
            Arm64ISA::Mtetc => self.mtetc,
            Arm64ISA::Occmo => self.occmo,
            Arm64ISA::Pauth => self.pauth,
            Arm64ISA::PauthLr => self.pauth_lr,
            Arm64ISA::Pmuv3 => self.pmuv3,
            Arm64ISA::Poe2 => self.poe2,
            Arm64ISA::Pops => self.pops,
            Arm64ISA::Predres => self.predres,
            Arm64ISA::Predres2 => self.predres2,
            Arm64ISA::Profile => self.profile,
            Arm64ISA::Ras => self.ras,
            Arm64ISA::Rasv2 => self.rasv2,
            Arm64ISA::Rcpc => self.rcpc,
            Arm64ISA::Rcpc3 => self.rcpc3,
            Arm64ISA::Rdm => self.rdm,
            Arm64ISA::Rng => self.rng,
            Arm64ISA::Sb => self.sb,
            Arm64ISA::Sha2 => self.sha2,
            Arm64ISA::Sha3 => self.sha3,
            Arm64ISA::Simd => self.simd,
            Arm64ISA::Sm4 => self.sm4,
            Arm64ISA::Sme => self.sme,
            Arm64ISA::SmeB16b16 => self.sme_b16b16,
            Arm64ISA::SmeF16f16 => self.sme_f16f16,
            Arm64ISA::SmeF64f64 => self.sme_f64f64,
            Arm64ISA::SmeF8f16 => self.sme_f8f16,
            Arm64ISA::SmeF8f32 => self.sme_f8f32,
            Arm64ISA::SmeFa64 => self.sme_fa64,
            Arm64ISA::SmeI16i64 => self.sme_i16i64,
            Arm64ISA::SmeLutv2 => self.sme_lutv2,
            Arm64ISA::SmeMop4 => self.sme_mop4,
            Arm64ISA::SmeTmop => self.sme_tmop,
            Arm64ISA::Sme2 => self.sme2,
            Arm64ISA::Sme2p1 => self.sme2p1,
            Arm64ISA::Sme2p2 => self.sme2p2,
            Arm64ISA::Sme2p3 => self.sme2p3,
            Arm64ISA::Ssbs => self.ssbs,
            Arm64ISA::SsveAes => self.ssve_aes,
            Arm64ISA::SsveBitperm => self.ssve_bitperm,
            Arm64ISA::SsveFexpa => self.ssve_fexpa,
            Arm64ISA::SsveFp8dot2 => self.ssve_fp8dot2,
            Arm64ISA::SsveFp8dot4 => self.ssve_fp8dot4,
            Arm64ISA::SsveFp8fma => self.ssve_fp8fma,
            Arm64ISA::Sve => self.sve,
            Arm64ISA::SveAes => self.sve_aes,
            Arm64ISA::SveAes2 => self.sve_aes2,
            Arm64ISA::SveB16b16 => self.sve_b16b16,
            Arm64ISA::SveB16mm => self.sve_b16mm,
            Arm64ISA::SveBfscale => self.sve_bfscale,
            Arm64ISA::SveBitperm => self.sve_bitperm,
            Arm64ISA::SveF16f32mm => self.sve_f16f32mm,
            Arm64ISA::SveSha3 => self.sve_sha3,
            Arm64ISA::SveSm4 => self.sve_sm4,
            Arm64ISA::Sve2 => self.sve2,
            Arm64ISA::Sve2Aes => self.sve2_aes,
            Arm64ISA::Sve2Bitperm => self.sve2_bitperm,
            Arm64ISA::Sve2Sha3 => self.sve2_sha3,
            Arm64ISA::Sve2Sm4 => self.sve2_sm4,
            Arm64ISA::Sve2p1 => self.sve2p1,
            Arm64ISA::Sve2p2 => self.sve2p2,
            Arm64ISA::Sve2p3 => self.sve2p3,
            Arm64ISA::Tev => self.tev,
            Arm64ISA::The => self.the_feat,
            Arm64ISA::Tlbid => self.tlbid,
            Arm64ISA::Tlbiw => self.tlbiw,
            Arm64ISA::Wfxt => self.wfxt,
        }
    }

    pub fn set_feature(&mut self, isa: Arm64ISA, enabled: bool) {
        match isa {
            Arm64ISA::None => {}
            Arm64ISA::Aes => self.aes = enabled,
            Arm64ISA::Bf16 => self.bf16 = enabled,
            Arm64ISA::Brbe => self.brbe = enabled,
            Arm64ISA::Bti => self.bti = enabled,
            Arm64ISA::Btie => self.btie = enabled,
            Arm64ISA::Cmpbr => self.cmpbr = enabled,
            Arm64ISA::Cpa => self.cpa = enabled,
            Arm64ISA::Crc => self.crc = enabled,
            Arm64ISA::Crypto => self.crypto = enabled,
            Arm64ISA::Cssc => self.cssc = enabled,
            Arm64ISA::D128 => self.d128 = enabled,
            Arm64ISA::Dit => self.dit = enabled,
            Arm64ISA::Dotprod => self.dotprod = enabled,
            Arm64ISA::F16f32dot => self.f16f32dot = enabled,
            Arm64ISA::F16f32mm => self.f16f32mm = enabled,
            Arm64ISA::F16mm => self.f16mm = enabled,
            Arm64ISA::F32mm => self.f32mm = enabled,
            Arm64ISA::F64mm => self.f64mm = enabled,
            Arm64ISA::F8f16mm => self.f8f16mm = enabled,
            Arm64ISA::F8f32mm => self.f8f32mm = enabled,
            Arm64ISA::Faminmax => self.faminmax = enabled,
            Arm64ISA::Fcma => self.fcma = enabled,
            Arm64ISA::Flagm => self.flagm = enabled,
            Arm64ISA::Fp => self.fp = enabled,
            Arm64ISA::Fp16 => self.fp16 = enabled,
            Arm64ISA::Fp16fml => self.fp16fml = enabled,
            Arm64ISA::Fp8 => self.fp8 = enabled,
            Arm64ISA::Fp8dot2 => self.fp8dot2 = enabled,
            Arm64ISA::Fp8dot4 => self.fp8dot4 = enabled,
            Arm64ISA::Fp8fma => self.fp8fma = enabled,
            Arm64ISA::Fprcvt => self.fprcvt = enabled,
            Arm64ISA::Gcie => self.gcie = enabled,
            Arm64ISA::Gcs => self.gcs = enabled,
            Arm64ISA::Hbc => self.hbc = enabled,
            Arm64ISA::Hinte => self.hinte = enabled,
            Arm64ISA::I8mm => self.i8mm = enabled,
            Arm64ISA::Ite => self.ite = enabled,
            Arm64ISA::Jscvt => self.jscvt = enabled,
            Arm64ISA::Ls64 => self.ls64 = enabled,
            Arm64ISA::Lscp => self.lscp = enabled,
            Arm64ISA::Lse => self.lse = enabled,
            Arm64ISA::Lse128 => self.lse128 = enabled,
            Arm64ISA::Lsfe => self.lsfe = enabled,
            Arm64ISA::Lsui => self.lsui = enabled,
            Arm64ISA::Lut => self.lut = enabled,
            Arm64ISA::Memtag => self.memtag = enabled,
            Arm64ISA::Mops => self.mops = enabled,
            Arm64ISA::MopsGo => self.mops_go = enabled,
            Arm64ISA::Mtetc => self.mtetc = enabled,
            Arm64ISA::Occmo => self.occmo = enabled,
            Arm64ISA::Pauth => self.pauth = enabled,
            Arm64ISA::PauthLr => self.pauth_lr = enabled,
            Arm64ISA::Pmuv3 => self.pmuv3 = enabled,
            Arm64ISA::Poe2 => self.poe2 = enabled,
            Arm64ISA::Pops => self.pops = enabled,
            Arm64ISA::Predres => self.predres = enabled,
            Arm64ISA::Predres2 => self.predres2 = enabled,
            Arm64ISA::Profile => self.profile = enabled,
            Arm64ISA::Ras => self.ras = enabled,
            Arm64ISA::Rasv2 => self.rasv2 = enabled,
            Arm64ISA::Rcpc => self.rcpc = enabled,
            Arm64ISA::Rcpc3 => self.rcpc3 = enabled,
            Arm64ISA::Rdm => self.rdm = enabled,
            Arm64ISA::Rng => self.rng = enabled,
            Arm64ISA::Sb => self.sb = enabled,
            Arm64ISA::Sha2 => self.sha2 = enabled,
            Arm64ISA::Sha3 => self.sha3 = enabled,
            Arm64ISA::Simd => self.simd = enabled,
            Arm64ISA::Sm4 => self.sm4 = enabled,
            Arm64ISA::Sme => self.sme = enabled,
            Arm64ISA::SmeB16b16 => self.sme_b16b16 = enabled,
            Arm64ISA::SmeF16f16 => self.sme_f16f16 = enabled,
            Arm64ISA::SmeF64f64 => self.sme_f64f64 = enabled,
            Arm64ISA::SmeF8f16 => self.sme_f8f16 = enabled,
            Arm64ISA::SmeF8f32 => self.sme_f8f32 = enabled,
            Arm64ISA::SmeFa64 => self.sme_fa64 = enabled,
            Arm64ISA::SmeI16i64 => self.sme_i16i64 = enabled,
            Arm64ISA::SmeLutv2 => self.sme_lutv2 = enabled,
            Arm64ISA::SmeMop4 => self.sme_mop4 = enabled,
            Arm64ISA::SmeTmop => self.sme_tmop = enabled,
            Arm64ISA::Sme2 => self.sme2 = enabled,
            Arm64ISA::Sme2p1 => self.sme2p1 = enabled,
            Arm64ISA::Sme2p2 => self.sme2p2 = enabled,
            Arm64ISA::Sme2p3 => self.sme2p3 = enabled,
            Arm64ISA::Ssbs => self.ssbs = enabled,
            Arm64ISA::SsveAes => self.ssve_aes = enabled,
            Arm64ISA::SsveBitperm => self.ssve_bitperm = enabled,
            Arm64ISA::SsveFexpa => self.ssve_fexpa = enabled,
            Arm64ISA::SsveFp8dot2 => self.ssve_fp8dot2 = enabled,
            Arm64ISA::SsveFp8dot4 => self.ssve_fp8dot4 = enabled,
            Arm64ISA::SsveFp8fma => self.ssve_fp8fma = enabled,
            Arm64ISA::Sve => self.sve = enabled,
            Arm64ISA::SveAes => self.sve_aes = enabled,
            Arm64ISA::SveAes2 => self.sve_aes2 = enabled,
            Arm64ISA::SveB16b16 => self.sve_b16b16 = enabled,
            Arm64ISA::SveB16mm => self.sve_b16mm = enabled,
            Arm64ISA::SveBfscale => self.sve_bfscale = enabled,
            Arm64ISA::SveBitperm => self.sve_bitperm = enabled,
            Arm64ISA::SveF16f32mm => self.sve_f16f32mm = enabled,
            Arm64ISA::SveSha3 => self.sve_sha3 = enabled,
            Arm64ISA::SveSm4 => self.sve_sm4 = enabled,
            Arm64ISA::Sve2 => self.sve2 = enabled,
            Arm64ISA::Sve2Aes => self.sve2_aes = enabled,
            Arm64ISA::Sve2Bitperm => self.sve2_bitperm = enabled,
            Arm64ISA::Sve2Sha3 => self.sve2_sha3 = enabled,
            Arm64ISA::Sve2Sm4 => self.sve2_sm4 = enabled,
            Arm64ISA::Sve2p1 => self.sve2p1 = enabled,
            Arm64ISA::Sve2p2 => self.sve2p2 = enabled,
            Arm64ISA::Sve2p3 => self.sve2p3 = enabled,
            Arm64ISA::Tev => self.tev = enabled,
            Arm64ISA::The => self.the_feat = enabled,
            Arm64ISA::Tlbid => self.tlbid = enabled,
            Arm64ISA::Tlbiw => self.tlbiw = enabled,
            Arm64ISA::Wfxt => self.wfxt = enabled,
        }
    }

    /// Generates Clang ISA argument (-m'arch=<min_arch>+<exts>') using ClangArm64ISANames and ClangArm64NOISANames
    pub fn generate_clang_isaarch(&self, min_arch: MinimumCpuArchitectureArm64, disabled_isas: &[Arm64ISA]) -> String {
        let min_name = MinimumCpuArchitectureArm64ClangNames::name(min_arch);
        let mut parts = Vec::new();
        for &isa in Arm64ISA::all() {
            if self.has_feature(isa) && !disabled_isas.contains(&isa) {
                let name = ClangArm64ISANames::name(isa);
                if !name.is_empty() && name != "none" {
                    parts.push(format!("+{}", name));
                }
            }
        }
        for &isa in disabled_isas {
            let no_name = ClangArm64NOISANames::name(isa);
            if !no_name.is_empty() {
                parts.push(format!("+{}", no_name));
            }
        }
        format!("-m'arch={}{}'", min_name, parts.join(""))
    }

    /// Evaluates the highest minimum architecture supported
    pub fn minimum_architecture(&self) -> MinimumCpuArchitectureArm64 {
        if self.d128 {
            MinimumCpuArchitectureArm64::ARMv9_7A
        } else if self.cmpbr || self.sve2p2 {
            MinimumCpuArchitectureArm64::ARMv9_6A
        } else if self.cpa || self.faminmax || self.fp8 {
            MinimumCpuArchitectureArm64::ARMv9_5A
        } else if self.cssc || self.gcs || self.lse128 || self.sve2p1 {
            MinimumCpuArchitectureArm64::ARMv9_4A
        } else if self.hbc || self.mops {
            MinimumCpuArchitectureArm64::ARMv9_3A
        } else if self.wfxt || self.sme {
            MinimumCpuArchitectureArm64::ARMv9_2A
        } else if self.sve2 || self.sve {
            MinimumCpuArchitectureArm64::ARMv9_A
        } else if self.bf16 || self.i8mm {
            MinimumCpuArchitectureArm64::ARMv8_6A
        } else if self.sb || self.predres {
            MinimumCpuArchitectureArm64::ARMv8_5A
        } else if self.dotprod || self.dit {
            MinimumCpuArchitectureArm64::ARMv8_4A
        } else if self.pauth || self.jscvt || self.fcma {
            MinimumCpuArchitectureArm64::ARMv8_3A
        } else if self.rcpc || self.fp16 {
            MinimumCpuArchitectureArm64::ARMv8_2A
        } else if self.lse || self.crc {
            MinimumCpuArchitectureArm64::ARMv8_1A
        } else if self.simd || self.fp {
            MinimumCpuArchitectureArm64::ARMv8_A
        } else {
            MinimumCpuArchitectureArm64::None
        }
    }

    /// Evaluates optimal vector length (fixed normal VL128 for Arm64)
    pub fn vector_length(&self) -> crate::vector_length::CpuArchitectureVectorLength {
        crate::vector_length::CpuArchitectureVectorLength::VL128
    }

    /// Evaluates vector length (fixed normal VL128 for Arm64)
    pub fn resolve_vector_length(&self, _requested: Option<crate::vector_length::CpuArchitectureVectorLength>) -> crate::vector_length::CpuArchitectureVectorLength {
        crate::vector_length::CpuArchitectureVectorLength::VL128
    }

    /// Generates '+' delimited enabled extensions string
    pub fn to_extensions_string(&self) -> String {
        let mut extensions = Vec::new();
        for &isa in Arm64ISA::all() {
            if self.has_feature(isa) {
                let name = isa.as_str();
                if !name.is_empty() && name != "none" {
                    extensions.push(name);
                }
            }
        }
        extensions.join("+")
    }

    /// Creates a key-value dictionary of boolean features
    pub fn to_map(&self) -> BTreeMap<String, bool> {
        let mut map = BTreeMap::new();
        for &isa in Arm64ISA::all() {
            let name = isa.as_str();
            if !name.is_empty() && name != "none" {
                map.insert(name.replace(['-', '.'], "_"), self.has_feature(isa));
            }
        }
        map
    }
}
