// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/cache_align.rs

use serde_json::{json, Value};

pub const CACHE_ALIGNMENT_TOKEN_BLOCK: usize = 1024;

/// Estimates the token count of a given string using BPE character ratio heuristic
pub fn estimate_tokens(text: &str) -> usize {
    let char_count = text.chars().count();
    if char_count == 0 {
        return 0;
    }
    // Heuristic: ~3.8 to 4.0 chars per token for English & code
    // Max of word-based and char-based estimates
    let words = text.split_whitespace().count();
    let char_est = (char_count + 3) / 4;
    words.max(char_est).max(1)
}

/// Aligns a static system prompt prefix to the nearest 1,024-token boundary
pub fn align_to_1024_tokens(text: &str) -> String {
    let tokens = estimate_tokens(text);
    if tokens == 0 {
        return String::new();
    }

    let remainder = tokens % CACHE_ALIGNMENT_TOKEN_BLOCK;
    if remainder == 0 {
        return text.to_string();
    }

    let deficit_tokens = CACHE_ALIGNMENT_TOKEN_BLOCK - remainder;
    // Pad ~4 chars per deficit token
    let pad_chars = deficit_tokens * 4;
    let marker = format!("\n\n<!-- tokless:cache-align pad_tokens={} -->\n", deficit_tokens);

    let mut out = String::with_capacity(text.len() + pad_chars + marker.len() + 16);
    out.push_str(text);
    out.push_str(&marker);

    // Fill remaining bytes cleanly with neutral comment spaces
    let marker_len = marker.len();
    if pad_chars > marker_len {
        let spaces = pad_chars - marker_len;
        out.push_str("<!-- ");
        for _ in 0..spaces {
            out.push(' ');
        }
        out.push_str(" -->\n");
    }

    out
}

/// Auto-places Anthropic `cache_control: {"type": "ephemeral"}` marker on request payload
///
/// Invariant: If any cache_control marker already exists in system, tools, or messages,
/// this function leaves the request untouched and returns `false`.
pub fn auto_place_anthropic_cache_control(body: &mut Value) -> bool {
    // Gate 1: Check if any cache_control marker already exists
    if has_cache_control_marker(body) {
        return false;
    }

    let Value::Object(map) = body else {
        return false;
    };

    // Gate 2: Prefer placing on the last tool definition (highest reuse rate across multi-turn interactions)
    if let Some(Value::Array(tools)) = map.get_mut("tools") {
        if let Some(last_tool) = tools.last_mut() {
            if let Value::Object(tool_map) = last_tool {
                tool_map.insert(
                    "cache_control".to_string(),
                    json!({ "type": "ephemeral" }),
                );
                return true;
            }
        }
    }

    // Gate 3: If no tools, place on system prompt
    if let Some(system_val) = map.get_mut("system") {
        match system_val {
            Value::Array(blocks) => {
                if let Some(last_block) = blocks.last_mut() {
                    if let Value::Object(block_map) = last_block {
                        block_map.insert(
                            "cache_control".to_string(),
                            json!({ "type": "ephemeral" }),
                        );
                        return true;
                    }
                }
            }
            Value::String(sys_str) => {
                let text = sys_str.clone();
                *system_val = json!([
                    {
                        "type": "text",
                        "text": text,
                        "cache_control": { "type": "ephemeral" }
                    }
                ]);
                return true;
            }
            _ => {}
        }
    }

    // Gate 4: Fall back to last message in conversation if history >= 2 turns
    if let Some(Value::Array(messages)) = map.get_mut("messages") {
        if messages.len() >= 2 {
            if let Some(last_msg) = messages.last_mut() {
                if let Value::Object(msg_map) = last_msg {
                    if let Some(Value::Array(content_blocks)) = msg_map.get_mut("content") {
                        if let Some(last_block) = content_blocks.last_mut() {
                            if let Value::Object(block_map) = last_block {
                                block_map.insert(
                                    "cache_control".to_string(),
                                    json!({ "type": "ephemeral" }),
                                );
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn has_cache_control_marker(val: &Value) -> bool {
    match val {
        Value::Object(map) => {
            if map.contains_key("cache_control") {
                return true;
            }
            for v in map.values() {
                if has_cache_control_marker(v) {
                    return true;
                }
            }
            false
        }
        Value::Array(arr) => arr.iter().any(has_cache_control_marker),
        _ => false,
    }
}

/// Normalizes schemas and system text for deterministic OpenAI / Gemini prompt cache hits
pub fn stabilize_cache_keys(body: &mut Value) {
    let Value::Object(map) = body else {
        return;
    };

    // Sort tool parameters properties if tools exist
    if let Some(Value::Array(tools)) = map.get_mut("tools") {
        for tool in tools {
            if let Value::Object(tool_map) = tool {
                if let Some(Value::Object(func_map)) = tool_map.get_mut("function") {
                    if let Some(Value::Object(param_map)) = func_map.get_mut("parameters") {
                        if let Some(Value::Object(prop_map)) = param_map.get_mut("properties") {
                            // Sort keys into a deterministic BTreeMap representation
                            let mut sorted_props: std::collections::BTreeMap<String, Value> =
                                std::collections::BTreeMap::new();
                            let keys: Vec<String> = prop_map.keys().cloned().collect();
                            for k in keys {
                                if let Some(v) = prop_map.remove(&k) {
                                    sorted_props.insert(k, v);
                                }
                            }
                            for (k, v) in sorted_props {
                                prop_map.insert(k, v);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Pads and normalizes system headers to 1,024-token boundaries for 90% prompt cache discounts
/// on Anthropic, OpenAI, and Gemini.
pub fn align_system_prompt_to_1024(body: &mut Value) -> bool {
    let Value::Object(map) = body else {
        return false;
    };

    // Anthropic: "system" field
    if let Some(sys_val) = map.get_mut("system") {
        match sys_val {
            Value::String(s) => {
                if !s.contains("tokless:cache-align") {
                    *s = align_to_1024_tokens(s);
                    return true;
                }
            }
            Value::Array(blocks) => {
                if let Some(first_block) = blocks.first_mut() {
                    if let Value::Object(block_map) = first_block {
                        if let Some(Value::String(text)) = block_map.get_mut("text") {
                            if !text.contains("tokless:cache-align") {
                                *text = align_to_1024_tokens(text);
                                return true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // OpenAI / Gemini: system message in messages[0]
    if let Some(Value::Array(messages)) = map.get_mut("messages") {
        if let Some(first_msg) = messages.first_mut() {
            let role = first_msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if role == "system" || role == "developer" {
                if let Some(Value::String(content)) = first_msg.get_mut("content") {
                    if !content.contains("tokless:cache-align") {
                        *content = align_to_1024_tokens(content);
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_system_prompt_to_1024() {
        let mut anthropic_req = json!({
            "model": "claude-3-7-sonnet",
            "system": "You are a helpful coding assistant.",
            "messages": [{"role": "user", "content": "hi"}]
        });
        assert!(align_system_prompt_to_1024(&mut anthropic_req));
        let sys = anthropic_req["system"].as_str().unwrap();
        assert!(sys.contains("<!-- tokless:cache-align pad_tokens="));
        assert!(estimate_tokens(sys) >= 1024);

        let mut openai_req = json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are a specialized rust expert."},
                {"role": "user", "content": "hi"}
            ]
        });
        assert!(align_system_prompt_to_1024(&mut openai_req));
        let open_sys = openai_req["messages"][0]["content"].as_str().unwrap();
        assert!(open_sys.contains("<!-- tokless:cache-align pad_tokens="));
        assert!(estimate_tokens(open_sys) >= 1024);
    }

    #[test]
    fn test_align_to_1024_tokens_padding() {
        let text = "System prompt header with basic instructions.";
        let aligned = align_to_1024_tokens(text);
        assert!(aligned.contains("<!-- tokless:cache-align pad_tokens="));
        let est = estimate_tokens(&aligned);
        assert!(est >= 1024);
    }

    #[test]
    fn test_auto_place_anthropic_cache_control() {
        let mut req = json!({
            "model": "claude-3-7-sonnet",
            "system": "You are a senior engineer.",
            "tools": [
                {
                    "name": "read_file",
                    "description": "Reads a file"
                }
            ],
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        let placed = auto_place_anthropic_cache_control(&mut req);
        assert!(placed);

        // Marker should be on last tool
        let tool = &req["tools"][0];
        assert_eq!(tool["cache_control"]["type"], "ephemeral");

        // Running again is a no-op (customer/existing placement wins)
        let placed_again = auto_place_anthropic_cache_control(&mut req);
        assert!(!placed_again);
    }
}
