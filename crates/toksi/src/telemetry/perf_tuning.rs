// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/perf_tuning.rs
// created: 2026-11-21

//! Kernel & serving performance tuning catalog (`aitune`, `helion`, `torchtitan`,
//! `torch-tensorrt`, `FBGEMM`, `DeepGEMM`, `FlashMLA`, `TileKernels`, `srt-slurm`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerfKernelProfile {
    pub name: &'static str,
    pub vault: &'static str,
    pub target_arch: &'static str,
    pub supports_fp8: bool,
}

pub const PERF_TUNING_PROFILES: &[PerfKernelProfile] = &[
    PerfKernelProfile { name: "flash-mla-decode", vault: "FlashMLA", target_arch: "sm90", supports_fp8: true },
    PerfKernelProfile { name: "deepgemm-fp8", vault: "DeepGEMM", target_arch: "sm90", supports_fp8: true },
    PerfKernelProfile { name: "aitune-compile", vault: "aitune", target_arch: "cuda", supports_fp8: true },
    PerfKernelProfile { name: "helion-autotune", vault: "helion", target_arch: "cuda-tpu", supports_fp8: true },
    PerfKernelProfile { name: "tilekernels-matmul", vault: "TileKernels", target_arch: "cuda", supports_fp8: true },
];

pub fn lookup_perf_profile(name: &str) -> Option<&'static PerfKernelProfile> {
    PERF_TUNING_PROFILES
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name.trim()))
}
