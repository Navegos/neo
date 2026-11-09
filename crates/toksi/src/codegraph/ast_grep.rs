// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/ast_grep.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum AstGrepError {
    #[error("Invalid search pattern: {0}")]
    InvalidPattern(String),
    #[error("Regex error in pattern rule: {0}")]
    Regex(#[from] regex::Error),
    #[error("I/O error during AST grep traversal: {0}")]
    Io(#[from] std::io::Error),
}

/// Structural search rule specification conforming to ast-grep rule schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstRule {
    pub id: String,
    pub language: String,
    pub pattern: String,
    pub kind: Option<String>,
    pub regex: Option<String>,
    pub message: Option<String>,
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

/// AstGrep engine providing structural code pattern matching and AST searches.
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

    /// Searches within a single source file for structural pattern matches.
    pub fn search_source(&self, file_path: &str, content: &str) -> Result<Vec<AstMatch>, AstGrepError> {
        let norm_path = normalize_path(file_path);
        let mut matches = Vec::new();

        for rule in &self.rules {
            let compiled_pattern = compile_structural_pattern(&rule.pattern)?;
            let regex_filter = if let Some(re) = &rule.regex {
                Some(regex::Regex::new(re)?)
            } else {
                None
            };

            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                if let Some(captures) = match_line_structural(line, &compiled_pattern) {
                    if let Some(ref re) = regex_filter {
                        if !re.is_match(line) {
                            continue;
                        }
                    }

                    matches.push(AstMatch {
                        rule_id: rule.id.clone(),
                        file_path: norm_path.clone(),
                        line: line_num,
                        column: line.find(line.trim_start()).unwrap_or(0) + 1,
                        matched_text: line.trim().to_string(),
                        captures,
                    });
                }
            }
        }

        Ok(matches)
    }

    /// Traverses a workspace scanning source files for AST pattern matches.
    pub fn search_workspace(&self, root_dir: &Path) -> Result<Vec<AstMatch>, AstGrepError> {
        let mut results = Vec::new();
        walk_ast_search(root_dir, self, &mut results)?;
        Ok(results)
    }
}

/// Token sequence extracted from a pattern with meta-variables.
#[derive(Debug, Clone)]
enum PatternToken {
    Literal(String),
    MetaVariable(String), // $VAR
    WildcardSpread,       // $$$ARGS
    WildcardSingle,       // $_
}

/// Compiles a structural pattern string like `foo($ARG, $$$REST)` into tokens.
fn compile_structural_pattern(pattern: &str) -> Result<Vec<PatternToken>, AstGrepError> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();
    let mut buf = String::new();

    while let Some(&c) = chars.peek() {
        if c == '$' {
            if !buf.is_empty() {
                tokens.push(PatternToken::Literal(buf.clone()));
                buf.clear();
            }
            chars.next(); // consume '$'
            if chars.peek() == Some(&'$') {
                chars.next(); // consume 2nd '$'
                if chars.peek() == Some(&'$') {
                    chars.next(); // consume 3rd '$'
                    // read var name
                    let mut var_name = String::new();
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_alphanumeric() || next_c == '_' {
                            var_name.push(next_c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(PatternToken::WildcardSpread);
                } else {
                    tokens.push(PatternToken::Literal("$$".to_string()));
                }
            } else if chars.peek() == Some(&'_') {
                chars.next();
                tokens.push(PatternToken::WildcardSingle);
            } else {
                let mut var_name = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        var_name.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if var_name.is_empty() {
                    tokens.push(PatternToken::Literal("$".to_string()));
                } else {
                    tokens.push(PatternToken::MetaVariable(var_name));
                }
            }
        } else {
            buf.push(c);
            chars.next();
        }
    }

    if !buf.is_empty() {
        tokens.push(PatternToken::Literal(buf));
    }

    Ok(tokens)
}

fn find_expr_start(s: &str, end: usize) -> usize {
    let bytes = s.as_bytes();
    let mut i = end;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while i > 0 {
        let prev = bytes[i - 1];
        if prev == b')' {
            paren_depth += 1;
            i -= 1;
        } else if prev == b'(' {
            if paren_depth > 0 {
                paren_depth -= 1;
                i -= 1;
            } else {
                break;
            }
        } else if prev == b']' {
            bracket_depth += 1;
            i -= 1;
        } else if prev == b'[' {
            if bracket_depth > 0 {
                bracket_depth -= 1;
                i -= 1;
            } else {
                break;
            }
        } else if paren_depth > 0 || bracket_depth > 0 {
            i -= 1;
        } else if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' || prev == b':' || prev == b'>' || prev == b'-' {
            i -= 1;
        } else {
            break;
        }
    }
    i
}

/// Matches a single code line against compiled structural tokens, extracting meta-variables.
fn match_line_structural(line: &str, tokens: &[PatternToken]) -> Option<HashMap<String, String>> {
    let trimmed = line.trim();
    let mut captures = HashMap::new();
    let mut cursor = 0usize;

    for (i, tok) in tokens.iter().enumerate() {
        match tok {
            PatternToken::Literal(lit) => {
                let remaining = &trimmed[cursor..];
                if let Some(pos) = remaining.find(lit) {
                    cursor += pos + lit.len();
                } else {
                    return None;
                }
            }
            PatternToken::MetaVariable(var) => {
                let remaining = &trimmed[cursor..];
                // Lookahead to find termination boundary
                let (start_offset, end_pos) = if let Some(PatternToken::Literal(next_lit)) = tokens.get(i + 1) {
                    if let Some(pos) = remaining.find(next_lit) {
                        let expr_start = find_expr_start(remaining, pos);
                        (expr_start, pos)
                    } else {
                        return None;
                    }
                } else {
                    let end = remaining.find(|c: char| c == ',' || c == ')' || c == ';' || c == ' ' || c == '}').unwrap_or(remaining.len());
                    (0, end)
                };

                let matched_val = remaining[start_offset..end_pos].trim();
                if matched_val.is_empty() {
                    return None;
                }
                captures.insert(var.clone(), matched_val.to_string());
                cursor += end_pos;
            }
            PatternToken::WildcardSingle => {
                let remaining = &trimmed[cursor..];
                let end_pos = remaining.find(|c: char| c == ',' || c == ')' || c == ';' || c == ' ').unwrap_or(remaining.len());
                cursor += end_pos;
            }
            PatternToken::WildcardSpread => {
                // Greedily consume until next token
                if let Some(PatternToken::Literal(next_lit)) = tokens.get(i + 1) {
                    let remaining = &trimmed[cursor..];
                    if let Some(pos) = remaining.find(next_lit) {
                        cursor += pos;
                    } else {
                        return None;
                    }
                } else {
                    cursor = trimmed.len();
                }
            }
        }
    }

    Some(captures)
}

fn walk_ast_search(dir: &Path, engine: &AstGrepEngine, out: &mut Vec<AstMatch>) -> Result<(), AstGrepError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => return Err(AstGrepError::Io(err)),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" {
            continue;
        }

        if path.is_dir() {
            walk_ast_search(&path, engine, out)?;
        } else if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(matches) = engine.search_source(&path.to_string_lossy(), &content) {
                    out.extend(matches);
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
    fn test_ast_grep_pattern_matching() {
        let mut engine = AstGrepEngine::new();
        engine.add_rule(AstRule {
            id: "no-unwrap".to_string(),
            language: "rust".to_string(),
            pattern: "$OBJ.unwrap()".to_string(),
            kind: None,
            regex: None,
            message: Some("Avoid unwraps".to_string()),
        });

        let code = r#"
fn process() {
    let x = conn.lock().unwrap();
    let y = safe_val;
}
"#;
        let matches = engine.search_source("src/test.rs", code).expect("Search should succeed");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_id, "no-unwrap");
        assert_eq!(matches[0].captures.get("OBJ").map(|s| s.as_str()), Some("conn.lock()"));
    }

    #[test]
    fn test_ast_grep_wildcard_spread() {
        let mut engine = AstGrepEngine::new();
        engine.add_rule(AstRule {
            id: "call-with-args".to_string(),
            language: "rust".to_string(),
            pattern: "execute($$$ARGS)".to_string(),
            kind: None,
            regex: None,
            message: None,
        });

        let code = "conn.execute(sql, params);";
        let matches = engine.search_source("src/db.rs", code).expect("Search");
        assert_eq!(matches.len(), 1);
    }
}
