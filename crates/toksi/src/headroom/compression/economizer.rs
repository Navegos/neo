// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/compression/economizer.rs
// created: 2026-11-09
// lastModified: 2026-11-16

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

/// Stage 2 distiller collapsing repetitive environment metadata blocks (OS/PATH/PWD dumps).
#[derive(Debug, Clone, Default)]
pub struct EnvFactsDistiller {
    seen_facts: HashSet<String>,
}

impl EnvFactsDistiller {
    pub fn distill(&mut self, text: &str) -> String {
        let mut out = Vec::with_capacity(text.lines().count());
        for line in text.lines() {
            let trimmed = line.trim();
            let is_env_fact = trimmed.starts_with("OS=")
                || trimmed.starts_with("PWD=")
                || trimmed.starts_with("SHELL=")
                || trimmed.starts_with("PATH=");
            if is_env_fact
                && !self.seen_facts.insert(trimmed.to_string()) {
                    continue;
                }
            out.push(line);
        }
        out.join("\n")
    }
}

/// Stage 3 loop detector suppressing repeated tool-loop outputs across consecutive turns.
#[derive(Debug, Clone)]
pub struct LoopGuard {
    threshold: usize,
}

impl Default for LoopGuard {
    fn default() -> Self {
        Self { threshold: 3 }
    }
}

impl LoopGuard {
    pub fn new(threshold: usize) -> Self {
        Self { threshold: threshold.max(2) }
    }

    pub fn guard_lines(&self, text: &str) -> (String, bool) {
        let mut out = Vec::new();
        let mut repeat = 0usize;
        let mut prev = "";
        let mut triggered = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed == prev {
                repeat += 1;
                if repeat >= self.threshold {
                    triggered = true;
                    continue;
                }
            } else {
                repeat = 0;
                prev = trimmed;
            }
            out.push(line);
        }
        (out.join("\n"), triggered)
    }
}

/// Dynamic 3-stage Token Economizer implementing prompt minification, environment-fact
/// distillation, instruction deduplication, and loop-guard collapse.
pub struct TokenEconomizer {
    options: EconomizerOptions,
    seen_instructions: HashSet<String>,
    env_distiller: EnvFactsDistiller,
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
            env_distiller: EnvFactsDistiller::default(),
        }
    }

    /// Reset seen instruction and environment-fact history between separate conversations.
    pub fn reset_history(&mut self) {
        self.seen_instructions.clear();
        self.env_distiller = EnvFactsDistiller::default();
    }

    /// 3-stage pipeline: (1) EnvFactsDistiller, (2) LoopGuard, (3) prompt minification + dedupe.
    pub fn economize_prompt(&mut self, prompt: &str) -> (String, EconomizerReport) {
        if prompt.trim().is_empty() {
            return (
                String::new(),
                EconomizerReport {
                    original_chars: prompt.len(),
                    economized_chars: 0,
                    original_estimated_tokens: 0,
                    economized_estimated_tokens: 0,
                    tokens_saved: 0,
                    duplicate_instructions_removed: 0,
                    loop_detected: false,
                },
            );
        }
        let distilled = self.env_distiller.distill(prompt);
        let guard = LoopGuard::new(self.options.loop_guard_threshold);
        let (guarded, guard_hit) = guard.guard_lines(&distilled);
        let orig_chars = prompt.len();
        let orig_tokens = (orig_chars / 4) as u64;

        let mut lines = Vec::new();
        let mut dupes_removed = 0usize;
        let mut loop_detected = guard_hit;
        let mut repeat_counter = 0usize;
        let mut last_line = "";

        for raw_line in guarded.lines() {
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
            if self.options.strip_redundant_comments
                && trimmed.starts_with("//") && !trimmed.starts_with("///") && trimmed.len() < 80 {
                    let comment_body = trimmed.trim_start_matches('/').trim().to_ascii_lowercase();
                    if comment_body.contains("todo") || comment_body.contains("fixme") || comment_body.contains("note") {
                        // keep significant comments
                    } else if comment_body.len() < 30 {
                        continue;
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

    /// Runs the 3-stage economizer followed by deterministic LLMLingua-style entropy pruning
    /// (preserving code blocks, numbers, identifiers, and negation words) to approach `target_ratio`.
    pub fn economize_with_entropy(&mut self, prompt: &str, target_ratio: f32) -> (String, EconomizerReport) {
        let (stage2, mut report) = self.economize_prompt(prompt);
        if stage2.is_empty() {
            return (stage2, report);
        }
        let pruned = llmlingua_entropy_prune(&stage2, target_ratio);
        report.economized_chars = pruned.len();
        report.economized_estimated_tokens = (pruned.len() / 4) as u64;
        report.tokens_saved = report
            .original_estimated_tokens
            .saturating_sub(report.economized_estimated_tokens);
        (pruned, report)
    }
}

/// Deterministic LLMLingua-inspired token-entropy pruner.
/// Drops low-information English filler/stopwords in prose lines while preserving fenced code blocks,
/// JSON lines, numbers, identifiers, and negation words (`not`, `never`, `no`, `only`, `except`).
pub fn llmlingua_entropy_prune(text: &str, target_ratio: f32) -> String {
    if text.trim().is_empty() || target_ratio >= 0.99 {
        return text.to_string();
    }
    let ratio = target_ratio.clamp(0.25, 0.99);
    const LOW_ENTROPY_FILLER: &[&str] = &[
        "actually", "basically", "really", "simply", "just", "very", "quite", "rather",
        "somewhat", "please", "kindly", "note", "that", "basically,", "actually,",
    ];
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            out.push(line.to_string());
            continue;
        }
        if in_fence || trimmed.starts_with('{') || trimmed.starts_with('[') || trimmed.len() < 40 {
            out.push(line.to_string());
            continue;
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        let target_words = ((words.len() as f32) * ratio).ceil().max(1.0) as usize;
        let mut kept = Vec::with_capacity(words.len());
        let mut dropped = 0usize;
        let max_drop = words.len().saturating_sub(target_words);
        for w in &words {
            let low = w.to_ascii_lowercase();
            let is_negation = matches!(low.as_str(), "not" | "never" | "no" | "only" | "except");
            if !is_negation && dropped < max_drop && LOW_ENTROPY_FILLER.contains(&low.as_str()) {
                dropped += 1;
                continue;
            }
            kept.push(*w);
        }
        out.push(kept.join(" "));
    }
    out.join("\n")
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
        // Strip or truncate verbose description fields in tool parameters (UTF-8 char-boundary safe)
        if let Some(desc) = map.get_mut("description")
            && let Some(s) = desc.as_str()
                && s.chars().count() > 120 {
                    let prefix: String = s.chars().take(100).collect();
                    let truncated = format!("{}...", prefix);
                    *desc = serde_json::Value::String(truncated);
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

/// 11 supported tool-output compaction formats for `ToolCompact`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCompactFormat {
    Json,
    Yaml,
    Toml,
    Xml,
    Csv,
    Tsv,
    MarkdownTable,
    Diff,
    Log,
    StackTrace,
    ShellOutput,
}

impl ToolCompactFormat {
    pub const ALL: [ToolCompactFormat; 11] = [
        Self::Json,
        Self::Yaml,
        Self::Toml,
        Self::Xml,
        Self::Csv,
        Self::Tsv,
        Self::MarkdownTable,
        Self::Diff,
        Self::Log,
        Self::StackTrace,
        Self::ShellOutput,
    ];
}

/// Compacts verbose tool outputs across 11 structured and semi-structured formats.
pub struct ToolCompact;

impl ToolCompact {
    pub fn compact(format: ToolCompactFormat, input: &str) -> String {
        if input.trim().is_empty() {
            return String::new();
        }
        match format {
            ToolCompactFormat::Json => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(input) {
                    serde_json::to_string(&val).unwrap_or_else(|_| input.trim().to_string())
                } else {
                    collapse_line_whitespace(input.trim())
                }
            }
            ToolCompactFormat::Xml => input
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(""),
            ToolCompactFormat::Yaml | ToolCompactFormat::Toml => input
                .lines()
                .map(str::trim_end)
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with('#')
                })
                .collect::<Vec<_>>()
                .join("\n"),
            ToolCompactFormat::Csv | ToolCompactFormat::Tsv => input
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
            ToolCompactFormat::MarkdownTable => input
                .lines()
                .map(|l| {
                    l.split('|')
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join("|")
                })
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
            ToolCompactFormat::Diff => input
                .lines()
                .filter(|l| {
                    l.starts_with("+++")
                        || l.starts_with("---")
                        || l.starts_with("@@")
                        || l.starts_with('+')
                        || l.starts_with('-')
                })
                .collect::<Vec<_>>()
                .join("\n"),
            ToolCompactFormat::Log | ToolCompactFormat::ShellOutput => {
                let (_, guarded) = {
                    let g = LoopGuard::default();
                    let (out, _) = g.guard_lines(input);
                    ((), out)
                };
                guarded
                    .lines()
                    .map(str::trim_end)
                    .filter(|l| !l.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            ToolCompactFormat::StackTrace => input
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .take(24)
                .collect::<Vec<_>>()
                .join("\n"),
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
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "This is an extremely verbose explanation that goes on and on describing in exhaustive detail how the code string parameter should be supplied by the model to satisfy the compiler toolchain requirements."
                }
            }
        });
        let pruned = econ.prune_json_schema(&schema);
        let desc = pruned["properties"]["code"]["description"].as_str().unwrap_or("");
        assert!(desc.ends_with("..."));
        assert!(desc.len() <= 105);
    }

    #[test]
    fn test_empty_prompt_utf8_schema_and_tool_compact_11_formats() {
        let mut econ = TokenEconomizer::default();
        let (out, rep) = econ.economize_prompt("   ");
        assert!(out.is_empty());
        assert_eq!(rep.tokens_saved, 0);

        // Multi-byte UTF-8 truncation safety
        let long_utf8 = "🚀".repeat(130);
        let schema = serde_json::json!({ "description": long_utf8 });
        let pruned = econ.prune_json_schema(&schema);
        assert!(pruned["description"].as_str().unwrap_or("").ends_with("..."));

        assert_eq!(ToolCompactFormat::ALL.len(), 11);
        let compacted_json = ToolCompact::compact(ToolCompactFormat::Json, "{\n  \"a\": 1,\n  \"b\": 2\n}");
        assert_eq!(compacted_json, "{\"a\":1,\"b\":2}");
        let compacted_diff = ToolCompact::compact(ToolCompactFormat::Diff, "context\n+added\n-removed\ncontext");
        assert_eq!(compacted_diff, "+added\n-removed");
    }

    #[test]
    fn test_llmlingua_entropy_prune_preserves_negations_and_code() {
        let mut econ = TokenEconomizer::default();
        let prompt = "Please really basically just check that we do not ever skip verification here.\n```rust\nlet x = 1;\n```";
        let (pruned, rep) = econ.economize_with_entropy(prompt, 0.6);
        assert!(pruned.contains("not"));
        assert!(pruned.contains("let x = 1;"));
        assert!(rep.economized_chars <= rep.original_chars);
    }
}
