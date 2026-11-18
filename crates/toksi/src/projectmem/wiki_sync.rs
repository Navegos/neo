// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/wiki_sync.rs
// created: 2026-11-18

#![deny(unsafe_code)]

use std::fs;
use std::path::Path;

use super::error::Result;
use super::store::{atomic_write_file, find_repo_root, project_map_path};
use super::structure::{build_structure, StructureNode};

/// Synchronize `<repo>/.toksi/projectmem/PROJECT_MAP.md` deterministically from
/// workspace structure and relationship edges without spending LLM tokens.
pub fn sync_project_map_from_structure(root: Option<&Path>) -> Result<String> {
    let repo_root = match root {
        Some(r) => r.to_path_buf(),
        None => find_repo_root(None),
    };
    let structure = build_structure(Some(&repo_root))?;
    let project_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    let map_file = project_map_path(Some(&repo_root))?;
    let existing = fs::read_to_string(&map_file).unwrap_or_default();

    // Preserve user-authored "## Project purpose" if present
    let purpose_block = extract_section(&existing, "## Project purpose")
        .unwrap_or_else(|| "Synchronized deterministically from workspace structure.".to_string());

    let mut out = String::new();
    out.push_str(&format!("# Project Map - {}\n\n", project_name));
    out.push_str(&format!(
        "Status: synced ({} files, {} directories, {} relationships).\n\n",
        structure.stats.files,
        structure.stats.dirs,
        structure.stats.relationships
    ));
    out.push_str("## Project purpose\n");
    out.push_str(purpose_block.trim());
    out.push_str("\n\n## Structure\n");

    let mut dirs = Vec::new();
    collect_dirs(std::slice::from_ref(&structure.tree), &mut dirs);
    dirs.sort_by(|a, b| a.0.cmp(&b.0));
    if dirs.is_empty() {
        out.push_str("- (root)\n");
    } else {
        for (path, children) in dirs.into_iter().take(40) {
            out.push_str(&format!("- `{}/` ({} children)\n", path, children));
        }
    }

    out.push_str("\n## Relationships\n");
    if structure.relationships.is_empty() {
        out.push_str("- None detected.\n");
    } else {
        for rel in structure.relationships.iter().take(40) {
            out.push_str(&format!(
                "- `{}` -> `{}` ({})\n",
                rel.source, rel.target, rel.kind
            ));
        }
    }

    atomic_write_file(&map_file, &out)?;
    Ok(out)
}

fn collect_dirs(nodes: &[StructureNode], out: &mut Vec<(String, usize)>) {
    for node in nodes {
        if node.node_type == "dir" {
            out.push((node.path.clone(), node.children.len()));
            collect_dirs(&node.children, out);
        }
    }
}

fn extract_section(doc: &str, heading: &str) -> Option<String> {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed == heading {
            in_section = true;
            continue;
        }
        if in_section {
            if trimmed.starts_with("## ") {
                break;
            }
            lines.push(line);
        }
    }
    let joined = lines.join("\n").trim().to_string();
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}
