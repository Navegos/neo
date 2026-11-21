// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/epss.rs
// created: 2026-11-21

//! EPSS exploit probability & remediation priority scoring (`osv-scanner` / `cve-lite-cli` parity).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpssScore {
    pub cve_id: String,
    pub epss: f32,
    pub percentile: f32,
    pub priority: String,
}

/// Computes remediation priority tier from EPSS score and CVSS severity.
pub fn score_remediation_priority(cve_id: &str, epss: f32, percentile: f32, severity: &str) -> EpssScore {
    let sev = severity.trim().to_ascii_uppercase();
    let priority = if epss >= 0.7 || (sev == "CRITICAL" && epss >= 0.2) {
        "P0-Immediate"
    } else if epss >= 0.3 || sev == "HIGH" {
        "P1-Urgent"
    } else if epss >= 0.05 || sev == "MEDIUM" {
        "P2-Scheduled"
    } else {
        "P3-Low"
    };

    EpssScore {
        cve_id: cve_id.to_string(),
        epss,
        percentile,
        priority: priority.to_string(),
    }
}
