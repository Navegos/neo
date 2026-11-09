// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/jcodemunch.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum JCodeMunchError {
    #[error("I/O error reading file '{0}': {1}")]
    Io(String, #[source] std::io::Error),
    #[error("File '{0}' not found or inaccessible")]
    FileNotFound(String),
    #[error("Unsupported language '{0}' for AST chunking")]
    UnsupportedLanguage(String),
}

/// Category/Kind of extracted code symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Interface,
    Trait,
    Module,
    Constant,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Class => "class",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Interface => "interface",
            Self::Trait => "trait",
            Self::Module => "module",
            Self::Constant => "constant",
        }
    }
}

/// Outline record for a single code symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolOutline {
    pub name: String,
    pub kind: SymbolKind,
    pub signature: String,
    pub docstring: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub parent_symbol: Option<String>,
    pub visibility: String,
}

/// Extracted outline metadata for a single source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutline {
    pub file_path: String,
    pub language: String,
    pub raw_bytes: usize,
    pub outline_bytes: usize,
    pub tokens_saved: u64,
    pub symbols: Vec<SymbolOutline>,
}

/// Summary outline for an entire repository or directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOutline {
    pub root_dir: String,
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_raw_bytes: usize,
    pub total_outline_bytes: usize,
    pub estimated_tokens_saved: u64,
    pub files: Vec<FileOutline>,
}

/// Detects language from file extension.
pub fn detect_language(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        "rust"
    } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
        "typescript"
    } else if lower.ends_with(".js") || lower.ends_with(".jsx") || lower.ends_with(".mjs") || lower.ends_with(".cjs") {
        "javascript"
    } else if lower.ends_with(".py") {
        "python"
    } else if lower.ends_with(".go") {
        "go"
    } else if lower.ends_with(".c") || lower.ends_with(".h") {
        "c"
    } else if lower.ends_with(".cpp") || lower.ends_with(".hpp") || lower.ends_with(".cc") {
        "cpp"
    } else {
        "unknown"
    }
}

/// Estimates tokens saved between raw source bytes and outline payload bytes.
pub fn estimate_token_savings(raw_bytes: usize, outline_bytes: usize) -> u64 {
    let reduction = raw_bytes.saturating_sub(outline_bytes);
    (reduction / 4) as u64
}

/// Parses symbol outlines from source code without including execution bodies.
pub fn extract_file_outline(file_path: &str, content: &str) -> FileOutline {
    let norm_path = normalize_path(file_path);
    let lang = detect_language(&norm_path);
    let mut symbols = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_doc = String::new();
    let mut current_parent: Option<String> = None;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // Capture doc comments
        if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**") || trimmed.starts_with('*') {
            let doc_line = trimmed.trim_start_matches(|c| c == '/' || c == '*' || c == '!').trim();
            if !doc_line.is_empty() {
                if !current_doc.is_empty() {
                    current_doc.push(' ');
                }
                current_doc.push_str(doc_line);
            }
            continue;
        }

        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        let doc = if current_doc.is_empty() {
            None
        } else {
            let d = current_doc.clone();
            current_doc.clear();
            Some(d)
        };

        // Match symbols per language
        match lang {
            "rust" => {
                if let Some(sym) = parse_rust_line(trimmed, line_num, doc, current_parent.as_deref()) {
                    if matches!(sym.kind, SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Trait) {
                        current_parent = Some(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "python" => {
                if let Some(sym) = parse_python_line(trimmed, line_num, doc, current_parent.as_deref()) {
                    if sym.kind == SymbolKind::Class {
                        current_parent = Some(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "typescript" | "javascript" => {
                if let Some(sym) = parse_ts_js_line(trimmed, line_num, doc, current_parent.as_deref()) {
                    if matches!(sym.kind, SymbolKind::Class | SymbolKind::Interface) {
                        current_parent = Some(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "go" => {
                if let Some(sym) = parse_go_line(trimmed, line_num, doc, current_parent.as_deref()) {
                    if sym.kind == SymbolKind::Struct || sym.kind == SymbolKind::Interface {
                        current_parent = Some(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            _ => {
                // Generic heuristic
                if let Some(sym) = parse_generic_line(trimmed, line_num, doc) {
                    symbols.push(sym);
                }
            }
        }
    }

    let raw_bytes = content.len();
    let outline_bytes = serde_json::to_string(&symbols).map(|s| s.len()).unwrap_or(0);
    let tokens_saved = estimate_token_savings(raw_bytes, outline_bytes);

    FileOutline {
        file_path: norm_path,
        language: lang.to_string(),
        raw_bytes,
        outline_bytes,
        tokens_saved,
        symbols,
    }
}

fn parse_rust_line(
    line: &str,
    line_num: usize,
    doc: Option<String>,
    parent: Option<&str>,
) -> Option<SymbolOutline> {
    let visibility = if line.starts_with("pub(crate) ") {
        "pub(crate)"
    } else if line.starts_with("pub ") {
        "pub"
    } else {
        "private"
    };

    let stripped = line.strip_prefix("pub(crate) ").or_else(|| line.strip_prefix("pub ")).unwrap_or(line);

    if stripped.starts_with("fn ") || stripped.starts_with("async fn ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name_part = stripped.strip_prefix("async fn ").unwrap_or_else(|| stripped.strip_prefix("fn ").unwrap_or(""));
        let name = name_part.split('(').next()?.split('<').next()?.trim().to_string();
        if !name.is_empty() {
            let kind = if parent.is_some() { SymbolKind::Method } else { SymbolKind::Function };
            return Some(SymbolOutline {
                name,
                kind,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: parent.map(|s| s.to_string()),
                visibility: visibility.to_string(),
            });
        }
    } else if stripped.starts_with("struct ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name = stripped.strip_prefix("struct ")?.split('{').next()?.split('<').next()?.split(';').next()?.trim().to_string();
        if !name.is_empty() {
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Struct,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: visibility.to_string(),
            });
        }
    } else if stripped.starts_with("enum ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name = stripped.strip_prefix("enum ")?.split('{').next()?.split('<').next()?.trim().to_string();
        if !name.is_empty() {
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Enum,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: visibility.to_string(),
            });
        }
    } else if stripped.starts_with("trait ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name = stripped.strip_prefix("trait ")?.split('{').next()?.split('<').next()?.split(':').next()?.trim().to_string();
        if !name.is_empty() {
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Trait,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: visibility.to_string(),
            });
        }
    }

    None
}

fn parse_python_line(
    line: &str,
    line_num: usize,
    doc: Option<String>,
    parent: Option<&str>,
) -> Option<SymbolOutline> {
    if line.starts_with("def ") || line.starts_with("async def ") {
        let sig = line.trim_end_matches(':').trim();
        let name_part = line.strip_prefix("async def ").unwrap_or_else(|| line.strip_prefix("def ").unwrap_or(""));
        let name = name_part.split('(').next()?.trim().to_string();
        if !name.is_empty() {
            let visibility = if name.starts_with("__") {
                "private"
            } else if name.starts_with('_') {
                "protected"
            } else {
                "public"
            };
            let kind = if parent.is_some() { SymbolKind::Method } else { SymbolKind::Function };
            return Some(SymbolOutline {
                name,
                kind,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: parent.map(|s| s.to_string()),
                visibility: visibility.to_string(),
            });
        }
    } else if line.starts_with("class ") {
        let sig = line.trim_end_matches(':').trim();
        let name = line.strip_prefix("class ")?.split('(').next()?.split(':').next()?.trim().to_string();
        if !name.is_empty() {
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Class,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: "public".to_string(),
            });
        }
    }

    None
}

fn parse_ts_js_line(
    line: &str,
    line_num: usize,
    doc: Option<String>,
    parent: Option<&str>,
) -> Option<SymbolOutline> {
    let visibility = if line.starts_with("export ") {
        "export"
    } else {
        "local"
    };

    let stripped = line.strip_prefix("export default ").or_else(|| line.strip_prefix("export ")).unwrap_or(line);

    if stripped.starts_with("function ") || stripped.starts_with("async function ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name_part = stripped.strip_prefix("async function ").unwrap_or_else(|| stripped.strip_prefix("function ").unwrap_or(""));
        let name = name_part.split('(').next()?.split('<').next()?.trim().to_string();
        if !name.is_empty() {
            let kind = if parent.is_some() { SymbolKind::Method } else { SymbolKind::Function };
            return Some(SymbolOutline {
                name,
                kind,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: parent.map(|s| s.to_string()),
                visibility: visibility.to_string(),
            });
        }
    } else if stripped.starts_with("class ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name = stripped.strip_prefix("class ")?.split(' ').next()?.split('{').next()?.split('<').next()?.trim().to_string();
        if !name.is_empty() {
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Class,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: visibility.to_string(),
            });
        }
    } else if stripped.starts_with("interface ") {
        let sig = stripped.trim_end_matches('{').trim();
        let name = stripped.strip_prefix("interface ")?.split(' ').next()?.split('{').next()?.split('<').next()?.trim().to_string();
        if !name.is_empty() {
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Interface,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: visibility.to_string(),
            });
        }
    }

    None
}

fn parse_go_line(
    line: &str,
    line_num: usize,
    doc: Option<String>,
    _parent: Option<&str>,
) -> Option<SymbolOutline> {
    if line.starts_with("func ") {
        let sig = line.trim_end_matches('{').trim();
        let rest = line.strip_prefix("func ")?;
        let (receiver, name_part) = if rest.starts_with('(') {
            let close = rest.find(')')?;
            let rec = rest[1..close].trim();
            (Some(rec.to_string()), rest[close + 1..].trim())
        } else {
            (None, rest)
        };

        let name = name_part.split('(').next()?.trim().to_string();
        if !name.is_empty() {
            let is_exported = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            let kind = if receiver.is_some() { SymbolKind::Method } else { SymbolKind::Function };
            return Some(SymbolOutline {
                name,
                kind,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: receiver,
                visibility: if is_exported { "public".to_string() } else { "private".to_string() },
            });
        }
    } else if line.starts_with("type ") && line.contains("struct") {
        let sig = line.trim_end_matches('{').trim();
        let name = line.strip_prefix("type ")?.split(' ').next()?.trim().to_string();
        if !name.is_empty() {
            let is_exported = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            return Some(SymbolOutline {
                name,
                kind: SymbolKind::Struct,
                signature: sig.to_string(),
                docstring: doc,
                start_line: line_num,
                end_line: line_num,
                parent_symbol: None,
                visibility: if is_exported { "public".to_string() } else { "private".to_string() },
            });
        }
    }

    None
}

fn parse_generic_line(line: &str, line_num: usize, doc: Option<String>) -> Option<SymbolOutline> {
    if (line.starts_with("function ") || line.starts_with("def ") || line.starts_with("fn ")) && line.contains('(') {
        let sig = line.trim_end_matches('{').trim_end_matches(':').trim();
        return Some(SymbolOutline {
            name: "symbol".to_string(),
            kind: SymbolKind::Function,
            signature: sig.to_string(),
            docstring: doc,
            start_line: line_num,
            end_line: line_num,
            parent_symbol: None,
            visibility: "unknown".to_string(),
        });
    }
    None
}

/// Generates a prompt-optimized code representation stripping internal method bodies
/// to save context tokens while retaining API signatures and contracts.
pub fn summarize_code_for_prompt(file_path: &str, content: &str) -> String {
    let outline = extract_file_outline(file_path, content);
    let mut out = String::with_capacity(outline.symbols.len() * 80);

    out.push_str(&format!("// [Outline of {} - {} symbols]\n", outline.file_path, outline.symbols.len()));

    for sym in &outline.symbols {
        if let Some(doc) = &sym.docstring {
            out.push_str(&format!("/// {}\n", doc));
        }
        out.push_str(&sym.signature);
        if !sym.signature.ends_with(';') {
            out.push_str(" { ... }\n");
        } else {
            out.push('\n');
        }
    }

    out
}

/// Recursively scans a workspace directory generating a compact repository outline.
pub fn generate_repo_outline(root_dir: &Path, max_depth: usize) -> Result<RepoOutline, JCodeMunchError> {
    let norm_root = normalize_path(root_dir.to_string_lossy().as_ref());
    let mut files = Vec::new();
    let mut total_raw = 0usize;
    let mut total_outline = 0usize;
    let mut total_symbols = 0usize;

    walk_dir_outline(root_dir, 0, max_depth, &mut files)?;

    for f in &files {
        total_raw += f.raw_bytes;
        total_outline += f.outline_bytes;
        total_symbols += f.symbols.len();
    }

    let tokens_saved = estimate_token_savings(total_raw, total_outline);

    Ok(RepoOutline {
        root_dir: norm_root,
        total_files: files.len(),
        total_symbols,
        total_raw_bytes: total_raw,
        total_outline_bytes: total_outline,
        estimated_tokens_saved: tokens_saved,
        files,
    })
}

fn walk_dir_outline(
    dir: &Path,
    current_depth: usize,
    max_depth: usize,
    files: &mut Vec<FileOutline>,
) -> Result<(), JCodeMunchError> {
    if current_depth > max_depth {
        return Ok(());
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => return Err(JCodeMunchError::Io(dir.to_string_lossy().to_string(), err)),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" {
            continue;
        }

        if path.is_dir() {
            walk_dir_outline(&path, current_depth + 1, max_depth, files)?;
        } else if path.is_file() {
            let lang = detect_language(&name);
            if lang != "unknown" {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let outline = extract_file_outline(&path.to_string_lossy(), &content);
                    if !outline.symbols.is_empty() {
                        files.push(outline);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_outline_extraction() {
        let code = r#"
/// Global master runner
pub fn run_pipeline(timeout: u64) -> Result<(), String> {
    let mut x = 10;
    println!("running");
    Ok(())
}

pub struct ConfigModel {
    pub name: String,
}

enum Status {
    Active,
    Idle,
}
"#;
        let outline = extract_file_outline("src/runner.rs", code);
        assert_eq!(outline.language, "rust");
        assert_eq!(outline.symbols.len(), 3);
        assert_eq!(outline.symbols[0].name, "run_pipeline");
        assert_eq!(outline.symbols[0].kind, SymbolKind::Function);
        assert_eq!(outline.symbols[0].visibility, "pub");
        assert_eq!(outline.symbols[0].docstring.as_deref(), Some("Global master runner"));

        assert_eq!(outline.symbols[1].name, "ConfigModel");
        assert_eq!(outline.symbols[1].kind, SymbolKind::Struct);

        assert_eq!(outline.symbols[2].name, "Status");
        assert_eq!(outline.symbols[2].kind, SymbolKind::Enum);

        let prompt = summarize_code_for_prompt("src/runner.rs", code);
        assert!(prompt.contains("run_pipeline"));
        assert!(prompt.contains("{ ... }"));
        assert!(!prompt.contains("println!"));
    }

    #[test]
    fn test_python_outline_extraction() {
        let code = r#"
class DataEngine:
    def process_record(self, item: dict) -> bool:
        return True
"#;
        let outline = extract_file_outline("app/engine.py", code);
        assert_eq!(outline.language, "python");
        assert_eq!(outline.symbols.len(), 2);
        assert_eq!(outline.symbols[0].name, "DataEngine");
        assert_eq!(outline.symbols[0].kind, SymbolKind::Class);
        assert_eq!(outline.symbols[1].name, "process_record");
        assert_eq!(outline.symbols[1].kind, SymbolKind::Method);
    }
}
