// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/x86_64/xcr0.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use super::cpuid::query_xcr0;

#[derive(Debug, Clone, Copy, Default)]
pub struct Xcr0State {
    pub xmm_enabled: bool,
    pub ymm_enabled: bool,
    pub opmask_enabled: bool,
    pub zmm_hi256_enabled: bool,
    pub hi16_zmm_enabled: bool,
    pub apx_enabled: bool,
}

impl Xcr0State {
    pub fn query(osxsave: bool) -> Self {
        if !osxsave {
            return Self::default();
        }

        let xcr0 = query_xcr0();
        Self {
            xmm_enabled: (xcr0 & 0x02) != 0,          // bit 1: XMM
            ymm_enabled: (xcr0 & 0x04) != 0,          // bit 2: YMM
            opmask_enabled: (xcr0 & 0x20) != 0,       // bit 5: AVX-512 Opmask
            zmm_hi256_enabled: (xcr0 & 0x40) != 0,    // bit 6: ZMM_Hi256
            hi16_zmm_enabled: (xcr0 & 0x80) != 0,     // bit 7: Hi16_ZMM
            apx_enabled: (xcr0 & (1 << 19)) != 0,     // bit 19: APX Extended GPRs (R16-R31)
        }
    }

    #[inline]
    pub fn is_avx_usable(&self) -> bool {
        self.xmm_enabled && self.ymm_enabled
    }

    #[inline]
    pub fn is_avx512_usable(&self) -> bool {
        self.xmm_enabled && self.ymm_enabled && self.opmask_enabled && self.zmm_hi256_enabled && self.hi16_zmm_enabled
    }

    #[inline]
    pub fn is_apx_usable(&self) -> bool {
        self.apx_enabled
    }
}
