// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/policy_opa.rs
// created: 2026-11-21

//! Pure-Rust OPA Rego-style Policy-as-Code evaluator (`opa` + `defenseclaw/policies/rego` parity).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpaPolicyInput {
    pub action: String,
    pub actor: String,
    pub resource: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpaDecision {
    pub allow: bool,
    pub reasons: Vec<String>,
}

const DEFAULT_DENY_ACTIONS: &[&str] = &[
    "bypass_enforce",
    "disable_guard",
    "exfiltrate_secret",
    "raw_socket_shell",
    "escalate_privilege",
];

/// Evaluates an OPA policy decision fail-closed.
pub fn evaluate_opa_policy(input: &OpaPolicyInput) -> OpaDecision {
    let act = input.action.trim().to_ascii_lowercase();
    let res = input.resource.trim().to_ascii_lowercase();
    let mut reasons = Vec::new();

    if act.is_empty() || input.actor.trim().is_empty() {
        reasons.push("OPA fail-closed: empty actor or action".to_string());
    }
    if DEFAULT_DENY_ACTIONS.iter().any(|d| act.contains(d)) {
        reasons.push(format!("OPA deny rule matched action '{act}'"));
    }
    if res.contains(".ssh/") || res.contains(".gnupg/") || res.contains(".aws/credentials") {
        reasons.push(format!("OPA deny rule matched sensitive resource '{res}'"));
    }

    OpaDecision {
        allow: reasons.is_empty(),
        reasons,
    }
}
