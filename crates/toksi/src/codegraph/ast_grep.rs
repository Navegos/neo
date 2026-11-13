// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/ast_grep.rs
// created: 2026-11-09
// lastModified: 2026-11-13

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tree_sitter::{Node, Parser};

use super::kernel::langs;
use super::walker;
use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum AstGrepError {
    #[error("Invalid search pattern: {0}")]
    InvalidPattern(String),
    #[error("Regex error in pattern rule: {0}")]
    Regex(#[from] regex::Error),
    #[error("Tree-sitter parse failure: {0}")]
    Parse(String),
    #[error("I/O error during AST grep traversal: {0}")]
    Io(#[from] std::io::Error),
}

/// AST matching strictness semantics aligned with ast-grep `MatchStrictness`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchStrictness {
    /// Exact match on all named and anonymous CST nodes.
    Cst,
    /// Match named AST nodes and significant punctuation tokens while skipping trivia.
    #[default]
    Smart,
    /// Match named AST nodes only.
    Ast,
    /// Match named AST nodes ignoring comments and trivia.
    Relaxed,
    /// Match structural AST node kinds without comparing leaf literal/identifier text.
    Signature,
}

/// Structural search rule specification conforming to ast-grep rule schemas.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AstRule {
    pub id: String,
    pub language: String,
    pub pattern: String,
    pub kind: Option<String>,
    pub regex: Option<String>,
    pub message: Option<String>,
    #[serde(default)]
    pub strictness: MatchStrictness,
}

/// A captured match result from structural code searching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstMatch {
    pub rule_id: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub matched_text: String,
    pub captures: HashMap<String, String>,
}

/// AstGrep engine backed by real Tree-sitter CSTs and `MatchStrictness` semantics.
pub struct AstGrepEngine {
    rules: Vec<AstRule>,
}

impl Default for AstGrepEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AstGrepEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_rules(rules: Vec<AstRule>) -> Self {
        Self { rules }
    }

    pub fn add_rule(&mut self, rule: AstRule) {
        self.rules.push(rule);
    }

    /// Searches a single source file using Tree-sitter CST matching (with structural token fallback).
    pub fn search_source(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<Vec<AstMatch>, AstGrepError> {
        let norm_path = normalize_path(file_path);
        let mut matches = Vec::new();

        for rule in &self.rules {
            if rule.pattern.trim().is_empty() {
                return Err(AstGrepError::InvalidPattern("Pattern cannot be empty".into()));
            }
            let regex_filter = match &rule.regex {
                Some(re) => Some(regex::Regex::new(re)?),
                None => None,
            };

            let canon_lang = langs::canonical_language(&rule.language);
            let mut matched_via_cst = false;

            if let Some(ts_lang) = langs::grammar_for(canon_lang) {
                if let Some(cst_matches) = search_via_treesitter(
                    &norm_path,
                    content,
                    ts_lang,
                    rule,
                    regex_filter.as_ref(),
                )? {
                    matches.extend(cst_matches);
                    matched_via_cst = true;
                }
            }

            if !matched_via_cst {
                let compiled = compile_structural_pattern(&rule.pattern)?;
                for (idx, line) in content.lines().enumerate() {
                    if let Some(captures) = match_line_structural(line, &compiled) {
                        if let Some(ref re) = regex_filter {
                            if !re.is_match(line) {
                                continue;
                            }
                        }
                        matches.push(AstMatch {
                            rule_id: rule.id.clone(),
                            file_path: norm_path.clone(),
                            line: idx + 1,
                            column: line.find(line.trim_start()).unwrap_or(0) + 1,
                            matched_text: line.trim().to_string(),
                            captures,
                        });
                    }
                }
            }
        }

        Ok(matches)
    }

    /// Searches workspace directory using ripgrep-grade `ignore` traversal (respecting `.gitignore`/`.ignore`).
    pub fn search_directory(&self, dir: &Path) -> Result<Vec<AstMatch>, AstGrepError> {
        let mut all_matches = Vec::new();
        let walked = walker::walk_project_files(dir);
        for wf in walked {
            let file_matches_rule = self.rules.iter().any(|r| {
                r.language.is_empty()
                    || langs::canonical_language(&r.language) == langs::canonical_language(wf.language)
            });
            if !file_matches_rule {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&wf.path) {
                let file_matches = self.search_source(&wf.rel_path, &content)?;
                all_matches.extend(file_matches);
            }
        }
        Ok(all_matches)
    }
}

#[derive(Debug, Clone)]
enum MetaPlaceholder {
    Single(String),
    Multi(String),
    Wildcard,
}

fn prepare_pattern_source(pattern: &str) -> (String, HashMap<String, MetaPlaceholder>) {
    let mut out = String::with_capacity(pattern.len() + 32);
    let mut map = HashMap::new();
    let bytes = pattern.as_bytes();
    let mut i = 0usize;
    let mut seq = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'$' {
            if i + 3 <= bytes.len() && &bytes[i..i + 3] == b"$$$" {
                let start = i + 3;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                let name = if end > start {
                    String::from_utf8_lossy(&bytes[start..end]).into_owned()
                } else {
                    format!("ARGS_{seq}")
                };
                let token = format!("__sg_multi_{seq}__");
                seq += 1;
                map.insert(token.clone(), MetaPlaceholder::Multi(name));
                out.push_str(&token);
                i = end;
                continue;
            }
            if i + 1 < bytes.len()
                && (bytes[i + 1].is_ascii_uppercase() || bytes[i + 1] == b'_')
            {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                let raw = String::from_utf8_lossy(&bytes[start..end]).into_owned();
                let token = format!("__sg_mv_{seq}__");
                seq += 1;
                if raw == "_" {
                    map.insert(token.clone(), MetaPlaceholder::Wildcard);
                } else {
                    map.insert(token.clone(), MetaPlaceholder::Single(raw));
                }
                out.push_str(&token);
                i = end;
                continue;
            }
        }
        if let Some(ch) = pattern[i..].chars().next() {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    (out, map)
}

fn unwrap_pattern_root<'a>(mut node: Node<'a>) -> Option<Node<'a>> {
    // Strip top-level module/program/source_file & expression_statement wrappers
    for _ in 0..4 {
        let kind = node.kind();
        if node.is_error() {
            return None;
        }
        if matches!(
            kind,
            "source_file"
                | "program"
                | "module"
                | "compilation_unit"
                | "translation_unit"
                | "chunk"
                | "expression_statement"
        ) && node.named_child_count() == 1
        {
            if let Some(child) = node.named_child(0) {
                node = child;
                continue;
            }
        }
        break;
    }
    Some(node)
}

fn search_via_treesitter(
    norm_path: &str,
    content: &str,
    ts_lang: tree_sitter::Language,
    rule: &AstRule,
    regex_filter: Option<&regex::Regex>,
) -> Result<Option<Vec<AstMatch>>, AstGrepError> {
    let mut parser = Parser::new();
    parser
        .set_language(&ts_lang)
        .map_err(|e| AstGrepError::Parse(e.to_string()))?;

    let (pat_src, placeholders) = prepare_pattern_source(&rule.pattern);
    let Some(pat_tree) = parser.parse(&pat_src, None) else {
        return Ok(None);
    };
    let Some(pat_node) = unwrap_pattern_root(pat_tree.root_node()) else {
        // Fallback when snippet doesn't parse cleanly as standalone AST root
        return Ok(None);
    };
    if pat_node.has_error() {
        return Ok(None);
    }

    let Some(src_tree) = parser.parse(content, None) else {
        return Ok(None);
    };

    let mut results = Vec::new();
    let mut cursor_stack = vec![src_tree.root_node()];
    let mut seen_ranges = std::collections::HashSet::new();

    while let Some(candidate) = cursor_stack.pop() {
        if let Some(ref req_kind) = rule.kind {
            if candidate.kind() != req_kind {
                push_children_rev(candidate, &mut cursor_stack);
                continue;
            }
        }

        let mut captures = HashMap::new();
        if match_cst_nodes(
            candidate,
            pat_node,
            content.as_bytes(),
            pat_src.as_bytes(),
            &placeholders,
            rule.strictness,
            &mut captures,
        ) {
            let range_key = (candidate.start_byte(), candidate.end_byte());
            if seen_ranges.insert(range_key) {
                let matched_text = candidate
                    .utf8_text(content.as_bytes())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let re_ok = regex_filter
                    .map(|re| re.is_match(&matched_text))
                    .unwrap_or(true);
                if re_ok {
                    let pos = candidate.start_position();
                    results.push(AstMatch {
                        rule_id: rule.id.clone(),
                        file_path: norm_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        matched_text,
                        captures,
                    });
                }
            }
        }

        push_children_rev(candidate, &mut cursor_stack);
    }

    Ok(Some(results))
}

fn push_children_rev<'a>(node: Node<'a>, stack: &mut Vec<Node<'a>>) {
    let count = node.named_child_count();
    for idx in (0..count).rev() {
        if let Some(ch) = node.named_child(idx) {
            stack.push(ch);
        }
    }
}

fn is_trivia_kind(kind: &str) -> bool {
    kind.contains("comment") || kind == ";" || kind == ","
}

fn collect_children<'a>(node: Node<'a>, strictness: MatchStrictness) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    let mut c = node.walk();
    for child in node.children(&mut c) {
        match strictness {
            MatchStrictness::Cst => out.push(child),
            MatchStrictness::Smart => {
                if !child.kind().contains("comment") {
                    out.push(child);
                }
            }
            MatchStrictness::Ast | MatchStrictness::Signature => {
                if child.is_named() && !child.kind().contains("comment") {
                    out.push(child);
                }
            }
            MatchStrictness::Relaxed => {
                if child.is_named() && !is_trivia_kind(child.kind()) {
                    out.push(child);
                }
            }
        }
    }
    out
}

fn match_cst_nodes(
    cand: Node,
    pat: Node,
    src_bytes: &[u8],
    pat_bytes: &[u8],
    placeholders: &HashMap<String, MetaPlaceholder>,
    strictness: MatchStrictness,
    captures: &mut HashMap<String, String>,
) -> bool {
    let pat_text = pat.utf8_text(pat_bytes).unwrap_or("").trim();
    if let Some(ph) = placeholders.get(pat_text) {
        let cand_text = cand.utf8_text(src_bytes).unwrap_or("").trim().to_string();
        match ph {
            MetaPlaceholder::Wildcard => return true,
            MetaPlaceholder::Single(name) | MetaPlaceholder::Multi(name) => {
                if let Some(prev) = captures.get(name) {
                    return prev == &cand_text;
                }
                captures.insert(name.clone(), cand_text);
                return true;
            }
        }
    }

    if cand.kind() != pat.kind() {
        return false;
    }

    let pat_children = collect_children(pat, strictness);
    let cand_children = collect_children(cand, strictness);

    if pat_children.is_empty() && cand_children.is_empty() {
        if strictness == MatchStrictness::Signature {
            return true;
        }
        let c_txt = cand.utf8_text(src_bytes).unwrap_or("").trim();
        let p_txt = pat.utf8_text(pat_bytes).unwrap_or("").trim();
        return c_txt == p_txt;
    }

    match_child_slice(
        &cand_children,
        &pat_children,
        src_bytes,
        pat_bytes,
        placeholders,
        strictness,
        captures,
    )
}

fn match_child_slice(
    cand: &[Node],
    pat: &[Node],
    src_bytes: &[u8],
    pat_bytes: &[u8],
    placeholders: &HashMap<String, MetaPlaceholder>,
    strictness: MatchStrictness,
    captures: &mut HashMap<String, String>,
) -> bool {
    if pat.is_empty() {
        return cand.is_empty();
    }
    let pat_first_txt = pat[0].utf8_text(pat_bytes).unwrap_or("").trim();
    if let Some(MetaPlaceholder::Multi(name)) = placeholders.get(pat_first_txt) {
        if pat.len() == 1 {
            let joined = cand
                .iter()
                .filter_map(|n| n.utf8_text(src_bytes).ok())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(", ");
            captures.insert(name.clone(), joined);
            return true;
        }
        for split in 0..=cand.len() {
            let mut snapshot = captures.clone();
            if match_child_slice(
                &cand[split..],
                &pat[1..],
                src_bytes,
                pat_bytes,
                placeholders,
                strictness,
                &mut snapshot,
            ) {
                let joined = cand[..split]
                    .iter()
                    .filter_map(|n| n.utf8_text(src_bytes).ok())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                snapshot.insert(name.clone(), joined);
                *captures = snapshot;
                return true;
            }
        }
        return false;
    }

    if cand.is_empty() {
        return false;
    }

    let mut snapshot = captures.clone();
    if match_cst_nodes(
        cand[0],
        pat[0],
        src_bytes,
        pat_bytes,
        placeholders,
        strictness,
        &mut snapshot,
    ) && match_child_slice(
        &cand[1..],
        &pat[1..],
        src_bytes,
        pat_bytes,
        placeholders,
        strictness,
        &mut snapshot,
    ) {
        *captures = snapshot;
        return true;
    }

    false
}

#[derive(Debug, Clone)]
enum PatternToken {
    Literal(String),
    Metavar(String),
    WildcardSpread(String),
}

fn compile_structural_pattern(pattern: &str) -> Result<Vec<PatternToken>, AstGrepError> {
    if pattern.trim().is_empty() {
        return Err(AstGrepError::InvalidPattern("Pattern cannot be empty".into()));
    }
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();
    let mut current = String::new();

    while let Some(&c) = chars.peek() {
        if c == '$' {
            if !current.is_empty() {
                tokens.push(PatternToken::Literal(std::mem::take(&mut current)));
            }
            chars.next();
            let mut spread = false;
            if chars.peek() == Some(&'$') {
                chars.next();
                if chars.peek() == Some(&'$') {
                    chars.next();
                    spread = true;
                }
            }
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_alphanumeric() || nc == '_' {
                    name.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if spread {
                tokens.push(PatternToken::WildcardSpread(name));
            } else if !name.is_empty() {
                tokens.push(PatternToken::Metavar(name));
            } else {
                tokens.push(PatternToken::Literal("$".to_string()));
            }
        } else {
            current.push(c);
            chars.next();
        }
    }
    if !current.is_empty() {
        tokens.push(PatternToken::Literal(current));
    }
    Ok(tokens)
}

fn match_line_structural(line: &str, pattern: &[PatternToken]) -> Option<HashMap<String, String>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut captures = HashMap::new();
    let mut rest = trimmed;

    for (idx, token) in pattern.iter().enumerate() {
        match token {
            PatternToken::Literal(lit) => {
                if idx == 0 {
                    let pos = rest.find(lit.as_str())?;
                    rest = &rest[pos + lit.len()..];
                } else {
                    let trimmed_rest = rest.trim_start();
                    if let Some(after) = trimmed_rest.strip_prefix(lit.as_str()) {
                        rest = after;
                    } else {
                        let pos = rest.find(lit.as_str())?;
                        rest = &rest[pos + lit.len()..];
                    }
                }
            }
            PatternToken::Metavar(name) => {
                let next_lit = pattern[idx + 1..].iter().find_map(|t| match t {
                    PatternToken::Literal(l) => Some(l.as_str()),
                    _ => None,
                });
                let val_str = if let Some(lit) = next_lit {
                    let pos = rest.find(lit)?;
                    let val = rest[..pos].trim().to_string();
                    rest = &rest[pos..];
                    val
                } else {
                    let val = rest.split_whitespace().next().unwrap_or("").to_string();
                    rest = &rest[val.len()..];
                    val
                };
                if val_str.is_empty() {
                    return None;
                }
                if let Some(prev) = captures.get(name) {
                    if prev != &val_str {
                        return None;
                    }
                } else {
                    captures.insert(name.clone(), val_str);
                }
            }
            PatternToken::WildcardSpread(name) => {
                let next_lit = pattern[idx + 1..].iter().find_map(|t| match t {
                    PatternToken::Literal(l) => Some(l.as_str()),
                    _ => None,
                });
                let val_str = if let Some(lit) = next_lit {
                    let pos = rest.find(lit)?;
                    let val = rest[..pos].trim().to_string();
                    rest = &rest[pos..];
                    val
                } else {
                    let val = rest.trim().to_string();
                    rest = "";
                    val
                };
                if !name.is_empty() {
                    captures.insert(name.clone(), val_str);
                }
            }
        }
    }

    Some(captures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_grep_cst_pattern_matching() {
        let mut engine = AstGrepEngine::new();
        engine.add_rule(AstRule {
            id: "no-unwrap".to_string(),
            language: "rust".to_string(),
            pattern: "$OBJ.unwrap()".to_string(),
            kind: None,
            regex: None,
            message: Some("Avoid .unwrap() in production code".to_string()),
            strictness: MatchStrictness::Smart,
        });

        let code = "fn foo(x: Option<i32>) -> i32 { let y = x.unwrap(); y }";
        let matches = engine.search_source("src/test.rs", code).expect("Search should succeed");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures.get("OBJ").map(|s| s.as_str()), Some("x"));
    }

    #[test]
    fn test_ast_grep_wildcard_spread() {
        let mut engine = AstGrepEngine::new();
        engine.add_rule(AstRule {
            id: "find-query".to_string(),
            language: "rust".to_string(),
            pattern: "$DB.query($$$ARGS)".to_string(),
            kind: None,
            regex: None,
            message: None,
            strictness: MatchStrictness::Ast,
        });

        let code = "fn run(conn: Db) { conn.query(\"SELECT * FROM users\", id); }";
        let matches = engine.search_source("src/db.rs", code).expect("Search");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures.get("DB").map(|s| s.as_str()), Some("conn"));
        assert!(matches[0]
            .captures
            .get("ARGS")
            .is_some_and(|a| a.contains("SELECT * FROM users")));
    }

    #[test]
    fn test_ast_grep_signature_strictness_and_gitignore() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join(".git")).expect("git");
        fs::create_dir_all(dir.path().join("src")).expect("src");
        fs::write(dir.path().join(".gitignore"), "src/ignored.rs\n").expect("gitignore");
        fs::write(
            dir.path().join("src/kept.rs"),
            "fn alpha() { println!(\"hi\"); }\n",
        )
        .expect("kept");
        fs::write(
            dir.path().join("src/ignored.rs"),
            "fn beta() { println!(\"secret\"); }\n",
        )
        .expect("ignored");

        let mut engine = AstGrepEngine::new();
        engine.add_rule(AstRule {
            id: "find-fn".to_string(),
            language: "rust".to_string(),
            pattern: "fn dummy() { println!(\"x\"); }".to_string(),
            kind: Some("function_item".to_string()),
            regex: None,
            message: None,
            strictness: MatchStrictness::Signature,
        });

        let hits = engine.search_directory(dir.path()).expect("dir search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].file_path, "src/kept.rs");
    }
}
