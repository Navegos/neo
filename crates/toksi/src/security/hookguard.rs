// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/hookguard.rs
// created: 2026-11-21

//! `defenseclaw` fail-closed JSON-RPC / tool-call / plugin hookguard & ordered
//! `tool-chains.json` policy evaluator.

#![deny(unsafe_code)]

use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::security::audit::record_audit_evidence;
use crate::security::guardrails_validators::validate_and_anonymize;
use crate::security::yara_mcp::scan_yara_mcp;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolChainRule {
    pub tool: String,
    #[serde(default)]
    pub requires_prior: Vec<String>,
    #[serde(default)]
    pub forbid_after: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolChainsPolicy {
    #[serde(default)]
    pub rules: Vec<ToolChainRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookGuardVerdict {
    pub allowed: bool,
    pub fail_closed: bool,
    pub reasons: Vec<String>,
}

impl ToolChainsPolicy {
    pub fn load_from_repo(repo_root: &Path) -> Self {
        let p = repo_root.join(".toksi").join("security").join("tool-chains.json");
        std::fs::read_to_string(p)
            .ok()
            .and_then(|raw| serde_json::from_str::<ToolChainsPolicy>(&raw).ok())
            .unwrap_or_default()
    }

    pub fn check_sequence(&self, history: &[String], next_tool: &str) -> Result<(), String> {
        let seen: HashSet<&str> = history.iter().map(String::as_str).collect();
        for rule in &self.rules {
            if rule.tool == next_tool {
                for req in &rule.requires_prior {
                    if !seen.contains(req.as_str()) {
                        return Err(format!(
                            "Tool-chain violation: '{next_tool}' requires prior '{req}'"
                        ));
                    }
                }
                for forb in &rule.forbid_after {
                    if seen.contains(forb.as_str()) {
                        return Err(format!(
                            "Tool-chain violation: '{next_tool}' forbidden after '{forb}'"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Fail-closed hookguard inspecting tool-call inputs, outputs, and tool-chain ordering.
pub fn evaluate_hookguard(
    repo_root: Option<&Path>,
    actor: &str,
    tool_name: &str,
    payload: &Value,
    prior_tools: &[String],
    fail_closed: bool,
) -> HookGuardVerdict {
    let mut reasons = Vec::new();
    let serialized = payload.to_string();

    if actor.trim().is_empty() || tool_name.trim().is_empty() {
        reasons.push("HookGuard fail-closed: empty actor or tool_name".to_string());
    }

    let g = validate_and_anonymize(&serialized);
    if !g.passed {
        reasons.extend(g.findings);
    }

    for ym in scan_yara_mcp(&serialized) {
        reasons.push(format!("YARA-MCP {}: {}", ym.rule_name, ym.description));
    }

    if let Some(root) = repo_root {
        let chain = ToolChainsPolicy::load_from_repo(root);
        if let Err(err) = chain.check_sequence(prior_tools, tool_name) {
            reasons.push(err);
        }
    }

    let allowed = reasons.is_empty() || (!fail_closed && reasons.iter().all(|r| !r.contains("fail-closed")));
    let outcome = if allowed { "allowed" } else { "blocked_fail_closed" };
    let _ = record_audit_evidence(
        repo_root,
        actor,
        "hookguard",
        tool_name,
        outcome,
        &reasons.join("; "),
        tool_name,
    );

    HookGuardVerdict {
        allowed,
        fail_closed,
        reasons,
    }
}
