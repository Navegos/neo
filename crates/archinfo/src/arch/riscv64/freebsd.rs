// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/riscv64/freebsd.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeBsdRiscv64Probe {
    pub hwcap: u64,
    pub hwcap2: u64,

    // Base & Standard Extensions (G = IMAFD, C, V)
    pub i_available: bool,
    pub m_available: bool,
    pub a_available: bool,
    pub f_available: bool,
    pub d_available: bool,
    pub c_available: bool,
    pub v_available: bool,

    // Bitmanip (Zba, Zbb, Zbc, Zbs) & Cryptography Scalar (Zbk*, Zk*)
    pub zba_available: bool,
    pub zbb_available: bool,
    pub zbc_available: bool,
    pub zbs_available: bool,
    pub zbkb_available: bool,
    pub zbkc_available: bool,
    pub zbkx_available: bool,
    pub zknd_available: bool,
    pub zkne_available: bool,
    pub zknh_available: bool,
    pub zksed_available: bool,
    pub zksh_available: bool,
    pub zkt_available: bool,

    // Vector Cryptography & Vector Float
    pub zvbb_available: bool,
    pub zvbc_available: bool,
    pub zvkb_available: bool,
    pub zvkg_available: bool,
    pub zvkned_available: bool,
    pub zvknha_available: bool,
    pub zvknhb_available: bool,
    pub zvksed_available: bool,
    pub zvksh_available: bool,
    pub zvkt_available: bool,
    pub zvfh_available: bool,
    pub zvfhmin_available: bool,
    pub zvfbfmin_available: bool,
    pub zvfbfwma_available: bool,
    pub zve32x_available: bool,
    pub zve32f_available: bool,
    pub zve64x_available: bool,
    pub zve64f_available: bool,
    pub zve64d_available: bool,

    // Scalar Floating-Point & Bfloat16
    pub zfh_available: bool,
    pub zfhmin_available: bool,
    pub zfa_available: bool,
    pub zfbfmin_available: bool,

    // Cache Management & Memory Operations
    pub zicbom_available: bool,
    pub zicbop_available: bool,
    pub zicboz_available: bool,

    // Atomics & Memory Model
    pub zaamo_available: bool,
    pub zalrsc_available: bool,
    pub zabha_available: bool,
    pub zalasr_available: bool,
    pub zacas_available: bool,
    pub ztso_available: bool,
    pub za64rs_available: bool,

    // Compressed Extensions
    pub zca_available: bool,
    pub zcb_available: bool,
    pub zcd_available: bool,
    pub zcf_available: bool,
    pub zcmop_available: bool,
    pub zclsd_available: bool,

    // Instruction Fetch, Counters & Hints
    pub zicntr_available: bool,
    pub zihpm_available: bool,
    pub zihintpause_available: bool,
    pub zihintntl_available: bool,
    pub zimop_available: bool,
    pub zicond_available: bool,
    pub zawrs_available: bool,
    pub zilsd_available: bool,

    // Control Flow Integrity & Supervisor/Cache Attributes
    pub zicfilp_available: bool,
    pub zicfiss_available: bool,
    pub zicclsm_available: bool,
    pub ziccamoa_available: bool,
    pub ziccif_available: bool,
    pub ziccrse_available: bool,
    pub supm_available: bool,
}

impl FreeBsdRiscv64Probe {
    pub fn query() -> Self {
        #[allow(unused_mut)]
        let mut probe = Self::default();

        #[cfg(all(target_os = "freebsd", target_arch = "riscv64"))]
        {
            // HWCAP (Auxiliary Vector Type 25) - FreeBSD sys/riscv/include/elf.h
            // #define HWCAP_ISA_BIT(c) (1 << ((c) - 'a'))
            const HWCAP_ISA_A: u64 = 1 << (b'a' - b'a'); // 1 << 0
            const HWCAP_ISA_B: u64 = 1 << (b'b' - b'a'); // 1 << 1 (alias for Zba, Zbb, Zbs)
            const HWCAP_ISA_C: u64 = 1 << (b'c' - b'a'); // 1 << 2
            const HWCAP_ISA_D: u64 = 1 << (b'd' - b'a'); // 1 << 3
            const HWCAP_ISA_F: u64 = 1 << (b'f' - b'a'); // 1 << 5
            const HWCAP_ISA_H: u64 = 1 << (b'h' - b'a'); // 1 << 7
            const HWCAP_ISA_I: u64 = 1 << (b'i' - b'a'); // 1 << 8
            const HWCAP_ISA_M: u64 = 1 << (b'm' - b'a'); // 1 << 12
            const HWCAP_ISA_V: u64 = 1 << (b'v' - b'a'); // 1 << 21

            // Auxiliary Vector Types from FreeBSD sys/sys/elf_common.h
            const AT_HWCAP: libc::c_int = 25;
            const AT_HWCAP2: libc::c_int = 26;

            let mut hwcap: libc::c_ulong = 0;
            let mut hwcap2: libc::c_ulong = 0;

            unsafe {
                let _ = libc::elf_aux_info(
                    AT_HWCAP,
                    &mut hwcap as *mut _ as *mut libc::c_void,
                    std::mem::size_of::<libc::c_ulong>() as libc::c_int,
                );
                let _ = libc::elf_aux_info(
                    AT_HWCAP2,
                    &mut hwcap2 as *mut _ as *mut libc::c_void,
                    std::mem::size_of::<libc::c_ulong>() as libc::c_int,
                );
            }

            let hwcap = hwcap as u64;
            let hwcap2 = hwcap2 as u64;

            #[inline(always)]
            fn check(mask: u64, flag: u64) -> bool {
                (mask & flag) != 0
            }

            probe.hwcap = hwcap;
            probe.hwcap2 = hwcap2;

            // Base & Standard Extensions (G = IMAFD, C, V)
            probe.i_available = check(hwcap, HWCAP_ISA_I);
            probe.m_available = check(hwcap, HWCAP_ISA_M);
            probe.a_available = check(hwcap, HWCAP_ISA_A);
            probe.f_available = check(hwcap, HWCAP_ISA_F);
            probe.d_available = check(hwcap, HWCAP_ISA_D);
            probe.c_available = check(hwcap, HWCAP_ISA_C);
            probe.v_available = check(hwcap, HWCAP_ISA_V);

            // B extension (Bitmanip alias: Zba, Zbb, Zbs)
            let has_b = check(hwcap, HWCAP_ISA_B);
            if has_b {
                probe.zba_available = true;
                probe.zbb_available = true;
                probe.zbs_available = true;
            }

            // Vector extension implies Zve64d, Zve64f, Zve64x, Zve32f, Zve32x
            if probe.v_available {
                probe.zve32x_available = true;
                probe.zve32f_available = true;
                probe.zve64x_available = true;
                probe.zve64f_available = true;
                probe.zve64d_available = true;
            }
        }

        probe
    }
}

