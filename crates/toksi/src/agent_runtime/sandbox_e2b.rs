// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/sandbox_e2b.rs
// created: 2026-11-21

//! `E2B` / `E2B-runtime` Firecracker `envd` microVM lifecycle, template definitions,
//! and isolated filesystem/process/code-interpreter RPC contracts.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2bTemplateSpec {
    pub template_id: String,
    pub vcpu_count: u8,
    pub memory_mb: u32,
    pub envd_port: u16,
    pub network_isolated: bool,
}

impl Default for E2bTemplateSpec {
    fn default() -> Self {
        Self {
            template_id: "toksi-firecracker-base".to_string(),
            vcpu_count: 2,
            memory_mb: 1024,
            envd_port: 49983,
            network_isolated: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2bMicroVmSession {
    pub sandbox_id: String,
    pub template_id: String,
    pub envd_endpoint: String,
    pub state: String,
    pub created_at: i64,
    pub ttl_secs: u64,
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "rpc", rename_all = "snake_case")]
pub enum EnvdRpcRequest {
    ProcessStart { cmd: String, args: Vec<String>, cwd: String },
    FsRead { path: String },
    FsWrite { path: String, content_b64: String },
    CodeInterpret { language: String, code: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvdRpcResponse {
    pub ok: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn create_e2b_microvm_session(
    sandbox_id: &str,
    template: &E2bTemplateSpec,
    ttl_secs: u64,
) -> E2bMicroVmSession {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    E2bMicroVmSession {
        sandbox_id: sandbox_id.to_string(),
        template_id: template.template_id.clone(),
        envd_endpoint: format!("http://127.0.0.1:{}", template.envd_port),
        state: "running".to_string(),
        created_at: now,
        ttl_secs,
        env_vars: HashMap::new(),
    }
}

pub fn build_envd_rpc_payload(req: &EnvdRpcRequest) -> String {
    serde_json::to_string(req).unwrap_or_else(|_| "{}".to_string())
}
