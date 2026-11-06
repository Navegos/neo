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
    Anthropic,
    Gemini,
}

pub fn detect_sdk_protocol(path: &str) -> Option<SdkProtocol> {
    let p = path.to_lowercase();
    if p.contains("/v1/chat/completions") || p.starts_with("/openai") {
        Some(SdkProtocol::OpenAI)
    } else if p.contains("/v1/messages") || p.starts_with("/anthropic") {
        Some(SdkProtocol::Anthropic)
    } else if p.contains("/v1beta/models") || p.starts_with("/gemini") {
        Some(SdkProtocol::Gemini)
    } else {
        None
    }
}

/// Normalizes SDK requests according to target provider protocol
pub fn adapt_sdk_payload(protocol: SdkProtocol, target_protocol: SdkProtocol, body: &Value) -> Value {
    if protocol == target_protocol {
        return body.clone();
    }
    match (protocol, target_protocol) {
        (SdkProtocol::OpenAI, SdkProtocol::Anthropic) => openai_to_anthropic(body),
        (SdkProtocol::Anthropic, SdkProtocol::OpenAI) => anthropic_to_openai(body),
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
