// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/filters/toml.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use regex::{Regex, RegexSet};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::rtk::ansi::strip_ansi;

const BUILTIN_TOML: &str = include_str!(concat!(env!("OUT_DIR"), "/builtin_filters.toml"));

#[derive(Debug, Clone, Deserialize)]
pub struct MatchOutputRule {
    pub pattern: String,
    pub message: String,
    #[serde(default)]
    pub unless: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplaceRule {
    pub pattern: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterTest {
    pub input: String,
    pub expected: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TomlFilterDef {
    pub match_command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub filter_stderr: bool,
    #[serde(default)]
    pub replace: Vec<ReplaceRule>,
    #[serde(default)]
    pub match_output: Vec<MatchOutputRule>,
    #[serde(default)]
    pub strip_lines_matching: Vec<String>,
    #[serde(default)]
    pub keep_lines_matching: Vec<String>,
    #[serde(default)]
    pub truncate_lines_at: Option<usize>,
    #[serde(default)]
    pub head_lines: Option<usize>,
    #[serde(default)]
    pub tail_lines: Option<usize>,
    #[serde(default)]
    pub max_lines: Option<usize>,
    #[serde(default)]
    pub on_empty: Option<String>,
    #[serde(default)]
    pub tests: Vec<FilterTest>,
}

#[derive(Debug, Clone, Deserialize)]
struct TomlFilterFile {
    pub schema_version: u32,
    #[serde(default)]
    pub filters: BTreeMap<String, TomlFilterDef>,
}

pub struct CompiledReplaceRule {
    pub pattern: Regex,
    pub replacement: String,
}

pub struct CompiledMatchOutputRule {
    pub pattern: Regex,
    pub message: String,
    pub unless: Option<Regex>,
}

pub struct CompiledFilter {
    pub name: String,
    pub match_command: Regex,
    pub filter_stderr: bool,
    pub replace: Vec<CompiledReplaceRule>,
    pub match_output: Vec<CompiledMatchOutputRule>,
    pub strip_lines: Option<RegexSet>,
    pub keep_lines: Option<RegexSet>,
    pub truncate_lines_at: Option<usize>,
    pub head_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub on_empty: Option<String>,
}

pub struct TomlRegistry {
    pub filters: Vec<CompiledFilter>,
}

impl TomlRegistry {
    pub fn init() -> Self {
        let file = match toml::from_str::<TomlFilterFile>(BUILTIN_TOML) {
            Ok(f) => {
                if f.schema_version != 1 {
                    eprintln!("[rtk] warning: unexpected schema_version {}", f.schema_version);
                }
                f
            }
            Err(e) => {
                eprintln!("[rtk] warning: failed to parse builtin filters TOML: {e}");
                return Self { filters: Vec::new() };
            }
        };

        let mut compiled = Vec::with_capacity(file.filters.len());
        for (name, def) in file.filters {
            let match_cmd_re = match Regex::new(&def.match_command) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[rtk] warning: invalid regex in filter '{}': {}", name, e);
                    continue;
                }
            };

            let mut replace_rules = Vec::new();
            for r in def.replace {
                if let Ok(pat) = Regex::new(&r.pattern) {
                    replace_rules.push(CompiledReplaceRule {
                        pattern: pat,
                        replacement: r.replacement,
                    });
                }
            }

            let mut match_out_rules = Vec::new();
            for m in def.match_output {
                if let Ok(pat) = Regex::new(&m.pattern) {
                    let unless_pat = m.unless.as_deref().and_then(|u| Regex::new(u).ok());
                    match_out_rules.push(CompiledMatchOutputRule {
                        pattern: pat,
                        message: m.message,
                        unless: unless_pat,
                    });
                }
            }

            let strip_lines = if !def.strip_lines_matching.is_empty() {
                RegexSet::new(&def.strip_lines_matching).ok()
            } else {
                None
            };

            let keep_lines = if !def.keep_lines_matching.is_empty() {
                RegexSet::new(&def.keep_lines_matching).ok()
            } else {
                None
            };

            compiled.push(CompiledFilter {
                name,
                match_command: match_cmd_re,
                filter_stderr: def.filter_stderr,
                replace: replace_rules,
                match_output: match_out_rules,
                strip_lines,
                keep_lines,
                truncate_lines_at: def.truncate_lines_at,
                head_lines: def.head_lines,
                tail_lines: def.tail_lines,
                max_lines: def.max_lines,
                on_empty: def.on_empty,
            });
        }

        Self { filters: compiled }
    }
}

pub static TOML_REGISTRY: LazyLock<TomlRegistry> = LazyLock::new(TomlRegistry::init);

/// Find a matching TOML filter for a given command line string.
pub fn find_matching_filter(command: &str) -> Option<&'static CompiledFilter> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    TOML_REGISTRY
        .filters
        .iter()
        .find(|f| f.match_command.is_match(trimmed))
}

/// Check if a command line matches any declarative TOML filter.
pub fn command_matches_filter(command: &str) -> bool {
    find_matching_filter(command).is_some()
}

/// Get all compiled filter names.
pub fn list_filter_names() -> Vec<&'static str> {
    TOML_REGISTRY.filters.iter().map(|f| f.name.as_str()).collect()
}

/// Apply a compiled TOML filter across its 8 pipeline stages.
pub fn apply_filter(filter: &CompiledFilter, stdout: &str) -> String {
    // Stage 1: strip ANSI
    let clean = strip_ansi(stdout);

    // Stage 2: replace rules
    let mut text = clean;
    for rule in &filter.replace {
        text = rule.pattern.replace_all(&text, &rule.replacement).to_string();
    }

    // Stage 3: match_output (short-circuit on whole blob)
    for rule in &filter.match_output {
        if rule.pattern.is_match(&text) {
            let is_unless = rule.unless.as_ref().is_some_and(|u| u.is_match(&text));
            if !is_unless {
                return rule.message.clone();
            }
        }
    }

    // Split lines for line-based pipeline stages
    let mut lines: Vec<&str> = text.lines().collect();

    // Stage 4: strip_lines / keep_lines
    if let Some(strip_set) = &filter.strip_lines {
        lines.retain(|l| !strip_set.is_match(l));
    } else if let Some(keep_set) = &filter.keep_lines {
        lines.retain(|l| keep_set.is_match(l));
    }

    // Stage 5: truncate_lines_at
    let mut processed_lines: Vec<String> = if let Some(max_char) = filter.truncate_lines_at {
        lines
            .into_iter()
            .map(|l| {
                if l.chars().count() > max_char {
                    format!("{}...", l.chars().take(max_char.saturating_sub(3)).collect::<String>())
                } else {
                    l.to_string()
                }
            })
            .collect()
    } else {
        lines.into_iter().map(String::from).collect()
    };

    // Stage 6: head_lines / tail_lines
    if let Some(head) = filter.head_lines {
        if processed_lines.len() > head {
            let omitted = processed_lines.len() - head;
            processed_lines.truncate(head);
            processed_lines.push(format!("[... omitted {} lines]", omitted));
        }
    } else if let Some(tail) = filter.tail_lines {
        if processed_lines.len() > tail {
            let omitted = processed_lines.len() - tail;
            let start = processed_lines.len() - tail;
            let mut new_lines = vec![format!("[... omitted {} lines]", omitted)];
            new_lines.extend_from_slice(&processed_lines[start..]);
            processed_lines = new_lines;
        }
    }

    // Stage 7: max_lines cap
    if let Some(max_l) = filter.max_lines {
        if processed_lines.len() > max_l {
            let omitted = processed_lines.len() - max_l;
            processed_lines.truncate(max_l);
            processed_lines.push(format!("[... capped: {} lines omitted]", omitted));
        }
    }

    // Stage 8: on_empty
    if processed_lines.is_empty() {
        if let Some(on_empty) = &filter.on_empty {
            return on_empty.clone();
        }
    }

    processed_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_filters_loaded() {
        assert!(!TOML_REGISTRY.filters.is_empty());
        assert!(TOML_REGISTRY.filters.len() >= 50);
    }

    #[test]
    fn test_command_matching() {
        assert!(command_matches_filter("ps aux"));
        assert!(command_matches_filter("df -h"));
        assert!(command_matches_filter("make test"));
        assert!(command_matches_filter("shellcheck script.sh"));
        assert!(command_matches_filter("terraform plan"));
        assert!(!command_matches_filter("nonexistent_random_cmd_12345"));
    }

    #[test]
    fn test_apply_filter_stages() {
        let filter = find_matching_filter("df -h").expect("df filter should exist");
        let raw_output = "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1        50G   20G   30G  40% /\ntmpfs           1.0G     0  1.0G   0% /dev/shm\n";
        let filtered = apply_filter(filter, raw_output);
        assert!(!filtered.is_empty());
    }
}
