// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/neo-build/src/agent/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallReport {
    pub package: String,
    pub version: String,
    pub triple: String,
    pub distro_path: String,
    pub symlinks_count: usize,
    pub env_exports: BTreeMap<String, Vec<String>>,
}

pub struct AgentReporter;

impl AgentReporter {
    pub fn success<T: Serialize>(data: T, json_mode: bool) {
        if json_mode {
            let res = AgentResponse {
                success: true,
                data: Some(data),
                error: None,
            };
            if let Ok(j) = serde_json::to_string(&res) {
                println!("{j}");
            }
        }
    }

    pub fn error(err: &str, json_mode: bool) {
        if json_mode {
            let res: AgentResponse<()> = AgentResponse {
                success: false,
                data: None,
                error: Some(err.to_string()),
            };
            if let Ok(j) = serde_json::to_string(&res) {
                eprintln!("{j}");
            }
        } else {
            eprintln!("[ERROR] {err}");
        }
    }
}

