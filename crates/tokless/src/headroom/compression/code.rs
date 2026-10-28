// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/compression/code.rs

use tree_sitter::{Language, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeLanguage {
    Rust,
    Typescript,
    Javascript,
    Python,
    Go,
    C,
    Cpp,
    Unknown,
}

impl CodeLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeLanguage::Rust => "rust",
            CodeLanguage::Typescript => "typescript",
            CodeLanguage::Javascript => "javascript",
            CodeLanguage::Python => "python",
            CodeLanguage::Go => "go",
            CodeLanguage::C => "c",
            CodeLanguage::Cpp => "cpp",
            CodeLanguage::Unknown => "unknown",
        }
    }

    pub fn from_hint(hint: &str) -> Self {
        let clean = hint.trim().to_lowercase();
        match clean.as_str() {
            "rs" | "rust" => CodeLanguage::Rust,
            "ts" | "typescript" | "tsx" => CodeLanguage::Typescript,
            "js" | "javascript" | "jsx" | "mjs" | "cjs" => CodeLanguage::Javascript,
            "py" | "python" => CodeLanguage::Python,
            "go" | "golang" => CodeLanguage::Go,
            "c" | "h" => CodeLanguage::C,
            "cpp" | "cxx" | "cc" | "hpp" => CodeLanguage::Cpp,
            _ => CodeLanguage::Unknown,
        }
    }

    pub fn grammar(&self) -> Option<Language> {
        match self {
            CodeLanguage::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            CodeLanguage::Typescript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            CodeLanguage::Javascript => Some(tree_sitter_javascript::LANGUAGE.into()),
            CodeLanguage::Python => Some(tree_sitter_python::LANGUAGE.into()),
            CodeLanguage::Go => Some(tree_sitter_go::LANGUAGE.into()),
            CodeLanguage::C => Some(tree_sitter_c::LANGUAGE.into()),
            CodeLanguage::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            CodeLanguage::Unknown => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeCompressorConfig {
    pub strip_comments: bool,
    pub preserve_docstrings: bool,
    pub strip_function_bodies: bool,
    pub max_blank_lines: usize,
}

impl Default for CodeCompressorConfig {
    fn default() -> Self {
        Self {
            strip_comments: true,
            preserve_docstrings: true,
            strip_function_bodies: false,
            max_blank_lines: 1,
        }
    }
}

pub fn detect_code_language(code_or_hint: &str) -> CodeLanguage {
    let lang = CodeLanguage::from_hint(code_or_hint);
    if lang != CodeLanguage::Unknown {
        return lang;
    }

    // Heuristics
    let trimmed = code_or_hint.trim();
    if trimmed.contains("fn ") && trimmed.contains("pub ") && (trimmed.contains("let mut ") || trimmed.contains("impl ")) {
        return CodeLanguage::Rust;
    }
    if (trimmed.contains("def ") || trimmed.contains("import ")) && trimmed.contains(":") && !trimmed.contains("function ") {
        return CodeLanguage::Python;
    }
    if (trimmed.contains("interface ") || trimmed.contains(": string") || trimmed.contains("const ")) && trimmed.contains("=>") {
        return CodeLanguage::Typescript;
    }
    if trimmed.contains("package ") && trimmed.contains("func ") {
        return CodeLanguage::Go;
    }
    if trimmed.contains("#include <") {
        return if trimmed.contains("std::") || trimmed.contains("class ") {
            CodeLanguage::Cpp
        } else {
            CodeLanguage::C
        };
    }

    CodeLanguage::Unknown
}

fn strip_line_comments_heuristic(source: &str, lang: CodeLanguage, preserve_docs: bool) -> String {
    let mut out = Vec::new();
    let comment_prefix = match lang {
        CodeLanguage::Python => "#",
        _ => "//",
    };

    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Handle C-style block comments
        if !in_block_comment && trimmed.starts_with("/*") {
            if preserve_docs && (trimmed.starts_with("/**") || trimmed.starts_with("/*!")) {
                out.push(line.to_string());
                if !trimmed.ends_with("*/") {
                    in_block_comment = true;
                }
                continue;
            }
            if !trimmed.ends_with("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if in_block_comment {
            if trimmed.ends_with("*/") {
                in_block_comment = false;
            }
            continue;
        }

        // Single line comments
        if trimmed.starts_with(comment_prefix) {
            // Keep doc comments or compiler directives
            if preserve_docs && (trimmed.starts_with("///") || trimmed.starts_with("//!")) {
                out.push(line.to_string());
                continue;
            }
            if trimmed.starts_with("#[") || trimmed.starts_with("#!") || trimmed.starts_with("#include") || trimmed.starts_with("#pragma") {
                out.push(line.to_string());
                continue;
            }
            continue;
        }

        // Inline comment stripping (conservative: only when not inside quotes)
        let mut clean_line = line.to_string();
        if let Some(pos) = line.find(comment_prefix) {
            let before = &line[..pos];
            let quote_count = before.chars().filter(|&c| c == '"' || c == '\'').count();
            if quote_count % 2 == 0 && !before.trim().is_empty() {
                clean_line = before.trim_end().to_string();
            }
        }

        out.push(clean_line);
    }

    out.join("\n")
}

fn normalize_blank_lines(source: &str, max_consecutive_blanks: usize) -> String {
    let mut out = Vec::new();
    let mut consecutive_blanks = 0usize;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= max_consecutive_blanks {
                out.push("");
            }
        } else {
            consecutive_blanks = 0;
            out.push(line.trim_end());
        }
    }

    out.join("\n")
}

pub fn compress_code(source: &str, lang: CodeLanguage, config: &CodeCompressorConfig) -> String {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let grammar = lang.grammar();

    // Step 1: Strip comments if requested
    let mut processed = if config.strip_comments {
        strip_line_comments_heuristic(source, lang, config.preserve_docstrings)
    } else {
        source.to_string()
    };

    // Step 2: Normalize blank lines
    processed = normalize_blank_lines(&processed, config.max_blank_lines);

    // Step 3: Verify AST syntax if grammar is available
    if let Some(grammar) = grammar {
        let mut parser = Parser::new();
        if parser.set_language(&grammar).is_ok() {
            let tree = parser.parse(&processed, None);
            if let Some(tree) = tree {
                // If comment removal introduced syntax errors, fallback to original
                if tree.root_node().has_error() {
                    let orig_tree = parser.parse(source, None);
                    if let Some(orig) = orig_tree {
                        if !orig.root_node().has_error() {
                            // The original was valid, so fallback to normalized original
                            return normalize_blank_lines(source, config.max_blank_lines);
                        }
                    }
                }
            }
        }
    }

    processed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_code_compression_and_ast_validity() {
        let code = r#"
        // A helper function
        /// Adds two numbers
        pub fn add(a: i32, b: i32) -> i32 {
            // Internal calculation


            a + b
        }
        "#;

        let res = compress_code(code, CodeLanguage::Rust, &CodeCompressorConfig::default());
        assert!(res.contains("pub fn add(a: i32, b: i32) -> i32"));
        assert!(res.contains("/// Adds two numbers"));
        assert!(!res.contains("// A helper function"));
        assert!(!res.contains("// Internal calculation"));
        assert!(!res.contains("\n\n\n"));
    }

    #[test]
    fn test_python_code_compression() {
        let code = r#"
        # Top-level comment
        def compute_sum(x: int, y: int) -> int:
            # Inline comment
            return x + y
        "#;

        let res = compress_code(code, CodeLanguage::Python, &CodeCompressorConfig::default());
        assert!(res.contains("def compute_sum(x: int, y: int) -> int:"));
        assert!(res.contains("return x + y"));
        assert!(!res.contains("# Top-level comment"));
        assert!(!res.contains("# Inline comment"));
    }

    #[test]
    fn test_typescript_code_compression() {
        let code = r#"
        // Interface definition
        export interface User {
            id: string; // user id
            name: string;
        }

        export const getUser = (id: string): User => {
            return { id, name: "Alice" };
        };
        "#;

        let res = compress_code(code, CodeLanguage::Typescript, &CodeCompressorConfig::default());
        assert!(res.contains("export interface User"));
        assert!(res.contains("id: string;"));
        assert!(res.contains("export const getUser"));
        assert!(!res.contains("// Interface definition"));
    }
}

