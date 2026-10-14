// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/riscv64/linux.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinuxRiscv64Probe {
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

#[cfg(all(target_os = "linux", target_arch = "riscv64"))]
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct RiscvHwprobe {
    key: i64,
    value: u64,
}

impl LinuxRiscv64Probe {
    pub fn query() -> Self {
        #[allow(unused_mut)]
        let mut probe = Self::default();

        #[cfg(all(target_os = "linux", target_arch = "riscv64"))]
        {
            // Auxiliary Vector AT_HWCAP
            pub const AT_HWCAP: u64 = 16;
            pub const COMPAT_HWCAP_ISA_I: u64 = 1 << (b'I' - b'A');
            pub const COMPAT_HWCAP_ISA_M: u64 = 1 << (b'M' - b'A');
            pub const COMPAT_HWCAP_ISA_A: u64 = 1 << (b'A' - b'A');
            pub const COMPAT_HWCAP_ISA_F: u64 = 1 << (b'F' - b'A');
            pub const COMPAT_HWCAP_ISA_D: u64 = 1 << (b'D' - b'A');
            pub const COMPAT_HWCAP_ISA_C: u64 = 1 << (b'C' - b'A');
            pub const COMPAT_HWCAP_ISA_V: u64 = 1 << (b'V' - b'A');

            // Syscall & Keys
            pub const NR_RISCV_HWPROBE: libc::c_long = 258;
            pub const RISCV_HWPROBE_KEY_BASE_BEHAVIOR: i64 = 3;
            pub const RISCV_HWPROBE_KEY_IMA_EXT_0: i64     = 4;
            pub const RISCV_HWPROBE_KEY_IMA_EXT_1: i64     = 16;

            // Base Behavior Flags
            pub const RISCV_HWPROBE_BASE_BEHAVIOR_IMA: u64 = 1 << 0;

            // RISCV_HWPROBE_KEY_IMA_EXT_0 Flags
            pub const RISCV_HWPROBE_IMA_FD: u64          = 1 << 0;
            pub const RISCV_HWPROBE_IMA_C: u64           = 1 << 1;
            pub const RISCV_HWPROBE_IMA_V: u64           = 1 << 2;
            pub const RISCV_HWPROBE_EXT_ZBA: u64         = 1 << 3;
            pub const RISCV_HWPROBE_EXT_ZBB: u64         = 1 << 4;
            pub const RISCV_HWPROBE_EXT_ZBS: u64         = 1 << 5;
            pub const RISCV_HWPROBE_EXT_ZICBOZ: u64      = 1 << 6;
            pub const RISCV_HWPROBE_EXT_ZBC: u64         = 1 << 7;
            pub const RISCV_HWPROBE_EXT_ZBKB: u64        = 1 << 8;
            pub const RISCV_HWPROBE_EXT_ZBKC: u64        = 1 << 9;
            pub const RISCV_HWPROBE_EXT_ZBKX: u64        = 1 << 10;
            pub const RISCV_HWPROBE_EXT_ZKND: u64        = 1 << 11;
            pub const RISCV_HWPROBE_EXT_ZKNE: u64        = 1 << 12;
            pub const RISCV_HWPROBE_EXT_ZKNH: u64        = 1 << 13;
            pub const RISCV_HWPROBE_EXT_ZKSED: u64       = 1 << 14;
            pub const RISCV_HWPROBE_EXT_ZKSH: u64        = 1 << 15;
            pub const RISCV_HWPROBE_EXT_ZKT: u64         = 1 << 16;
            pub const RISCV_HWPROBE_EXT_ZVBB: u64        = 1 << 17;
            pub const RISCV_HWPROBE_EXT_ZVBC: u64        = 1 << 18;
            pub const RISCV_HWPROBE_EXT_ZVKB: u64        = 1 << 19;
            pub const RISCV_HWPROBE_EXT_ZVKG: u64        = 1 << 20;
            pub const RISCV_HWPROBE_EXT_ZVKNED: u64      = 1 << 21;
            pub const RISCV_HWPROBE_EXT_ZVKNHA: u64      = 1 << 22;
            pub const RISCV_HWPROBE_EXT_ZVKNHB: u64      = 1 << 23;
            pub const RISCV_HWPROBE_EXT_ZVKSED: u64      = 1 << 24;
            pub const RISCV_HWPROBE_EXT_ZVKSH: u64       = 1 << 25;
            pub const RISCV_HWPROBE_EXT_ZVKT: u64        = 1 << 26;
            pub const RISCV_HWPROBE_EXT_ZFH: u64         = 1 << 27;
            pub const RISCV_HWPROBE_EXT_ZFHMIN: u64      = 1 << 28;
            pub const RISCV_HWPROBE_EXT_ZIHINTNTL: u64   = 1 << 29;
            pub const RISCV_HWPROBE_EXT_ZVFH: u64        = 1 << 30;
            pub const RISCV_HWPROBE_EXT_ZVFHMIN: u64     = 1 << 31;
            pub const RISCV_HWPROBE_EXT_ZFA: u64         = 1 << 32;
            pub const RISCV_HWPROBE_EXT_ZTSO: u64        = 1 << 33;
            pub const RISCV_HWPROBE_EXT_ZACAS: u64       = 1 << 34;
            pub const RISCV_HWPROBE_EXT_ZICOND: u64      = 1 << 35;
            pub const RISCV_HWPROBE_EXT_ZIHINTPAUSE: u64 = 1 << 36;
            pub const RISCV_HWPROBE_EXT_ZVE32X: u64      = 1 << 37;
            pub const RISCV_HWPROBE_EXT_ZVE32F: u64      = 1 << 38;
            pub const RISCV_HWPROBE_EXT_ZVE64X: u64      = 1 << 39;
            pub const RISCV_HWPROBE_EXT_ZVE64F: u64      = 1 << 40;
            pub const RISCV_HWPROBE_EXT_ZVE64D: u64      = 1 << 41;
            pub const RISCV_HWPROBE_EXT_ZIMOP: u64       = 1 << 42;
            pub const RISCV_HWPROBE_EXT_ZCA: u64         = 1 << 43;
            pub const RISCV_HWPROBE_EXT_ZCB: u64         = 1 << 44;
            pub const RISCV_HWPROBE_EXT_ZCD: u64         = 1 << 45;
            pub const RISCV_HWPROBE_EXT_ZCF: u64         = 1 << 46;
            pub const RISCV_HWPROBE_EXT_ZCMOP: u64       = 1 << 47;
            pub const RISCV_HWPROBE_EXT_ZAWRS: u64       = 1 << 48;
            pub const RISCV_HWPROBE_EXT_SUPM: u64        = 1 << 49;
            pub const RISCV_HWPROBE_EXT_ZICNTR: u64      = 1 << 50;
            pub const RISCV_HWPROBE_EXT_ZIHPM: u64       = 1 << 51;
            pub const RISCV_HWPROBE_EXT_ZFBFMIN: u64     = 1 << 52;
            pub const RISCV_HWPROBE_EXT_ZVFBFMIN: u64    = 1 << 53;
            pub const RISCV_HWPROBE_EXT_ZVFBFWMA: u64    = 1 << 54;
            pub const RISCV_HWPROBE_EXT_ZICBOM: u64      = 1 << 55;
            pub const RISCV_HWPROBE_EXT_ZAAMO: u64       = 1 << 56;
            pub const RISCV_HWPROBE_EXT_ZALRSC: u64      = 1 << 57;
            pub const RISCV_HWPROBE_EXT_ZABHA: u64       = 1 << 58;
            pub const RISCV_HWPROBE_EXT_ZALASR: u64      = 1 << 59;
            pub const RISCV_HWPROBE_EXT_ZICBOP: u64      = 1 << 60;
            pub const RISCV_HWPROBE_EXT_ZILSD: u64       = 1 << 61;
            pub const RISCV_HWPROBE_EXT_ZCLSD: u64       = 1 << 62;
            pub const RISCV_HWPROBE_EXT_ZICFILP: u64     = 1 << 63;

            // RISCV_HWPROBE_KEY_IMA_EXT_1 Flags
            pub const RISCV_HWPROBE_EXT_ZICFISS: u64   = 1 << 0;
            pub const RISCV_HWPROBE_EXT_ZICCLSM: u64   = 1 << 1;
            pub const RISCV_HWPROBE_EXT_ZICCAMOA: u64  = 1 << 2;
            pub const RISCV_HWPROBE_EXT_ZICCIF: u64    = 1 << 3;
            pub const RISCV_HWPROBE_EXT_ZICCRSE: u64   = 1 << 4;
            pub const RISCV_HWPROBE_EXT_ZA64RS: u64    = 1 << 5;

            #[inline(always)]
            fn check(mask: u64, flag: u64) -> bool {
                (mask & flag) != 0
            }

            // AT_HWCAP fallback base detection
            let hwcap = unsafe { libc::getauxval(AT_HWCAP as libc::c_ulong) as u64 };
            probe.i_available = check(hwcap, COMPAT_HWCAP_ISA_I);
            probe.m_available = check(hwcap, COMPAT_HWCAP_ISA_M);
            probe.a_available = check(hwcap, COMPAT_HWCAP_ISA_A);
            probe.f_available = check(hwcap, COMPAT_HWCAP_ISA_F);
            probe.d_available = check(hwcap, COMPAT_HWCAP_ISA_D);
            probe.c_available = check(hwcap, COMPAT_HWCAP_ISA_C);
            probe.v_available = check(hwcap, COMPAT_HWCAP_ISA_V);

            // riscv_hwprobe syscall query
            let mut pairs: [RiscvHwprobe; 3] = [
                RiscvHwprobe { key: RISCV_HWPROBE_KEY_IMA_EXT_0, value: 0 },
                RiscvHwprobe { key: RISCV_HWPROBE_KEY_IMA_EXT_1, value: 0 },
                RiscvHwprobe { key: RISCV_HWPROBE_KEY_BASE_BEHAVIOR, value: 0 },
            ];

            let ret = unsafe {
                libc::syscall(
                    NR_RISCV_HWPROBE,
                    pairs.as_mut_ptr(),
                    pairs.len(),
                    0usize,
                    std::ptr::null_mut::<libc::c_void>(),
                    0u32,
                )
            };

            if ret == 0 {
                let ima_ext_0 = pairs[0].value;
                if ima_ext_0 != 0 {
                    probe.zba_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBA);
                    probe.zbb_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBB);
                    probe.zbc_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBC);
                    probe.zbs_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBS);
                    probe.zicboz_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZICBOZ);
                    probe.zbkb_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBKB);
                    probe.zbkc_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBKC);
                    probe.zbkx_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZBKX);
                    probe.zknd_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZKND);
                    probe.zkne_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZKNE);
                    probe.zknh_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZKNH);
                    probe.zksed_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZKSED);
                    probe.zksh_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZKSH);
                    probe.zkt_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZKT);
                    probe.zvbb_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVBB);
                    probe.zvbc_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVBC);
                    probe.zvkb_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKB);
                    probe.zvkg_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKG);
                    probe.zvkned_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKNED);
                    probe.zvknha_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKNHA);
                    probe.zvknhb_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKNHB);
                    probe.zvksed_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKSED);
                    probe.zvksh_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKSH);
                    probe.zvkt_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVKT);
                    probe.zfh_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZFH);
                    probe.zfhmin_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZFHMIN);
                    probe.zihintntl_available   = check(ima_ext_0, RISCV_HWPROBE_EXT_ZIHINTNTL);
                    probe.zvfh_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVFH);
                    probe.zvfhmin_available     = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVFHMIN);
                    probe.zfa_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZFA);
                    probe.ztso_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_ZTSO);
                    probe.zacas_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZACAS);
                    probe.zicond_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZICOND);
                    probe.zihintpause_available = check(ima_ext_0, RISCV_HWPROBE_EXT_ZIHINTPAUSE);
                    probe.zve32x_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVE32X);
                    probe.zve32f_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVE32F);
                    probe.zve64x_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVE64X);
                    probe.zve64f_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVE64F);
                    probe.zve64d_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVE64D);
                    probe.zimop_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZIMOP);
                    probe.zca_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZCA);
                    probe.zcb_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZCB);
                    probe.zcd_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZCD);
                    probe.zcf_available         = check(ima_ext_0, RISCV_HWPROBE_EXT_ZCF);
                    probe.zcmop_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZCMOP);
                    probe.zawrs_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZAWRS);
                    probe.supm_available        = check(ima_ext_0, RISCV_HWPROBE_EXT_SUPM);
                    probe.zicntr_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZICNTR);
                    probe.zihpm_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZIHPM);
                    probe.zfbfmin_available     = check(ima_ext_0, RISCV_HWPROBE_EXT_ZFBFMIN);
                    probe.zvfbfmin_available    = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVFBFMIN);
                    probe.zvfbfwma_available    = check(ima_ext_0, RISCV_HWPROBE_EXT_ZVFBFWMA);
                    probe.zicbom_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZICBOM);
                    probe.zaamo_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZAAMO);
                    probe.zalrsc_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZALRSC);
                    probe.zabha_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZABHA);
                    probe.zalasr_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZALASR);
                    probe.zicbop_available      = check(ima_ext_0, RISCV_HWPROBE_EXT_ZICBOP);
                    probe.zilsd_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZILSD);
                    probe.zclsd_available       = check(ima_ext_0, RISCV_HWPROBE_EXT_ZCLSD);
                    probe.zicfilp_available     = check(ima_ext_0, RISCV_HWPROBE_EXT_ZICFILP);

                    probe.f_available |= check(ima_ext_0, RISCV_HWPROBE_IMA_FD);
                    probe.d_available |= check(ima_ext_0, RISCV_HWPROBE_IMA_FD);
                    probe.c_available |= check(ima_ext_0, RISCV_HWPROBE_IMA_C);
                    probe.v_available |= check(ima_ext_0, RISCV_HWPROBE_IMA_V);
                }

                let ima_ext_1 = pairs[1].value;
                if ima_ext_1 != 0 {
                    probe.zicfiss_available  = check(ima_ext_1, RISCV_HWPROBE_EXT_ZICFISS);
                    probe.zicclsm_available  = check(ima_ext_1, RISCV_HWPROBE_EXT_ZICCLSM);
                    probe.ziccamoa_available = check(ima_ext_1, RISCV_HWPROBE_EXT_ZICCAMOA);
                    probe.ziccif_available   = check(ima_ext_1, RISCV_HWPROBE_EXT_ZICCIF);
                    probe.ziccrse_available  = check(ima_ext_1, RISCV_HWPROBE_EXT_ZICCRSE);
                    probe.za64rs_available   = check(ima_ext_1, RISCV_HWPROBE_EXT_ZA64RS);
                }

                let base_behavior = pairs[2].value;
                if base_behavior != 0 {
                    let has_ima = check(base_behavior, RISCV_HWPROBE_BASE_BEHAVIOR_IMA);
                    probe.i_available |= has_ima;
                    probe.m_available |= has_ima;
                    probe.a_available |= has_ima;
                }
            }
        }

        probe
    }
}

