// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/adr.rs
// created: 2026-11-21

//! Agent Detection & Response (ADR) Layer.
//!
//! Intercepts every `Command`, `FileMutation`, `WebRequest`, and `A2aMessage` prior to host
//! execution across `<toksi-siagent>` and hooked peer channels, attesting via SPIFFE SVIDs and
//! OPA/Cedar policies, diverting unsafe/untrusted/unknown payloads into the Isolated Agent
//! Sandbox, alerting user + orchestrating agent, and writing zero-plaintext incident artifacts.

#![deny(unsafe_code)]

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::agent_runtime::sandbox::{global_sandbox_manager, ssrf_guard, SandboxBackend};
use crate::commands::guard::{CommandGuard, GuardDecision};
use crate::db::{open_default_db, AdrAuditEventRecord};
use crate::security::audit::record_audit_evidence;
use crate::security::blind_compiler::DEFAULT_BLIND_KEY;
use crate::security::iam_attestation::SpiffeSvid;
use crate::security::llm_guard::{scan_llm_guard, LlmGuardConfig};
use crate::security::policy_cedar::{CedarEffect, CedarPolicyEngine};
use crate::security::policy_opa::{evaluate_opa_policy, OpaPolicyInput};
use crate::security::safeagents::verify_agent_handoff;
use crate::security::sage_rules::scan_sage_rules;
use crate::security::sarif::{format_findings, ReportFormat, UnifiedSecurityFinding};
use crate::security::scanner::scan_with_blind_signatures;
use crate::security::schema::BlindSignatures;
use crate::security::skill_scanner::scan_skill_or_plugin_with_preset;
use crate::security::{CelVerdict, ScanPreset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOrigin {
    VerifiedWorkspace,
    Untrusted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentAction {
    Command {
        program: String,
        args: Vec<String>,
    },
    FileMutation {
        path: String,
        operation: String,
        content_preview: String,
    },
    WebRequest {
        url: String,
        method: String,
    },
    A2aMessage {
        from_agent: String,
        to_agent: String,
        payload: String,
    },
}

impl AgentAction {
    pub fn channel_name(&self) -> &'static str {
        match self {
            Self::Command { .. } => "command",
            Self::FileMutation { .. } => "file",
            Self::WebRequest { .. } => "web",
            Self::A2aMessage { .. } => "a2a",
        }
    }

    pub fn summary_payload(&self) -> String {
        match self {
            Self::Command { program, args } => {
                if args.is_empty() {
                    program.clone()
                } else {
                    format!("{program} {}", args.join(" "))
                }
            }
            Self::FileMutation {
                path,
                operation,
                content_preview,
            } => format!("{operation} {path} {content_preview}"),
            Self::WebRequest { url, method } => format!("{method} {url}"),
            Self::A2aMessage {
                from_agent,
                to_agent,
                payload,
            } => format!("{from_agent}->{to_agent}: {payload}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrDisposition {
    VerifiedSafe,
    Sandboxed,
    Blocked,
    KillSwitchTriggered,
}

impl AdrDisposition {
    pub fn db_str(&self) -> &'static str {
        match self {
            Self::VerifiedSafe => "allowed",
            Self::Sandboxed => "sandboxed",
            Self::Blocked => "blocked",
            Self::KillSwitchTriggered => "kill_switch_triggered",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrAlert {
    pub notify_user: bool,
    pub notify_orchestrator: bool,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrDecision {
    pub event_id: String,
    pub disposition: AdrDisposition,
    pub sandbox_id: Option<String>,
    pub indicators: Vec<String>,
    pub alert: Option<AdrAlert>,
    pub sarif_json: Option<String>,
    pub osv_record_id: Option<String>,
    pub stix_indicator_id: Option<String>,
    pub svid_spiffe_id: String,
    pub persisted_to_db: bool,
}

pub struct AdrEngine;

impl AdrEngine {
    pub fn inspect(
        action: &AgentAction,
        actor_agent: &str,
        origin: ActionOrigin,
        workspace_root: Option<&Path>,
        blind_sigs: Option<&BlindSignatures>,
    ) -> AdrDecision {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let payload = action.summary_payload();
        let digest_hex: String = Sha256::digest(payload.as_bytes())
            .iter()
            .take(6)
            .map(|b| format!("{b:02x}"))
            .collect();
        let event_id = format!("adr-{}-{}", now, digest_hex);

        let svid = SpiffeSvid::issue("toksi.local", actor_agent, 300)
            .map(|s| s.spiffe_id)
            .unwrap_or_else(|_| format!("spiffe://toksi.local/agent/{actor_agent}"));

        // 1. Pre-exec CommandGuard & KillSwitch check
        let guard = CommandGuard::check_pre_exec(actor_agent, &payload, workspace_root);
        match guard {
            GuardDecision::KillSwitchTriggered { reason, .. } => {
                return Self::finalize_incident(
                    event_id,
                    actor_agent,
                    action,
                    AdrDisposition::KillSwitchTriggered,
                    vec!["command_guard_bypass".to_string()],
                    reason,
                    "critical",
                    svid,
                    workspace_root,
                    now,
                    false,
                );
            }
            GuardDecision::Deny { matched, reason, .. } => {
                return Self::finalize_incident(
                    event_id,
                    actor_agent,
                    action,
                    AdrDisposition::Blocked,
                    vec![format!("command_guard:{matched}")],
                    reason,
                    "high",
                    svid,
                    workspace_root,
                    now,
                    true,
                );
            }
            GuardDecision::Allow => {}
        }

        // 2. OPA & Cedar Policy-as-Code + A2A SafeAgents check
        let opa = evaluate_opa_policy(&OpaPolicyInput {
            action: action.channel_name().to_string(),
            actor: actor_agent.to_string(),
            resource: payload.clone(),
            metadata: json!({ "origin": origin }),
        });
        let cedar_principal = if actor_agent == "siagent-core" {
            "toksi-siagent"
        } else {
            actor_agent
        };
        let cedar_action = if matches!(origin, ActionOrigin::VerifiedWorkspace) {
            "read_workspace"
        } else {
            action.channel_name()
        };
        let cedar = CedarPolicyEngine::with_default_toksi_rules().evaluate(
            cedar_principal,
            cedar_action,
            &payload,
        );
        let mut indicators = Vec::new();
        if !opa.allow {
            indicators.push(format!("opa_deny:{}", opa.reasons.join("; ")));
        }
        if cedar.decision == CedarEffect::Forbid {
            indicators.push("cedar_forbid".to_string());
        }
        if let AgentAction::A2aMessage {
            from_agent,
            to_agent,
            payload: msg,
        } = action
        {
            let sa = verify_agent_handoff(from_agent, to_agent, "a2a_message", msg);
            if !sa.allowed {
                indicators.push(format!("safeagents_block:{}", sa.violations.join("; ")));
            }
        }

        // 3. Threat behavior & indicator engines: sage, llm-guard, skill-scanner, blind_signatures, ssrf
        let llm = scan_llm_guard(&payload, &LlmGuardConfig::default());
        if !llm.is_valid {
            indicators.push(format!("llm_guard:risk={:.2}", llm.risk_score));
        }
        let sage = scan_sage_rules("adr-payload", &payload);
        for f in &sage {
            indicators.push(format!("sage:{}", f.rule_id));
        }
        let ss = scan_skill_or_plugin_with_preset(
            "adr-action",
            &payload,
            workspace_root,
            ScanPreset::Balanced,
        );
        if ss.verdict != CelVerdict::Allow {
            indicators.push(format!("skill_scanner:{:?}", ss.verdict));
        }
        if let Some(sigs) = blind_sigs {
            let blind_hits = scan_with_blind_signatures(&payload, sigs, &DEFAULT_BLIND_KEY);
            for h in blind_hits {
                indicators.push(format!("blind_sig:{}:{}", h.kind, h.digest_or_index));
            }
        }
        if let AgentAction::WebRequest { url, .. } = action
            && let Err(e) = ssrf_guard(url)
        {
            indicators.push(format!("ssrf_guard:{e}"));
        }

        // 4. Route decision
        let untrusted_origin = matches!(origin, ActionOrigin::Untrusted | ActionOrigin::Unknown);
        if !indicators.is_empty() || untrusted_origin {
            if untrusted_origin && indicators.is_empty() {
                indicators.push(format!("origin:{origin:?}"));
            }
            let sandbox_id = workspace_root.map(|root| {
                let mgr = global_sandbox_manager();
                let inst = mgr.create(format!("adr-sbx-{}", &digest_hex[..8]), SandboxBackend::Native, root);
                inst.id
            });
            let reason = format!(
                "Diverted {} action from '{}' into isolated sandbox (indicators={})",
                action.channel_name(),
                actor_agent,
                indicators.join(",")
            );
            let mut dec = Self::finalize_incident(
                event_id,
                actor_agent,
                action,
                AdrDisposition::Sandboxed,
                indicators,
                reason,
                "medium",
                svid,
                workspace_root,
                now,
                true,
            );
            dec.sandbox_id = sandbox_id;
            return dec;
        }

        let persisted_to_db = Self::persist_db_event(
            &event_id,
            actor_agent,
            action.channel_name(),
            AdrDisposition::VerifiedSafe.db_str(),
            "verified_safe",
            now,
        );

        AdrDecision {
            event_id,
            disposition: AdrDisposition::VerifiedSafe,
            sandbox_id: None,
            indicators: Vec::new(),
            alert: None,
            sarif_json: None,
            osv_record_id: None,
            stix_indicator_id: None,
            svid_spiffe_id: svid,
            persisted_to_db,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finalize_incident(
        event_id: String,
        actor_agent: &str,
        action: &AgentAction,
        disposition: AdrDisposition,
        indicators: Vec<String>,
        reason: String,
        severity: &str,
        svid_spiffe_id: String,
        workspace_root: Option<&Path>,
        now: i64,
        write_db: bool,
    ) -> AdrDecision {
        // Zero-plaintext sanitized SARIF 2.1.0 finding (never embeds raw exploit/payload)
        let finding = UnifiedSecurityFinding {
            rule_id: format!("TOKSI-ADR-{}", action.channel_name().to_ascii_uppercase()),
            severity: severity.to_string(),
            cwe: "CWE-78".to_string(),
            aitech: "AITech-ADR-01".to_string(),
            file: format!("channel://{}", action.channel_name()),
            line: 1,
            message: format!("ADR {:?} on {}", disposition, action.channel_name()),
            remediation: "Execute inside Isolated Agent Sandbox or remediate policy violation.".to_string(),
        };
        let sarif_json = format_findings(&[finding], ReportFormat::Sarif);
        let osv_record_id = format!("TOKSI-ADR-{now}");
        let stix_indicator_id = format!("indicator--{event_id}");

        let _ = record_audit_evidence(
            workspace_root,
            actor_agent,
            "adr",
            action.channel_name(),
            disposition.db_str(),
            &reason,
            &svid_spiffe_id,
        );

        let persisted_to_db = if write_db {
            Self::persist_db_event(
                &event_id,
                actor_agent,
                action.channel_name(),
                disposition.db_str(),
                &reason,
                now,
            )
        } else {
            true
        };

        AdrDecision {
            event_id,
            disposition,
            sandbox_id: None,
            indicators,
            alert: Some(AdrAlert {
                notify_user: true,
                notify_orchestrator: true,
                severity: severity.to_string(),
                message: reason,
            }),
            sarif_json: Some(sarif_json),
            osv_record_id: Some(osv_record_id),
            stix_indicator_id: Some(stix_indicator_id),
            svid_spiffe_id,
            persisted_to_db,
        }
    }

    fn persist_db_event(
        id: &str,
        agent_id: &str,
        action: &str,
        decision: &str,
        reason: &str,
        created_at: i64,
    ) -> bool {
        match open_default_db() {
            Ok(mut db) => db
                .insert_adr_audit_event(&AdrAuditEventRecord {
                    id: id.to_string(),
                    agent_id: agent_id.to_string(),
                    action: action.to_string(),
                    decision: decision.to_string(),
                    reason: reason.to_string(),
                    created_at,
                })
                .is_ok(),
            Err(_) => false,
        }
    }
}
