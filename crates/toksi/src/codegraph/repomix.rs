// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/repomix.rs
// created: 2026-11-09
// lastModified: 2026-11-13

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::walker;
use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum RepomixError {
    #[error("Token budget exceeded: {actual} tokens > limit {limit}")]
    TokenBudgetExceeded { actual: usize, limit: usize },
    #[error("I/O error scanning files for repomix: {0}")]
    Io(#[from] std::io::Error),
}

/// Output formatting protocol for packed codebase contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepomixFormat {
    Xml,
    Markdown,
    Json,
}

impl Default for RepomixFormat {
    fn default() -> Self {
        Self::Xml
    }
}

fn default_true() -> bool {
    true
}

/// Configuration options for Repomix packing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepomixOptions {
    pub format: RepomixFormat,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_tokens: Option<usize>,
    pub header_comment: Option<String>,
    #[serde(default = "default_true")]
    pub redact_secrets: bool,
}

impl Default for RepomixOptions {
    fn default() -> Self {
        Self {
            format: RepomixFormat::Xml,
            include_patterns: vec![],
            exclude_patterns: vec![
                "node_modules".into(),
                "target".into(),
                ".git".into(),
                ".toksi".into(),
            ],
            max_tokens: None,
            header_comment: None,
            redact_secrets: true,
        }
    }
}

/// Packed file entry inside a Repomix bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedFile {
    pub path: String,
    pub content: String,
    pub lines: usize,
    pub chars: usize,
    pub estimated_tokens: usize,
    pub redacted_secrets: usize,
}

/// Resulting Repomix context archive container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepomixBundle {
    pub files: Vec<PackedFile>,
    pub total_files: usize,
    pub total_lines: usize,
    pub total_chars: usize,
    pub total_estimated_tokens: usize,
    pub total_redacted_secrets: usize,
    pub rendered_output: String,
}

/// Built-in secretlint engine: redacts API keys, private keys, JWTs, and URL credentials.
pub fn redact_secrets(content: &str) -> (String, usize) {
    const RULES: &[(&str, &str)] = &[
        ("AWS_ACCESS_KEY", r"\bAKIA[0-9A-Z]{16}\b"),
        ("GITHUB_TOKEN", r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b"),
        ("GITHUB_PAT", r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
        ("API_SECRET_KEY", r"\b(?:sk-[A-Za-z0-9_-]{20,}|sk-ant-[A-Za-z0-9_-]{20,}|rk_live_[A-Za-z0-9]{16,})\b"),
        ("PRIVATE_KEY", r"-----BEGIN (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----"),
        ("JWT_TOKEN", r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"),
        ("URL_PASSWORD", r"(https?://[^:\s/@]+:)([^@\s/]+)(@)"),
    ];

    let mut out = content.to_string();
    let mut count = 0usize;

    for &(label, pat) in RULES {
        let Ok(re) = Regex::new(pat) else {
            continue;
        };
        if label == "URL_PASSWORD" {
            let matches = re.find_iter(&out).count();
            if matches > 0 {
                count += matches;
                out = re
                    .replace_all(&out, format!("$1[REDACTED:{}]$3", label))
                    .into_owned();
            }
        } else {
            let matches = re.find_iter(&out).count();
            if matches > 0 {
                count += matches;
                out = re
                    .replace_all(&out, format!("[REDACTED:{}]", label))
                    .into_owned();
            }
        }
    }

    (out, count)
}

/// BPE-calibrated cl100k/o200k token counter splitting identifiers, camelCase/snake_case
/// subwords, numbers, punctuation operators, and whitespace chunks without external data files.
pub fn estimate_bpe_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let bytes = text.as_bytes();
    let mut tokens = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            let start = i;
            while i < bytes.len() && bytes[i] == b && (i - start) < 4 {
                i += 1;
            }
            // Single space attaches to adjacent token in BPE most of the time; newlines count as 1 token
            if b == b'\n' || (i - start) > 1 {
                tokens += 1;
            }
            continue;
        }
        if b.is_ascii_alphabetic() {
            let start = i;
            i += 1;
            // Split camelCase / PascalCase boundaries and ~6-char common subwords
            while i < bytes.len() && bytes[i].is_ascii_lowercase() && (i - start) < 6 {
                i += 1;
            }
            tokens += 1;
            continue;
        }
        if b.is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() && (i - start) < 3 {
                i += 1;
            }
            tokens += 1;
            continue;
        }
        // Group repeated punctuation operators (`::`, `->`, `=>`, `===`, etc.) up to 3 chars
        let start = i;
        i += 1;
        while i < bytes.len()
            && !bytes[i].is_ascii_alphanumeric()
            && !bytes[i].is_ascii_whitespace()
            && (i - start) < 3
            && bytes[i] == b
        {
            i += 1;
        }
        tokens += 1;
    }

    tokens.max(1)
}

/// Packages a map of files into structured context payload with secretlint redaction and BPE estimation.
pub fn package_files(
    files: &BTreeMap<String, String>,
    options: &RepomixOptions,
) -> Result<RepomixBundle, RepomixError> {
    let mut packed_files = Vec::new();
    let mut total_lines = 0;
    let mut total_chars = 0;
    let mut total_estimated_tokens = 0;
    let mut total_redacted_secrets = 0;

    for (raw_path, raw_content) in files {
        let norm_path = normalize_path(raw_path);

        if options.exclude_patterns.iter().any(|p| norm_path.contains(p.as_str())) {
            continue;
        }
        if !options.include_patterns.is_empty()
            && !options.include_patterns.iter().any(|p| norm_path.contains(p.as_str()))
        {
            continue;
        }

        let (content, redacted) = if options.redact_secrets {
            redact_secrets(raw_content)
        } else {
            (raw_content.clone(), 0)
        };

        let lines = content.lines().count();
        let chars = content.len();
        let est_tokens = estimate_bpe_tokens(&content);

        total_lines += lines;
        total_chars += chars;
        total_estimated_tokens += est_tokens;
        total_redacted_secrets += redacted;

        packed_files.push(PackedFile {
            path: norm_path,
            content,
            lines,
            chars,
            estimated_tokens: est_tokens,
            redacted_secrets: redacted,
        });
    }

    if let Some(budget) = options.max_tokens {
        if total_estimated_tokens > budget {
            return Err(RepomixError::TokenBudgetExceeded {
                actual: total_estimated_tokens,
                limit: budget,
            });
        }
    }

    let mut rendered = String::new();
    if let Some(ref hdr) = options.header_comment {
        rendered.push_str(hdr);
        rendered.push('\n');
    }

    match options.format {
        RepomixFormat::Xml => {
            rendered.push_str("<repomix_output>\n");
            rendered.push_str(&format!(
                "  <summary total_files=\"{}\" total_tokens=\"{}\" redacted_secrets=\"{}\" />\n",
                packed_files.len(),
                total_estimated_tokens,
                total_redacted_secrets
            ));
            rendered.push_str("  <files>\n");
            for f in &packed_files {
                rendered.push_str(&format!("    <file path=\"{}\">\n", xml_escape(&f.path)));
                rendered.push_str(&f.content);
                if !f.content.ends_with('\n') {
                    rendered.push('\n');
                }
                rendered.push_str("    </file>\n");
            }
            rendered.push_str("  </files>\n");
            rendered.push_str("</repomix_output>\n");
        }
        RepomixFormat::Markdown => {
            rendered.push_str(&format!(
                "# Codebase Context\n\nTotal Files: {} | BPE Estimated Tokens: {} | Redacted Secrets: {}\n\n",
                packed_files.len(),
                total_estimated_tokens,
                total_redacted_secrets
            ));
            for f in &packed_files {
                let ext = Path::new(&f.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                rendered.push_str(&format!("## {}\n```{}\n", f.path, ext));
                rendered.push_str(&f.content);
                if !f.content.ends_with('\n') {
                    rendered.push('\n');
                }
                rendered.push_str("```\n\n");
            }
        }
        RepomixFormat::Json => {
            rendered = serde_json::to_string_pretty(&packed_files).unwrap_or_else(|_| "[]".into());
        }
    }

    Ok(RepomixBundle {
        total_files: packed_files.len(),
        files: packed_files,
        total_lines,
        total_chars,
        total_estimated_tokens,
        total_redacted_secrets,
        rendered_output: rendered,
    })
}

/// Walks a workspace respecting `.gitignore`, `.ignore`, and `.repomixignore`, then packages into a bundle.
pub fn pack_workspace(root: &Path, options: &RepomixOptions) -> Result<RepomixBundle, RepomixError> {
    let mut files = BTreeMap::new();
    let mut wb = WalkBuilder::new(root);
    wb.hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .ignore(true)
        .add_custom_ignore_filename(".repomixignore")
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".repomixignore" && !walker::is_build_or_vendor_artifact(&name)
        });

    for entry in wb.build().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        if meta.len() > walker::MAX_SOURCE_FILE_SIZE {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map(|p| normalize_path(&p.to_string_lossy()))
            .unwrap_or_else(|_| normalize_path(&path.to_string_lossy()));
        if let Ok(content) = fs::read_to_string(path) {
            files.insert(rel, content);
        }
    }

    package_files(&files, options)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repomix_xml_packaging_and_secretlint() {
        let mut files = BTreeMap::new();
        files.insert("src\\main.rs".to_string(), "fn main() { let k = \"AKIA1234567890ABCDEF\"; }\n".to_string());
        files.insert("src/lib.rs".to_string(), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n".to_string());

        let options = RepomixOptions {
            format: RepomixFormat::Xml,
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_tokens: None,
            header_comment: Some("<!-- Generated by Toksi Repomix Engine -->".to_string()),
            redact_secrets: true,
        };

        let bundle = package_files(&files, &options).expect("Packaging should succeed");
        assert_eq!(bundle.total_files, 2);
        assert_eq!(bundle.total_redacted_secrets, 1);
        assert!(bundle.rendered_output.contains("[REDACTED:AWS_ACCESS_KEY]"));
        assert!(!bundle.rendered_output.contains("AKIA1234567890ABCDEF"));
        assert!(bundle.rendered_output.contains("<file path=\"src/main.rs\">"));
    }

    #[test]
    fn test_repomix_markdown_and_bpe_budget() {
        let mut files = BTreeMap::new();
        files.insert("README.md".to_string(), "# Hello World\nWelcome.\n".to_string());

        let options = RepomixOptions {
            format: RepomixFormat::Markdown,
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_tokens: None,
            header_comment: None,
            redact_secrets: true,
        };

        let bundle = package_files(&files, &options).expect("Packaging markdown");
        assert!(bundle.rendered_output.contains("## README.md"));
        assert!(bundle.total_estimated_tokens >= 4);

        let tight = RepomixOptions {
            max_tokens: Some(1),
            ..options
        };
        assert!(matches!(
            package_files(&files, &tight),
            Err(RepomixError::TokenBudgetExceeded { .. })
        ));
    }

    #[test]
    fn test_repomix_workspace_respects_repomixignore() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join(".git")).expect("git");
        fs::write(dir.path().join(".repomixignore"), "*.secret\n").expect("repomixignore");
        fs::write(dir.path().join("keep.rs"), "pub fn ok() {}\n").expect("keep");
        fs::write(dir.path().join("drop.secret"), "ghp_1234567890abcdefghij1234567890\n").expect("drop");

        let bundle = pack_workspace(dir.path(), &RepomixOptions::default()).expect("pack");
        assert_eq!(bundle.total_files, 1);
        assert_eq!(bundle.files[0].path, "keep.rs");
    }
}
