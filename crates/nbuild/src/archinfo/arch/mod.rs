// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-14

pub mod arm64;
pub mod riscv64;
pub mod x86_64;

use crate::archinfo::platform::{Arch, Platform};
use crate::archinfo::vector_length::CpuArchitectureVectorLength;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum CPUFeatures {
    X86_64(x86_64::X64CPUFeatures),
    Arm64(arm64::Arm64CPUFeatures),
    Riscv64(riscv64::Riscv64CPUFeatures),
}

impl CPUFeatures {
    /// Detect host CPU features
    pub fn detect_host() -> Self {
        match Arch::current().resolve() {
            Arch::X86_64 => CPUFeatures::X86_64(x86_64::X64CPUFeatures::detect_host()),
            Arch::Arm64 | Arch::Arm64EC | Arch::Arm64E => CPUFeatures::Arm64(arm64::Arm64CPUFeatures::detect_host()),
            Arch::Riscv64 => CPUFeatures::Riscv64(riscv64::Riscv64CPUFeatures::detect_host()),
            Arch::Native => unreachable!(),
        }
    }

    pub fn vector_length(&self) -> CpuArchitectureVectorLength {
        match self {
            CPUFeatures::X86_64(f) => f.vector_length(),
            CPUFeatures::Arm64(f) => f.vector_length(),
            CPUFeatures::Riscv64(f) => f.vector_length(),
        }
    }

    pub fn resolve_vector_length(&self, requested: Option<CpuArchitectureVectorLength>) -> CpuArchitectureVectorLength {
        match self {
            CPUFeatures::X86_64(f) => f.resolve_vector_length(requested),
            CPUFeatures::Arm64(f) => f.resolve_vector_length(requested),
            CPUFeatures::Riscv64(f) => f.resolve_vector_length(requested),
        }
    }

    pub fn to_extensions_string(&self) -> String {
        match self {
            CPUFeatures::X86_64(f) => f.to_extensions_string(),
            CPUFeatures::Arm64(f) => f.to_extensions_string(),
            CPUFeatures::Riscv64(f) => f.to_extensions_string(),
        }
    }

    pub fn to_map(&self) -> BTreeMap<String, bool> {
        match self {
            CPUFeatures::X86_64(f) => f.to_map(),
            CPUFeatures::Arm64(f) => f.to_map(),
            CPUFeatures::Riscv64(f) => f.to_map(),
        }
    }
}
