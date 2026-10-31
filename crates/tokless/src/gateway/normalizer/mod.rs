// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/normalizer/mod.rs

use serde_json::{json, Value};
use crate::gateway::providers::UnifiedRequest;

/// Translates OpenAI ChatCompletions format to Anthropic Messages format
pub fn openai_to_anthropic(req: &Value) -> Value {
    let model = req.get("model").and_then(Value::as_str).unwrap_or("claude-3-5-sonnet");
    let max_tokens = req.get("max_tokens").and_then(Value::as_u64).unwrap_or(4096);

    let mut system_prompt = None;
    let mut anthropic_messages = Vec::new();

    if let Some(msgs) = req.get("messages").and_then(Value::as_array) {
        for m in msgs {
            let role = m.get("role").and_then(Value::as_str).unwrap_or("user");
            let content = m.get("content").cloned().unwrap_or(json!(""));

            if role == "system" {
                if let Some(s) = content.as_str() {
                    system_prompt = Some(s.to_string());
                }
            } else {
                anthropic_messages.push(json!({
                    "role": if role == "assistant" { "assistant" } else { "user" },
                    "content": content
                }));
            }
        }
    }

    let mut out = json!({
        "model": model,
        "messages": anthropic_messages,
        "max_tokens": max_tokens,
    });

    if let Some(sys) = system_prompt {
        out["system"] = json!(sys);
    }
    if let Some(tools) = req.get("tools") {
        out["tools"] = tools.clone();
    }
    out
}

/// Translates Anthropic Messages format to OpenAI ChatCompletions format
pub fn anthropic_to_openai(req: &Value) -> Value {
    let model = req.get("model").and_then(Value::as_str).unwrap_or("gpt-4o");
    let mut msgs = Vec::new();

    if let Some(sys) = req.get("system").and_then(Value::as_str) {
        msgs.push(json!({ "role": "system", "content": sys }));
    }

    if let Some(m_arr) = req.get("messages").and_then(Value::as_array) {
        msgs.extend(m_arr.clone());
    }

    let mut out = json!({
        "model": model,
        "messages": msgs,
    });

    if let Some(max_tokens) = req.get("max_tokens") {
        out["max_tokens"] = max_tokens.clone();
    }
    if let Some(temp) = req.get("temperature") {
        out["temperature"] = temp.clone();
    }
    if let Some(tools) = req.get("tools") {
        out["tools"] = tools.clone();
    }
    out
}

/// Translates Gemini format to UnifiedRequest
pub fn gemini_to_unified(req: &Value) -> UnifiedRequest {
    let mut messages = Vec::new();

    if let Some(contents) = req.get("contents").and_then(Value::as_array) {
        for c in contents {
            let role = c.get("role").and_then(Value::as_str).unwrap_or("user");
            let mut text_acc = String::new();
            if let Some(parts) = c.get("parts").and_then(Value::as_array) {
                for p in parts {
                    if let Some(t) = p.get("text").and_then(Value::as_str) {
                        text_acc.push_str(t);
                    }
                }
            }
            messages.push(json!({
                "role": if role == "model" { "assistant" } else { "user" },
                "content": text_acc
            }));
        }
    }

    UnifiedRequest {
        model: "gemini-2.0-flash".to_string(),
        messages,
        system: None,
        max_tokens: None,
        temperature: None,
        stream: None,
        tools: None,
        extra: std::collections::HashMap::new(),
    }
}

