// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/external_analyzers.rs
// created: 2026-11-21

//! Optional `PATH`-verified external compiler/security analyzers
//! (`rust-analyzer`, `gopls`, `clang-tidy`, `cargo-audit`) obeying the "Rule of Test and Inform User".

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::util::env_path::check_executable_in_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalAnalyzerStatus {
    pub binary: String,
    pub available_in_path: bool,
    pub remediation: Option<String>,
}

const KNOWN_EXTERNAL_ANALYZERS: &[&str] = &["rust-analyzer", "gopls", "clang-tidy", "cargo-audit"];

/// Probes `PATH` for optional external analyzers without hardcoding absolute paths.
pub fn probe_external_analyzers() -> Vec<ExternalAnalyzerStatus> {
    KNOWN_EXTERNAL_ANALYZERS
        .iter()
        .map(|bin| match check_executable_in_path(bin) {
            Ok(_) => ExternalAnalyzerStatus {
                binary: (*bin).to_string(),
                available_in_path: true,
                remediation: None,
            },
            Err(err) => ExternalAnalyzerStatus {
                binary: (*bin).to_string(),
                available_in_path: false,
                remediation: Some(err),
            },
        })
        .collect()
}
