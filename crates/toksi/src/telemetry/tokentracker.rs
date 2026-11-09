// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/tokentracker.rs
// created: 2026-11-08
// lastModified: 2026-11-09

use std::fmt;
use std::sync::RwLock;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use super::collector::try_send_telemetry_event;
use super::collector::TelemetryEvent;
use super::cost::{calculate_cost, calculate_savings};

#[derive(Debug)]
pub enum TokenTrackerError {
    InvalidSseChunk(String),
    JsonError(serde_json::Error),
    LockPoisoned,
    NetworkError(String),
}

impl fmt::Display for TokenTrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSseChunk(msg) => write!(f, "Invalid SSE chunk: {}", msg),
            Self::JsonError(e) => write!(f, "JSON parse error: {}", e),
            Self::LockPoisoned => write!(f, "Internal lock poisoned"),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
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

/// Token usage parsed from live SSE or completion payload.
/// When idle, all counters strictly read 0 (strict prohibition of synthetic/virtual tokens).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedTokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: u64,
    pub saved_tokens: u64,
}

/// A ground-truth session token usage record.
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

/// Rate limits and quotas extracted from upstream HTTP response headers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRateLimits {
    pub remaining_tokens: Option<u64>,
    pub remaining_requests: Option<u64>,
    pub limit_tokens: Option<u64>,
    pub limit_requests: Option<u64>,
    pub reset_tokens: Option<String>,
    pub reset_requests: Option<String>,
    pub retry_after_secs: Option<u64>,
}

/// Headroom token savings report capturing exact byte reduction and pruned schemas.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadroomSavingsReport {
    pub raw_bytes: usize,
    pub compressed_bytes: usize,
    pub byte_reduction: usize,
    pub pruned_tool_schemas: usize,
    pub tokens_saved: u64,
}

/// Status of provider quota queried via free agent AI API channels without token costs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderQuotaStatus {
    pub provider: String,
    pub is_available: bool,
    pub rate_limits: ProviderRateLimits,
    pub message: String,
    pub queried_epoch_ms: u64,
}

/// In-memory ground-truth session token tracker engine.
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

    /// Returns a snapshot of recent session records.
    pub fn get_recent_sessions(&self) -> Vec<SessionUsageRecord> {
        self.sessions
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Aggregates lifetime ground-truth statistics across recorded sessions.
    /// When idle (zero sessions recorded), all totals strictly return exactly 0.
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

// ---------------------------------------------------------------------------
// Content Type Aware Byte-to-Token Ratio Calculators
// ---------------------------------------------------------------------------

/// Content type categorization for refined byte-to-token ratio estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Prose,
    Code,
    Json,
    Cjk,
    Markdown,
}

impl ContentType {
    /// Nominal average bytes per token for typical model tokenizers (tiktoken, sentencepiece).
    pub fn bytes_per_token(&self) -> f64 {
        match self {
            Self::Prose => 4.0,
            Self::Code => 3.2,
            Self::Json => 2.8,
            Self::Cjk => 1.5,
            Self::Markdown => 3.8,
        }
    }
}

/// Detects the probable content type based on character distribution.
pub fn detect_content_type(sample: &str) -> ContentType {
    if sample.is_empty() {
        return ContentType::Prose;
    }

    let trimmed = sample.trim();
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        return ContentType::Json;
    }

    let mut cjk_count = 0usize;
    let mut json_indicator = 0usize;
    let mut code_indicator = 0usize;
    let total_chars = sample.chars().count();

    for c in sample.chars() {
        if ('\u{4e00}'..='\u{9fff}').contains(&c)
            || ('\u{3040}'..='\u{30ff}').contains(&c)
            || ('\u{ac00}'..='\u{d7af}').contains(&c)
        {
            cjk_count += 1;
        } else if matches!(c, '{' | '}' | '[' | ']' | '"' | ':') {
            json_indicator += 1;
        } else if matches!(c, ';' | '(' | ')' | '=' | '<' | '>' | '&' | '|' | '`') {
            code_indicator += 1;
        }
    }

    let code_keywords = [
        "fn ", "def ", "class ", "function", "pub ", "struct ", "impl ", "let ",
        "var ", "val ", "const ", "import ", "export ", "return ", "package ",
    ];
    let has_code_keyword = code_keywords.iter().any(|k| sample.contains(k));

    if cjk_count * 10 >= total_chars * 2 {
        ContentType::Cjk
    } else if json_indicator * 10 >= total_chars * 3 && (trimmed.starts_with('{') || trimmed.starts_with('[')) {
        ContentType::Json
    } else if has_code_keyword || code_indicator * 10 >= total_chars * 15 / 10 {
        ContentType::Code
    } else if sample.contains("# ") || sample.contains("```") || sample.contains("- ") {
        ContentType::Markdown
    } else {
        ContentType::Prose
    }
}

/// Calculates token count from byte size according to specific content type ratio.
pub fn calculate_tokens_from_bytes(bytes: usize, content_type: ContentType) -> u64 {
    let ratio = content_type.bytes_per_token();
    if ratio <= 0.0 {
        return (bytes / BYTES_PER_TOKEN) as u64;
    }
    ((bytes as f64) / ratio).round() as u64
}

/// Known agent plan quotas and limits for popular coding assistants and providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPlanTier {
    ClaudePro,
    ClaudeTeam,
    OpenAiPlus,
    OpenAiTeam,
    CursorPro,
    CursorBusiness,
    CodexPaid,
    DeepSeekPaid,
    Custom { name: String, limit_requests: u64, window_secs: u64 },
}

impl AgentPlanTier {
    pub fn window_duration_secs(&self) -> u64 {
        match self {
            Self::ClaudePro | Self::ClaudeTeam => 5 * 3600, // 5 hours rolling
            Self::OpenAiPlus | Self::OpenAiTeam => 3 * 3600, // 3 hours rolling
            Self::CursorPro | Self::CursorBusiness => 30 * 86400, // monthly
            Self::CodexPaid => 86400, // daily
            Self::DeepSeekPaid => 60, // per minute TPM/RPM
            Self::Custom { window_secs, .. } => *window_secs,
        }
    }

    pub fn nominal_request_limit(&self) -> u64 {
        match self {
            Self::ClaudePro => 45,
            Self::ClaudeTeam => 150,
            Self::OpenAiPlus => 40,
            Self::OpenAiTeam => 100,
            Self::CursorPro => 500,
            Self::CursorBusiness => 2000,
            Self::CodexPaid => 1000,
            Self::DeepSeekPaid => 60,
            Self::Custom { limit_requests, .. } => *limit_requests,
        }
    }
}

/// Tracks rolling quota consumption for agent subscription plans.
#[derive(Debug, Clone)]
pub struct RollingQuotaTracker {
    pub plan: AgentPlanTier,
    pub request_timestamps: Vec<u64>, // epoch seconds
    pub token_history: Vec<(u64, u64)>, // (epoch seconds, tokens consumed)
}

impl RollingQuotaTracker {
    pub fn new(plan: AgentPlanTier) -> Self {
        Self {
            plan,
            request_timestamps: Vec::new(),
            token_history: Vec::new(),
        }
    }

    /// Records a request invocation and its tokens.
    pub fn record_turn(&mut self, now_epoch_secs: u64, tokens: u64) {
        self.request_timestamps.push(now_epoch_secs);
        self.token_history.push((now_epoch_secs, tokens));
        self.prune(now_epoch_secs);
    }

    /// Prunes timestamps older than the plan's rolling window.
    pub fn prune(&mut self, now_epoch_secs: u64) {
        let window = self.plan.window_duration_secs();
        let cutoff = now_epoch_secs.saturating_sub(window);
        self.request_timestamps.retain(|&ts| ts >= cutoff);
        self.token_history.retain(|&(ts, _)| ts >= cutoff);
    }

    /// Returns the number of requests used in the current rolling window.
    pub fn current_window_requests(&self, now_epoch_secs: u64) -> usize {
        let window = self.plan.window_duration_secs();
        let cutoff = now_epoch_secs.saturating_sub(window);
        self.request_timestamps.iter().filter(|&&ts| ts >= cutoff).count()
    }

    /// Returns the total tokens consumed in the current rolling window.
    pub fn current_window_tokens(&self, now_epoch_secs: u64) -> u64 {
        let window = self.plan.window_duration_secs();
        let cutoff = now_epoch_secs.saturating_sub(window);
        self.token_history
            .iter()
            .filter(|&&(ts, _)| ts >= cutoff)
            .map(|&(_, tok)| tok)
            .sum()
    }

    /// Evaluates quota consumption status: (used_requests, limit_requests, percentage_consumed).
    pub fn consumption_ratio(&self, now_epoch_secs: u64) -> (usize, u64, f64) {
        let used = self.current_window_requests(now_epoch_secs);
        let limit = self.plan.nominal_request_limit();
        let ratio = if limit > 0 {
            (used as f64) / (limit as f64)
        } else {
            0.0
        };
        (used, limit, (ratio * 100.0).min(100.0))
    }
}

// ---------------------------------------------------------------------------
// Rate Limits Extraction from Upstream HTTP Response Headers
// ---------------------------------------------------------------------------

/// Extracts upstream provider rate limits and quotas from HTTP response headers.
/// Inspects headers case-insensitively for OpenAI, Anthropic, Google, and standard RFC headers.
pub fn extract_provider_rate_limits(headers: &[(String, String)]) -> ProviderRateLimits {
    let mut limits = ProviderRateLimits::default();

    for (k, v) in headers {
        let key = k.to_ascii_lowercase();
        let val = v.trim();

        match key.as_str() {
            "x-ratelimit-remaining-tokens"
            | "x-ratelimit-remaining-tokens-requests"
            | "anthropic-ratelimit-tokens-remaining"
            | "anthropic-ratelimit-input-tokens-remaining" => {
                if let Ok(num) = val.parse::<u64>() {
                    limits.remaining_tokens = Some(num);
                }
            }
            "x-ratelimit-remaining-requests"
            | "x-ratelimit-remaining"
            | "anthropic-ratelimit-requests-remaining" => {
                if let Ok(num) = val.parse::<u64>() {
                    limits.remaining_requests = Some(num);
                }
            }
            "x-ratelimit-limit-tokens"
            | "anthropic-ratelimit-tokens-limit"
            | "anthropic-ratelimit-input-tokens-limit" => {
                if let Ok(num) = val.parse::<u64>() {
                    limits.limit_tokens = Some(num);
                }
            }
            "x-ratelimit-limit-requests" | "anthropic-ratelimit-requests-limit" => {
                if let Ok(num) = val.parse::<u64>() {
                    limits.limit_requests = Some(num);
                }
            }
            "x-ratelimit-reset-tokens" | "anthropic-ratelimit-tokens-reset" => {
                if !val.is_empty() {
                    limits.reset_tokens = Some(val.to_string());
                }
            }
            "x-ratelimit-reset-requests" | "anthropic-ratelimit-requests-reset" => {
                if !val.is_empty() {
                    limits.reset_requests = Some(val.to_string());
                }
            }
            "retry-after" => {
                if let Ok(secs) = val.parse::<u64>() {
                    limits.retry_after_secs = Some(secs);
                }
            }
            _ => {}
        }
    }

    limits
}

// ---------------------------------------------------------------------------
// True Headroom Token Savings Calculations
// ---------------------------------------------------------------------------

/// Average token ratio constant: ~4 bytes per token for typical UTF-8 code/prose.
pub const BYTES_PER_TOKEN: usize = 4;

/// Average estimated tokens saved per pruned MCP tool schema definition.
pub const AVERAGE_TOKENS_PER_PRUNED_TOOL: u64 = 250;

/// Calculates byte reduction and corresponding token savings.
pub fn calculate_byte_savings(raw_bytes: usize, compressed_bytes: usize) -> (usize, u64) {
    let byte_reduction = raw_bytes.saturating_sub(compressed_bytes);
    let tokens_saved = (byte_reduction / BYTES_PER_TOKEN) as u64;
    (byte_reduction, tokens_saved)
}

/// Calculates exact token savings from pruned MCP tool schemas.
pub fn calculate_pruned_schema_savings(pruned_tool_count: usize, tokens_per_tool: u64) -> u64 {
    (pruned_tool_count as u64).saturating_mul(tokens_per_tool)
}

/// Generates a comprehensive Headroom savings report combining byte reduction and pruned schemas.
pub fn calculate_headroom_savings(
    raw_bytes: usize,
    compressed_bytes: usize,
    pruned_tool_schemas: usize,
) -> HeadroomSavingsReport {
    let (byte_reduction, byte_tokens) = calculate_byte_savings(raw_bytes, compressed_bytes);
    let schema_tokens = calculate_pruned_schema_savings(pruned_tool_schemas, AVERAGE_TOKENS_PER_PRUNED_TOOL);
    let tokens_saved = byte_tokens.saturating_add(schema_tokens);

    HeadroomSavingsReport {
        raw_bytes,
        compressed_bytes,
        byte_reduction,
        pruned_tool_schemas,
        tokens_saved,
    }
}

// ---------------------------------------------------------------------------
// Live SSE Stream Token Parsers
// ---------------------------------------------------------------------------

/// Parses tokens from an OpenAI SSE stream line or JSON payload.
pub fn parse_openai_sse_chunk(line: &str) -> Option<ParsedTokenUsage> {
    let payload = if let Some(stripped) = line.strip_prefix("data:") {
        stripped.trim()
    } else {
        line.trim()
    };

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

/// Parses tokens from an Anthropic SSE stream line or JSON payload.
pub fn parse_anthropic_sse_chunk(line: &str) -> Option<ParsedTokenUsage> {
    let payload = if let Some(stripped) = line.strip_prefix("data:") {
        stripped.trim()
    } else {
        line.trim()
    };

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
        "message" => {
            let usage = val.get("usage")?;
            let prompt_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let completion_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let cached_tokens = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Some(ParsedTokenUsage {
                prompt_tokens,
                completion_tokens,
                cached_tokens,
                saved_tokens: 0,
            })
        }
        _ => None,
    }
}

/// Parses tokens from a Google Gemini response or SSE chunk.
pub fn parse_gemini_chunk(json_str: &str) -> Option<ParsedTokenUsage> {
    let payload = if let Some(stripped) = json_str.trim().strip_prefix("data:") {
        stripped.trim()
    } else {
        json_str.trim()
    };

    if payload.is_empty() {
        return None;
    }

    let val: serde_json::Value = serde_json::from_str(payload).ok()?;
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

/// Streaming SSE token accumulator that processes incoming stream lines on-the-fly
/// without buffering the entire response payload in memory.
#[derive(Debug, Clone, Default)]
pub struct SseStreamTokenParser {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: u64,
    pub saved_tokens: u64,
}

impl SseStreamTokenParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests a single chunk line and updates accumulated ground-truth tokens.
    /// Returns the parsed delta if tokens were detected in the chunk.
    pub fn feed_line(&mut self, line: &str) -> Option<ParsedTokenUsage> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "data: [DONE]" {
            return None;
        }

        // Try OpenAI format first
        if let Some(usage) = parse_openai_sse_chunk(trimmed) {
            if usage.prompt_tokens > 0 {
                self.prompt_tokens = usage.prompt_tokens;
            }
            if usage.completion_tokens > 0 {
                self.completion_tokens = usage.completion_tokens;
            }
            if usage.cached_tokens > 0 {
                self.cached_tokens = usage.cached_tokens;
            }
            return Some(usage);
        }

        // Try Anthropic format
        if let Some(usage) = parse_anthropic_sse_chunk(trimmed) {
            if usage.prompt_tokens > 0 {
                self.prompt_tokens = self.prompt_tokens.max(usage.prompt_tokens);
            }
            if usage.completion_tokens > 0 {
                self.completion_tokens = self.completion_tokens.saturating_add(usage.completion_tokens);
            }
            if usage.cached_tokens > 0 {
                self.cached_tokens = self.cached_tokens.max(usage.cached_tokens);
            }
            return Some(usage);
        }

        // Try Gemini format
        if let Some(usage) = parse_gemini_chunk(trimmed) {
            if usage.prompt_tokens > 0 {
                self.prompt_tokens = usage.prompt_tokens;
            }
            if usage.completion_tokens > 0 {
                self.completion_tokens = usage.completion_tokens;
            }
            if usage.cached_tokens > 0 {
                self.cached_tokens = usage.cached_tokens;
            }
            return Some(usage);
        }

        None
    }

    /// Returns the total accumulated token usage.
    pub fn total_usage(&self) -> ParsedTokenUsage {
        ParsedTokenUsage {
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            cached_tokens: self.cached_tokens,
            saved_tokens: self.saved_tokens,
        }
    }
}

// ---------------------------------------------------------------------------
// Query Provider Quota via Free Channels (Zero Token Cost)
// ---------------------------------------------------------------------------

/// Queries upstream provider quota status via free agent AI API channels (e.g. metadata/model list endpoints)
/// without incurring any token generation costs.
pub fn query_provider_quota_free(provider: &str, api_key: Option<&str>) -> ProviderQuotaStatus {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let free_endpoint = match provider.to_ascii_lowercase().as_str() {
        "openai" => "https://api.openai.com/v1/models",
        "anthropic" => "https://api.anthropic.com/v1/models",
        "gemini" | "google" => "https://generativelanguage.googleapis.com/v1beta/models",
        "grok" | "xai" => "https://api.x.ai/v1/models",
        "deepseek" => "https://api.deepseek.com/models",
        "ollama" | "local" => "http://127.0.0.1:11434/api/tags",
        _ => "https://api.openai.com/v1/models",
    };

    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ProviderQuotaStatus {
                provider: provider.to_string(),
                is_available: false,
                rate_limits: ProviderRateLimits::default(),
                message: format!("HTTP client build error: {}", e),
                queried_epoch_ms: now_ms,
            };
        }
    };

    let mut req = client.get(free_endpoint);
    if let Some(key) = api_key {
        if !key.is_empty() {
            if provider.eq_ignore_ascii_case("anthropic") {
                req = req.header("x-api-key", key);
                req = req.header("anthropic-version", "2023-06-01");
            } else if provider.eq_ignore_ascii_case("gemini") || provider.eq_ignore_ascii_case("google") {
                let url = format!("{}?key={}", free_endpoint, key);
                req = client.get(url);
            } else {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
        }
    }

    match req.send() {
        Ok(resp) => {
            let status = resp.status();
            let mut headers_vec = Vec::new();
            for (k, v) in resp.headers() {
                if let Ok(v_str) = v.to_str() {
                    headers_vec.push((k.as_str().to_string(), v_str.to_string()));
                }
            }
            let rate_limits = extract_provider_rate_limits(&headers_vec);
            let is_available = status.is_success() || status.as_u16() == 429;
            let message = format!("HTTP {}", status);

            ProviderQuotaStatus {
                provider: provider.to_string(),
                is_available,
                rate_limits,
                message,
                queried_epoch_ms: now_ms,
            }
        }
        Err(e) => ProviderQuotaStatus {
            provider: provider.to_string(),
            is_available: false,
            rate_limits: ProviderRateLimits::default(),
            message: format!("Connection failed: {}", e),
            queried_epoch_ms: now_ms,
        },
    }
}

/// Asynchronous version of `query_provider_quota_free`.
pub async fn query_provider_quota_free_async(provider: &str, api_key: Option<&str>) -> ProviderQuotaStatus {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let free_endpoint = match provider.to_ascii_lowercase().as_str() {
        "openai" => "https://api.openai.com/v1/models",
        "anthropic" => "https://api.anthropic.com/v1/models",
        "gemini" | "google" => "https://generativelanguage.googleapis.com/v1beta/models",
        "grok" | "xai" => "https://api.x.ai/v1/models",
        "deepseek" => "https://api.deepseek.com/models",
        "ollama" | "local" => "http://127.0.0.1:11434/api/tags",
        _ => "https://api.openai.com/v1/models",
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ProviderQuotaStatus {
                provider: provider.to_string(),
                is_available: false,
                rate_limits: ProviderRateLimits::default(),
                message: format!("HTTP client build error: {}", e),
                queried_epoch_ms: now_ms,
            };
        }
    };

    let mut req = client.get(free_endpoint);
    if let Some(key) = api_key {
        if !key.is_empty() {
            if provider.eq_ignore_ascii_case("anthropic") {
                req = req.header("x-api-key", key);
                req = req.header("anthropic-version", "2023-06-01");
            } else if provider.eq_ignore_ascii_case("gemini") || provider.eq_ignore_ascii_case("google") {
                let url = format!("{}?key={}", free_endpoint, key);
                req = client.get(url);
            } else {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
        }
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let mut headers_vec = Vec::new();
            for (k, v) in resp.headers() {
                if let Ok(v_str) = v.to_str() {
                    headers_vec.push((k.as_str().to_string(), v_str.to_string()));
                }
            }
            let rate_limits = extract_provider_rate_limits(&headers_vec);
            let is_available = status.is_success() || status.as_u16() == 429;
            let message = format!("HTTP {}", status);

            ProviderQuotaStatus {
                provider: provider.to_string(),
                is_available,
                rate_limits,
                message,
                queried_epoch_ms: now_ms,
            }
        }
        Err(e) => ProviderQuotaStatus {
            provider: provider.to_string(),
            is_available: false,
            rate_limits: ProviderRateLimits::default(),
            message: format!("Connection failed: {}", e),
            queried_epoch_ms: now_ms,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_idle_guarantee() {
        let engine = TokenTrackerEngine::new(100);
        let summary = engine.lifetime_summary();
        assert_eq!(summary, (0, 0, 0, 0, 0.0, 0.0));
    }

    #[test]
    fn test_extract_provider_rate_limits() {
        let headers = vec![
            ("x-ratelimit-remaining-tokens".to_string(), "850000".to_string()),
            ("x-ratelimit-remaining-requests".to_string(), "990".to_string()),
            ("x-ratelimit-limit-tokens".to_string(), "1000000".to_string()),
            ("x-ratelimit-limit-requests".to_string(), "1000".to_string()),
            ("x-ratelimit-reset-tokens".to_string(), "15s".to_string()),
            ("retry-after".to_string(), "30".to_string()),
        ];

        let limits = extract_provider_rate_limits(&headers);
        assert_eq!(limits.remaining_tokens, Some(850000));
        assert_eq!(limits.remaining_requests, Some(990));
        assert_eq!(limits.limit_tokens, Some(1000000));
        assert_eq!(limits.limit_requests, Some(1000));
        assert_eq!(limits.reset_tokens.as_deref(), Some("15s"));
        assert_eq!(limits.retry_after_secs, Some(30));
    }

    #[test]
    fn test_anthropic_headers_extraction() {
        let headers = vec![
            ("anthropic-ratelimit-tokens-remaining".to_string(), "400000".to_string()),
            ("anthropic-ratelimit-requests-remaining".to_string(), "45".to_string()),
            ("anthropic-ratelimit-tokens-limit".to_string(), "500000".to_string()),
        ];

        let limits = extract_provider_rate_limits(&headers);
        assert_eq!(limits.remaining_tokens, Some(400000));
        assert_eq!(limits.remaining_requests, Some(45));
        assert_eq!(limits.limit_tokens, Some(500000));
    }

    #[test]
    fn test_headroom_savings_calculations() {
        let report = calculate_headroom_savings(12000, 4000, 2);
        assert_eq!(report.raw_bytes, 12000);
        assert_eq!(report.compressed_bytes, 4000);
        assert_eq!(report.byte_reduction, 8000);
        assert_eq!(report.pruned_tool_schemas, 2);
        // byte savings: 8000 / 4 = 2000 tokens. schema savings: 2 * 250 = 500 tokens. total = 2500
        assert_eq!(report.tokens_saved, 2500);
    }

    #[test]
    fn test_sse_stream_accumulator() {
        let mut parser = SseStreamTokenParser::new();
        assert_eq!(parser.total_usage(), ParsedTokenUsage::default());

        let line1 = "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":120,\"cache_read_input_tokens\":40}}}";
        let res1 = parser.feed_line(line1);
        assert!(res1.is_some());

        let line2 = "data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":85}}";
        let res2 = parser.feed_line(line2);
        assert!(res2.is_some());

        let total = parser.total_usage();
        assert_eq!(total.prompt_tokens, 120);
        assert_eq!(total.cached_tokens, 40);
        assert_eq!(total.completion_tokens, 85);
    }

    #[test]
    fn test_parse_gemini_chunk_with_and_without_data() {
        let raw = r#"{"usageMetadata":{"promptTokenCount":300,"candidatesTokenCount":50,"cachedContentTokenCount":100}}"#;
        let p1 = parse_gemini_chunk(raw);
        assert!(p1.is_some());
        if let Some(u1) = &p1 {
            assert_eq!(u1.prompt_tokens, 300);
            assert_eq!(u1.completion_tokens, 50);
            assert_eq!(u1.cached_tokens, 100);
        }

        let sse = format!("data: {}", raw);
        let p2 = parse_gemini_chunk(&sse);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_content_type_detection_and_ratios() {
        let code = "fn main() { println!(\"Hello World\"); let x = 42; }";
        assert_eq!(detect_content_type(code), ContentType::Code);
        let tokens = calculate_tokens_from_bytes(code.len(), ContentType::Code);
        assert!(tokens > 0);

        let json = "{\"name\": \"toksi\", \"status\": \"ok\", \"count\": 12}";
        assert_eq!(detect_content_type(json), ContentType::Json);

        let cjk = "这是一个测试，测试中文分词与标记化。";
        assert_eq!(detect_content_type(cjk), ContentType::Cjk);
    }

    #[test]
    fn test_rolling_quota_tracker() {
        let mut tracker = RollingQuotaTracker::new(AgentPlanTier::ClaudePro);
        let now = 100000;
        tracker.record_turn(now, 500);
        tracker.record_turn(now + 60, 800);
        assert_eq!(tracker.current_window_requests(now + 120), 2);
        assert_eq!(tracker.current_window_tokens(now + 120), 1300);

        let (used, limit, pct) = tracker.consumption_ratio(now + 120);
        assert_eq!(used, 2);
        assert_eq!(limit, 45);
        assert!(pct > 4.0 && pct < 5.0);

        // Advance past 5-hour window for all recorded turns
        let past = now + 60 + 5 * 3600 + 10;
        tracker.prune(past);
        assert_eq!(tracker.current_window_requests(past), 0);
        assert_eq!(tracker.current_window_tokens(past), 0);
    }
}
