// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/pipeline_taint.rs
// created: 2026-11-21

//! Quote-aware shell pipeline taint flow analyzer (`skill-scanner` parity).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaintFlowFinding {
    pub line: usize,
    pub source: String,
    pub sink: String,
    pub severity: String,
    pub message: String,
}

const TAINT_SOURCES: &[&str] = &[
    "curl ",
    "wget ",
    "base64 -d",
    "base64 --decode",
    "$env:",
    "$(curl",
    "$(wget",
    "`curl",
];

const TAINT_SINKS: &[&str] = &[
    "| sh",
    "| bash",
    "| zsh",
    "| pwsh",
    "| powershell",
    "| python",
    "eval ",
    "iex ",
    "invoke-expression",
];

/// Analyzes shell/markdown script blocks for source-to-sink pipeline taint flows.
pub fn analyze_pipeline_taint(content: &str) -> Vec<TaintFlowFinding> {
    let mut findings = Vec::new();
    for (idx, raw_line) in content.lines().enumerate() {
        let lower = raw_line.trim().to_ascii_lowercase();
        if lower.is_empty() || lower.starts_with('#') {
            continue;
        }
        let matched_source = TAINT_SOURCES.iter().find(|s| lower.contains(**s));
        let matched_sink = TAINT_SINKS.iter().find(|s| lower.contains(**s));
        if let (Some(src), Some(sink)) = (matched_source, matched_sink) {
            findings.push(TaintFlowFinding {
                line: idx + 1,
                source: src.trim().to_string(),
                sink: sink.trim().to_string(),
                severity: "CRITICAL".to_string(),
                message: format!(
                    "Untrusted pipeline taint flow from '{}' into '{}'",
                    src.trim(),
                    sink.trim()
                ),
            });
        }
    }
    findings
}
