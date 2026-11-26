// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/live_zone.rs
// created: 2026-11-13
// lastModified: 2026-11-18

//! Zero-unsafe `RawValue` byte-range surgery preserving frozen LLM prompt-cache prefixes
//! across Anthropic `/v1/messages`, OpenAI `/v1/chat/completions`, OpenAI `/v1/responses`,
//! and Gemini `contents` payloads.

use serde::Deserialize;
use serde_json::value::RawValue;
use serde_json::Value;

use super::ccr::{compute_key, global_ccr_store, marker_for, CcrStore};
use super::compression::compress_content;
use super::diff_compressor::DiffCompressor;
use super::search_compressor::SearchCompressor;
use super::smart_crusher::SmartCrusher;

const MIN_LIVE_BLOCK_BYTES: usize = 512;

/// Computes how many leading `messages` belong to a frozen cache prefix (`cache_control` or Bedrock `cachePoint`).
/// Every message up to and including the last `cache_control`/`cachePoint` breakpoint MUST stay byte-identical.
pub fn compute_frozen_count(parsed: &Value) -> usize {
    let Some(messages) = parsed.get("messages").and_then(|m| m.as_array()) else {
        return 0;
    };
    let mut frozen = 0usize;
    for (idx, msg) in messages.iter().enumerate() {
        if message_has_cache_control(msg) {
            frozen = idx + 1;
        }
    }
    frozen
}

fn message_has_cache_control(val: &Value) -> bool {
    match val {
        Value::Object(map) => {
            if map.contains_key("cache_control") || map.contains_key("cachePoint") {
                return true;
            }
            map.values().any(message_has_cache_control)
        }
        Value::Array(arr) => arr.iter().any(message_has_cache_control),
        _ => false,
    }
}

/// Extracts the latest user prompt text for query-aware SmartCrusher & SearchCompressor ranking.
pub fn extract_latest_user_query(parsed: &Value) -> String {
    if let Some(messages) = parsed.get("messages").and_then(|m| m.as_array()) {
        for msg in messages.iter().rev() {
            if msg.get("role").and_then(Value::as_str) == Some("user") {
                let text = extract_text_from_turn_value(msg);
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }
    if let Some(input) = parsed.get("input") {
        if let Some(s) = input.as_str() {
            return s.to_string();
        }
        if let Some(items) = input.as_array() {
            for item in items.iter().rev() {
                if item.get("role").and_then(Value::as_str) == Some("user") {
                    let text = extract_text_from_turn_value(item);
                    if !text.is_empty() {
                        return text;
                    }
                }
            }
        }
    }
    if let Some(contents) = parsed.get("contents").and_then(|c| c.as_array()) {
        for item in contents.iter().rev() {
            let role = item.get("role").and_then(Value::as_str).unwrap_or("user");
            if role == "user" {
                let text = extract_text_from_turn_value(item);
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }
    String::new()
}

fn extract_text_from_turn_value(turn: &Value) -> String {
    if let Some(s) = turn.as_str() {
        return s.to_string();
    }
    let Some(obj) = turn.as_object() else {
        return String::new();
    };
    if let Some(s) = obj.get("content").and_then(Value::as_str) {
        return s.to_string();
    }
    if let Some(blocks) = obj.get("content").and_then(Value::as_array) {
        let joined: Vec<&str> = blocks
            .iter()
            .filter_map(|b| {
                b.get("text")
                    .and_then(Value::as_str)
                    .or_else(|| b.get("content").and_then(Value::as_str))
            })
            .collect();
        if !joined.is_empty() {
            return joined.join("\n");
        }
    }
    if let Some(parts) = obj.get("parts").and_then(Value::as_array) {
        let joined: Vec<&str> = parts
            .iter()
            .filter(|p| p.get("thought").and_then(Value::as_bool) != Some(true))
            .filter_map(|p| p.get("text").and_then(Value::as_str))
            .collect();
        if !joined.is_empty() {
            return joined.join("\n");
        }
    }
    String::new()
}

/// Detects native Gemini `generateContent` and AWS Bedrock Converse shapes that cannot carry `headroom_retrieve`.
pub fn is_marker_free_native_shape(path: &str, parsed: &Value) -> bool {
    if (parsed.get("contents").is_some()
        && parsed.get("messages").is_none()
        && parsed.get("input").is_none())
        || path.contains("generateContent")
    {
        return true;
    }

    if let Some(messages) = parsed.get("messages").and_then(Value::as_array)
        && !messages.is_empty()
            && messages.iter().all(|m| {
                let role = m.get("role").and_then(Value::as_str).unwrap_or("");
                let role_ok = role == "user" || role == "assistant";
                let blocks_ok = m
                    .get("content")
                    .and_then(Value::as_array)
                    .is_some_and(|arr| {
                        !arr.is_empty()
                            && arr.iter().all(|b| b.as_object().is_some_and(|o| !o.contains_key("type")))
                    });
                role_ok && blocks_ok
            })
        {
            let has_converse_block = messages.iter().any(|m| {
                m.get("content")
                    .and_then(Value::as_array)
                    .is_some_and(|arr| {
                        arr.iter().any(|b| {
                            b.get("text").is_some()
                                || b.get("toolUse").is_some()
                                || b.get("toolResult").is_some()
                                || b.get("cachePoint").is_some()
                                || b.get("reasoningContent").is_some()
                        })
                    })
            });
            if has_converse_block {
                return true;
            }
        }
    false
}

/// Applies safe byte-range surgery on the live zone of an LLM JSON request payload.
/// Returns original `body` untouched if no live-zone turn shrunk or payload isn't eligible.
pub fn compress_request_body_safe(path: &str, body: &[u8]) -> Vec<u8> {
    let store = global_ccr_store();
    compress_request_body_with_store(path, body, store.as_ref())
}

pub fn compress_request_body_with_store(path: &str, body: &[u8], store: &dyn CcrStore) -> Vec<u8> {
    if body.is_empty() {
        return Vec::new();
    }
    let Ok(parsed) = serde_json::from_slice::<Value>(body) else {
        return body.to_vec();
    };
    let frozen_count = compute_frozen_count(&parsed);
    let query = extract_latest_user_query(&parsed);
    let ccr_opt: Option<&dyn CcrStore> = if is_marker_free_native_shape(path, &parsed) {
        None
    } else {
        Some(store)
    };

    if (path.contains("messages") || parsed.get("messages").is_some())
        && let Some(rewritten) =
            compress_field_array_live_zone(body, "messages", frozen_count, &query, ccr_opt)
        {
            return rewritten;
        }
    if (path.contains("responses") || parsed.get("input").is_some())
        && let Some(rewritten) =
            compress_field_array_live_zone(body, "input", frozen_count, &query, ccr_opt)
        {
            return rewritten;
        }
    if parsed.get("contents").is_some()
        && let Some(rewritten) =
            compress_field_array_live_zone(body, "contents", frozen_count, &query, ccr_opt)
        {
            return rewritten;
        }
    body.to_vec()
}

fn compress_field_array_live_zone(
    body: &[u8],
    field_name: &str,
    frozen_count: usize,
    query: &str,
    store: Option<&dyn CcrStore>,
) -> Option<Vec<u8>> {
    #[derive(Deserialize)]
    struct Shell<'a> {
        #[serde(borrow)]
        messages: Option<Vec<&'a RawValue>>,
        #[serde(borrow)]
        input: Option<Vec<&'a RawValue>>,
        #[serde(borrow)]
        contents: Option<Vec<&'a RawValue>>,
    }

    let shell: Shell<'_> = serde_json::from_slice(body).ok()?;
    let elements = match field_name {
        "messages" => shell.messages?,
        "input" => shell.input?,
        "contents" => shell.contents?,
        _ => return None,
    };

    if frozen_count >= elements.len() {
        return None;
    }

    let base_ptr = body.as_ptr() as usize;
    let body_end = base_ptr + body.len();

    let mut edits: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    for raw_item in elements.iter().skip(frozen_count) {
        let slice = raw_item.get().as_bytes();
        let start_ptr = slice.as_ptr() as usize;
        let end_ptr = start_ptr + slice.len();
        if start_ptr < base_ptr || end_ptr > body_end {
            return None;
        }
        let start = start_ptr - base_ptr;
        let end = end_ptr - base_ptr;
        if let Some(replacement) = compress_single_turn_raw(raw_item.get(), query, store)
            && replacement.len() < slice.len() {
                edits.push((start, end, replacement));
            }
    }

    if edits.is_empty() {
        return None;
    }

    // Apply edits in ascending order over disjoint raw slices, leaving frozen bytes 100% untouched.
    let mut out = Vec::with_capacity(body.len());
    let mut cursor = 0usize;
    for (start, end, replacement) in edits {
        if start < cursor || end > body.len() {
            return None;
        }
        out.extend_from_slice(&body[cursor..start]);
        out.extend_from_slice(&replacement);
        cursor = end;
    }
    out.extend_from_slice(&body[cursor..]);
    Some(out)
}

fn compress_single_turn_raw(
    raw_json: &str,
    query: &str,
    store: Option<&dyn CcrStore>,
) -> Option<Vec<u8>> {
    let mut val: Value = serde_json::from_str(raw_json).ok()?;
    let mut changed = false;

    if let Some(obj) = val.as_object_mut() {
        // String content (OpenAI chat / Anthropic simple content)
        if let Some(Value::String(s)) = obj.get_mut("content")
            && let Some(c) = compress_text_block_with_query(s, query, store) {
                *s = c;
                changed = true;
            }
        // Array of content blocks (Anthropic blocks & Bedrock Converse blocks)
        if let Some(Value::Array(blocks)) = obj.get_mut("content") {
            for blk in blocks {
                if compress_block_object(blk, query, store) {
                    changed = true;
                }
            }
        }
        // Gemini parts array (never touch thought parts, thoughtSignature, functionCall, or inlineData)
        if let Some(Value::Array(parts)) = obj.get_mut("parts") {
            for part in parts {
                let is_protected_part = part.get("thought").and_then(Value::as_bool) == Some(true)
                    || part.get("thoughtSignature").is_some()
                    || part.get("functionCall").is_some()
                    || part.get("inlineData").is_some();
                if is_protected_part {
                    continue;
                }
                if let Some(Value::String(s)) = part.get_mut("text")
                    && let Some(c) = compress_text_block_with_query(s, query, store) {
                        *s = c;
                        changed = true;
                    }
            }
        }
        // Responses output string inside input item
        if let Some(Value::String(s)) = obj.get_mut("output")
            && let Some(c) = compress_text_block_with_query(s, query, store) {
                *s = c;
                changed = true;
            }
    }

    if changed {
        serde_json::to_vec(&val).ok()
    } else {
        None
    }
}

fn compress_block_object(blk: &mut Value, query: &str, store: Option<&dyn CcrStore>) -> bool {
    // Protect blocks with top-level OR nested `cache_control` / `cachePoint` (e.g. inside `tool_result.content`).
    if message_has_cache_control(blk) {
        return false;
    }
    let Some(obj) = blk.as_object_mut() else {
        return false;
    };
    if obj.contains_key("reasoningContent")
        || obj.contains_key("toolUse")
        || obj.contains_key("thoughtSignature")
        || obj.get("thought").and_then(Value::as_bool) == Some(true)
    {
        return false;
    }
    let mut changed = false;
    for field in ["text", "content"] {
        if let Some(Value::String(s)) = obj.get_mut(field)
            && let Some(c) = compress_text_block_with_query(s, query, store) {
                *s = c;
                changed = true;
            }
    }
    // Anthropic `tool_result` with array `content` sub-blocks
    if let Some(Value::Array(inner_blocks)) = obj.get_mut("content") {
        for inner in inner_blocks {
            if compress_block_object(inner, query, store) {
                changed = true;
            }
        }
    }
    // Bedrock Converse `toolResult.content` inner array
    if let Some(Value::Object(tool_res)) = obj.get_mut("toolResult")
        && let Some(Value::Array(inner_blocks)) = tool_res.get_mut("content") {
            for inner in inner_blocks {
                if compress_block_object(inner, query, store) {
                    changed = true;
                }
            }
        }
    changed
}

pub fn compress_text_block(raw: &str, store: &dyn CcrStore) -> Option<String> {
    compress_text_block_with_query(raw, "", Some(store))
}

pub fn compress_text_block_with_query(
    raw: &str,
    query: &str,
    store: Option<&dyn CcrStore>,
) -> Option<String> {
    if raw.len() < MIN_LIVE_BLOCK_BYTES || raw.contains("<<ccr:") {
        return None;
    }
    let trimmed = raw.trim();

    if trimmed.starts_with('[') && trimmed.ends_with(']')
        && let Some(crushed) = SmartCrusher::default().compress_json_str_with_query(trimmed, query, store)
            && crushed.len() < raw.len() {
                return Some(crushed);
            }
    if DiffCompressor::looks_like_unified_diff(trimmed)
        && let Some(diff) = DiffCompressor::default().compress(trimmed, store)
            && diff.len() < raw.len() {
                return Some(diff);
            }
    if SearchCompressor::looks_like_search_output(trimmed)
        && let Some(search) = SearchCompressor::default().compress(trimmed, query, store)
            && search.len() < raw.len() {
                return Some(search);
            }

    let rtk_pre = if !query.is_empty() && crate::rtk::can_filter(query) {
        Some(crate::rtk::compress_tool_output(query, raw))
    } else if crate::rtk::has_ansi(raw) {
        Some(crate::rtk::strip_ansi(raw))
    } else {
        None
    };
    let effective_raw = rtk_pre.as_deref().unwrap_or(raw);
    let compressed = compress_content(effective_raw, None);
    if let Some(ccr) = store {
        if compressed.len() + 64 < raw.len() {
            let key = compute_key(raw.as_bytes());
            ccr.put(&key, raw);
            let with_ccr = format!("{}\n[Full content: headroom_retrieve({})]", compressed, marker_for(&key));
            if with_ccr.len() < raw.len() {
                return Some(with_ccr);
            }
            return Some(compressed);
        }
    } else if compressed.len() < raw.len() {
        return Some(compressed);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headroom::ccr::InMemoryCcrStore;

    #[test]
    fn test_live_zone_preserves_frozen_cache_prefix_byte_for_byte() {
        let large_grep: String = (1..=40)
            .map(|i| format!(r"C:\repo\src\file.rs:{}:let val_{} = {};", i, i, i))
            .collect::<Vec<_>>()
            .join("\n");

        let payload = serde_json::json!({
            "model": "claude-3-7-sonnet",
            "system": [{"type": "text", "text": "FROZEN_SYSTEM_PROMPT", "cache_control": {"type": "ephemeral"}}],
            "messages": [
                {
                    "role": "user",
                    "content": [{"type": "text", "text": "FROZEN_TURN_0_UNTOUCHED", "cache_control": {"type": "ephemeral"}}]
                },
                {
                    "role": "assistant",
                    "content": "Ack."
                },
                {
                    "role": "user",
                    "content": [{"type": "tool_result", "content": large_grep}]
                }
            ]
        });
        let raw_bytes = serde_json::to_vec(&payload).unwrap_or_default();
        let raw_str = String::from_utf8(raw_bytes.clone()).unwrap_or_default();
        let prefix_marker = "\"role\":\"assistant\"";
        let prefix_len = raw_str.find(prefix_marker).unwrap_or(0);
        assert!(prefix_len > 0);

        let store = InMemoryCcrStore::default();
        let compressed = compress_request_body_with_store("/v1/messages", &raw_bytes, &store);
        assert!(compressed.len() < raw_bytes.len());
        assert_eq!(&compressed[..prefix_len], &raw_bytes[..prefix_len]);
        let compressed_str = String::from_utf8(compressed).unwrap_or_default();
        assert!(compressed_str.contains("<<ccr:"));
        assert!(compressed_str.contains("FROZEN_TURN_0_UNTOUCHED"));
    }

    #[test]
    fn test_gemini_and_bedrock_converse_marker_free_compression() {
        let repeated_logs = "2026-11-15T10:00:00Z [INFO] worker pool healthy\n".repeat(30);
        let gemini_payload = serde_json::json!({
            "contents": [
                {
                    "role": "model",
                    "parts": [
                        {"text": "thinking trace", "thought": true, "thoughtSignature": "sig-abc"}
                    ]
                },
                {
                    "role": "user",
                    "parts": [
                        {"text": repeated_logs}
                    ]
                }
            ]
        });
        let gem_raw = serde_json::to_vec(&gemini_payload).unwrap_or_default();
        let store = InMemoryCcrStore::default();
        let gem_comp = compress_request_body_with_store("/v1beta/models/gemini-2.5-pro:generateContent", &gem_raw, &store);
        assert!(gem_comp.len() < gem_raw.len());
        let gem_str = String::from_utf8_lossy(&gem_comp);
        assert!(!gem_str.contains("<<ccr:"));
        assert!(gem_str.contains("sig-abc"));

        let converse_payload = serde_json::json!({
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {"toolResult": {"toolUseId": "t1", "content": [{"text": repeated_logs}]}}
                    ]
                }
            ]
        });
        let conv_raw = serde_json::to_vec(&converse_payload).unwrap_or_default();
        let conv_comp = compress_request_body_with_store("/converse", &conv_raw, &store);
        assert!(conv_comp.len() < conv_raw.len());
        let conv_str = String::from_utf8_lossy(&conv_comp);
        assert!(!conv_str.contains("<<ccr:"));
    }

    #[test]
    fn test_live_zone_protects_nested_cache_control_in_tool_result() {
        let repeated_logs = "2026-11-18T10:00:00Z [INFO] worker pool healthy\n".repeat(40);
        let payload = serde_json::json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": "tu_1",
                            "content": [
                                {
                                    "type": "text",
                                    "text": repeated_logs,
                                    "cache_control": {"type": "ephemeral"}
                                }
                            ]
                        }
                    ]
                }
            ]
        });
        let raw_bytes = serde_json::to_vec(&payload).unwrap_or_default();
        let store = InMemoryCcrStore::default();
        let out = compress_request_body_with_store("/v1/messages", &raw_bytes, &store);
        assert_eq!(out, raw_bytes);
    }
}
