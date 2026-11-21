// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/policy.rs
// created: 2026-11-21

//! Gateway Policy-as-Code (`opa`, `cedar`) & SPIFFE/SPIRE IAM attestation gate
//! for deterministic tool, file, network, and agent-to-agent authorization.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::security::iam_attestation::SpiffeSvid;
use crate::security::policy_cedar::{CedarEffect, CedarPolicyEngine, CedarRule};
use crate::security::policy_opa::{evaluate_opa_policy, OpaPolicyInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationScope {
    Tool,
    File,
    Network,
    AgentToAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayPolicyRequest {
    pub scope: AuthorizationScope,
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub svid: Option<SpiffeSvid>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayPolicyDecision {
    pub allow: bool,
    pub spiffe_verified: bool,
    pub spiffe_id: Option<String>,
    pub opa_allow: bool,
    pub opa_reasons: Vec<String>,
    pub cedar_decision: CedarEffect,
    pub matched_rules: Vec<String>,
}

/// Unified Gateway Policy Enforcer combining SPIFFE SVID attestation, OPA Rego rules,
/// and pure-Rust Cedar PBAC evaluation.
#[derive(Debug)]
pub struct GatewayPolicyEnforcer {
    trust_domain: String,
    require_svid: bool,
    cedar: CedarPolicyEngine,
}

impl Default for GatewayPolicyEnforcer {
    fn default() -> Self {
        Self::new("toksi.local", false)
    }
}

impl GatewayPolicyEnforcer {
    pub fn new(trust_domain: impl Into<String>, require_svid: bool) -> Self {
        Self {
            trust_domain: trust_domain.into(),
            require_svid,
            cedar: CedarPolicyEngine::with_default_toksi_rules(),
        }
    }

    pub fn add_cedar_rule(&mut self, rule: CedarRule) {
        self.cedar.add_rule(rule);
    }

    pub fn authorize(&self, req: &GatewayPolicyRequest, now_unix: i64) -> GatewayPolicyDecision {
        let (spiffe_verified, spiffe_id) = match &req.svid {
            Some(svid) => {
                let ok = svid.trust_domain == self.trust_domain && svid.is_valid_at(now_unix);
                (ok, Some(svid.spiffe_id.clone()))
            }
            None => (!self.require_svid, None),
        };

        let opa = evaluate_opa_policy(&OpaPolicyInput {
            action: req.action.clone(),
            actor: req.principal.clone(),
            resource: req.resource.clone(),
            metadata: req.metadata.clone(),
        });

        let cedar = self
            .cedar
            .evaluate(&req.principal, &req.action, &req.resource);

        let allow = spiffe_verified && opa.allow && cedar.decision == CedarEffect::Permit;
        GatewayPolicyDecision {
            allow,
            spiffe_verified,
            spiffe_id,
            opa_allow: opa.allow,
            opa_reasons: opa.reasons,
            cedar_decision: cedar.decision,
            matched_rules: cedar.matched_rules,
        }
    }
}
