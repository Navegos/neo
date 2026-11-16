// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/effort.rs
// created: 2026-11-16
// lastModified: 2026-11-16

//! Shared Thinking-Effort-Aware Headroom Compressor (`Low` / `Medium` / `High`)
//! enforced across all 7 inference and protocol entrypoints (`:8787`–`:8790`,
//! `:9098`, `:9099`, `:18787`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::adaptive_mem::inject_adaptive_memory;
use super::auth_mode::classify_auth_mode;
use super::cache_align::{
    align_system_prompt_to_1024, auto_place_anthropic_cache_control, stabilize_cache_keys,
};
use super::compression::economizer::{EconomizerOptions, TokenEconomizer};
use super::live_zone::{
    compress_request_body_safe, compute_frozen_count, is_marker_free_native_shape,
};
use super::pruner::{prune_mcp_request_payload, McpPrunerConfig};

/// Thinking-effort tier governing upstream reasoning budgets and Headroom aggressiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Low,
    #[default]
    Medium,
    High,
}

impl EffortLevel {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "low" | "l" | "fast" | "minimal" => Some(Self::Low),
            "medium" | "med" | "m" | "balanced" | "default" => Some(Self::Medium),
            "high" | "h" | "deep" | "max" => Some(Self::High),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub const fn mcp_max_description_len(self) -> usize {
        match self {
            Self::Low => 160,
            Self::Medium => 400,
            Self::High => 800,
        }
    }

    pub const fn loop_guard_threshold(self) -> usize {
        match self {
            Self::Low => 2,
            Self::Medium => 3,
            Self::High => 5,
        }
    }

    pub const fn anthropic_budget_tokens(self) -> u64 {
        match self {
            Self::Low => 1024,
            Self::Medium => 8192,
            Self::High => 32768,
        }
    }

    pub const fn gemini_thinking_budget(self) -> u64 {
        match self {
            Self::Low => 1024,
            Self::Medium => 8192,
            Self::High => 24576,
        }
    }
}

/// Metrics returned by [`HeadroomEffortCompressor`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffortCompressionStats {
    pub effort: EffortLevel,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub bytes_saved: usize,
    pub estimated_tokens_saved: usize,
    pub mcp_tools_pruned: usize,
    pub economizer_dupes_removed: usize,
    pub cache_aligned: bool,
    pub frozen_prefix_turns: usize,
}

/// Detects requested effort tier from `x-toksi-effort` header or JSON body reasoning/thinking hints.
pub fn detect_effort_level(headers: &[(String, String)], payload: Option<&Value>) -> EffortLevel {
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("x-toksi-effort") {
            if let Some(level) = EffortLevel::from_str_loose(v) {
                return level;
            }
        }
    }

    if let Some(val) = payload {
        if let Some(raw) = val.get("reasoning_effort").and_then(Value::as_str) {
            if let Some(level) = EffortLevel::from_str_loose(raw) {
                return level;
            }
        }
        if let Some(budget) = val
            .get("thinking")
            .and_then(|t| t.get("budget_tokens"))
            .and_then(Value::as_u64)
        {
            return if budget <= 2048 {
                EffortLevel::Low
            } else if budget <= 16384 {
                EffortLevel::Medium
            } else {
                EffortLevel::High
            };
        }
        if let Some(budget) = val
            .get("generationConfig")
            .and_then(|g| g.get("thinkingConfig"))
            .and_then(|t| t.get("thinkingBudget"))
            .and_then(Value::as_u64)
        {
            return if budget <= 2048 {
                EffortLevel::Low
            } else if budget <= 12288 {
                EffortLevel::Medium
            } else {
                EffortLevel::High
            };
        }
    }

    EffortLevel::Medium
}

/// Translates an explicit `EffortLevel` into OpenAI / Anthropic / Gemini wire parameters
/// when the model supports thinking or the payload already carries thinking settings.
pub fn apply_effort_wire_translation(
    path: &str,
    payload: &mut Value,
    effort: EffortLevel,
    explicit_override: bool,
) -> bool {
    let Some(obj) = payload.as_object_mut() else {
        return false;
    };
    let model = obj
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();

    let is_openai_reasoning = model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
        || model.starts_with("gpt-5")
        || model.contains("deepseek-r1")
        || model.contains("deepseek-reasoner")
        || model.contains("grok-3-mini")
        || obj.contains_key("reasoning_effort");

    if is_openai_reasoning && (explicit_override || obj.contains_key("reasoning_effort")) {
        obj.insert(
            "reasoning_effort".to_string(),
            Value::String(effort.as_str().to_string()),
        );
        return true;
    }

    let is_anthropic_thinking = path.contains("/v1/messages")
        || model.contains("claude-3-7")
        || model.contains("claude-sonnet-4")
        || model.contains("claude-opus-4")
        || obj.contains_key("thinking");

    if is_anthropic_thinking && (explicit_override || obj.contains_key("thinking")) {
        let budget = effort.anthropic_budget_tokens();
        let mut thinking = obj
            .get("thinking")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        thinking.insert("type".to_string(), Value::String("enabled".to_string()));
        thinking.insert("budget_tokens".to_string(), json!(budget));
        obj.insert("thinking".to_string(), Value::Object(thinking));
        return true;
    }

    let is_gemini_thinking = path.contains("generateContent")
        || model.contains("gemini-2.5")
        || obj
            .get("generationConfig")
            .and_then(|g| g.get("thinkingConfig"))
            .is_some();

    if is_gemini_thinking
        && (explicit_override
            || obj
                .get("generationConfig")
                .and_then(|g| g.get("thinkingConfig"))
                .is_some())
    {
        let budget = effort.gemini_thinking_budget();
        let gen_cfg = obj
            .entry("generationConfig")
            .or_insert_with(|| Value::Object(Default::default()));
        if let Some(gen_map) = gen_cfg.as_object_mut() {
            let tc = gen_map
                .entry("thinkingConfig")
                .or_insert_with(|| Value::Object(Default::default()));
            if let Some(tc_map) = tc.as_object_mut() {
                tc_map.insert("thinkingBudget".to_string(), json!(budget));
                return true;
            }
        }
    }

    false
}

/// Shared Headroom effort-scaled compression pipeline.
#[derive(Debug, Clone)]
pub struct HeadroomEffortCompressor {
    pub align_1024: bool,
    pub inject_memory: bool,
}

impl Default for HeadroomEffortCompressor {
    fn default() -> Self {
        Self {
            align_1024: false,
            inject_memory: true,
        }
    }
}

impl HeadroomEffortCompressor {
    pub fn gateway_default() -> Self {
        Self {
            align_1024: true,
            inject_memory: false,
        }
    }

    /// Compresses a mutable JSON payload in place (used by `:9099` Gateway and `:9098` Protocol Bus).
    pub fn compress_json_payload(
        &self,
        path: &str,
        headers: &[(String, String)],
        payload: &mut Value,
    ) -> EffortCompressionStats {
        let orig_bytes = serde_json::to_vec(payload).map(|v| v.len()).unwrap_or(0);
        let has_header_override = headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("x-toksi-effort"));
        let effort = detect_effort_level(headers, Some(payload));
        let auth_mode = classify_auth_mode(headers);
        let frozen_count = compute_frozen_count(payload);

        let mut stats = EffortCompressionStats {
            effort,
            original_bytes: orig_bytes,
            compressed_bytes: orig_bytes,
            frozen_prefix_turns: frozen_count,
            ..Default::default()
        };

        // Never mutate frozen cache prefixes or OAuth/Subscription cache-control headers.
        if auth_mode.should_mutate_cache_control() && frozen_count == 0 {
            if self.inject_memory {
                let _ = inject_adaptive_memory(payload);
            }
            let pruner_cfg = McpPrunerConfig {
                max_description_len: effort.mcp_max_description_len(),
                ..Default::default()
            };
            if let Some(p_stats) = prune_mcp_request_payload(payload, &pruner_cfg) {
                stats.mcp_tools_pruned = p_stats.tokens_saved;
            }
            if path.contains("/v1/messages") {
                let _ = auto_place_anthropic_cache_control(payload);
            }
            if self.align_1024 {
                stats.cache_aligned = align_system_prompt_to_1024(payload);
            }
            stabilize_cache_keys(payload);
        }

        if has_header_override {
            let _ = apply_effort_wire_translation(path, payload, effort, true);
        }

        // Apply TokenEconomizer to unfrozen live-zone turns
        let econ_opts = EconomizerOptions {
            deduplicate_instructions: effort != EffortLevel::High,
            minify_whitespace: effort == EffortLevel::Low,
            strip_redundant_comments: effort != EffortLevel::High,
            prune_schema_verbose_descriptions: effort != EffortLevel::High,
            loop_guard_threshold: effort.loop_guard_threshold(),
        };
        let mut economizer = TokenEconomizer::new(econ_opts);
        stats.economizer_dupes_removed =
            economize_unfrozen_turns(payload, frozen_count, &mut economizer);

        // Apply live-zone byte-range surgery (SmartCrusher, SearchCompressor, DiffCompressor, Log/Code/JSON + CCR)
        if let Ok(serialized) = serde_json::to_vec(payload) {
            let compressed = compress_request_body_safe(path, &serialized);
            if compressed.len() < serialized.len() {
                if let Ok(re_parsed) = serde_json::from_slice::<Value>(&compressed) {
                    *payload = re_parsed;
                }
            }
        }

        let final_bytes = serde_json::to_vec(payload).map(|v| v.len()).unwrap_or(orig_bytes);
        stats.compressed_bytes = final_bytes;
        stats.bytes_saved = orig_bytes.saturating_sub(final_bytes);
        stats.estimated_tokens_saved = stats.bytes_saved / 4;
        stats
    }

    /// Compresses raw HTTP JSON request bytes while strictly preserving any frozen `cache_control` prefix bytes.
    pub fn compress_http_request(
        &self,
        path: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> (Vec<u8>, EffortCompressionStats) {
        let mut stats = EffortCompressionStats {
            effort: detect_effort_level(headers, None),
            original_bytes: body.len(),
            compressed_bytes: body.len(),
            ..Default::default()
        };
        if body.is_empty() {
            return (Vec::new(), stats);
        }

        let Ok(mut parsed) = serde_json::from_slice::<Value>(body) else {
            return (body.to_vec(), stats);
        };
        let has_prompt_body = parsed.get("messages").is_some()
            || parsed.get("contents").is_some()
            || parsed.get("input").is_some();
        if !has_prompt_body {
            return (body.to_vec(), stats);
        }

        let has_header_override = headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("x-toksi-effort"));
        let effort = detect_effort_level(headers, Some(&parsed));
        let auth_mode = classify_auth_mode(headers);
        let frozen_count = compute_frozen_count(&parsed);
        let native_shape = is_marker_free_native_shape(path, &parsed);
        stats.effort = effort;
        stats.frozen_prefix_turns = frozen_count;

        // When a cache prefix is frozen or AuthMode forbids prefix mutation, skip JSON rewrite
        // and apply pure byte-range live_zone surgery directly on `body`.
        let mut working_body = body.to_vec();
        if auth_mode.should_mutate_cache_control() && frozen_count == 0 && !native_shape {
            if self.inject_memory {
                let _ = inject_adaptive_memory(&mut parsed);
            }
            let pruner_cfg = McpPrunerConfig {
                max_description_len: effort.mcp_max_description_len(),
                ..Default::default()
            };
            if let Some(p_stats) = prune_mcp_request_payload(&mut parsed, &pruner_cfg) {
                stats.mcp_tools_pruned = p_stats.tokens_saved;
            }
            if path.contains("/v1/messages") {
                let _ = auto_place_anthropic_cache_control(&mut parsed);
            }
            if self.align_1024 {
                stats.cache_aligned = align_system_prompt_to_1024(&mut parsed);
            }
            stabilize_cache_keys(&mut parsed);

            let econ_opts = EconomizerOptions {
                deduplicate_instructions: effort != EffortLevel::High,
                minify_whitespace: effort == EffortLevel::Low,
                strip_redundant_comments: effort != EffortLevel::High,
                prune_schema_verbose_descriptions: effort != EffortLevel::High,
                loop_guard_threshold: effort.loop_guard_threshold(),
            };
            let mut economizer = TokenEconomizer::new(econ_opts);
            stats.economizer_dupes_removed =
                economize_unfrozen_turns(&mut parsed, 0, &mut economizer);

            let translated = if has_header_override {
                apply_effort_wire_translation(path, &mut parsed, effort, true)
            } else {
                false
            };

            if let Ok(bytes) = serde_json::to_vec(&parsed) {
                if bytes.len() <= working_body.len() || translated {
                    working_body = bytes;
                }
            }
        }

        let live_compressed = compress_request_body_safe(path, &working_body);
        let final_body = if live_compressed.len() < working_body.len() {
            live_compressed
        } else {
            working_body
        };

        stats.compressed_bytes = final_body.len();
        stats.bytes_saved = body.len().saturating_sub(final_body.len());
        stats.estimated_tokens_saved = stats.bytes_saved / 4;
        (final_body, stats)
    }
}

fn economize_unfrozen_turns(
    payload: &mut Value,
    frozen_count: usize,
    economizer: &mut TokenEconomizer,
) -> usize {
    let mut dupes = 0usize;
    let Some(obj) = payload.as_object_mut() else {
        return 0;
    };

    if let Some(Value::Array(messages)) = obj.get_mut("messages") {
        for msg in messages.iter_mut().skip(frozen_count) {
            if let Some(Value::String(s)) = msg.get_mut("content") {
                if s.len() >= 256 && !s.contains("<<ccr:") {
                    let (econ, report) = economizer.economize_prompt(s);
                    dupes += report.duplicate_instructions_removed;
                    if econ.len() < s.len() {
                        *s = econ;
                    }
                }
            }
        }
    }
    dupes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effort_detection_and_wire_translation() {
        let headers = vec![("X-Toksi-Effort".to_string(), "high".to_string())];
        let mut oai = json!({
            "model": "o3-mini",
            "messages": [{"role": "user", "content": "Solve this"}]
        });
        let effort = detect_effort_level(&headers, Some(&oai));
        assert_eq!(effort, EffortLevel::High);
        assert!(apply_effort_wire_translation("/v1/chat/completions", &mut oai, effort, true));
        assert_eq!(oai["reasoning_effort"], "high");

        let mut anth = json!({
            "model": "claude-3-7-sonnet",
            "messages": [{"role": "user", "content": "Solve this"}]
        });
        assert!(apply_effort_wire_translation("/v1/messages", &mut anth, EffortLevel::Low, true));
        assert_eq!(anth["thinking"]["budget_tokens"], 1024);

        let mut gem = json!({
            "model": "gemini-2.5-pro",
            "contents": [{"role": "user", "parts": [{"text": "Solve this"}]}]
        });
        assert!(apply_effort_wire_translation(
            "/v1beta/models/gemini-2.5-pro:generateContent",
            &mut gem,
            EffortLevel::Medium,
            true
        ));
        assert_eq!(gem["generationConfig"]["thinkingConfig"]["thinkingBudget"], 8192);
    }

    #[test]
    fn test_effort_compressor_preserves_frozen_prefix_and_compresses_tail() {
        let large_grep: String = (1..=35)
            .map(|i| format!(r"C:\repo\src\file.rs:{}:let val_{} = {};", i, i, i))
            .collect::<Vec<_>>()
            .join("\n");
        let payload = json!({
            "model": "claude-3-7-sonnet",
            "messages": [
                {
                    "role": "user",
                    "content": [{"type": "text", "text": "FROZEN_PREFIX", "cache_control": {"type": "ephemeral"}}]
                },
                {
                    "role": "user",
                    "content": [{"type": "tool_result", "content": large_grep}]
                }
            ]
        });
        let raw = serde_json::to_vec(&payload).unwrap_or_default();
        let raw_str = String::from_utf8(raw.clone()).unwrap_or_default();
        let frozen_end = raw_str.find("tool_result").unwrap_or(0);
        assert!(frozen_end > 0);

        let headers = vec![("x-api-key".to_string(), "sk-ant-api03-test".to_string())];
        let (compressed, stats) = HeadroomEffortCompressor::default().compress_http_request(
            "/v1/messages",
            &headers,
            &raw,
        );
        assert!(compressed.len() < raw.len());
        assert!(stats.bytes_saved > 0);
        assert_eq!(stats.frozen_prefix_turns, 1);
        assert_eq!(&compressed[..frozen_end - 10], &raw[..frozen_end - 10]);
    }
}
