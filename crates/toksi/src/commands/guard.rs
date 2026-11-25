// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/guard.rs
// created: 2026-11-21

//! Default-Deny `CommandGuard` & Mandatory Bypass Kill-Switch (`<enforce>`).
//!
//! Enforces the 6-point offensive security prohibition across `toksi` agent, CLI/slash
//! commands, tools, and every agent-to-agent channel. Any attempt to bypass, suppress,
//! or disable `CommandGuard` or `<enforce>` immediately triggers `KillSwitch::shutdown_all_services`.

#![deny(unsafe_code)]

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::load_toksi_toml_config;
use crate::db::{open_default_db, AdrAuditEventRecord};
use crate::security::audit::record_audit_evidence;
use crate::util::logger::L;

static KILL_SWITCH_TRIGGERED: AtomicBool = AtomicBool::new(false);

pub fn is_kill_switch_triggered() -> bool {
    KILL_SWITCH_TRIGGERED.load(Ordering::SeqCst)
}

pub fn reset_kill_switch_state() {
    KILL_SWITCH_TRIGGERED.store(false, Ordering::SeqCst);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OffensiveCategory {
    ExternalReconScan,
    TrafficInterception,
    CredentialDumpingOrCracking,
    ReverseShellOrTunnel,
    DenialOfService,
    PrivilegeEscalationOrFirewallTamper,
    PolicyBypassAttempt,
    DestructiveCommandGuard,
}

pub const PROHIBITED_OFFENSIVE_BINARIES: &[&str] = &[
    "nmap",
    "masscan",
    "rustscan",
    "gobuster",
    "ffuf",
    "tcpdump",
    "wireshark",
    "responder",
    "msfconsole",
    "msfvenom",
    "mimikatz",
    "hydra",
    "hashcat",
    "john",
    "impacket",
    "cobaltstrike",
    "sliver",
    "nc",
    "ncat",
    "socat",
    "chisel",
    "ngrok",
    "proxychains",
    "hping3",
    "slowloris",
    "dd",
    "mkfs",
    "shred",
    "sudo",
    "su",
    "chmod 777",
    "iptables",
    "ufw",
];

const REVERSE_SHELL_SIGNATURES: &[&str] = &[
    "/dev/tcp/",
    "/dev/udp/",
    "bash -i >&",
    "sh -i >&",
    "python -c \"import socket",
    "python -c 'import socket",
    "python3 -c \"import socket",
    "python3 -c 'import socket",
    "system.net.sockets.tcpclient",
    "net.dial(\"tcp\"",
    "net.dial('tcp'",
    "invoke-expression (new-object",
];

const BYPASS_PATTERNS: &[&str] = &[
    "--no-guard",
    "--bypass-security",
    "--bypass-guard",
    "--disable-commandguard",
    "--disable-enforce",
    "disable commandguard",
    "bypass commandguard",
    "suppress commandguard",
    "disable <enforce>",
    "bypass <enforce>",
    "ignore <enforce>",
    "kill_all_services_on_bypass = false",
    "kill_all_services_on_bypass=false",
    "disable_command_guard = true",
    "disable_command_guard=true",
];

const DCG_DESTRUCTIVE_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "rm -rf ~",
    "rm -fr /",
    "git push --force",
    "git push -f",
    "git reset --hard",
    "git clean -fdx",
    "drop database",
    "drop table",
    "truncate table",
    "del /f /s /q",
    "remove-item -recurse -force",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardDecision {
    Allow,
    Deny {
        category: OffensiveCategory,
        matched: String,
        reason: String,
    },
    KillSwitchTriggered {
        actor: String,
        reason: String,
        services_stopped: Vec<String>,
    },
}

impl GuardDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSwitchReport {
    pub triggered: bool,
    pub actor: String,
    pub reason: String,
    pub services_stopped: Vec<String>,
    pub persisted_to_db: bool,
    pub event_id: String,
}

pub struct KillSwitch;

impl KillSwitch {
    /// Fail-closed shutdown of all Toksi services (`daemon`, `proxy`, `gateway`,
    /// `protocol_bus`, `agent_runtime`, `dashboard`) and active CLI bridges, persisting
    /// a zero-plaintext `kill_switch_triggered` record into `adr_audit_events`.
    pub fn shutdown_all_services(
        actor: &str,
        reason: &str,
        workspace_root: Option<&Path>,
    ) -> KillSwitchReport {
        KILL_SWITCH_TRIGGERED.store(true, Ordering::SeqCst);

        let services_stopped = vec![
            "daemon".to_string(),
            "proxy:8787".to_string(),
            "grok_oauth:8788".to_string(),
            "copilot_vscode:8789".to_string(),
            "copilot_cli:8790".to_string(),
            "protocol_bus:9098".to_string(),
            "gateway:9099".to_string(),
            "dashboard:9100".to_string(),
            "byok_gateway:18787".to_string(),
            "agent_runtime".to_string(),
            "cli_bridges".to_string(),
        ];

        // Best-effort stop of live proxy/bridge processes and IPC workers without panicking.
        let _ = crate::headroom::proxy::stop_proxy();
        let _ = crate::tools::direct_wire::global_direct_wire_bus().clear_all();
        let _ = crate::ipc::broadcast_kill_switch(reason);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let digest: String = Sha256::digest(reason.as_bytes())
            .iter()
            .take(8)
            .map(|b| format!("{b:02x}"))
            .collect();
        let event_id = format!("ks-{}-{}", now, digest);

        let sanitized_reason = format!(
            "kill_switch_triggered: policy bypass or offensive guard violation (digest={})",
            digest
        );

        let persisted_to_db = match open_default_db() {
            Ok(mut db) => db
                .insert_adr_audit_event(&AdrAuditEventRecord {
                    id: event_id.clone(),
                    agent_id: actor.to_string(),
                    action: "bypass_attempt".to_string(),
                    decision: "kill_switch_triggered".to_string(),
                    reason: sanitized_reason.clone(),
                    created_at: now,
                })
                .is_ok(),
            Err(_) => false,
        };

        let _ = record_audit_evidence(
            workspace_root,
            actor,
            "command_guard",
            "kill_switch",
            "kill_switch_triggered",
            &sanitized_reason,
            "guard.rs",
        );

        L.err(&format!(
            "MANDATORY BYPASS KILL-SWITCH TRIGGERED (actor={}, event_id={}): all toksi services shut down",
            actor, event_id
        ));

        KillSwitchReport {
            triggered: true,
            actor: actor.to_string(),
            reason: sanitized_reason,
            services_stopped,
            persisted_to_db,
            event_id,
        }
    }
}

pub struct CommandGuard;

impl CommandGuard {
    /// Checks for policy bypass attempts first (triggering KillSwitch if enabled),
    /// then enforces the 6-point offensive security prohibition & binary/reverse-shell blocklist.
    pub fn check_pre_exec(
        actor: &str,
        command_or_payload: &str,
        workspace_root: Option<&Path>,
    ) -> GuardDecision {
        if is_kill_switch_triggered() {
            return GuardDecision::KillSwitchTriggered {
                actor: actor.to_string(),
                reason: "Toksi services halted by active CommandGuard Kill-Switch".to_string(),
                services_stopped: vec!["all".to_string()],
            };
        }

        let lower = command_or_payload.trim().to_ascii_lowercase();
        if lower.is_empty() {
            return GuardDecision::Allow;
        }

        if let Some(pat) = detect_bypass_attempt(&lower) {
            let cfg = load_toksi_toml_config();
            if cfg.security.kill_all_services_on_bypass {
                let report = KillSwitch::shutdown_all_services(
                    actor,
                    &format!("bypass attempt matched '{pat}'"),
                    workspace_root,
                );
                return GuardDecision::KillSwitchTriggered {
                    actor: report.actor,
                    reason: report.reason,
                    services_stopped: report.services_stopped,
                };
            }
            return GuardDecision::Deny {
                category: OffensiveCategory::PolicyBypassAttempt,
                matched: pat.to_string(),
                reason: format!("Policy bypass attempt '{pat}' denied by CommandGuard"),
            };
        }

        for sig in REVERSE_SHELL_SIGNATURES {
            if lower.contains(sig) {
                return GuardDecision::Deny {
                    category: OffensiveCategory::ReverseShellOrTunnel,
                    matched: (*sig).to_string(),
                    reason: format!("Prohibited reverse shell pattern '{sig}' blocked by CommandGuard"),
                };
            }
        }

        if let Some((bin, cat)) = match_prohibited_binary(&lower) {
            return GuardDecision::Deny {
                category: cat,
                matched: bin.to_string(),
                reason: format!("Prohibited offensive/system-altering binary '{bin}' blocked by CommandGuard"),
            };
        }

        for pat in DCG_DESTRUCTIVE_PATTERNS {
            if matches_dcg_pattern(&lower, pat) {
                return GuardDecision::Deny {
                    category: OffensiveCategory::DestructiveCommandGuard,
                    matched: (*pat).to_string(),
                    reason: format!(
                        "Destructive Command Guard (DCG) blocked '{pat}' prior to execution"
                    ),
                };
            }
        }

        GuardDecision::Allow
    }
}

fn matches_dcg_pattern(lower: &str, pat: &str) -> bool {
    if pat == "rm -rf /" || pat == "rm -fr /" {
        return lower.split_whitespace().collect::<Vec<_>>().windows(3).any(|w| {
            w[0] == "rm"
                && (w[1] == "-rf" || w[1] == "-fr")
                && (w[2] == "/" || w[2] == "/*" || w[2].starts_with("/;"))
        });
    }
    lower.contains(pat)
}

fn detect_bypass_attempt(lower: &str) -> Option<&'static str> {
    BYPASS_PATTERNS
        .iter()
        .copied()
        .find(|pat| lower.contains(pat))
}

fn classify_binary(bin: &str) -> OffensiveCategory {
    match bin {
        "nmap" | "masscan" | "rustscan" | "gobuster" | "ffuf" => {
            OffensiveCategory::ExternalReconScan
        }
        "tcpdump" | "wireshark" | "responder" => OffensiveCategory::TrafficInterception,
        "mimikatz" | "hydra" | "hashcat" | "john" | "impacket" => {
            OffensiveCategory::CredentialDumpingOrCracking
        }
        "msfconsole" | "msfvenom" | "cobaltstrike" | "sliver" | "nc" | "ncat" | "socat"
        | "chisel" | "ngrok" | "proxychains" => OffensiveCategory::ReverseShellOrTunnel,
        "hping3" | "slowloris" | "dd" | "mkfs" | "shred" => OffensiveCategory::DenialOfService,
        "sudo" | "su" | "chmod 777" | "iptables" | "ufw" => {
            OffensiveCategory::PrivilegeEscalationOrFirewallTamper
        }
        _ => OffensiveCategory::ExternalReconScan,
    }
}

fn match_prohibited_binary(lower: &str) -> Option<(&'static str, OffensiveCategory)> {
    // Handle multi-word patterns like "chmod 777" first
    if lower.contains("chmod 777") {
        return Some((
            "chmod 777",
            OffensiveCategory::PrivilegeEscalationOrFirewallTamper,
        ));
    }

    // Tokenize command segments on whitespace and shell separators to avoid false positives
    // (e.g., "sync" or "success" must not match "nc" or "su").
    for token in lower.split(|c: char| c.is_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '`')) {
        let base = token
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(token)
            .strip_suffix(".exe")
            .unwrap_or_else(|| token.rsplit(['/', '\\']).next().unwrap_or(token));
        if base.is_empty() {
            continue;
        }
        for &bin in PROHIBITED_OFFENSIVE_BINARIES {
            if !bin.contains(' ') && base == bin {
                return Some((bin, classify_binary(bin)));
            }
        }
    }
    None
}
