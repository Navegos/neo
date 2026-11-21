// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/acp.rs
// created: 2026-11-21

//! JSON-RPC 2.0 Agent Client Protocol (`agent-client-protocol`) served on `:9098/acp`
//! with shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) payload compression.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::headroom::effort::{EffortLevel, HeadroomEffortCompressor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpJsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpJsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// Dispatches an ACP JSON-RPC 2.0 request (`initialize`, `session/new`, `session/prompt`, `session/cancel`)
/// and compresses prompt payloads through `HeadroomEffortCompressor`.
pub fn handle_acp_jsonrpc(raw: &Value, effort: EffortLevel) -> AcpJsonRpcResponse {
    let id = raw.get("id").cloned().unwrap_or(Value::Null);
    let method = raw
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("");
    let params = raw.get("params").cloned().unwrap_or_else(|| json!({}));

    match method {
        "initialize" => AcpJsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": 1,
                "agentInfo": {
                    "name": "toksi-siagent",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {
                    "loadSession": true,
                    "promptCapabilities": { "embeddedContext": true, "image": false },
                    "mcpCapabilities": { "http": true, "sse": true }
                }
            })),
            error: None,
        },
        "session/new" => {
            let sid = format!("acp-{}", chrono::Utc::now().timestamp_millis());
            AcpJsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "sessionId": sid })),
                error: None,
            }
        }
        "session/prompt" => {
            let mut body = json!({
                "model": "toksi-siagent",
                "messages": [{
                    "role": "user",
                    "content": params.get("prompt").cloned().unwrap_or_else(|| json!(""))
                }]
            });
            let compressor = HeadroomEffortCompressor::gateway_default();
            let headers = vec![("x-toksi-effort".to_string(), effort.as_str().to_string())];
            let stats = compressor.compress_json_payload("/acp", &headers, &mut body);
            AcpJsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "stopReason": "end_turn",
                    "effort": effort.as_str(),
                    "headroom": {
                        "original_bytes": stats.original_bytes,
                        "compressed_bytes": stats.compressed_bytes,
                        "bytes_saved": stats.bytes_saved,
                    }
                })),
                error: None,
            }
        }
        "session/cancel" => AcpJsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "cancelled": true })),
            error: None,
        },
        other => AcpJsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": format!("Unsupported ACP method: {other}")
            })),
        },
    }
}
