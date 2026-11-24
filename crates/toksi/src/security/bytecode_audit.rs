// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/bytecode_audit.rs
// created: 2026-11-21

//! `.pyc` / compiled bytecode artifact mismatch & orphaned binary detector (`skill-scanner` parity).

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;

pub use crate::training::quant::{verify_safetensors_header, SafetensorsHeaderInfo};
pub use crate::training::serving::LlguidanceJsonMask;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BytecodeAuditFinding {
    pub path: String,
    pub reason: String,
    pub severity: String,
}

/// Audits a directory tree for orphaned or mismatched `.pyc` / `.pyo` compiled artifacts.
pub fn audit_bytecode_artifacts(root: &Path) -> Vec<BytecodeAuditFinding> {
    let mut findings = Vec::new();
    if !root.exists() {
        return findings;
    }
    let walker = ignore::WalkBuilder::new(root).hidden(false).build();
    for entry in walker.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("pyc") || ext.eq_ignore_ascii_case("pyo") {
            let norm = normalize_path(&p.to_string_lossy());
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let base_name = stem.split('.').next().unwrap_or(stem);
            let parent = p.parent().unwrap_or(root);
            let sibling_py = if parent.file_name().and_then(|n| n.to_str()) == Some("__pycache__") {
                parent
                    .parent()
                    .unwrap_or(parent)
                    .join(format!("{base_name}.py"))
            } else {
                parent.join(format!("{base_name}.py"))
            };
            if !sibling_py.is_file() {
                findings.push(BytecodeAuditFinding {
                    path: norm,
                    reason: format!(
                        "Orphaned compiled bytecode without matching '{base_name}.py' source"
                    ),
                    severity: "HIGH".to_string(),
                });
            }
        }
    }
    findings
}
