// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/vector_length.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Specifies the preferred vector length for CPU architecture SIMD code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CpuArchitectureVectorLength {
    #[default]
    None,
    VL128,
    VL256,
    VL512,
}

impl fmt::Display for CpuArchitectureVectorLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuArchitectureVectorLength::None => write!(f, "none"),
            CpuArchitectureVectorLength::VL128 => write!(f, "vl128"),
            CpuArchitectureVectorLength::VL256 => write!(f, "vl256"),
            CpuArchitectureVectorLength::VL512 => write!(f, "vl512"),
        }
    }
}

impl FromStr for CpuArchitectureVectorLength {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.trim().to_ascii_lowercase();
        match norm.as_str() {
            "128" | "vl128" => Ok(CpuArchitectureVectorLength::VL128),
            "256" | "vl256" => Ok(CpuArchitectureVectorLength::VL256),
            "512" | "vl512" => Ok(CpuArchitectureVectorLength::VL512),
            "none" | "0" | "default" => Ok(CpuArchitectureVectorLength::None),
            other => Err(format!(
                "Invalid vector length: '{}'. Acceptable numbers/values are 128, 256, 512 (or vl128, vl256, vl512)",
                other
            )),
        }
    }
}

/// Provides a mapping between CpuArchitectureVectorLength and Clang preferred vector width command line arguments (-mprefer-vector-width=).
pub struct ClangCpuArchitecturePreferredVectorLength;

impl ClangCpuArchitecturePreferredVectorLength {
    pub fn name(vl: CpuArchitectureVectorLength) -> &'static str {
        match vl {
            CpuArchitectureVectorLength::None => "",
            CpuArchitectureVectorLength::VL128 => "-m'prefer-vector-width=128'",
            CpuArchitectureVectorLength::VL256 => "-m'prefer-vector-width=256'",
            CpuArchitectureVectorLength::VL512 => "-m'prefer-vector-width=512'",
        }
    }
}

/// Provides a mapping between CpuArchitectureVectorLength and Clang vector length suffix representations (e.g. for -mavx10.1-256 / -mavx10.1-512).
pub struct ClangCpuArchitectureVectorLengthNames;

impl ClangCpuArchitectureVectorLengthNames {
    pub fn name(vl: CpuArchitectureVectorLength) -> &'static str {
        match vl {
            CpuArchitectureVectorLength::None => "",
            CpuArchitectureVectorLength::VL128 => "",
            CpuArchitectureVectorLength::VL256 => "-256",
            CpuArchitectureVectorLength::VL512 => "-512",
        }
    }
}

/// Provides a mapping between CpuArchitectureVectorLength and MSVC preferred vector length command line arguments (/vlen=).
pub struct MSVCCpuArchitecturePreferredVectorLength;

impl MSVCCpuArchitecturePreferredVectorLength {
    pub fn name(vl: CpuArchitectureVectorLength) -> &'static str {
        match vl {
            CpuArchitectureVectorLength::None => "",
            CpuArchitectureVectorLength::VL128 => "",
            CpuArchitectureVectorLength::VL256 => "/vlen=256",
            CpuArchitectureVectorLength::VL512 => "/vlen=512",
        }
    }
}
