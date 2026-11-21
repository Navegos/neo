// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/sage_rules.rs
// created: 2026-11-21

//! Prompt/agent/SSRF threat detection rules (`sage`, `codex-security`, `deepsec` parity).

#![deny(unsafe_code)]

use crate::security::sarif::UnifiedSecurityFinding;

/// Scans source or skill/config content for SSRF, metadata endpoint access, and agent threat patterns.
pub fn scan_sage_rules(file_path: &str, content: &str) -> Vec<UnifiedSecurityFinding> {
    let mut findings = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("169.254.169.254") || lower.contains("metadata.google.internal") {
            findings.push(UnifiedSecurityFinding {
                rule_id: "sage.ssrf.cloud-metadata".to_string(),
                severity: "CRITICAL".to_string(),
                cwe: "CWE-918".to_string(),
                aitech: "AITech-8.3-SSRF".to_string(),
                file: file_path.to_string(),
                line: idx + 1,
                message: "Cloud instance metadata endpoint reference detected (potential SSRF)".to_string(),
                remediation: "Block link-local metadata IPs in outbound HTTP/sandbox egress policy.".to_string(),
            });
        }
        if lower.contains("dangerously_skip_permissions") || lower.contains("--no-sandbox") {
            findings.push(UnifiedSecurityFinding {
                rule_id: "sage.agent.sandbox-bypass".to_string(),
                severity: "HIGH".to_string(),
                cwe: "CWE-284".to_string(),
                aitech: "AITech-4.1-SandboxBypass".to_string(),
                file: file_path.to_string(),
                line: idx + 1,
                message: "Agent sandbox bypass flag detected".to_string(),
                remediation: "Remove sandbox bypass flags; execute under <enforce> isolation.".to_string(),
            });
        }
    }
    findings
}
