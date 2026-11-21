// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/analyzer.rs
// created: 2026-11-21

//! Unified 41+ Language Static Code Security Analyzer (`src/security/analyzer.rs`).
//!
//! Combines Tree-sitter AST SAST (`semgrep_engine`), prompt/agent/SSRF rules (`sage_rules`),
//! secret scanning (`redaction`), zero-plaintext blind signature matching (`scanner`),
//! and lockfile OSV parsing (`osv`) across all 45 registered Tree-sitter languages.

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::projectmem::redaction::redact_cow;
use crate::security::osv::{parse_lockfile, ParsedDependency};
use crate::security::polyglot::{detect_language_from_path, walk_polyglot_files};
use crate::security::sage_rules::scan_sage_rules;
use crate::security::sarif::{format_findings, ReportFormat, UnifiedSecurityFinding};
use crate::security::scanner::scan_file_blind_to_findings;
use crate::security::schema::BlindSignatures;
use crate::security::semgrep_engine::scan_source_ast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanSummary {
    pub scanned_files: usize,
    pub dependencies_parsed: usize,
    pub findings: Vec<UnifiedSecurityFinding>,
    pub dependencies: Vec<ParsedDependency>,
}

/// Scans a single file or directory across all 41+ Tree-sitter languages and lockfiles.
pub fn scan_path_security(
    target: &Path,
    blind_sigs: Option<&BlindSignatures>,
) -> SecurityScanSummary {
    let mut findings = Vec::new();
    let mut dependencies = Vec::new();
    let mut scanned_files = 0usize;

    let file_list: Vec<(String, String)> = if target.is_file() {
        let lang = detect_language_from_path(target).unwrap_or("text");
        vec![(
            crate::util::paths::normalize_path(&target.to_string_lossy()),
            lang.to_string(),
        )]
    } else {
        walk_polyglot_files(target)
    };

    for (norm_path, lang) in file_list {
        scanned_files += 1;
        let p = Path::new(&norm_path);
        let fname = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if matches!(
            fname.as_str(),
            "cargo.lock" | "package-lock.json" | "requirements.txt" | "go.sum"
        ) {
            dependencies.extend(parse_lockfile(p));
        }

        let Ok(raw) = std::fs::read_to_string(p) else {
            continue;
        };

        findings.extend(scan_source_ast(&norm_path, &lang, &raw));
        findings.extend(scan_sage_rules(&norm_path, &raw));

        let (_, secret_hits) = redact_cow(&raw);
        for hit in secret_hits {
            findings.push(UnifiedSecurityFinding {
                rule_id: format!("toksi.secret.{hit}"),
                severity: "HIGH".to_string(),
                cwe: "CWE-798".to_string(),
                aitech: "AITech-8.2-SecretLeak".to_string(),
                file: norm_path.clone(),
                line: 1,
                message: format!("Hardcoded secret pattern '{hit}' detected"),
                remediation: "Move secret into OS Keyring or environment variable.".to_string(),
            });
        }

        if let Some(sigs) = blind_sigs {
            findings.extend(scan_file_blind_to_findings(&norm_path, &raw, sigs));
        }
    }

    SecurityScanSummary {
        scanned_files,
        dependencies_parsed: dependencies.len(),
        findings,
        dependencies,
    }
}

/// Formats a `SecurityScanSummary`'s findings in `json | sarif | markdown | table | html`.
pub fn format_scan_summary(summary: &SecurityScanSummary, format: ReportFormat) -> String {
    format_findings(&summary.findings, format)
}
