// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/live_zone.rs
// created: 2026-11-13
// lastModified: 2026-11-13

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

/// Computes how many leading `messages` belong to a frozen cache prefix (`cache_control`).
/// Every message up to and including the last `cache_control` breakpoint MUST stay byte-identical.
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
            if map.contains_key("cache_control") {
                return true;
            }
            map.values().any(message_has_cache_control)
        }
        Value::Array(arr) => arr.iter().any(message_has_cache_control),
        _ => false,
    }
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

    if path.contains("messages") || parsed.get("messages").is_some() {
        if let Some(rewritten) = compress_field_array_live_zone(body, "messages", frozen_count, store) {
            return rewritten;
        }
    }
    if path.contains("responses") || parsed.get("input").is_some() {
        if let Some(rewritten) = compress_field_array_live_zone(body, "input", frozen_count, store) {
            return rewritten;
        }
    }
    if parsed.get("contents").is_some() {
        if let Some(rewritten) = compress_field_array_live_zone(body, "contents", frozen_count, store) {
            return rewritten;
        }
    }
    body.to_vec()
}

fn compress_field_array_live_zone(
    body: &[u8],
    field_name: &str,
    frozen_count: usize,
    store: &dyn CcrStore,
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
        if let Some(replacement) = compress_single_turn_raw(raw_item.get(), store) {
            if replacement.len() < slice.len() {
                edits.push((start, end, replacement));
            }
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

fn compress_single_turn_raw(raw_json: &str, store: &dyn CcrStore) -> Option<Vec<u8>> {
    let mut val: Value = serde_json::from_str(raw_json).ok()?;
    let mut changed = false;

    if let Some(obj) = val.as_object_mut() {
        // String content (OpenAI chat / Anthropic simple content)
        if let Some(Value::String(s)) = obj.get_mut("content") {
            if let Some(c) = compress_text_block(s, store) {
                *s = c;
                changed = true;
            }
        }
        // Array of content blocks (Anthropic blocks)
        if let Some(Value::Array(blocks)) = obj.get_mut("content") {
            for blk in blocks {
                if compress_block_object(blk, store) {
                    changed = true;
                }
            }
        }
        // Gemini parts array
        if let Some(Value::Array(parts)) = obj.get_mut("parts") {
            for part in parts {
                if let Some(Value::String(s)) = part.get_mut("text") {
                    if let Some(c) = compress_text_block(s, store) {
                        *s = c;
                        changed = true;
                    }
                }
            }
        }
        // Responses output string inside input item
        if let Some(Value::String(s)) = obj.get_mut("output") {
            if let Some(c) = compress_text_block(s, store) {
                *s = c;
                changed = true;
            }
        }
    }

    if changed {
        serde_json::to_vec(&val).ok()
    } else {
        None
    }
}

fn compress_block_object(blk: &mut Value, store: &dyn CcrStore) -> bool {
    let Some(obj) = blk.as_object_mut() else {
        return false;
    };
    if obj.contains_key("cache_control") {
        return false;
    }
    let mut changed = false;
    for field in ["text", "content"] {
        if let Some(Value::String(s)) = obj.get_mut(field) {
            if let Some(c) = compress_text_block(s, store) {
                *s = c;
                changed = true;
            }
        }
    }
    changed
}

pub fn compress_text_block(raw: &str, store: &dyn CcrStore) -> Option<String> {
    if raw.len() < MIN_LIVE_BLOCK_BYTES || raw.contains("<<ccr:") {
        return None;
    }
    let trimmed = raw.trim();

    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Some(crushed) = SmartCrusher::default().compress_json_str(trimmed, Some(store)) {
            if crushed.len() < raw.len() {
                return Some(crushed);
            }
        }
    }
    if DiffCompressor::looks_like_unified_diff(trimmed) {
        if let Some(diff) = DiffCompressor::default().compress(trimmed, Some(store)) {
            if diff.len() < raw.len() {
                return Some(diff);
            }
        }
    }
    if SearchCompressor::looks_like_search_output(trimmed) {
        if let Some(search) = SearchCompressor::default().compress(trimmed, "", Some(store)) {
            if search.len() < raw.len() {
                return Some(search);
            }
        }
    }

    let compressed = compress_content(raw, None);
    if compressed.len() + 64 < raw.len() {
        let key = compute_key(raw.as_bytes());
        store.put(&key, raw);
        let with_ccr = format!("{}\n[Full content: headroom_retrieve({})]", compressed, marker_for(&key));
        if with_ccr.len() < raw.len() {
            return Some(with_ccr);
        }
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
        let raw_bytes = serde_json::to_vec(&payload).expect("serialize");
        let raw_str = String::from_utf8(raw_bytes.clone()).expect("utf8");
        let prefix_marker = "\"role\":\"assistant\"";
        let prefix_len = raw_str.find(prefix_marker).expect("prefix offset");

        let store = InMemoryCcrStore::default();
        let compressed = compress_request_body_with_store("/v1/messages", &raw_bytes, &store);
        assert!(compressed.len() < raw_bytes.len());
        assert_eq!(&compressed[..prefix_len], &raw_bytes[..prefix_len]);
        let compressed_str = String::from_utf8(compressed).expect("utf8");
        assert!(compressed_str.contains("<<ccr:"));
        assert!(compressed_str.contains("FROZEN_TURN_0_UNTOUCHED"));
    }
}
