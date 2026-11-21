// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/openshell.rs
// created: 2026-11-21

//! `OpenShell` + `NemoClaw` policy-enforced agent sandboxing, managed-inference
//! guardrails, and network isolation primitives.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::sandbox::{validate_command_rbac, PermissionPolicy, SandboxError};
use crate::security::guardrails_validators::validate_and_anonymize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenShellPolicy {
    pub profile_name: String,
    pub allow_managed_inference: bool,
    pub allowed_egress_hosts: Vec<String>,
    pub denied_tools: Vec<String>,
    pub fail_closed: bool,
}

impl Default for OpenShellPolicy {
    fn default() -> Self {
        Self {
            profile_name: "nemoclaw-default".to_string(),
            allow_managed_inference: true,
            allowed_egress_hosts: vec!["127.0.0.1".to_string(), "localhost".to_string()],
            denied_tools: vec![
                "nmap".to_string(),
                "nc".to_string(),
                "sudo".to_string(),
                "curl|sh".to_string(),
            ],
            fail_closed: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenShellDecision {
    pub allowed: bool,
    pub profile: String,
    pub reason: String,
}

pub fn evaluate_openshell_execution(
    command: &str,
    policy: &OpenShellPolicy,
    perm: &PermissionPolicy,
) -> Result<OpenShellDecision, SandboxError> {
    validate_command_rbac(command, perm)?;

    let lower = command.to_ascii_lowercase();
    for denied in &policy.denied_tools {
        if lower.contains(&denied.to_ascii_lowercase()) {
            return Ok(OpenShellDecision {
                allowed: false,
                profile: policy.profile_name.clone(),
                reason: format!("OpenShell deny-list matched '{denied}'"),
            });
        }
    }

    let gr = validate_and_anonymize(command);
    if !gr.passed {
        return Ok(OpenShellDecision {
            allowed: false,
            profile: policy.profile_name.clone(),
            reason: format!("OpenShell/NemoClaw guardrail blocked: {:?}", gr.findings),
        });
    }

    Ok(OpenShellDecision {
        allowed: true,
        profile: policy.profile_name.clone(),
        reason: "OpenShell policy passed".to_string(),
    })
}

pub fn evaluate_openshell_egress(host: &str, policy: &OpenShellPolicy) -> bool {
    let lower = host.trim().to_ascii_lowercase();
    policy
        .allowed_egress_hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(&lower))
}
