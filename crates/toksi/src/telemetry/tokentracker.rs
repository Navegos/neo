// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/tokentracker.rs
// created: 2026-11-08
// lastModified: 2026-11-08

use std::fmt;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

use super::collector::try_send_telemetry_event;
use super::collector::TelemetryEvent;
use super::cost::{calculate_cost, calculate_savings};

#[derive(Debug)]
pub enum TokenTrackerError {
    InvalidSseChunk(String),
    JsonError(serde_json::Error),
    LockPoisoned,
}

impl fmt::Display for TokenTrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSseChunk(msg) => write!(f, "Invalid SSE chunk: {}", msg),
            Self::JsonError(e) => write!(f, "JSON parse error: {}", e),
            Self::LockPoisoned => write!(f, "Internal lock poisoned"),
        }
    }
}

impl std::error::Error for TokenTrackerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::JsonError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for TokenTrackerError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err)
    }
}

/// Token usage parsed from live SSE or completion payload
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedTokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: u64,
    pub saved_tokens: u64,
}

/// A ground-truth session token usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUsageRecord {
    pub session_id: String,
    pub agent: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub saved_tokens: u64,
    pub cost_usd: f64,
    pub saved_usd: f64,
    pub timestamp_epoch_ms: u64,
    pub duration_ms: u64,
}

/// In-memory ground-truth session token tracker engine
pub struct TokenTrackerEngine {
    sessions: RwLock<Vec<SessionUsageRecord>>,
    max_history: usize,
}

impl Default for TokenTrackerEngine {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl TokenTrackerEngine {
    pub fn new(max_history: usize) -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
            max_history,
        }
    }

    /// Records a ground-truth session usage record, dispatching metrics to the telemetry collector.
    pub fn record_usage(
        &self,
        session_id: &str,
        agent: &str,
        provider: &str,
        model: &str,
        tokens: ParsedTokenUsage,
        duration_ms: u64,
    ) -> Result<SessionUsageRecord, TokenTrackerError> {
        let cost_usd = calculate_cost(model, tokens.prompt_tokens, tokens.completion_tokens, tokens.cached_tokens);
        let saved_usd = calculate_savings(model, tokens.saved_tokens);

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let record = SessionUsageRecord {
            session_id: session_id.to_string(),
            agent: agent.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens: tokens.prompt_tokens,
            output_tokens: tokens.completion_tokens,
            cached_tokens: tokens.cached_tokens,
            saved_tokens: tokens.saved_tokens,
            cost_usd,
            saved_usd,
            timestamp_epoch_ms: now_ms,
            duration_ms,
        };

        // Dispatch telemetry event asynchronously
        try_send_telemetry_event(TelemetryEvent::UpstreamRequest {
            provider: provider.to_string(),
            model: model.to_string(),
            status_code: 200,
            error_type: None,
            latency_secs: duration_ms as f64 / 1000.0,
            input_tokens: tokens.prompt_tokens,
            output_tokens: tokens.completion_tokens,
            cached_tokens: tokens.cached_tokens,
            tokens_saved: tokens.saved_tokens,
            agent: agent.to_string(),
            virtual_key: "default".to_string(),
        });

        let mut lock = self.sessions.write().map_err(|_| TokenTrackerError::LockPoisoned)?;
        if lock.len() >= self.max_history {
            lock.remove(0);
        }
        lock.push(record.clone());

        Ok(record)
    }

    /// Returns a snapshot of recent session records
    pub fn get_recent_sessions(&self) -> Vec<SessionUsageRecord> {
        self.sessions
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Aggregates lifetime ground-truth statistics across recorded sessions.
    /// When idle (zero sessions recorded), all totals return exactly 0.
    pub fn lifetime_summary(&self) -> (u64, u64, u64, u64, f64, f64) {
        let lock = match self.sessions.read() {
            Ok(l) => l,
            Err(_) => return (0, 0, 0, 0, 0.0, 0.0),
        };

        let mut prompt = 0u64;
        let mut completion = 0u64;
        let mut cached = 0u64;
        let mut saved = 0u64;
        let mut cost = 0.0;
        let mut savings = 0.0;

        for r in lock.iter() {
            prompt = prompt.saturating_add(r.input_tokens);
            completion = completion.saturating_add(r.output_tokens);
            cached = cached.saturating_add(r.cached_tokens);
            saved = saved.saturating_add(r.saved_tokens);
            cost += r.cost_usd;
            savings += r.saved_usd;
        }

        (prompt, completion, cached, saved, cost, savings)
    }
}

/// Parses tokens from an OpenAI SSE stream line or JSON payload
pub fn parse_openai_sse_chunk(line: &str) -> Option<ParsedTokenUsage> {
    let payload = line.strip_prefix("data:")?.trim();
    if payload == "[DONE]" || payload.is_empty() {
        return None;
    }

    let val: serde_json::Value = serde_json::from_str(payload).ok()?;
    let usage = val.get("usage")?;

    let prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let completion_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let cached_tokens = usage
        .get("prompt_tokens_details")
        .and_then(|d| d.get("cached_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if prompt_tokens == 0 && completion_tokens == 0 && cached_tokens == 0 {
        return None;
    }

    Some(ParsedTokenUsage {
        prompt_tokens,
        completion_tokens,
        cached_tokens,
        saved_tokens: 0,
    })
}

/// Parses tokens from an Anthropic SSE stream line or JSON payload
pub fn parse_anthropic_sse_chunk(line: &str) -> Option<ParsedTokenUsage> {
    let payload = line.strip_prefix("data:")?.trim();
    if payload.is_empty() {
        return None;
    }

    let val: serde_json::Value = serde_json::from_str(payload).ok()?;
    let event_type = val.get("type").and_then(|v| v.as_str())?;

    match event_type {
        "message_start" => {
            let msg = val.get("message")?;
            let usage = msg.get("usage")?;
            let prompt_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let cached_tokens = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Some(ParsedTokenUsage {
                prompt_tokens,
                completion_tokens: 0,
                cached_tokens,
                saved_tokens: 0,
            })
        }
        "message_delta" => {
            let usage = val.get("usage")?;
            let completion_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            Some(ParsedTokenUsage {
                prompt_tokens: 0,
                completion_tokens,
                cached_tokens: 0,
                saved_tokens: 0,
            })
        }
        _ => None,
    }
}

/// Parses tokens from a Google Gemini response or SSE chunk
pub fn parse_gemini_chunk(json_str: &str) -> Option<ParsedTokenUsage> {
    let val: serde_json::Value = serde_json::from_str(json_str.trim()).ok()?;
    let meta = val.get("usageMetadata")?;

    let prompt_tokens = meta.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);
    let completion_tokens = meta.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);
    let cached_tokens = meta.get("cachedContentTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);

    if prompt_tokens == 0 && completion_tokens == 0 && cached_tokens == 0 {
        return None;
    }

    Some(ParsedTokenUsage {
        prompt_tokens,
        completion_tokens,
        cached_tokens,
        saved_tokens: 0,
    })
}
