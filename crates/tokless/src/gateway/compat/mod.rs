// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/compat/mod.rs

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
