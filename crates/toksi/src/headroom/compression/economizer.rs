// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/compression/economizer.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EconomizerError {
    #[error("JSON minification error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid prompt configuration")]
    InvalidConfig,
}

/// Token economizer options and aggressiveness controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomizerOptions {
    pub deduplicate_instructions: bool,
    pub minify_whitespace: bool,
    pub strip_redundant_comments: bool,
    pub prune_schema_verbose_descriptions: bool,
    pub loop_guard_threshold: usize,
}

impl Default for EconomizerOptions {
    fn default() -> Self {
        Self {
            deduplicate_instructions: true,
            minify_whitespace: true,
            strip_redundant_comments: true,
            prune_schema_verbose_descriptions: true,
            loop_guard_threshold: 3,
        }
    }
}

/// Report summarizing prompt optimization and token economization savings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomizerReport {
    pub original_chars: usize,
    pub economized_chars: usize,
    pub original_estimated_tokens: u64,
    pub economized_estimated_tokens: u64,
    pub tokens_saved: u64,
    pub duplicate_instructions_removed: usize,
    pub loop_detected: bool,
}

/// Dynamic Token Economizer implementing prompt minification and instruction deduplication.
pub struct TokenEconomizer {
    options: EconomizerOptions,
    seen_instructions: HashSet<String>,
}

impl Default for TokenEconomizer {
    fn default() -> Self {
        Self::new(EconomizerOptions::default())
    }
}

impl TokenEconomizer {
    pub fn new(options: EconomizerOptions) -> Self {
        Self {
            options,
            seen_instructions: HashSet::new(),
        }
    }

    /// Reset seen instruction history between separate conversations.
    pub fn reset_history(&mut self) {
        self.seen_instructions.clear();
    }

    /// Economizes a multi-line prompt string.
    pub fn economize_prompt(&mut self, prompt: &str) -> (String, EconomizerReport) {
        let orig_chars = prompt.len();
        let orig_tokens = (orig_chars / 4) as u64;

        let mut lines = Vec::new();
        let mut dupes_removed = 0usize;
        let mut loop_detected = false;
        let mut repeat_counter = 0usize;
        let mut last_line = "";

        for raw_line in prompt.lines() {
            let trimmed = raw_line.trim();

            if trimmed.is_empty() {
                // Collapse multiple empty lines
                if lines.last().map(|s: &String| s.is_empty()).unwrap_or(false) {
                    continue;
                }
                lines.push(String::new());
                continue;
            }

            // Loop guard check
            if trimmed == last_line {
                repeat_counter += 1;
                if repeat_counter >= self.options.loop_guard_threshold {
                    loop_detected = true;
                    continue; // drop circular repetition
                }
            } else {
                repeat_counter = 0;
                last_line = trimmed;
            }

            // Redundant comments
            if self.options.strip_redundant_comments {
                if trimmed.starts_with("//") && !trimmed.starts_with("///") && trimmed.len() < 80 {
                    let comment_body = trimmed.trim_start_matches('/').trim().to_ascii_lowercase();
                    if comment_body.contains("todo") || comment_body.contains("fixme") || comment_body.contains("note") {
                        // keep significant comments
                    } else if comment_body.len() < 30 {
                        continue;
                    }
                }
            }

            // Deduplicate repetitive instructions across turns
            if self.options.deduplicate_instructions && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) && trimmed.len() > 15 {
                let norm_instruction = trimmed.to_ascii_lowercase();
                if self.seen_instructions.contains(&norm_instruction) {
                    dupes_removed += 1;
                    continue;
                } else {
                    self.seen_instructions.insert(norm_instruction);
                }
            }

            if self.options.minify_whitespace {
                lines.push(collapse_line_whitespace(trimmed));
            } else {
                lines.push(trimmed.to_string());
            }
        }

        let economized_text = lines.join("\n");
        let econ_chars = economized_text.len();
        let econ_tokens = (econ_chars / 4) as u64;
        let tokens_saved = orig_tokens.saturating_sub(econ_tokens);

        let report = EconomizerReport {
            original_chars: orig_chars,
            economized_chars: econ_chars,
            original_estimated_tokens: orig_tokens,
            economized_estimated_tokens: econ_tokens,
            tokens_saved,
            duplicate_instructions_removed: dupes_removed,
            loop_detected,
        };

        (economized_text, report)
    }

    /// Prunes verbose descriptions from structured JSON schemas.
    pub fn prune_json_schema(&self, schema_json: &serde_json::Value) -> serde_json::Value {
        if !self.options.prune_schema_verbose_descriptions {
            return schema_json.clone();
        }

        let mut val = schema_json.clone();
        prune_schema_recursive(&mut val);
        val
    }
}

fn collapse_line_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out
}

fn prune_schema_recursive(val: &mut serde_json::Value) {
    if let serde_json::Value::Object(map) = val {
        // Strip or truncate verbose description fields in tool parameters
        if let Some(desc) = map.get_mut("description") {
            if let Some(s) = desc.as_str() {
                if s.len() > 120 {
                    let truncated = format!("{}...", &s[..100]);
                    *desc = serde_json::Value::String(truncated);
                }
            }
        }
        for (_, v) in map.iter_mut() {
            prune_schema_recursive(v);
        }
    } else if let serde_json::Value::Array(arr) = val {
        for v in arr {
            prune_schema_recursive(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economizer_dedup_and_loop_guard() {
        let mut econ = TokenEconomizer::default();
        let prompt = r#"
Please analyze this project.

- Always follow Rust edition 2024 standards.
- Always follow Rust edition 2024 standards.

Loop line
Loop line
Loop line
Loop line
Loop line

End of prompt.
"#;
        let (res, report) = econ.economize_prompt(prompt);
        assert!(report.tokens_saved > 0);
        assert!(report.duplicate_instructions_removed >= 1);
        assert!(report.loop_detected);
        assert!(!res.contains("Loop line\nLoop line\nLoop line\nLoop line"));
    }

    #[test]
    fn test_prune_schema_verbose_descriptions() {
        let econ = TokenEconomizer::default();
        let mut schema = serde_json::json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "This is an extremely verbose explanation that goes on and on describing in exhaustive detail how the code string parameter should be supplied by the model to satisfy the compiler toolchain requirements."
                }
            }
        });
        let pruned = econ.prune_json_schema(&mut schema);
        let desc = pruned["properties"]["code"]["description"].as_str().unwrap();
        assert!(desc.ends_with("..."));
        assert!(desc.len() <= 105);
    }
}
