// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/skill_scanner.rs
// created: 2026-11-21

//! `skill-scanner` + `defenseclaw` multi-engine & bounded CEL decision scanner
//! for externalized `<skills>` and plugins.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::plugins::{scan_plugin_content, PluginSecurityStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CelVerdict {
    Allow,
    DenyFailClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillScannerReport {
    pub target_id: String,
    pub verdict: CelVerdict,
    pub aitech_tags: Vec<String>,
    pub findings: Vec<String>,
}

/// Evaluates a skill or plugin payload using `scan_plugin_content` + `defenseclaw`
/// fail-closed shell/exfiltration rules.
pub fn scan_skill_or_plugin(target_id: &str, content: &str) -> SkillScannerReport {
    let base = scan_plugin_content(target_id, content);
    let mut findings = base.findings;
    let mut aitech_tags = Vec::new();
    let lower = content.to_ascii_lowercase();

    if (lower.contains("curl ") || lower.contains("wget "))
        && (lower.contains("| sh") || lower.contains("| bash"))
    {
        findings.push("SKILL-SCANNER: remote pipe-to-shell execution detected".to_string());
        aitech_tags.push("AITech-9.1-RemoteExec".to_string());
    }
    if lower.contains("chmod 777") || lower.contains("sudo ") {
        findings.push("DEFENSECLAW: privileged escalation pattern detected".to_string());
        aitech_tags.push("AITech-4.2-PrivilegeEscalation".to_string());
    }
    if base.status == PluginSecurityStatus::BlockedFailClosed && aitech_tags.is_empty() {
        aitech_tags.push("AITech-1.1-PromptInjection".to_string());
    }

    let verdict = if findings.is_empty() {
        CelVerdict::Allow
    } else {
        CelVerdict::DenyFailClosed
    };

    SkillScannerReport {
        target_id: target_id.to_string(),
        verdict,
        aitech_tags,
        findings,
    }
}
