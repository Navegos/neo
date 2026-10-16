// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/x86_64/cpuid.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__cpuid, __cpuid_count, _xgetbv};

#[derive(Default, Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[inline]
pub fn query_cpuid(leaf: u32) -> CpuidResult {
    #[cfg(target_arch = "x86_64")]
    {
        let r = __cpuid(leaf);
        CpuidResult {
            eax: r.eax,
            ebx: r.ebx,
            ecx: r.ecx,
            edx: r.edx,
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = leaf;
        CpuidResult::default()
    }
}

#[inline]
pub fn query_cpuid_count(leaf: u32, sub_leaf: u32) -> CpuidResult {
    #[cfg(target_arch = "x86_64")]
    {
        let r = __cpuid_count(leaf, sub_leaf);
        CpuidResult {
            eax: r.eax,
            ebx: r.ebx,
            ecx: r.ecx,
            edx: r.edx,
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (leaf, sub_leaf);
        CpuidResult::default()
    }
}

#[inline]
pub fn query_xcr0() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        _xgetbv(0)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        0
    }
}

