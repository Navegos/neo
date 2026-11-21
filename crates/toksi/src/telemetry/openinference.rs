// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/openinference.rs
// created: 2026-11-21

//! Arize Phoenix OpenInference span schema & attribution helpers.

#![deny(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OpenInferenceSpanKind {
    Llm,
    Chain,
    Retriever,
    Embedding,
    Tool,
    Agent,
    Guardrail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenInferenceSpan {
    pub trace_id: String,
    pub span_id: String,
    pub kind: OpenInferenceSpanKind,
    pub name: String,
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub start_time_ns: u64,
    pub end_time_ns: u64,
}

impl OpenInferenceSpan {
    pub fn llm(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        model_name: impl Into<String>,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Self {
        let mut attributes = BTreeMap::new();
        attributes.insert("openinference.span.kind".to_string(), serde_json::json!("LLM"));
        attributes.insert("llm.model_name".to_string(), serde_json::json!(model_name.into()));
        attributes.insert("llm.token_count.prompt".to_string(), serde_json::json!(prompt_tokens));
        attributes.insert("llm.token_count.completion".to_string(), serde_json::json!(completion_tokens));
        attributes.insert(
            "llm.token_count.total".to_string(),
            serde_json::json!(prompt_tokens + completion_tokens),
        );
        let now_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).max(0) as u64;
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            kind: OpenInferenceSpanKind::Llm,
            name: "llm.completion".to_string(),
            attributes,
            start_time_ns: now_ns,
            end_time_ns: now_ns,
        }
    }
}
