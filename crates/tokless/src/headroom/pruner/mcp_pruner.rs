// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/pruner/mcp_pruner.rs

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Configuration options for dynamic MCP schema pruning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrunerConfig {
    /// Strip "$schema" URLs from JSON schemas
    pub strip_schema_url: bool,
    /// Strip redundant "title" properties matching the parameter name
    pub strip_redundant_titles: bool,
    /// Maximum character length for tool and parameter descriptions
    pub max_description_len: usize,
    /// Whitelist of allowed tool names (if Some, unlisted tools are pruned)
    pub allowed_tools: Option<Vec<String>>,
    /// Strip empty property descriptions or schemas
    pub strip_empty_fields: bool,
}

impl Default for McpPrunerConfig {
    fn default() -> Self {
        Self {
            strip_schema_url: true,
            strip_redundant_titles: true,
            max_description_len: 400,
            allowed_tools: None,
            strip_empty_fields: true,
        }
    }
}

/// Statistics reporting token savings from MCP pruning.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpPruneStats {
    pub tools_before: usize,
    pub tools_after: usize,
    pub original_tokens_est: usize,
    pub pruned_tokens_est: usize,
    pub tokens_saved: usize,
}

/// Resolves the tool name regardless of provider schema format (Anthropic vs OpenAI).
pub fn extract_tool_name(tool: &Value) -> Option<&str> {
    if let Some(name) = tool.get("name").and_then(Value::as_str) {
        return Some(name);
    }
    if let Some(name) = tool
        .get("function")
        .and_then(|f| f.get("name"))
        .and_then(Value::as_str)
    {
        return Some(name);
    }
    None
}

/// Estimates tokens for a JSON value using character ratio heuristic.
pub fn estimate_json_tokens(val: &Value) -> usize {
    let s = val.to_string();
    let char_count = s.chars().count();
    (char_count + 3) / 4
}

/// Recursively optimizes and strips redundant attributes from a JSON schema.
pub fn optimize_json_schema(schema: &mut Value, config: &McpPrunerConfig) {
    let Value::Object(map) = schema else {
        return;
    };

    if config.strip_schema_url {
        map.remove("$schema");
    }

    if let Some(desc) = map.get("description").and_then(Value::as_str).map(|s| s.to_string()) {
        if desc.chars().count() > config.max_description_len {
            let truncated: String = desc.chars().take(config.max_description_len).collect();
            map.insert("description".to_string(), json!(format!("{}...", truncated.trim())));
        }
    }

    if config.strip_empty_fields {
        if let Some(Value::Object(req)) = map.get("required") {
            if req.is_empty() {
                map.remove("required");
            }
        }
    }

    if let Some(Value::Object(properties)) = map.get_mut("properties") {
        for (prop_name, prop_val) in properties.iter_mut() {
            if let Value::Object(prop_map) = prop_val {
                if config.strip_redundant_titles {
                    if let Some(title) = prop_map.get("title").and_then(Value::as_str) {
                        if title.eq_ignore_ascii_case(prop_name) {
                            prop_map.remove("title");
                        }
                    }
                }
                if let Some(desc) = prop_map.get("description").and_then(Value::as_str).map(|s| s.to_string()) {
                    if desc.chars().count() > config.max_description_len {
                        let truncated: String = desc.chars().take(config.max_description_len).collect();
                        prop_map.insert("description".to_string(), json!(format!("{}...", truncated.trim())));
                    }
                }
                optimize_json_schema(prop_val, config);
            }
        }
    }

    if let Some(items) = map.get_mut("items") {
        optimize_json_schema(items, config);
    }
}

/// Prunes an array of tool definitions in-place, removing disallowed tools and compressing schemas.
pub fn prune_mcp_tools(tools: &mut Vec<Value>, config: &McpPrunerConfig) -> McpPruneStats {
    let tools_before = tools.len();
    let original_tokens_est: usize = tools.iter().map(estimate_json_tokens).sum();

    if let Some(ref allowed) = config.allowed_tools {
        tools.retain(|tool| {
            if let Some(name) = extract_tool_name(tool) {
                allowed.iter().any(|a| a == name)
            } else {
                true
            }
        });
    }

    for tool in tools.iter_mut() {
        if let Value::Object(map) = tool {
            if let Some(desc) = map.get("description").and_then(Value::as_str).map(|s| s.to_string()) {
                if desc.chars().count() > config.max_description_len {
                    let truncated: String = desc.chars().take(config.max_description_len).collect();
                    map.insert("description".to_string(), json!(format!("{}...", truncated.trim())));
                }
            }

            // Anthropic input_schema
            if let Some(schema) = map.get_mut("input_schema") {
                optimize_json_schema(schema, config);
            }

            // OpenAI function parameters
            if let Some(Value::Object(func)) = map.get_mut("function") {
                if let Some(desc) = func.get("description").and_then(Value::as_str).map(|s| s.to_string()) {
                    if desc.chars().count() > config.max_description_len {
                        let truncated: String = desc.chars().take(config.max_description_len).collect();
                        func.insert("description".to_string(), json!(format!("{}...", truncated.trim())));
                    }
                }
                if let Some(params) = func.get_mut("parameters") {
                    optimize_json_schema(params, config);
                }
            }
        }
    }

    let tools_after = tools.len();
    let pruned_tokens_est: usize = tools.iter().map(estimate_json_tokens).sum();
    let tokens_saved = original_tokens_est.saturating_sub(pruned_tokens_est);

    McpPruneStats {
        tools_before,
        tools_after,
        original_tokens_est,
        pruned_tokens_est,
        tokens_saved,
    }
}

/// Dynamically prunes tools in an agent request payload.
pub fn prune_mcp_request_payload(payload: &mut Value, config: &McpPrunerConfig) -> Option<McpPruneStats> {
    let Value::Object(map) = payload else {
        return None;
    };

    if let Some(Value::Array(tools)) = map.get_mut("tools") {
        let stats = prune_mcp_tools(tools, config);
        Some(stats)
    } else {
        None
    }
}
