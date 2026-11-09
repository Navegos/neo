// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/repomix.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::collections::BTreeMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum RepomixError {
    #[error("I/O error during packaging: {0}")]
    Io(#[from] std::io::Error),
    #[error("Token limit exceeded: required {required}, budget {budget}")]
    TokenBudgetExceeded { required: u64, budget: u64 },
}

/// Output formatting style for the packaged repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepomixFormat {
    Xml,
    Markdown,
    Json,
}

/// Packaged file metadata and token stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagedFile {
    pub path: String,
    pub content: String,
    pub char_count: usize,
    pub token_count: u64,
}

/// Configuration options for Repomix packing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepomixOptions {
    pub format: RepomixFormat,
    pub max_tokens: Option<u64>,
    pub include_tree: bool,
    pub header_comment: Option<String>,
}

impl Default for RepomixOptions {
    fn default() -> Self {
        Self {
            format: RepomixFormat::Xml,
            max_tokens: None,
            include_tree: true,
            header_comment: None,
        }
    }
}

/// Packaged codebase bundle with directory tree, files, and token analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepomixBundle {
    pub total_files: usize,
    pub total_chars: usize,
    pub total_tokens: u64,
    pub directory_tree: String,
    pub rendered_output: String,
    pub files: Vec<PackagedFile>,
}

/// Estimates tokens for a given text (~4 characters / bytes per token).
pub fn estimate_tokens(text: &str) -> u64 {
    (text.len() / 4).max(1) as u64
}

/// Generates a visual directory tree structure string from a list of normalized paths.
pub fn generate_directory_tree(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }

    let mut tree: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in paths {
        let norm = normalize_path(p);
        let parts: Vec<&str> = norm.split('/').collect();
        if parts.len() <= 1 {
            tree.entry(".".to_string()).or_default().push(norm);
        } else {
            let parent = parts[0..parts.len() - 1].join("/");
            let file_name = parts[parts.len() - 1].to_string();
            tree.entry(parent).or_default().push(file_name);
        }
    }

    let mut out = String::new();
    for (dir, entries) in tree {
        if dir == "." {
            out.push_str(".\n");
        } else {
            out.push_str(&format!("{}/\n", dir));
        }
        for entry in entries {
            out.push_str(&format!("  └── {}\n", entry));
        }
    }

    out
}

/// Packages a map of files into structured context payload according to Repomix formatting conventions.
pub fn package_files(
    files_map: &[(String, String)], // (relative_path, content)
    options: &RepomixOptions,
) -> Result<RepomixBundle, RepomixError> {
    let mut packaged = Vec::with_capacity(files_map.len());
    let mut total_chars = 0usize;
    let mut total_tokens = 0u64;
    let mut paths_list = Vec::with_capacity(files_map.len());

    for (rel_path, content) in files_map {
        let norm_path = normalize_path(rel_path);
        let chars = content.chars().count();
        let tokens = estimate_tokens(content);

        paths_list.push(norm_path.clone());
        total_chars += chars;
        total_tokens = total_tokens.saturating_add(tokens);

        packaged.push(PackagedFile {
            path: norm_path,
            content: content.clone(),
            char_count: chars,
            token_count: tokens,
        });
    }

    let dir_tree = if options.include_tree {
        generate_directory_tree(&paths_list)
    } else {
        String::new()
    };

    let tree_tokens = estimate_tokens(&dir_tree);
    total_tokens = total_tokens.saturating_add(tree_tokens);

    if let Some(budget) = options.max_tokens {
        if total_tokens > budget {
            return Err(RepomixError::TokenBudgetExceeded {
                required: total_tokens,
                budget,
            });
        }
    }

    let mut rendered = String::with_capacity(total_chars + 1024);

    if let Some(header) = &options.header_comment {
        rendered.push_str(header);
        rendered.push('\n');
    }

    match options.format {
        RepomixFormat::Xml => {
            rendered.push_str("<repomix_output>\n");
            if options.include_tree && !dir_tree.is_empty() {
                rendered.push_str("<directory_structure>\n");
                rendered.push_str(&dir_tree);
                rendered.push_str("</directory_structure>\n\n");
            }
            rendered.push_str("<files>\n");
            for f in &packaged {
                rendered.push_str(&format!("<file path=\"{}\">\n", f.path));
                rendered.push_str(&f.content);
                if !f.content.ends_with('\n') {
                    rendered.push('\n');
                }
                rendered.push_str("</file>\n");
            }
            rendered.push_str("</files>\n");
            rendered.push_str("</repomix_output>\n");
        }
        RepomixFormat::Markdown => {
            rendered.push_str("# Codebase Context\n\n");
            if options.include_tree && !dir_tree.is_empty() {
                rendered.push_str("## Directory Structure\n```text\n");
                rendered.push_str(&dir_tree);
                rendered.push_str("```\n\n");
            }
            rendered.push_str("## Files\n\n");
            for f in &packaged {
                rendered.push_str(&format!("### File: `{}`\n```\n", f.path));
                rendered.push_str(&f.content);
                if !f.content.ends_with('\n') {
                    rendered.push('\n');
                }
                rendered.push_str("```\n\n");
            }
        }
        RepomixFormat::Json => {
            let json_str = serde_json::to_string_pretty(&packaged).unwrap_or_default();
            rendered.push_str(&json_str);
        }
    }

    Ok(RepomixBundle {
        total_files: packaged.len(),
        total_chars,
        total_tokens,
        directory_tree: dir_tree,
        rendered_output: rendered,
        files: packaged,
    })
}

/// Recursively traverses a root workspace and packages codebase files while respecting ignore patterns.
pub fn pack_workspace(root: &Path, options: &RepomixOptions) -> Result<RepomixBundle, RepomixError> {
    let mut files_vec = Vec::new();
    walk_and_collect(root, root, &mut files_vec)?;
    package_files(&files_vec, options)
}

fn walk_and_collect(
    base: &Path,
    current: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), RepomixError> {
    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(err) => return Err(RepomixError::Io(err)),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" {
            continue;
        }

        if path.is_dir() {
            walk_and_collect(base, &path, out)?;
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                let rel_str = normalize_path(rel.to_string_lossy().as_ref());
                if let Ok(content) = std::fs::read_to_string(&path) {
                    out.push((rel_str, content));
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
    fn test_repomix_xml_packaging() {
        let files = vec![
            ("src/main.rs".to_string(), "fn main() { println!(\"Hi\"); }".to_string()),
            ("Cargo.toml".to_string(), "[package]\nname = \"toksi\"".to_string()),
        ];
        let options = RepomixOptions {
            format: RepomixFormat::Xml,
            max_tokens: Some(5000),
            include_tree: true,
            header_comment: Some("<!-- Generated by Toksi Repomix Engine -->".to_string()),
        };

        let bundle = package_files(&files, &options).expect("Packaging should succeed");
        assert_eq!(bundle.total_files, 2);
        assert!(bundle.rendered_output.contains("<repomix_output>"));
        assert!(bundle.rendered_output.contains("<file path=\"src/main.rs\">"));
        assert!(bundle.rendered_output.contains("<!-- Generated by Toksi Repomix Engine -->"));
        assert!(bundle.total_tokens > 0);
    }

    #[test]
    fn test_repomix_markdown_packaging() {
        let files = vec![
            ("lib.py".to_string(), "def calc(): return 42".to_string()),
        ];
        let options = RepomixOptions {
            format: RepomixFormat::Markdown,
            max_tokens: None,
            include_tree: false,
            header_comment: None,
        };

        let bundle = package_files(&files, &options).expect("Packaging markdown");
        assert!(bundle.rendered_output.contains("### File: `lib.py`"));
    }

    #[test]
    fn test_token_budget_rejection() {
        let files = vec![
            ("big.txt".to_string(), "A".repeat(4000)),
        ];
        let options = RepomixOptions {
            format: RepomixFormat::Xml,
            max_tokens: Some(50), // strict budget
            include_tree: false,
            header_comment: None,
        };

        let result = package_files(&files, &options);
        assert!(matches!(result, Err(RepomixError::TokenBudgetExceeded { .. })));
    }
}
