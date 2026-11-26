// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/inference_bridge.rs
// created: 2026-11-21

//! `olla` zero-cost local node health probing and pure-Rust OpenAI/Responses/multimodal
//! conformance parsers (`Switchyard`, `NeMo-Fabric`, `NeMo-Relay`, `physicsnemo-serve`, `frontend-crates`).

#![deny(unsafe_code)]

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeHealthStatus {
    pub node_id: String,
    pub endpoint: String,
    pub healthy: bool,
    pub models_discovered: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedInferenceRequest {
    pub model: String,
    pub prompt_text: String,
    pub has_multimodal_parts: bool,
    pub stream: bool,
    pub wire_format: String,
}

/// Zero-cost local node health probe (`olla` parity) hitting `/v1/models` or `/health` without inference tokens.
pub async fn probe_local_node_health(node_id: &str, base_url: &str) -> NodeHealthStatus {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let trimmed = base_url.trim_end_matches('/');
    let models_url = format!("{trimmed}/v1/models");
    if let Ok(resp) = client.get(&models_url).send().await
        && resp.status().is_success() {
            let count = resp
                .json::<Value>()
                .await
                .ok()
                .and_then(|v| v.get("data").and_then(Value::as_array).map(|a| a.len()))
                .unwrap_or(0);
            return NodeHealthStatus {
                node_id: node_id.to_string(),
                endpoint: trimmed.to_string(),
                healthy: true,
                models_discovered: count,
            };
        }
    NodeHealthStatus {
        node_id: node_id.to_string(),
        endpoint: trimmed.to_string(),
        healthy: false,
        models_discovered: 0,
    }
}

/// Pure-Rust OpenAI Chat Completions (`/v1/chat/completions`), Responses API (`/v1/responses`),
/// and multimodal content conformance parser (`frontend-crates` / `NeMo-Relay` parity).
pub fn parse_conformance_request(payload: &Value) -> Result<NormalizedInferenceRequest, String> {
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .filter(|m| !m.trim().is_empty())
        .ok_or_else(|| "Missing required 'model' field".to_string())?
        .to_string();
    let stream = payload
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if let Some(messages) = payload.get("messages").and_then(Value::as_array) {
        let mut parts = Vec::new();
        let mut multimodal = false;
        for msg in messages {
            match msg.get("content") {
                Some(Value::String(s)) => parts.push(s.clone()),
                Some(Value::Array(arr)) => {
                    multimodal = true;
                    for item in arr {
                        if let Some(t) = item.get("text").and_then(Value::as_str) {
                            parts.push(t.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        return Ok(NormalizedInferenceRequest {
            model,
            prompt_text: parts.join("\n"),
            has_multimodal_parts: multimodal,
            stream,
            wire_format: "openai.chat.completions".to_string(),
        });
    }

    if let Some(input) = payload.get("input") {
        let (prompt_text, multimodal) = match input {
            Value::String(s) => (s.clone(), false),
            Value::Array(arr) => {
                let mut buf = Vec::new();
                for item in arr {
                    if let Some(s) = item.as_str() {
                        buf.push(s.to_string());
                    } else if let Some(s) = item.get("text").and_then(Value::as_str) {
                        buf.push(s.to_string());
                    }
                }
                (buf.join("\n"), true)
            }
            _ => (input.to_string(), false),
        };
        return Ok(NormalizedInferenceRequest {
            model,
            prompt_text,
            has_multimodal_parts: multimodal,
            stream,
            wire_format: "openai.responses".to_string(),
        });
    }

    Err("Payload must contain 'messages' or 'input'".to_string())
}
