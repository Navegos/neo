// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/safeagents.rs
// created: 2026-11-21

//! `safeagents` multi-agent delegation & tool-boundary safety verifier.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::security::guardrails_validators::validate_and_anonymize;
use crate::security::policy_opa::{evaluate_opa_policy, OpaPolicyInput};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafeAgentsCheck {
    pub allowed: bool,
    pub source_agent: String,
    pub target_agent: String,
    pub violations: Vec<String>,
}

/// Validates an agent-to-agent delegation or tool handoff against `safeagents` + OPA + Guardrails.
pub fn verify_agent_handoff(
    source_agent: &str,
    target_agent: &str,
    action: &str,
    payload: &str,
) -> SafeAgentsCheck {
    let mut violations = Vec::new();

    if source_agent.trim().is_empty() || target_agent.trim().is_empty() {
        violations.push("SafeAgents: empty source or target agent ID".to_string());
    }

    let opa = evaluate_opa_policy(&OpaPolicyInput {
        action: action.to_string(),
        actor: source_agent.to_string(),
        resource: target_agent.to_string(),
        metadata: serde_json::json!({ "channel": "agent_to_agent" }),
    });
    if !opa.allow {
        violations.extend(opa.reasons);
    }

    let guard = validate_and_anonymize(payload);
    if !guard.passed {
        violations.extend(guard.findings);
    }

    SafeAgentsCheck {
        allowed: violations.is_empty(),
        source_agent: source_agent.to_string(),
        target_agent: target_agent.to_string(),
        violations,
    }
}
