// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/polyglot.rs
// created: 2026-11-13

use super::buffers::EmitOut;
use super::tsjs;
use super::{FileExtraction, SymbolNode};
use crate::util::paths::normalize_path;

/// Extracts Vue Single File Components (.vue) by isolating <script> contents with line padding
pub fn extract_vue(file_path: &str, content: &str) -> Result<EmitOut, String> {
    extract_sfc_script(file_path, content, "vue")
}

/// Extracts Svelte Single File Components (.svelte) by isolating <script> contents with line padding
pub fn extract_svelte(file_path: &str, content: &str) -> Result<EmitOut, String> {
    extract_sfc_script(file_path, content, "svelte")
}

fn extract_sfc_script(file_path: &str, content: &str, lang: &str) -> Result<EmitOut, String> {
    let lower = content.to_ascii_lowercase();
    let script_start_marker = "<script";
    let script_end_marker = "</script>";

    if let Some(start_idx) = lower.find(script_start_marker) {
        if let Some(tag_close) = lower[start_idx..].find('>') {
            let body_start = start_idx + tag_close + 1;
            let body_end = lower[body_start..]
                .find(script_end_marker)
                .map(|e| body_start + e)
                .unwrap_or(content.len());

            // Build padded source: preserve exact line numbers by replacing preceding chars with spaces/newlines
            let mut padded = String::with_capacity(content.len());
            for (idx, ch) in content.char_indices() {
                if idx < body_start {
                    if ch == '\n' {
                        padded.push('\n');
                    } else {
                        padded.push(' ');
                    }
                } else if idx < body_end {
                    padded.push(ch);
                } else if ch == '\n' {
                    padded.push('\n');
                } else {
                    padded.push(' ');
                }
            }

            let script_lang = if lower[start_idx..start_idx + tag_close].contains("ts") {
                "typescript"
            } else {
                "javascript"
            };

            return tsjs::extract(file_path, &padded, script_lang);
        }
    }

    Err(format!("no script block found in {}", lang))
}

/// Fallback structural outline extractor for polyglot languages without C Tree-sitter parsers
pub fn extract_structural(file_path: &str, content: &str, language: &str) -> FileExtraction {
    let norm_path = normalize_path(file_path);
    let mut nodes = Vec::new();

    let mut current_doc = String::new();
    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = (idx + 1) as u32;
        let trimmed = line.trim();

        // Capture doc comments
        if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**") || trimmed.starts_with('*') || trimmed.starts_with("##") || trimmed.starts_with("#!") {
            let doc_line = trimmed.trim_start_matches(|c| c == '/' || c == '*' || c == '!' || c == '#').trim();
            if !doc_line.is_empty() {
                if !current_doc.is_empty() {
                    current_doc.push(' ');
                }
                current_doc.push_str(doc_line);
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        let docstring = if current_doc.is_empty() {
            None
        } else {
            let d = current_doc.clone();
            current_doc.clear();
            Some(d)
        };

        if let Some((name, kind, signature)) = parse_line_symbol(trimmed, language) {
            let col = (line.find(&name).unwrap_or(0) + 1) as u32;
            let id = format!("{}:{}:{}", norm_path, kind, line_num);
            let qn = format!("{}::{}", norm_path, name);

            nodes.push(SymbolNode {
                id,
                name: name.clone(),
                qualified_name: qn,
                kind: kind.to_string(),
                visibility: Some("public".to_string()),
                file_path: norm_path.clone(),
                start_line: line_num,
                end_line: line_num,
                start_column: col,
                end_column: col + name.len() as u32,
                docstring,
                signature: Some(signature),
                return_type: None,
            });
        }
    }

    FileExtraction {
        nodes,
        edges: Vec::new(),
        refs: Vec::new(),
    }
}

fn parse_line_symbol(line: &str, lang: &str) -> Option<(String, &'static str, String)> {
    match lang {
        "zig" => {
            if line.starts_with("pub fn ") || line.starts_with("fn ") {
                let rest = line.trim_start_matches("pub ").trim_start_matches("fn ");
                let name = rest.split('(').next()?.trim();
                if !name.is_empty() {
                    return Some((name.to_string(), "function", line.to_string()));
                }
            } else if line.contains("struct {") {
                let name = line.split('=').next()?.trim().split_whitespace().last()?;
                return Some((name.to_string(), "struct", line.to_string()));
            }
        }
        "julia" => {
            if line.starts_with("function ") {
                let name = line["function ".len()..].split('(').next()?.trim();
                return Some((name.to_string(), "function", line.to_string()));
            } else if line.starts_with("struct ") {
                let name = line["struct ".len()..].split_whitespace().next()?.trim();
                return Some((name.to_string(), "struct", line.to_string()));
            } else if line.starts_with("module ") {
                let name = line["module ".len()..].split_whitespace().next()?.trim();
                return Some((name.to_string(), "module", line.to_string()));
            }
        }
        "elixir" => {
            if line.starts_with("def ") || line.starts_with("defp ") {
                let rest = line.split_whitespace().nth(1)?;
                let name = rest.split('(').next()?.trim();
                return Some((name.to_string(), "function", line.to_string()));
            } else if line.starts_with("defmodule ") {
                let name = line["defmodule ".len()..].split_whitespace().next()?.trim();
                return Some((name.to_string(), "module", line.to_string()));
            }
        }
        "erlang" => {
            if line.starts_with("-module(") {
                let name = line.trim_start_matches("-module(").trim_end_matches(").").trim();
                return Some((name.to_string(), "module", line.to_string()));
            } else if line.contains("->") && !line.starts_with('%') {
                let name = line.split('(').next()?.trim();
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Some((name.to_string(), "function", line.to_string()));
                }
            }
        }
        "bash" => {
            if line.starts_with("function ") {
                let name = line["function ".len()..].split('(').next()?.trim();
                return Some((name.to_string(), "function", line.to_string()));
            } else if line.contains("()") && (line.ends_with('{') || line.ends_with("()")) {
                let name = line.split("()").next()?.trim();
                if !name.is_empty() && !name.contains(' ') {
                    return Some((name.to_string(), "function", line.to_string()));
                }
            }
        }
        "sql" => {
            let upper = line.to_ascii_uppercase();
            if upper.starts_with("CREATE TABLE ") {
                let name = line.split_whitespace().nth(2)?.trim_matches('(').trim();
                return Some((name.to_string(), "table", line.to_string()));
            } else if upper.starts_with("CREATE FUNCTION ") || upper.starts_with("CREATE OR REPLACE FUNCTION ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&p| p.eq_ignore_ascii_case("FUNCTION")) {
                    if let Some(n) = parts.get(pos + 1) {
                        let name = n.split('(').next()?.trim();
                        return Some((name.to_string(), "function", line.to_string()));
                    }
                }
            } else if upper.starts_with("CREATE PROCEDURE ") {
                let name = line.split_whitespace().nth(2)?.split('(').next()?.trim();
                return Some((name.to_string(), "procedure", line.to_string()));
            }
        }
        "protobuf" => {
            if line.starts_with("message ") {
                let name = line["message ".len()..].split('{').next()?.trim();
                return Some((name.to_string(), "message", line.to_string()));
            } else if line.starts_with("service ") {
                let name = line["service ".len()..].split('{').next()?.trim();
                return Some((name.to_string(), "service", line.to_string()));
            } else if line.starts_with("rpc ") {
                let name = line["rpc ".len()..].split('(').next()?.trim();
                return Some((name.to_string(), "rpc", line.to_string()));
            }
        }
        "hcl" => {
            if line.starts_with("resource ") || line.starts_with("data ") {
                let parts: Vec<&str> = line.split('"').collect();
                if parts.len() >= 4 {
                    let name = format!("{}.{}", parts[1], parts[3]);
                    return Some((name, "resource", line.to_string()));
                }
            } else if line.starts_with("variable ") {
                let name = line.split('"').nth(1)?.trim();
                return Some((name.to_string(), "variable", line.to_string()));
            }
        }
        "dockerfile" => {
            if line.starts_with("FROM ") {
                let name = line["FROM ".len()..].trim();
                return Some((name.to_string(), "image", line.to_string()));
            }
        }
        "markdown" => {
            if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
                let name = line.trim_start_matches('#').trim();
                return Some((name.to_string(), "section", line.to_string()));
            }
        }
        _ => {
            // General heuristic: function/class/struct
            if line.starts_with("class ") || line.starts_with("struct ") || line.starts_with("interface ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim_matches('{').trim_matches(':').trim();
                    let kind = match parts[0] {
                        "class" => "class",
                        "struct" => "struct",
                        "interface" => "interface",
                        _ => "type",
                    };
                    return Some((name.to_string(), kind, line.to_string()));
                }
            } else if line.starts_with("fn ") || line.starts_with("def ") || line.starts_with("func ") || line.starts_with("function ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].split('(').next()?.trim();
                    return Some((name.to_string(), "function", line.to_string()));
                }
            }
        }
    }
    None
}
