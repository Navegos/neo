// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/skill_scanner.rs
// created: 2026-11-21

//! `skill-scanner` + `defenseclaw` multi-engine & bounded CEL decision scanner
//! for externalized `<skills>` and plugins across 5 presets (`strict|balanced|low-noise|quiet|permissive`).

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::plugins::{scan_plugin_content, PluginSecurityStatus};
use crate::security::bytecode_audit::audit_bytecode_artifacts;
use crate::security::cel_decision::{evaluate_cel_policy, CelDecisionVerdict, ScanPreset};
use crate::security::pipeline_taint::analyze_pipeline_taint;
use crate::security::yara_mcp::scan_yara_mcp;

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
    pub preset: ScanPreset,
    pub cel_reason: String,
    pub aitech_tags: Vec<String>,
    pub findings: Vec<String>,
}

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    let bytes = s.as_bytes();
    for &b in bytes {
        counts[b as usize] += 1;
    }
    let len = bytes.len() as f64;
    let mut ent = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / len;
            ent -= p * p.log2();
        }
    }
    ent
}

/// Evaluates a skill or plugin payload using multi-engine (`yara_mcp`, `pipeline_taint`,
/// entropy/link checks, and bounded CEL policy decisions) with default `Balanced` preset.
pub fn scan_skill_or_plugin(target_id: &str, content: &str) -> SkillScannerReport {
    scan_skill_or_plugin_with_preset(target_id, content, None, ScanPreset::Balanced)
}

/// Evaluates a skill or plugin payload with an explicit preset and optional directory for `.pyc` audit.
pub fn scan_skill_or_plugin_with_preset(
    target_id: &str,
    content: &str,
    dir_for_bytecode: Option<&Path>,
    preset: ScanPreset,
) -> SkillScannerReport {
    let base = scan_plugin_content(target_id, content);
    let mut findings = base.findings;
    let mut aitech_tags = Vec::new();
    let mut critical = 0usize;
    let mut high = 0usize;
    let mut medium = 0usize;

    if base.status == PluginSecurityStatus::BlockedFailClosed {
        critical += findings.len().max(1);
        aitech_tags.push("AITech-1.1-PromptInjection".to_string());
    }

    for ym in scan_yara_mcp(content) {
        findings.push(format!("YARA-MCP [{}]: {}", ym.rule_name, ym.description));
        aitech_tags.push(ym.aitech);
        if ym.severity == "CRITICAL" {
            critical += 1;
        } else {
            high += 1;
        }
    }

    for taint in analyze_pipeline_taint(content) {
        findings.push(format!("TAINT L{}: {}", taint.line, taint.message));
        aitech_tags.push("AITech-9.1-RemoteExec".to_string());
        critical += 1;
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.len() >= 64 && !trimmed.contains(' ') && shannon_entropy(trimmed) > 5.2 {
            findings.push("ENTROPY: High-entropy obfuscated token/payload blob detected".to_string());
            aitech_tags.push("AITech-5.1-Obfuscation".to_string());
            medium += 1;
            break;
        }
    }

    if let Some(dir) = dir_for_bytecode {
        for bc in audit_bytecode_artifacts(dir) {
            findings.push(format!("BYTECODE: {} ({})", bc.path, bc.reason));
            aitech_tags.push("AITech-9.3-BytecodeMismatch".to_string());
            high += 1;
        }
    }

    let cel = evaluate_cel_policy(preset, critical, high, medium, 0, None);
    let verdict = match cel.verdict {
        CelDecisionVerdict::DenyFailClosed => CelVerdict::DenyFailClosed,
        CelDecisionVerdict::Allow | CelDecisionVerdict::Warn => CelVerdict::Allow,
    };

    SkillScannerReport {
        target_id: target_id.to_string(),
        verdict,
        preset,
        cel_reason: cel.reason,
        aitech_tags,
        findings,
    }
}
