// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/policy_cedar.rs
// created: 2026-11-21

//! Pure-Rust Cedar PBAC (`principal`, `action`, `resource`) policy evaluator.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CedarEffect {
    Permit,
    Forbid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CedarRule {
    pub effect: CedarEffect,
    pub principal: String,
    pub action: String,
    pub resource_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CedarEvalResult {
    pub decision: CedarEffect,
    pub matched_rules: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CedarPolicyEngine {
    rules: Vec<CedarRule>,
}

impl CedarPolicyEngine {
    pub fn with_default_toksi_rules() -> Self {
        Self {
            rules: vec![
                CedarRule {
                    effect: CedarEffect::Forbid,
                    principal: "*".to_string(),
                    action: "bypass_security".to_string(),
                    resource_prefix: "*".to_string(),
                },
                CedarRule {
                    effect: CedarEffect::Permit,
                    principal: "toksi-siagent".to_string(),
                    action: "tool_invoke".to_string(),
                    resource_prefix: ".toksi/".to_string(),
                },
                CedarRule {
                    effect: CedarEffect::Permit,
                    principal: "toksi-siagent".to_string(),
                    action: "read_workspace".to_string(),
                    resource_prefix: "*".to_string(),
                },
            ],
        }
    }

    pub fn add_rule(&mut self, rule: CedarRule) {
        self.rules.push(rule);
    }

    /// Cedar semantics: any matching `Forbid` overrides `Permit`; default is `Forbid`.
    pub fn evaluate(&self, principal: &str, action: &str, resource: &str) -> CedarEvalResult {
        let mut permitted = false;
        let mut matched = Vec::new();
        for rule in &self.rules {
            let p_ok = rule.principal == "*" || rule.principal == principal;
            let a_ok = rule.action == "*" || rule.action == action;
            let r_ok = rule.resource_prefix == "*" || resource.starts_with(&rule.resource_prefix);
            if p_ok && a_ok && r_ok {
                matched.push(format!("{:?}:{}:{}:{}", rule.effect, rule.principal, rule.action, rule.resource_prefix));
                if rule.effect == CedarEffect::Forbid {
                    return CedarEvalResult {
                        decision: CedarEffect::Forbid,
                        matched_rules: matched,
                    };
                }
                permitted = true;
            }
        }
        CedarEvalResult {
            decision: if permitted {
                CedarEffect::Permit
            } else {
                CedarEffect::Forbid
            },
            matched_rules: matched,
        }
    }
}
