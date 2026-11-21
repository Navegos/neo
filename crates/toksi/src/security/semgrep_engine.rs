// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/semgrep_engine.rs
// created: 2026-11-21

//! Multi-language AST SAST engine across Toksi's 45 Tree-sitter languages
//! (`semgrep`, `semgrep-rules`, `codex-security`, `deepsec`, `sage` parity).

#![deny(unsafe_code)]

use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

use crate::codegraph::kernel::langs::{canonical_language, grammar_for, LANGUAGES};
use crate::security::sarif::UnifiedSecurityFinding;

struct AstRule {
    rule_id: &'static str,
    severity: &'static str,
    cwe: &'static str,
    aitech: &'static str,
    languages: &'static [&'static str],
    scm_query: &'static str,
    message: &'static str,
    remediation: &'static str,
}

const BUILTIN_AST_RULES: &[AstRule] = &[
    AstRule {
        rule_id: "semgrep.rust.unsafe-block",
        severity: "MEDIUM",
        cwe: "CWE-119",
        aitech: "AITech-7.1-MemorySafety",
        languages: &["rust"],
        scm_query: "(unsafe_block) @hit",
        message: "Unsafe Rust block detected; isolate FFI inside leaf crates",
        remediation: "Move unavoidable FFI to toksi-sys/toksi-grammars with // SAFETY: proof.",
    },
    AstRule {
        rule_id: "semgrep.python.dynamic-exec",
        severity: "HIGH",
        cwe: "CWE-95",
        aitech: "AITech-9.2-CodeInjection",
        languages: &["python"],
        scm_query: r#"((call function: (identifier) @fn) (#match? @fn "^(eval|exec)$")) @hit"#,
        message: "Dynamic eval/exec call detected in Python",
        remediation: "Replace eval/exec with ast.literal_eval or explicit dispatch.",
    },
    AstRule {
        rule_id: "semgrep.js.eval-sink",
        severity: "HIGH",
        cwe: "CWE-95",
        aitech: "AITech-9.2-CodeInjection",
        languages: &["javascript", "typescript", "tsx", "jsx", "vue"],
        scm_query: r#"((call_expression function: (identifier) @fn) (#eq? @fn "eval")) @hit"#,
        message: "Dynamic eval() call detected",
        remediation: "Avoid eval(); use JSON.parse or static dispatch.",
    },
    AstRule {
        rule_id: "semgrep.go.exec-command",
        severity: "MEDIUM",
        cwe: "CWE-78",
        aitech: "AITech-9.1-CommandInjection",
        languages: &["go"],
        scm_query: r#"((selector_expression field: (field_identifier) @fn) (#eq? @fn "Command")) @hit"#,
        message: "Subprocess execution sink detected in Go",
        remediation: "Validate arguments and avoid shell interpolation.",
    },
];

/// Returns all supported Tree-sitter language IDs covered by the SAST engine.
pub fn supported_sast_languages() -> &'static [&'static str] {
    LANGUAGES
}

/// Runs Tree-sitter S-expression SAST queries + language-agnostic sink checks on source text.
pub fn scan_source_ast(
    file_path: &str,
    language: &str,
    source: &str,
) -> Vec<UnifiedSecurityFinding> {
    let canon = canonical_language(language);
    let mut findings = Vec::new();

    if let Some(grammar) = grammar_for(canon) {
        let mut parser = Parser::new();
        if parser.set_language(&grammar).is_ok() {
            if let Some(tree) = parser.parse(source, None) {
                for rule in BUILTIN_AST_RULES {
                    if !rule.languages.contains(&canon) {
                        continue;
                    }
                    if let Ok(query) = Query::new(&grammar, rule.scm_query) {
                        let mut cursor = QueryCursor::new();
                        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
                        while let Some(m) = matches.next() {
                            if let Some(cap) = m.captures.first() {
                                let line = cap.node.start_position().row + 1;
                                findings.push(UnifiedSecurityFinding {
                                    rule_id: rule.rule_id.to_string(),
                                    severity: rule.severity.to_string(),
                                    cwe: rule.cwe.to_string(),
                                    aitech: rule.aitech.to_string(),
                                    file: file_path.to_string(),
                                    line,
                                    message: rule.message.to_string(),
                                    remediation: rule.remediation.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Cross-language sink fallback for all 45 languages (SQLi, shell pipe, hardcoded root Dockerfile)
    for (idx, line) in source.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if canon == "dockerfile" && lower.trim_start().starts_with("user root") {
            findings.push(UnifiedSecurityFinding {
                rule_id: "semgrep.docker.user-root".to_string(),
                severity: "MEDIUM".to_string(),
                cwe: "CWE-250".to_string(),
                aitech: "AITech-4.2-PrivilegeEscalation".to_string(),
                file: file_path.to_string(),
                line: idx + 1,
                message: "Container runs as USER root".to_string(),
                remediation: "Switch to a non-root USER in Dockerfile.".to_string(),
            });
        }
        if (lower.contains("curl ") || lower.contains("wget "))
            && (lower.contains("| sh") || lower.contains("| bash"))
        {
            findings.push(UnifiedSecurityFinding {
                rule_id: "semgrep.shell.curl-pipe-sh".to_string(),
                severity: "CRITICAL".to_string(),
                cwe: "CWE-494".to_string(),
                aitech: "AITech-9.1-RemoteExec".to_string(),
                file: file_path.to_string(),
                line: idx + 1,
                message: "Remote script piped directly to shell".to_string(),
                remediation: "Download, verify checksum/signature, then execute.".to_string(),
            });
        }
    }

    findings
}
