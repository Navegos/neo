// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/atr_engine.rs
// created: 2026-11-24

//! Pure-Rust Agent Threat Rules (`ATR`) deterministic evaluator for prompt injection,
//! reverse shells, data exfiltration, and tool poisoning.

#![deny(unsafe_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtrSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtrVerdict {
    Allow,
    Alert,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtrRule {
    pub id: String,
    pub category: String,
    pub severity: AtrSeverity,
    pub pattern: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtrMatch {
    pub rule_id: String,
    pub category: String,
    pub severity: AtrSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtrEvaluationReport {
    pub target: String,
    pub verdict: AtrVerdict,
    pub matches: Vec<AtrMatch>,
}

pub struct AtrEngine {
    rules: Vec<AtrRule>,
}

impl Default for AtrEngine {
    fn default() -> Self {
        Self::with_default_rules()
    }
}

impl AtrEngine {
    pub fn with_default_rules() -> Self {
        Self {
            rules: vec![
                AtrRule {
                    id: "ATR-INJ-001".to_string(),
                    category: "prompt_injection".to_string(),
                    severity: AtrSeverity::Critical,
                    pattern: r"(?i)ignore\s+(all\s+)?previous\s+instructions".to_string(),
                    description: "Instruction hierarchy override attempt".to_string(),
                },
                AtrRule {
                    id: "ATR-SHL-002".to_string(),
                    category: "reverse_shell".to_string(),
                    severity: AtrSeverity::Critical,
                    pattern: r"(?i)(/dev/tcp/|nc\s+-e|bash\s+-i\s+>&|curl\s+.+\|\s*(sh|bash))".to_string(),
                    description: "Remote pipe execution or reverse shell payload".to_string(),
                },
                AtrRule {
                    id: "ATR-EXF-003".to_string(),
                    category: "exfiltration".to_string(),
                    severity: AtrSeverity::High,
                    pattern: r"(?i)(~?/\.ssh/id_|/etc/shadow|AWS_SECRET_ACCESS_KEY\s*=)".to_string(),
                    description: "Credential / private key exfiltration marker".to_string(),
                },
                AtrRule {
                    id: "ATR-DST-004".to_string(),
                    category: "destructive_tool_poison".to_string(),
                    severity: AtrSeverity::Medium,
                    pattern: r"(?i)(rm\s+-rf\s+/(\s|$)|chmod\s+777\s+/)".to_string(),
                    description: "Destructive root filesystem command".to_string(),
                },
            ],
        }
    }

    pub fn evaluate_payload(&self, target: &str, payload: &str) -> AtrEvaluationReport {
        let mut matches = Vec::new();
        for rule in &self.rules {
            if let Ok(re) = Regex::new(&rule.pattern)
                && re.is_match(payload)
            {
                matches.push(AtrMatch {
                    rule_id: rule.id.clone(),
                    category: rule.category.clone(),
                    severity: rule.severity,
                    description: rule.description.clone(),
                });
            }
        }
        let verdict = if matches
            .iter()
            .any(|m| matches!(m.severity, AtrSeverity::Critical | AtrSeverity::High))
        {
            AtrVerdict::Block
        } else if !matches.is_empty() {
            AtrVerdict::Alert
        } else {
            AtrVerdict::Allow
        };

        AtrEvaluationReport {
            target: target.to_string(),
            verdict,
            matches,
        }
    }
}
