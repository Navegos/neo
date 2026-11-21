// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/modes.rs
// created: 2026-11-21

//! Agent mode switching (`cline`, `kilocode`, `opencode`, `alphacode`, `zeroclaw`)
//! for `<toksi-siagent>`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    Architect,
    Code,
    Ask,
    Debug,
    Orchestrator,
    Review,
}

impl AgentMode {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "architect" | "plan" => Some(Self::Architect),
            "code" | "build" | "act" => Some(Self::Code),
            "ask" | "chat" => Some(Self::Ask),
            "debug" | "fix" => Some(Self::Debug),
            "orchestrator" | "swarm" => Some(Self::Orchestrator),
            "review" | "audit" => Some(Self::Review),
            _ => None,
        }
    }

    pub fn allows_file_writes(self) -> bool {
        matches!(self, Self::Code | Self::Debug | Self::Orchestrator)
    }
}
