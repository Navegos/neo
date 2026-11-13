// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/compat/mod.rs
// created: 2026-10-31
// lastModified: 2026-10-31

use serde_json::Value;
use crate::gateway::normalizer::{anthropic_to_openai, openai_to_anthropic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdkProtocol {
    OpenAI,
    OpenAIResponses,
    OpenAIEmbeddings,
    Models,
    Anthropic,
    Gemini,
}

pub fn detect_sdk_protocol(path: &str) -> Option<SdkProtocol> {
    let p = path.to_lowercase();
    if p.contains("/v1/chat/completions") || p.starts_with("/openai") {
        Some(SdkProtocol::OpenAI)
    } else if p.contains("/v1/responses") {
        Some(SdkProtocol::OpenAIResponses)
    } else if p.contains("/v1/embeddings") {
        Some(SdkProtocol::OpenAIEmbeddings)
    } else if p == "/v1/models" || p.starts_with("/v1/models?") {
        Some(SdkProtocol::Models)
    } else if p.contains("/v1/messages") || p.starts_with("/anthropic") {
        Some(SdkProtocol::Anthropic)
    } else if p.contains("/v1beta/models") || p.starts_with("/gemini") {
        Some(SdkProtocol::Gemini)
    } else {
        None
    }
}

/// Convert an OpenAI `/v1/responses` request payload into canonical chat completions JSON.
pub fn responses_to_chat_completions(body: &Value) -> Value {
    let model = body.get("model").and_then(Value::as_str).unwrap_or("gpt-4o");
    let mut messages: Vec<Value> = Vec::new();
    if let Some(instructions) = body.get("instructions").and_then(Value::as_str) {
        if !instructions.trim().is_empty() {
            messages.push(serde_json::json!({"role": "system", "content": instructions}));
        }
    }
    match body.get("input") {
        Some(Value::String(s)) => {
            messages.push(serde_json::json!({"role": "user", "content": s}));
        }
        Some(Value::Array(items)) => {
            for item in items {
                if let Some(role) = item.get("role").and_then(Value::as_str) {
                    let content = item.get("content").cloned().unwrap_or_else(|| Value::String(String::new()));
                    messages.push(serde_json::json!({"role": role, "content": content}));
                } else if let Some(text) = item.as_str() {
                    messages.push(serde_json::json!({"role": "user", "content": text}));
                }
            }
        }
        _ => {
            if let Some(existing) = body.get("messages").and_then(Value::as_array) {
                messages.extend(existing.iter().cloned());
            }
        }
    }
    let mut out = serde_json::json!({
        "model": model,
        "messages": messages,
    });
    if let Some(max_toks) = body.get("max_output_tokens").or_else(|| body.get("max_tokens")) {
        out["max_tokens"] = max_toks.clone();
    }
    if let Some(stream) = body.get("stream") {
        out["stream"] = stream.clone();
    }
    if let Some(temp) = body.get("temperature") {
        out["temperature"] = temp.clone();
    }
    out
}

/// Format an OpenAI chat completion payload into `/v1/responses` response JSON.
pub fn chat_to_responses_json(resp: &Value, fallback_model: &str) -> Value {
    let id = resp
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("resp_toksi")
        .to_string();
    let model = resp
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(fallback_model);
    let content = resp
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let (input_tokens, output_tokens) = crate::gateway::providers::parse_openai_usage(resp);

    serde_json::json!({
        "id": id,
        "object": "response",
        "model": model,
        "output": [{
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": content
            }]
        }],
        "output_text": content,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens
        }
    })
}

/// Normalizes SDK requests according to target provider protocol
pub fn adapt_sdk_payload(protocol: SdkProtocol, target_protocol: SdkProtocol, body: &Value) -> Value {
    if protocol == target_protocol {
        return body.clone();
    }
    match (protocol, target_protocol) {
        (SdkProtocol::OpenAI, SdkProtocol::Anthropic) => openai_to_anthropic(body),
        (SdkProtocol::Anthropic, SdkProtocol::OpenAI) => anthropic_to_openai(body),
        (SdkProtocol::OpenAIResponses, SdkProtocol::OpenAI) => responses_to_chat_completions(body),
        (SdkProtocol::OpenAIResponses, SdkProtocol::Anthropic) => {
            openai_to_anthropic(&responses_to_chat_completions(body))
        }
        _ => body.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_sdk_protocol() {
        assert_eq!(detect_sdk_protocol("/v1/chat/completions"), Some(SdkProtocol::OpenAI));
        assert_eq!(detect_sdk_protocol("/openai/deployments/gpt-4o"), Some(SdkProtocol::OpenAI));
        assert_eq!(detect_sdk_protocol("/v1/messages"), Some(SdkProtocol::Anthropic));
        assert_eq!(detect_sdk_protocol("/anthropic/v1/messages"), Some(SdkProtocol::Anthropic));
        assert_eq!(detect_sdk_protocol("/v1beta/models/gemini-2.0-flash:streamGenerateContent"), Some(SdkProtocol::Gemini));
        assert_eq!(detect_sdk_protocol("/unknown/path"), None);
    }

    #[test]
    fn test_adapt_sdk_payload_same_protocol() {
        let payload = json!({"model": "gpt-4o", "messages": []});
        let adapted = adapt_sdk_payload(SdkProtocol::OpenAI, SdkProtocol::OpenAI, &payload);
        assert_eq!(adapted, payload);
    }

    #[test]
    fn test_adapt_sdk_payload_cross_conversion() {
        let openai_req = json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "Be concise"},
                {"role": "user", "content": "Hello"}
            ]
        });

        let anthropic_adapted = adapt_sdk_payload(SdkProtocol::OpenAI, SdkProtocol::Anthropic, &openai_req);
        assert_eq!(anthropic_adapted["system"], "Be concise");
        assert_eq!(anthropic_adapted["messages"][0]["role"], "user");
    }
}
