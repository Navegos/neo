// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/polyglot.rs
// created: 2026-11-21

//! Bridges `codegraph`'s 45 Tree-sitter languages + `ignore` walker to extract
//! normalized sliding token n-grams (`3, 4, 5`) and language taxonomy IDs.

#![deny(unsafe_code)]

use std::path::Path;

use tree_sitter::Parser;

use crate::codegraph::kernel::langs::{canonical_language, grammar_for};
use crate::util::paths::normalize_path;

/// Normalizes a command or code snippet (lowercase + whitespace collapse).
pub fn normalize_tokens(input: &str) -> Vec<String> {
    input
        .to_ascii_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Extracts sliding n-grams of sizes `[3, 4, 5]` from normalized tokens.
pub fn extract_sliding_ngrams(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for n in [3usize, 4, 5] {
        if tokens.len() >= n {
            for win in tokens.windows(n) {
                out.push(win.join(" "));
            }
        }
    }
    if out.is_empty() && !tokens.is_empty() {
        out.push(tokens.join(" "));
    }
    out
}

/// Detects canonical Tree-sitter language identifier from file path extension or filename.
pub fn detect_language_from_path(path: &Path) -> Option<&'static str> {
    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if fname == "dockerfile" || fname.starts_with("dockerfile.") {
        return Some("dockerfile");
    }
    if fname == "makefile" || fname == "gnumakefile" {
        return Some("make");
    }
    if fname == "cmakelists.txt" {
        return Some("cmake");
    }
    let ext = path.extension().and_then(|e| e.to_str())?.to_ascii_lowercase();
    let lang = match ext.as_str() {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "py" => "python",
        "go" => "go",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "zig" => "zig",
        "swift" => "swift",
        "java" => "java",
        "cs" => "csharp",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "php" => "php",
        "rb" => "ruby",
        "lua" => "lua",
        "luau" => "luau",
        "jl" => "julia",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "ml" | "mli" => "ocaml",
        "hs" => "haskell",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "vue" => "vue",
        "svelte" => "svelte",
        "gql" | "graphql" => "graphql",
        "sh" | "bash" | "zsh" => "bash",
        "tf" | "hcl" => "hcl",
        "nix" => "nix",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "proto" => "protobuf",
        "sql" => "sql",
        "md" => "markdown",
        "tex" => "latex",
        "dart" => "dart",
        "r" => "r",
        _ => return None,
    };
    Some(lang)
}

/// Parses source code with the corresponding Tree-sitter grammar and returns normalized AST leaf tokens.
pub fn extract_ast_leaf_tokens(language: &str, source: &str) -> Vec<String> {
    let canon = canonical_language(language);
    let Some(grammar) = grammar_for(canon) else {
        return normalize_tokens(source);
    };
    let mut parser = Parser::new();
    if parser.set_language(&grammar).is_err() {
        return normalize_tokens(source);
    }
    let Some(tree) = parser.parse(source, None) else {
        return normalize_tokens(source);
    };

    let mut tokens = Vec::new();
    let mut stack = vec![tree.root_node()];
    let bytes = source.as_bytes();
    while let Some(node) = stack.pop() {
        if node.child_count() == 0 {
            if let Ok(text) = node.utf8_text(bytes) {
                let norm = text.trim().to_ascii_lowercase();
                if !norm.is_empty() {
                    tokens.push(norm);
                }
            }
        } else {
            let count = node.child_count();
            for i in (0..count).rev() {
                if let Some(child) = node.child(i) {
                    stack.push(child);
                }
            }
        }
    }
    if tokens.is_empty() {
        normalize_tokens(source)
    } else {
        tokens
    }
}

/// Walks a directory respecting `.gitignore`/`.ignore` and returns normalized `(path, lang)` pairs.
pub fn walk_polyglot_files(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }
    let walker = ignore::WalkBuilder::new(root).build();
    for entry in walker.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        if let Some(lang) = detect_language_from_path(p) {
            files.push((normalize_path(&p.to_string_lossy()), lang.to_string()));
        }
    }
    files
}
