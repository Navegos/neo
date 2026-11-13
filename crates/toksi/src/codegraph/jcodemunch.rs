// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/jcodemunch.rs
// created: 2026-11-09
// lastModified: 2026-11-13

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::kernel;
use super::walker;
use super::CodeGraph;
use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum JCodeMunchError {
    #[error("Failed to read file {0}: {1}")]
    Io(String, #[source] std::io::Error),
    #[error("JSON serialization failure: {0}")]
    Json(#[from] serde_json::Error),
}

/// Condensed symbolic outline of a code element (token-saving alternative to raw source text).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOutline {
    pub kind: String,
    pub name: String,
    pub signature: String,
    pub start_line: usize,
    pub end_line: usize,
    pub children: Vec<SymbolOutline>,
    pub doc_comment: Option<String>,
}

/// Extracted condensed outline of an entire source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutline {
    pub file_path: String,
    pub language: String,
    pub line_count: usize,
    pub symbols: Vec<SymbolOutline>,
}

/// Repository tree representation mapping files cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOutline {
    pub root: String,
    pub total_files: usize,
    pub total_symbols: usize,
    pub files: Vec<FileOutline>,
}

/// Extracts a high-signal AST outline from source content across all supported languages
/// via the Tree-sitter kernel (no fragile regex heuristics).
pub fn extract_file_outline(file_path: &str, content: &str) -> FileOutline {
    let norm_path = normalize_path(file_path);
    let line_count = content.lines().count();
    let lang = walker::detect_language(Path::new(&norm_path)).unwrap_or("unknown");

    let mut symbols = Vec::new();
    if let Ok(extraction) = kernel::extract_file(&norm_path, content, lang) {
        let mut nodes: Vec<_> = extraction
            .nodes
            .into_iter()
            .filter(|n| n.kind != "file")
            .collect();
        nodes.sort_by_key(|n| (n.start_line, n.start_column));

        for node in nodes {
            let sig = node.signature.unwrap_or_else(|| {
                content
                    .lines()
                    .nth(node.start_line.saturating_sub(1) as usize)
                    .map(|l| l.trim().to_string())
                    .unwrap_or_else(|| node.name.clone())
            });
            symbols.push(SymbolOutline {
                kind: node.kind,
                name: node.name,
                signature: sig,
                start_line: node.start_line as usize,
                end_line: node.end_line as usize,
                children: Vec::new(),
                doc_comment: node.docstring,
            });
        }
    }

    FileOutline {
        file_path: norm_path,
        language: lang.to_string(),
        line_count,
        symbols,
    }
}

/// Generates an ultra-compact source skeleton prompt snippet preserving key signatures cleanly.
pub fn summarize_code_for_prompt(file_path: &str, content: &str, budget_lines: usize) -> String {
    let outline = extract_file_outline(file_path, content);
    let mut out = String::new();
    out.push_str(&format!(
        "=== {} ({}, {} lines) ===\n",
        outline.file_path, outline.language, outline.line_count
    ));

    let mut lines_written = 1;
    for sym in &outline.symbols {
        if lines_written >= budget_lines {
            out.push_str("... [truncated for token budget] ...\n");
            break;
        }
        if let Some(ref doc) = sym.doc_comment {
            let short_doc = doc.lines().next().unwrap_or("");
            if !short_doc.is_empty() {
                out.push_str(&format!("/// {}\n", short_doc));
                lines_written += 1;
            }
        }
        out.push_str(&format!(
            "[L{}-{}] {} {}\n",
            sym.start_line, sym.end_line, sym.kind, sym.signature
        ));
        lines_written += 1;
    }

    out
}

/// O(1) SQLite byte-offset slice read for a single symbol using `symbols.db` + Seek.
pub fn extract_symbol_slice_o1(cg: &CodeGraph, id_or_name: &str) -> Option<(String, String)> {
    let (node, text) = cg.cache.get_symbol_slice_o1(&cg.root, id_or_name)?;
    Some((normalize_path(&node.file_path), text))
}

/// Generates a compact O(1) cached symbol skeleton with byte-offset coordinates from `symbols.db`.
pub fn summarize_from_cache_o1(cg: &CodeGraph, rel_path: &str) -> String {
    let norm = normalize_path(rel_path);
    let nodes = cg.engine.get_file_symbols(&norm);
    let mut out = format!("=== {} (cached AST) ===\n", norm);
    for n in nodes {
        if n.kind == "file" {
            continue;
        }
        let range = cg
            .cache
            .get_symbol_byte_range(&n.id)
            .map(|(_, sb, eb)| format!("bytes {}..{}", sb, eb))
            .unwrap_or_else(|| format!("L{}-{}", n.start_line, n.end_line));
        out.push_str(&format!(
            "- {} `{}` [L{}-{}, {}]\n",
            n.kind,
            n.qualified_name,
            n.start_line,
            n.end_line,
            range
        ));
    }
    out
}

/// Walks a repository respecting `.gitignore`/`.ignore` and builds a condensed `RepoOutline`.
pub fn generate_repo_outline(root_dir: &Path, max_depth: usize) -> Result<RepoOutline, JCodeMunchError> {
    let walked = walker::walk_project_files(root_dir);
    let mut files = Vec::new();
    let mut total_symbols = 0;

    for wf in walked {
        let depth = wf.rel_path.split('/').count();
        if max_depth > 0 && depth > max_depth {
            continue;
        }
        let content = fs::read_to_string(&wf.path)
            .map_err(|e| JCodeMunchError::Io(wf.rel_path.clone(), e))?;
        let outline = extract_file_outline(&wf.rel_path, &content);
        total_symbols += outline.symbols.len();
        files.push(outline);
    }

    Ok(RepoOutline {
        root: normalize_path(&root_dir.to_string_lossy()),
        total_files: files.len(),
        total_symbols,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jcodemunch_ast_outline_extraction() {
        let code = r#"
/// Processes payment requests
pub struct PaymentEngine {
    pub api_key: String,
}

impl PaymentEngine {
    pub fn charge(&self, amount: u64) -> bool {
        amount > 0
    }
}
"#;
        let outline = extract_file_outline("src/payment.rs", code);
        assert_eq!(outline.language, "rust");
        assert!(outline.symbols.iter().any(|s| s.name == "PaymentEngine" && s.kind == "struct"));
        assert!(outline.symbols.iter().any(|s| s.name == "charge"));

        let summary = summarize_code_for_prompt("src/payment.rs", code, 10);
        assert!(summary.contains("src/payment.rs"));
        assert!(summary.contains("PaymentEngine"));
    }

    #[test]
    fn test_jcodemunch_o1_sqlite_byte_offset_slice() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join(".git")).expect("git");
        fs::create_dir_all(dir.path().join("src")).expect("src");
        let src = "pub fn first() -> i32 { 1 }\n\npub fn target_slice() -> i32 {\n    99\n}\n";
        fs::write(dir.path().join("src/slice.rs"), src).expect("write");

        let mut cg = CodeGraph::open(dir.path()).expect("open");
        cg.ensure_indexed().expect("index");

        let (file, snippet) = extract_symbol_slice_o1(&cg, "target_slice").expect("o1 slice");
        assert_eq!(file, "src/slice.rs");
        assert!(snippet.contains("target_slice"));
        assert!(snippet.contains("99"));
        assert!(!snippet.contains("first"));
    }
}
