// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/x86_64/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-14

pub mod cpuid;
pub mod isa;
pub mod names;
pub mod targets;
pub mod xcr0;

use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};

pub use isa::X64ISA;
pub use names::*;
pub use targets::*;
pub use xcr0::Xcr0State;

use crate::archinfo::vector_length::CpuArchitectureVectorLength;

/// Represents the detected or configured CPU features for x86_64 architecture.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct X64CPUFeatures {
    pub adx: bool,
    pub aes: bool,
    pub amx_avx512: bool,
    pub amx_bf16: bool,
    pub amx_complex: bool,
    pub amx_fp16: bool,
    pub amx_fp8: bool,
    pub amx_int8: bool,
    pub amx_movrs: bool,
    pub amx_tf32: bool,
    pub amx_tile: bool,
    pub apxf: bool,
    pub avx: bool,
    pub avx10_1: bool,
    pub avx10_2: bool,
    pub avx2: bool,
    pub avx512bf16: bool,
    pub avx512bitalg: bool,
    pub avx512bw: bool,
    pub avx512bmm: bool,
    pub avx512cd: bool,
    pub avx512dq: bool,
    pub avx512f: bool,
    pub avx512fp16: bool,
    pub avx512ifma: bool,
    pub avx512vbmi: bool,
    pub avx512vbmi2: bool,
    pub avx512vl: bool,
    pub avx512vnni: bool,
    pub avx512vp2intersect: bool,
    pub avx512vpopcntdq: bool,
    pub avxifma: bool,
    pub avxneconvert: bool,
    pub avxvnni: bool,
    pub avxvnniint16: bool,
    pub avxvnniint8: bool,
    pub bmi: bool,
    pub bmi2: bool,
    pub cldemote: bool,
    pub clflushopt: bool,
    pub clwb: bool,
    pub clzero: bool,
    pub cmpccxadd: bool,
    pub crc32: bool,
    pub cx16: bool,
    pub enqcmd: bool,
    pub f16c: bool,
    pub fma: bool,
    pub fma4: bool,
    pub fsgsbase: bool,
    pub fxsr: bool,
    pub gfni: bool,
    pub kl: bool,
    pub lwp: bool,
    pub lzcnt: bool,
    pub mmx: bool,
    pub movbe: bool,
    pub movdir64b: bool,
    pub movdiri: bool,
    pub movrs: bool,
    pub mwaitx: bool,
    pub pclmul: bool,
    pub pku: bool,
    pub popcnt: bool,
    pub prefetchi: bool,
    pub prfchw: bool,
    pub ptwrite: bool,
    pub raoint: bool,
    pub rdpid: bool,
    pub rdpru: bool,
    pub rdrnd: bool,
    pub rdseed: bool,
    pub rtm: bool,
    pub sahf: bool,
    pub serialize: bool,
    pub sgx: bool,
    pub sha: bool,
    pub sha512: bool,
    pub sm4: bool,
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub sse4a: bool,
    pub ssse3: bool,
    pub tbm: bool,
    pub tsxldtrk: bool,
    pub uintr: bool,
    pub usermsr: bool,
    pub vaes: bool,
    pub vpclmulqdq: bool,
    pub vzeroupper: bool,
    pub waitpkg: bool,
	pub aeskl: bool,
    pub widekl: bool,
    pub x87: bool,
    pub xop: bool,
    pub xsave: bool,
    pub osxsave: bool,
    pub xsavec: bool,
    pub xsaveopt: bool,
    pub xsaves: bool,
    pub avx_usable: bool,
    pub avx2_usable: bool,
    pub avx512_usable: bool,
    pub avx10v1_usable: bool,
    pub avx10v2_usable: bool,
    pub apxf_usable: bool,
}

impl X64CPUFeatures {
    /// Detects all features on the current host machine using CPUID and XCR0 register checks
    pub fn detect_host() -> Self {
        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::default()
        }

        #[cfg(target_arch = "x86_64")]
        {
            let mut f = Self::default();

            let leaf0 = cpuid::query_cpuid(0);
            let max_leaf = leaf0.eax;

            if max_leaf >= 1 {
                let leaf1 = cpuid::query_cpuid(1);
                f.sse3 = (leaf1.ecx & (1 << 0)) != 0;
                f.pclmul = (leaf1.ecx & (1 << 1)) != 0;
                f.ssse3 = (leaf1.ecx & (1 << 9)) != 0;
                f.fma = (leaf1.ecx & (1 << 12)) != 0;
                f.cx16 = (leaf1.ecx & (1 << 13)) != 0;
                f.sse4_1 = (leaf1.ecx & (1 << 19)) != 0;
                f.sse4_2 = (leaf1.ecx & (1 << 20)) != 0;
                f.crc32 = f.sse4_2;
                f.movbe = (leaf1.ecx & (1 << 22)) != 0;
                f.popcnt = (leaf1.ecx & (1 << 23)) != 0;
                f.aes = (leaf1.ecx & (1 << 25)) != 0;
                f.xsave = (leaf1.ecx & (1 << 26)) != 0;
                f.osxsave = (leaf1.ecx & (1 << 27)) != 0;
                let cpu_avx = (leaf1.ecx & (1 << 28)) != 0;
                f.f16c = (leaf1.ecx & (1 << 29)) != 0;
                f.rdrnd = (leaf1.ecx & (1 << 30)) != 0;

                f.fxsr = (leaf1.edx & (1 << 24)) != 0;
                f.sse = (leaf1.edx & (1 << 25)) != 0;
                f.sse2 = (leaf1.edx & (1 << 26)) != 0;

                let xcr0_state = Xcr0State::query(f.osxsave);
                f.avx_usable = xcr0_state.is_avx_usable();
                f.avx2_usable = f.avx_usable;
                f.avx512_usable = xcr0_state.is_avx512_usable();
				f.avx10v1_usable = f.avx512_usable;
				f.avx10v2_usable = f.avx10v1_usable;
                f.apxf_usable = xcr0_state.is_apx_usable();

                f.avx = cpu_avx && f.avx_usable;

                if max_leaf >= 7 {
                    let leaf7 = cpuid::query_cpuid_count(7, 0);
                    f.fsgsbase = (leaf7.ebx & (1 << 0)) != 0;
                    f.sgx = (leaf7.ebx & (1 << 2)) != 0;
                    f.bmi = (leaf7.ebx & (1 << 3)) != 0;
                    f.avx2 = ((leaf7.ebx & (1 << 5)) != 0) && f.avx2_usable;
                    f.pku = (leaf7.ebx & (1 << 7)) != 0;
                    f.bmi2 = (leaf7.ebx & (1 << 8)) != 0;
                    f.rtm = (leaf7.ebx & (1 << 11)) != 0;
                    f.avx512f = ((leaf7.ebx & (1 << 16)) != 0) && f.avx512_usable;
                    f.avx512dq = ((leaf7.ebx & (1 << 17)) != 0) && f.avx512f;
                    f.rdseed = (leaf7.ebx & (1 << 18)) != 0;
                    f.adx = (leaf7.ebx & (1 << 19)) != 0;
                    f.avx512ifma = ((leaf7.ebx & (1 << 21)) != 0) && f.avx512f;
                    f.clflushopt = (leaf7.ebx & (1 << 23)) != 0;
                    f.clwb = (leaf7.ebx & (1 << 24)) != 0;
                    f.avx512cd = ((leaf7.ebx & (1 << 28)) != 0) && f.avx512f;
                    f.sha = (leaf7.ebx & (1 << 29)) != 0;
                    f.avx512bw = ((leaf7.ebx & (1 << 30)) != 0) && f.avx512f;
                    f.avx512vl = ((leaf7.ebx & (1 << 31)) != 0) && f.avx512f;

                    f.avx512vbmi = ((leaf7.ecx & (1 << 1)) != 0) && f.avx512f;
                    f.waitpkg = (leaf7.ecx & (1 << 5)) != 0;
                    f.avx512vbmi2 = ((leaf7.ecx & (1 << 6)) != 0) && f.avx512f;
                    f.gfni = (leaf7.ecx & (1 << 8)) != 0;
                    f.vaes = (leaf7.ecx & (1 << 9)) != 0;
                    f.vpclmulqdq = (leaf7.ecx & (1 << 10)) != 0;
                    f.avx512vnni = ((leaf7.ecx & (1 << 11)) != 0) && f.avx512f;
                    f.avx512bitalg = ((leaf7.ecx & (1 << 12)) != 0) && f.avx512f;
                    f.avx512vpopcntdq = ((leaf7.ecx & (1 << 14)) != 0) && f.avx512f;
                    f.rdpid = (leaf7.ecx & (1 << 22)) != 0;
                    f.kl = (leaf7.ecx & (1 << 23)) != 0;
                    f.cldemote = (leaf7.ecx & (1 << 25)) != 0;
                    f.movdiri = (leaf7.ecx & (1 << 27)) != 0;
                    f.movdir64b = (leaf7.ecx & (1 << 28)) != 0;
                    f.enqcmd = (leaf7.ecx & (1 << 29)) != 0;

                    f.uintr = (leaf7.edx & (1 << 5)) != 0;
                    f.avx512vp2intersect = ((leaf7.edx & (1 << 8)) != 0) && f.avx512f;
                    f.serialize = (leaf7.edx & (1 << 14)) != 0;
                    f.tsxldtrk = (leaf7.edx & (1 << 16)) != 0;
                    f.avx512fp16 = ((leaf7.edx & (1 << 23)) != 0) && f.avx512f;

                    let leaf7_1 = cpuid::query_cpuid_count(7, 1);
                    f.sha512 = (leaf7_1.eax & (1 << 0)) != 0;
                    f.sm4 = (leaf7_1.eax & (1 << 2)) != 0;
                    f.raoint = (leaf7_1.eax & (1 << 3)) != 0;
                    f.avxvnni = (leaf7_1.eax & (1 << 4)) != 0;
                    f.avx512bf16 = ((leaf7_1.eax & (1 << 5)) != 0) && f.avx512f;
                    f.cmpccxadd = (leaf7_1.eax & (1 << 7)) != 0;
                    f.amx_fp16 = (leaf7_1.eax & (1 << 21)) != 0;
                    f.avxifma = (leaf7_1.eax & (1 << 23)) != 0;
                    f.movrs = (leaf7_1.eax & (1 << 31)) != 0;

                    f.avxvnniint8 = (leaf7_1.edx & (1 << 4)) != 0;
                    f.avxneconvert = (leaf7_1.edx & (1 << 5)) != 0;
                    f.amx_complex = (leaf7_1.edx & (1 << 8)) != 0;
                    f.avxvnniint16 = (leaf7_1.edx & (1 << 10)) != 0;
                    f.prefetchi = (leaf7_1.edx & (1 << 14)) != 0;
                    f.usermsr = (leaf7_1.edx & (1 << 15)) != 0;
                    f.avx10_1 = ((leaf7_1.edx & (1 << 19)) != 0) && f.avx10v1_usable;
                    f.apxf = ((leaf7_1.edx & (1 << 21)) != 0) && f.apxf_usable;
                    f.amx_bf16 = (leaf7_1.edx & (1 << 22)) != 0;
                    f.amx_tile = (leaf7_1.edx & (1 << 24)) != 0;
                    f.amx_int8 = (leaf7_1.edx & (1 << 25)) != 0;
                }

                if max_leaf >= 0xd {
                    let leaf_d1 = cpuid::query_cpuid_count(0xd, 1);
                    f.xsaveopt = (leaf_d1.eax & (1 << 0)) != 0;
                    f.xsavec = (leaf_d1.eax & (1 << 1)) != 0;
                    f.xsaves = (leaf_d1.eax & (1 << 3)) != 0;
                }

                if max_leaf >= 0x14 {
                    let leaf14 = cpuid::query_cpuid_count(0x14, 0);
                    f.ptwrite = (leaf14.ebx & (1 << 4)) != 0;
                }

                if max_leaf >= 0x19 {
                    let leaf19 = cpuid::query_cpuid_count(0x19, 0);
                    f.aeskl = ((leaf19.ebx & (1 << 0)) != 0) && f.kl;
                    f.widekl = ((leaf19.ebx & (1 << 2)) != 0) && f.kl;
                }

                if max_leaf >= 0x1e {
                    let leaf1e_1 = cpuid::query_cpuid_count(0x1e, 1);
                    f.amx_int8 = f.amx_int8 || ((leaf1e_1.eax & (1 << 0)) != 0);
                    f.amx_bf16 = f.amx_bf16 || ((leaf1e_1.eax & (1 << 1)) != 0);
                    f.amx_complex = f.amx_complex || ((leaf1e_1.eax & (1 << 2)) != 0);
                    f.amx_fp16 = f.amx_fp16 || ((leaf1e_1.eax & (1 << 3)) != 0);
                    f.amx_fp8 = (leaf1e_1.eax & (1 << 4)) != 0;
                    f.amx_tf32 = (leaf1e_1.eax & (1 << 6)) != 0;
                    f.amx_avx512 = (leaf1e_1.eax & (1 << 7)) != 0;
                    f.amx_movrs = (leaf1e_1.eax & (1 << 8)) != 0;
                }

                if max_leaf >= 0x21 {
                    let leaf21 = cpuid::query_cpuid_count(0x21, 0);
                    f.avx512bmm = ((leaf21.eax & (1 << 23)) != 0) && f.avx512f;
                }

                if max_leaf >= 0x24 && f.avx10_1 {
                    let leaf24 = cpuid::query_cpuid_count(0x24, 0);
                    f.avx10_2 = ((leaf24.ebx & 0xFF) >= 2) && f.avx10v2_usable;
                }
            }

            // Extended leaves (0x80000000+)
            let ext_leaf0 = cpuid::query_cpuid(0x80000000);
            if ext_leaf0.eax >= 0x80000001 {
                let ext1 = cpuid::query_cpuid(0x80000001);
                f.sahf = (ext1.ecx & (1 << 0)) != 0;
                f.lzcnt = (ext1.ecx & (1 << 5)) != 0;
                f.sse4a = (ext1.ecx & (1 << 6)) != 0;
                f.prfchw = (ext1.ecx & (1 << 8)) != 0;
                f.xop = (ext1.ecx & (1 << 11)) != 0;
                f.lwp = (ext1.ecx & (1 << 15)) != 0;
                f.fma4 = (ext1.ecx & (1 << 16)) != 0;
                f.tbm = (ext1.ecx & (1 << 21)) != 0;
                f.mwaitx = (ext1.ecx & (1 << 29)) != 0;
            }

            if ext_leaf0.eax >= 0x80000008 {
                let ext8 = cpuid::query_cpuid(0x80000008);
                f.clzero = (ext8.ebx & (1 << 0)) != 0;
                f.rdpru = (ext8.ebx & (1 << 4)) != 0;
            }

            f
        }
    }

    /// Constructs features from a '+' delimited extension string
    pub fn from_extensions_str(ext_str: &str) -> Self {
        let tokens: HashSet<&str> = ext_str.split('+').filter(|s| !s.is_empty()).collect();
        let mut f = Self::default();
        f.adx = tokens.contains("adx");
        f.aes = tokens.contains("aes");
        f.amx_avx512 = tokens.contains("amx_avx512") || tokens.contains("amx-avx512");
        f.amx_bf16 = tokens.contains("amx_bf16") || tokens.contains("amx-bf16");
        f.amx_complex = tokens.contains("amx_complex") || tokens.contains("amx-complex");
        f.amx_fp16 = tokens.contains("amx_fp16") || tokens.contains("amx-fp16");
        f.amx_fp8 = tokens.contains("amx_fp8") || tokens.contains("amx-fp8");
        f.amx_int8 = tokens.contains("amx_int8") || tokens.contains("amx-int8");
        f.amx_movrs = tokens.contains("amx_movrs") || tokens.contains("amx-movrs");
        f.amx_tf32 = tokens.contains("amx_tf32") || tokens.contains("amx-tf32");
        f.apxf = tokens.contains("apxf") || tokens.contains("apx_f") || tokens.contains("apx");
        f.avx = tokens.contains("avx");
        f.avx10_1 = tokens.contains("avx10.1-512") || tokens.contains("avx10.1_512") || tokens.contains("avx10_1") || tokens.contains("avx10.1");
        f.avx10_2 = tokens.contains("avx10.2-512") || tokens.contains("avx10.2_512") || tokens.contains("avx10_2") || tokens.contains("avx10.2");
        f.avx2 = tokens.contains("avx2");
        f.avx512bf16 = tokens.contains("avx512bf16");
        f.avx512bitalg = tokens.contains("avx512bitalg");
        f.avx512bmm = tokens.contains("avx512bmm");
        f.avx512bw = tokens.contains("avx512bw");
        f.avx512cd = tokens.contains("avx512cd");
        f.avx512dq = tokens.contains("avx512dq");
        f.avx512f = tokens.contains("avx512f");
        f.avx512fp16 = tokens.contains("avx512fp16");
        f.avx512ifma = tokens.contains("avx512ifma");
        f.avx512vbmi = tokens.contains("avx512vbmi");
        f.avx512vbmi2 = tokens.contains("avx512vbmi2");
        f.avx512vl = tokens.contains("avx512vl");
        f.avx512vnni = tokens.contains("avx512vnni");
        f.avx512vp2intersect = tokens.contains("avx512vp2intersect");
        f.avx512vpopcntdq = tokens.contains("avx512vpopcntdq");
        f.avxifma = tokens.contains("avxifma");
        f.avxneconvert = tokens.contains("avxneconvert");
        f.avxvnni = tokens.contains("avxvnni");
        f.avxvnniint16 = tokens.contains("avxvnniint16");
        f.avxvnniint8 = tokens.contains("avxvnniint8");
        f.bmi = tokens.contains("bmi");
        f.bmi2 = tokens.contains("bmi2");
        f.cldemote = tokens.contains("cldemote");
        f.clflushopt = tokens.contains("clflushopt");
        f.clwb = tokens.contains("clwb");
        f.clzero = tokens.contains("clzero");
        f.cmpccxadd = tokens.contains("cmpccxadd");
        f.crc32 = tokens.contains("crc32");
        f.cx16 = tokens.contains("cx16");
        f.enqcmd = tokens.contains("enqcmd");
        f.f16c = tokens.contains("f16c");
        f.fma = tokens.contains("fma");
        f.fma4 = tokens.contains("fma4");
        f.fsgsbase = tokens.contains("fsgsbase");
        f.fxsr = tokens.contains("fxsr");
        f.gfni = tokens.contains("gfni");
        f.kl = tokens.contains("kl");
        f.lwp = tokens.contains("lwp");
        f.lzcnt = tokens.contains("lzcnt");
        f.movbe = tokens.contains("movbe");
        f.movdir64b = tokens.contains("movdir64b");
        f.movdiri = tokens.contains("movdiri");
        f.movrs = tokens.contains("movrs");
        f.mwaitx = tokens.contains("mwaitx");
        f.pclmul = tokens.contains("pclmul");
        f.pku = tokens.contains("pku");
        f.popcnt = tokens.contains("popcnt");
        f.prefetchi = tokens.contains("prefetchi");
        f.prfchw = tokens.contains("prfchw");
        f.ptwrite = tokens.contains("ptwrite");
        f.raoint = tokens.contains("raoint");
        f.rdpid = tokens.contains("rdpid");
        f.rdpru = tokens.contains("rdpru");
        f.rdrnd = tokens.contains("rdrnd");
        f.rdseed = tokens.contains("rdseed");
        f.rtm = tokens.contains("rtm");
        f.sahf = tokens.contains("sahf");
        f.serialize = tokens.contains("serialize");
        f.sgx = tokens.contains("sgx");
        f.sha = tokens.contains("sha");
        f.sha512 = tokens.contains("sha512");
        f.sm4 = tokens.contains("sm4");
        f.sse = tokens.contains("sse");
        f.sse2 = tokens.contains("sse2");
        f.sse3 = tokens.contains("sse3");
        f.sse4_1 = tokens.contains("sse4.1") || tokens.contains("sse4_1");
        f.sse4_2 = tokens.contains("sse4.2") || tokens.contains("sse4_2");
        f.sse4a = tokens.contains("sse4a");
        f.ssse3 = tokens.contains("ssse3");
        f.tbm = tokens.contains("tbm");
        f.tsxldtrk = tokens.contains("tsxldtrk");
        f.uintr = tokens.contains("uintr");
        f.usermsr = tokens.contains("usermsr");
        f.vaes = tokens.contains("vaes");
        f.vpclmulqdq = tokens.contains("vpclmulqdq");
        f.vzeroupper = tokens.contains("vzeroupper");
        f.waitpkg = tokens.contains("waitpkg");
        f.aeskl = tokens.contains("aeskl");
        f.widekl = tokens.contains("widekl");
        f.xop = tokens.contains("xop");
        f.xsave = tokens.contains("xsave");
        f.xsavec = tokens.contains("xsavec");
        f.xsaveopt = tokens.contains("xsaveopt");
        f.xsaves = tokens.contains("xsaves");
        f.avx_usable = f.avx;
        f.avx2_usable = f.avx2;
        f.avx512_usable = f.avx512f;
        f.avx10v1_usable = f.avx10_1;
        f.avx10v2_usable = f.avx10_2;
        f.apxf_usable = f.apxf;
        f
    }

    /// Constructs features from a known TargetCpuArchitectureX64
    pub fn from_target(target: TargetCpuArchitectureX64) -> Self {
        match target {
            TargetCpuArchitectureX64::Native => Self::detect_host(),
            other => {
                let ext = ClangTargetCpuArchitectureX64ISANames::name(other);
                Self::from_extensions_str(ext)
            }
        }
    }

    /// Constructs features for Generic target given a requested MinimumCpuArchitectureX64
    pub fn from_min_arch(min_arch: MinimumCpuArchitectureX64) -> Self {
        match min_arch {
            MinimumCpuArchitectureX64::None => {
                Self::from_extensions_str("sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf")
            }
            MinimumCpuArchitectureX64::AVX => {
                Self::from_extensions_str("sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+avx+xsave")
            }
            MinimumCpuArchitectureX64::AVX2 => {
                Self::from_extensions_str("sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+fsgsbase+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c")
            }
            MinimumCpuArchitectureX64::AVX512 => {
                Self::from_extensions_str("sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+fsgsbase+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl")
            }
            MinimumCpuArchitectureX64::AVX10_1 => {
				Self::from_extensions_str("sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+fsgsbase+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+pku+gfni+movdiri+movdir64b+sha+vpclmulqdq+avxifma+avxneconvert+avxvnni+prefetchi+avx10_1")
            }
            MinimumCpuArchitectureX64::AVX10_2 => {
				Self::from_extensions_str("sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+fsgsbase+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+pku+gfni+movdiri+movdir64b+sha+vpclmulqdq+avxifma+avxneconvert+avxvnni+prefetchi+avx10_1+avxvnniint8+avxvnniint16+cmpccxadd+movrs+avx10_2")
            }
        }
    }

    pub fn has_feature(&self, isa: X64ISA) -> bool {
        match isa {
            X64ISA::None => false,
            X64ISA::Adx => self.adx,
            X64ISA::Aes => self.aes,
            X64ISA::AmxAvx512 => self.amx_avx512,
            X64ISA::AmxBf16 => self.amx_bf16,
            X64ISA::AmxComplex => self.amx_complex,
            X64ISA::AmxFp16 => self.amx_fp16,
            X64ISA::AmxFp8 => self.amx_fp8,
            X64ISA::AmxInt8 => self.amx_int8,
            X64ISA::AmxMovrs => self.amx_movrs,
            X64ISA::AmxTf32 => self.amx_tf32,
            X64ISA::AmxTile => self.amx_tile,
            X64ISA::Apxf => self.apxf,
            X64ISA::Avx => self.avx,
            X64ISA::Avx10_1 => self.avx10_1,
            X64ISA::Avx10_2 => self.avx10_2,
            X64ISA::Avx2 => self.avx2,
            X64ISA::Avx512bf16 => self.avx512bf16,
            X64ISA::Avx512bitalg => self.avx512bitalg,
            X64ISA::Avx512bw => self.avx512bw,
            X64ISA::Avx512bmm => self.avx512bmm,
            X64ISA::Avx512cd => self.avx512cd,
            X64ISA::Avx512dq => self.avx512dq,
            X64ISA::Avx512f => self.avx512f,
            X64ISA::Avx512fp16 => self.avx512fp16,
            X64ISA::Avx512ifma => self.avx512ifma,
            X64ISA::Avx512vbmi => self.avx512vbmi,
            X64ISA::Avx512vbmi2 => self.avx512vbmi2,
            X64ISA::Avx512vl => self.avx512vl,
            X64ISA::Avx512vnni => self.avx512vnni,
            X64ISA::Avx512vp2intersect => self.avx512vp2intersect,
            X64ISA::Avx512vpopcntdq => self.avx512vpopcntdq,
            X64ISA::Avxifma => self.avxifma,
            X64ISA::Avxneconvert => self.avxneconvert,
            X64ISA::Avxvnni => self.avxvnni,
            X64ISA::Avxvnniint16 => self.avxvnniint16,
            X64ISA::Avxvnniint8 => self.avxvnniint8,
            X64ISA::Bmi => self.bmi,
            X64ISA::Bmi2 => self.bmi2,
            X64ISA::Cldemote => self.cldemote,
            X64ISA::Clflushopt => self.clflushopt,
            X64ISA::Clwb => self.clwb,
            X64ISA::Clzero => self.clzero,
            X64ISA::Cmpccxadd => self.cmpccxadd,
            X64ISA::Crc32 => self.crc32,
            X64ISA::Cx16 => self.cx16,
            X64ISA::Enqcmd => self.enqcmd,
            X64ISA::F16c => self.f16c,
            X64ISA::Fma => self.fma,
            X64ISA::Fma4 => self.fma4,
            X64ISA::Fsgsbase => self.fsgsbase,
            X64ISA::Fxsr => self.fxsr,
            X64ISA::Gfni => self.gfni,
            X64ISA::Kl => self.kl,
            X64ISA::Lwp => self.lwp,
            X64ISA::Lzcnt => self.lzcnt,
            X64ISA::Movbe => self.movbe,
            X64ISA::Movdir64b => self.movdir64b,
            X64ISA::Movdiri => self.movdiri,
            X64ISA::Movrs => self.movrs,
            X64ISA::Mwaitx => self.mwaitx,
            X64ISA::Pclmul => self.pclmul,
            X64ISA::Pku => self.pku,
            X64ISA::Popcnt => self.popcnt,
            X64ISA::Prefetchi => self.prefetchi,
            X64ISA::Prfchw => self.prfchw,
            X64ISA::Ptwrite => self.ptwrite,
            X64ISA::Raoint => self.raoint,
            X64ISA::Rdpid => self.rdpid,
            X64ISA::Rdpru => self.rdpru,
            X64ISA::Rdrnd => self.rdrnd,
            X64ISA::Rdseed => self.rdseed,
            X64ISA::Rtm => self.rtm,
            X64ISA::Sahf => self.sahf,
            X64ISA::Serialize => self.serialize,
            X64ISA::Sgx => self.sgx,
            X64ISA::Sha => self.sha,
            X64ISA::Sha512 => self.sha512,
            X64ISA::Sm4 => self.sm4,
            X64ISA::Sse => self.sse,
            X64ISA::Sse2 => self.sse2,
            X64ISA::Sse3 => self.sse3,
            X64ISA::Sse4_1 => self.sse4_1,
            X64ISA::Sse4_2 => self.sse4_2,
            X64ISA::Sse4a => self.sse4a,
            X64ISA::Ssse3 => self.ssse3,
            X64ISA::Tbm => self.tbm,
            X64ISA::Tsxldtrk => self.tsxldtrk,
            X64ISA::Uintr => self.uintr,
            X64ISA::Usermsr => self.usermsr,
            X64ISA::Vaes => self.vaes,
            X64ISA::Vpclmulqdq => self.vpclmulqdq,
            X64ISA::Vzeroupper => self.vzeroupper,
            X64ISA::Waitpkg => self.waitpkg,
            X64ISA::Aeskl => self.aeskl,
            X64ISA::Widekl => self.widekl,
            X64ISA::Xop => self.xop,
            X64ISA::Xsave => self.xsave,
            X64ISA::Xsavec => self.xsavec,
            X64ISA::Xsaveopt => self.xsaveopt,
            X64ISA::Xsaves => self.xsaves,
        }
    }

    pub fn set_feature(&mut self, isa: X64ISA, enabled: bool) {
        match isa {
            X64ISA::None => {}
            X64ISA::Adx => self.adx = enabled,
            X64ISA::Aes => self.aes = enabled,
            X64ISA::AmxAvx512 => self.amx_avx512 = enabled,
            X64ISA::AmxBf16 => self.amx_bf16 = enabled,
            X64ISA::AmxComplex => self.amx_complex = enabled,
            X64ISA::AmxFp16 => self.amx_fp16 = enabled,
            X64ISA::AmxFp8 => self.amx_fp8 = enabled,
            X64ISA::AmxInt8 => self.amx_int8 = enabled,
            X64ISA::AmxMovrs => self.amx_movrs = enabled,
            X64ISA::AmxTf32 => self.amx_tf32 = enabled,
            X64ISA::AmxTile => self.amx_tile = enabled,
            X64ISA::Apxf => self.apxf = enabled,
            X64ISA::Avx => self.avx = enabled,
            X64ISA::Avx10_1 => self.avx10_1 = enabled,
            X64ISA::Avx10_2 => self.avx10_2 = enabled,
            X64ISA::Avx2 => self.avx2 = enabled,
            X64ISA::Avx512bf16 => self.avx512bf16 = enabled,
            X64ISA::Avx512bitalg => self.avx512bitalg = enabled,
            X64ISA::Avx512bw => self.avx512bw = enabled,
            X64ISA::Avx512bmm => self.avx512bmm = enabled,
            X64ISA::Avx512cd => self.avx512cd = enabled,
            X64ISA::Avx512dq => self.avx512dq = enabled,
            X64ISA::Avx512f => self.avx512f = enabled,
            X64ISA::Avx512fp16 => self.avx512fp16 = enabled,
            X64ISA::Avx512ifma => self.avx512ifma = enabled,
            X64ISA::Avx512vbmi => self.avx512vbmi = enabled,
            X64ISA::Avx512vbmi2 => self.avx512vbmi2 = enabled,
            X64ISA::Avx512vl => self.avx512vl = enabled,
            X64ISA::Avx512vnni => self.avx512vnni = enabled,
            X64ISA::Avx512vp2intersect => self.avx512vp2intersect = enabled,
            X64ISA::Avx512vpopcntdq => self.avx512vpopcntdq = enabled,
            X64ISA::Avxifma => self.avxifma = enabled,
            X64ISA::Avxneconvert => self.avxneconvert = enabled,
            X64ISA::Avxvnni => self.avxvnni = enabled,
            X64ISA::Avxvnniint16 => self.avxvnniint16 = enabled,
            X64ISA::Avxvnniint8 => self.avxvnniint8 = enabled,
            X64ISA::Bmi => self.bmi = enabled,
            X64ISA::Bmi2 => self.bmi2 = enabled,
            X64ISA::Cldemote => self.cldemote = enabled,
            X64ISA::Clflushopt => self.clflushopt = enabled,
            X64ISA::Clwb => self.clwb = enabled,
            X64ISA::Clzero => self.clzero = enabled,
            X64ISA::Cmpccxadd => self.cmpccxadd = enabled,
            X64ISA::Crc32 => self.crc32 = enabled,
            X64ISA::Cx16 => self.cx16 = enabled,
            X64ISA::Enqcmd => self.enqcmd = enabled,
            X64ISA::F16c => self.f16c = enabled,
            X64ISA::Fma => self.fma = enabled,
            X64ISA::Fma4 => self.fma4 = enabled,
            X64ISA::Fsgsbase => self.fsgsbase = enabled,
            X64ISA::Fxsr => self.fxsr = enabled,
            X64ISA::Gfni => self.gfni = enabled,
            X64ISA::Kl => self.kl = enabled,
            X64ISA::Lwp => self.lwp = enabled,
            X64ISA::Lzcnt => self.lzcnt = enabled,
            X64ISA::Movbe => self.movbe = enabled,
            X64ISA::Movdir64b => self.movdir64b = enabled,
            X64ISA::Movdiri => self.movdiri = enabled,
            X64ISA::Movrs => self.movrs = enabled,
            X64ISA::Mwaitx => self.mwaitx = enabled,
            X64ISA::Pclmul => self.pclmul = enabled,
            X64ISA::Pku => self.pku = enabled,
            X64ISA::Popcnt => self.popcnt = enabled,
            X64ISA::Prefetchi => self.prefetchi = enabled,
            X64ISA::Prfchw => self.prfchw = enabled,
            X64ISA::Ptwrite => self.ptwrite = enabled,
            X64ISA::Raoint => self.raoint = enabled,
            X64ISA::Rdpid => self.rdpid = enabled,
            X64ISA::Rdpru => self.rdpru = enabled,
            X64ISA::Rdrnd => self.rdrnd = enabled,
            X64ISA::Rdseed => self.rdseed = enabled,
            X64ISA::Rtm => self.rtm = enabled,
            X64ISA::Sahf => self.sahf = enabled,
            X64ISA::Serialize => self.serialize = enabled,
            X64ISA::Sgx => self.sgx = enabled,
            X64ISA::Sha => self.sha = enabled,
            X64ISA::Sha512 => self.sha512 = enabled,
            X64ISA::Sm4 => self.sm4 = enabled,
            X64ISA::Sse => self.sse = enabled,
            X64ISA::Sse2 => self.sse2 = enabled,
            X64ISA::Sse3 => self.sse3 = enabled,
            X64ISA::Sse4_1 => self.sse4_1 = enabled,
            X64ISA::Sse4_2 => self.sse4_2 = enabled,
            X64ISA::Sse4a => self.sse4a = enabled,
            X64ISA::Ssse3 => self.ssse3 = enabled,
            X64ISA::Tbm => self.tbm = enabled,
            X64ISA::Tsxldtrk => self.tsxldtrk = enabled,
            X64ISA::Uintr => self.uintr = enabled,
            X64ISA::Usermsr => self.usermsr = enabled,
            X64ISA::Vaes => self.vaes = enabled,
            X64ISA::Vpclmulqdq => self.vpclmulqdq = enabled,
            X64ISA::Vzeroupper => self.vzeroupper = enabled,
            X64ISA::Waitpkg => self.waitpkg = enabled,
            X64ISA::Aeskl => self.aeskl = enabled,
            X64ISA::Widekl => self.widekl = enabled,
            X64ISA::Xop => self.xop = enabled,
            X64ISA::Xsave => self.xsave = enabled,
            X64ISA::Xsavec => self.xsavec = enabled,
            X64ISA::Xsaveopt => self.xsaveopt = enabled,
            X64ISA::Xsaves => self.xsaves = enabled,
        }
    }

    /// Generates Clang ISA arguments (-m<ext>) using ClangX64ISANames and ClangX64NOISANames
    pub fn generate_clang_isaarch(&self, vl: CpuArchitectureVectorLength, disabled_isas: &[X64ISA]) -> String {
        let mut extensions = Vec::new();
        for &isa in X64ISA::all() {
            if self.has_feature(isa) && !disabled_isas.contains(&isa) {
                let name = match isa {
                    X64ISA::Avx10_1 | X64ISA::Avx10_2 => {
                        if vl == CpuArchitectureVectorLength::VL512 {
                            format!("{}-512", ClangX64ISANames::name(isa))
                        } else {
                            ClangX64ISANames::name(isa).to_string()
                        }
                    }
                    _ => ClangX64ISANames::name(isa).to_string(),
                };
                if !name.is_empty() {
                    extensions.push(format!("-m'{}'", name));
                }
            }
        }
        for &isa in disabled_isas {
            let no_name = ClangX64NOISANames::name(isa);
            if !no_name.is_empty() {
                extensions.push(format!("-m'{}'", no_name));
            }
        }
        extensions.join(" ")
    }

    /// Evaluates the highest minimum architecture supported
    pub fn minimum_architecture(&self) -> MinimumCpuArchitectureX64 {
        if self.avx10_2 {
            MinimumCpuArchitectureX64::AVX10_2
        } else if self.avx10_1 {
            MinimumCpuArchitectureX64::AVX10_1
        } else if self.avx512f {
            MinimumCpuArchitectureX64::AVX512
        } else if self.avx2 {
            MinimumCpuArchitectureX64::AVX2
        } else if self.avx {
            MinimumCpuArchitectureX64::AVX
        } else {
            MinimumCpuArchitectureX64::None
        }
    }

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
        self.minimum_architecture().resolve_vector_length(requested)
    }

    /// Generates '+' delimited enabled extensions string
    pub fn to_extensions_string(&self) -> String {
        let mut extensions = Vec::new();
        for &isa in X64ISA::all() {
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
        for &isa in X64ISA::all() {
            let name = isa.as_str();
            if !name.is_empty() && name != "none" {
                map.insert(name.replace(['-', '.'], "_"), self.has_feature(isa));
            }
        }
        map
    }
}
