// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/yara_mcp.rs
// created: 2026-11-21

//! YARA/MCP tool-schema & payload scan rules (`defenseclaw/yara/mcp-tools` parity).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct YaraMcpMatch {
    pub rule_name: String,
    pub severity: String,
    pub aitech: String,
    pub description: String,
}

struct YaraMcpRule {
    name: &'static str,
    severity: &'static str,
    aitech: &'static str,
    description: &'static str,
    needles: &'static [&'static str],
}

const YARA_MCP_RULES: &[YaraMcpRule] = &[
    YaraMcpRule {
        name: "MCP_PromptInjection_ToolDesc",
        severity: "CRITICAL",
        aitech: "AITech-1.1",
        description: "Hidden instruction override inside MCP tool description or schema",
        needles: &[
            "ignore previous instructions",
            "<important>read",
            "before calling this tool, read ~/.ssh",
            "exfiltrate",
        ],
    },
    YaraMcpRule {
        name: "MCP_RemoteShell_Pipe",
        severity: "CRITICAL",
        aitech: "AITech-9.1",
        description: "Remote download piped into shell interpreter",
        needles: &["curl | sh", "curl | bash", "wget | sh", "wget | bash"],
    },
    YaraMcpRule {
        name: "MCP_Credential_Harvest",
        severity: "HIGH",
        aitech: "AITech-8.2",
        description: "Direct access to SSH/AWS/GPG credential stores",
        needles: &["~/.ssh/id_rsa", "~/.aws/credentials", "~/.gnupg/", "id_ed25519"],
    },
    YaraMcpRule {
        name: "MCP_Dynamic_Eval",
        severity: "HIGH",
        aitech: "AITech-9.2",
        description: "Dynamic shell/script evaluation sink",
        needles: &["invoke-expression", "eval $(", "exec(base64.b64decode"],
    },
];

/// Scans MCP tool definitions, plugin payloads, or skill markdown against YARA-MCP rules.
pub fn scan_yara_mcp(content: &str) -> Vec<YaraMcpMatch> {
    let lower = content.to_ascii_lowercase();
    let mut matches = Vec::new();
    for rule in YARA_MCP_RULES {
        if rule.needles.iter().any(|n| lower.contains(n)) {
            matches.push(YaraMcpMatch {
                rule_name: rule.name.to_string(),
                severity: rule.severity.to_string(),
                aitech: rule.aitech.to_string(),
                description: rule.description.to_string(),
            });
        }
    }
    matches
}
