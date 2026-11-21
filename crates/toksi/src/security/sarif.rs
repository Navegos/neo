// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/sarif.rs
// created: 2026-11-21

//! SARIF 2.1.0, JSON, Markdown, Table, and HTML security report formatter + AITech taxonomy normalizer.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Sarif,
    Markdown,
    Table,
    Html,
}

impl ReportFormat {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "sarif" => Self::Sarif,
            "markdown" | "md" => Self::Markdown,
            "table" => Self::Table,
            "html" => Self::Html,
            _ => Self::Json,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnifiedSecurityFinding {
    pub rule_id: String,
    pub severity: String,
    pub cwe: String,
    pub aitech: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub remediation: String,
}

pub fn normalize_aitech(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "AITech-0.0-General".to_string()
    } else if trimmed.starts_with("AITech-") {
        trimmed.to_string()
    } else {
        format!("AITech-{trimmed}")
    }
}

pub fn format_findings(findings: &[UnifiedSecurityFinding], format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => {
            serde_json::to_string_pretty(findings).unwrap_or_else(|_| "[]".to_string())
        }
        ReportFormat::Sarif => {
            let results: Vec<serde_json::Value> = findings
                .iter()
                .map(|f| {
                    let level = match f.severity.to_ascii_uppercase().as_str() {
                        "CRITICAL" | "HIGH" => "error",
                        "MEDIUM" => "warning",
                        _ => "note",
                    };
                    json!({
                        "ruleId": f.rule_id,
                        "level": level,
                        "message": { "text": f.message },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": { "uri": f.file },
                                "region": { "startLine": f.line.max(1) }
                            }
                        }],
                        "properties": {
                            "cwe": f.cwe,
                            "aitech": normalize_aitech(&f.aitech),
                            "remediation": f.remediation
                        }
                    })
                })
                .collect();
            let sarif = json!({
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "version": "2.1.0",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "toksi-security",
                            "version": env!("CARGO_PKG_VERSION"),
                            "informationUri": "https://github.com/Navegos/neo"
                        }
                    },
                    "results": results
                }]
            });
            serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
        }
        ReportFormat::Markdown => {
            let mut out = String::from("# Toksi Security Report\n\n| Severity | Rule | CWE | Location | Message |\n|---|---|---|---|---|\n");
            for f in findings {
                out.push_str(&format!(
                    "| {} | `{}` | {} | `{}:{}` | {} |\n",
                    f.severity, f.rule_id, f.cwe, f.file, f.line, f.message
                ));
            }
            out
        }
        ReportFormat::Table => {
            if findings.is_empty() {
                return "No security findings detected.".to_string();
            }
            let mut out = String::new();
            for f in findings {
                out.push_str(&format!(
                    "[{:8}] {:24} {}:{:<5} ({}) {}\n",
                    f.severity, f.rule_id, f.file, f.line, f.cwe, f.message
                ));
            }
            out
        }
        ReportFormat::Html => {
            let mut rows = String::new();
            for f in findings {
                rows.push_str(&format!(
                    "<tr><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}:{}</td><td>{}</td></tr>",
                    f.severity, f.rule_id, f.cwe, f.file, f.line, f.message
                ));
            }
            format!(
                "<!doctype html><html><head><meta charset=\"utf-8\"><title>Toksi Security Report</title></head><body><h1>Toksi Security Report</h1><table><thead><tr><th>Severity</th><th>Rule</th><th>CWE</th><th>Location</th><th>Message</th></tr></thead><tbody>{rows}</tbody></table></body></html>"
            )
        }
    }
}
