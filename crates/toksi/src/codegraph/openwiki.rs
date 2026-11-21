// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/openwiki.rs
// created: 2026-11-21

//! Deterministic OpenWiki Mermaid architectural call-graph synthesizer
//! and `PROJECT_MAP.md` synchronization bridge.

#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::projectmem::wiki_sync::sync_project_map_from_structure;
use crate::util::paths::normalize_path;

/// Generates a deterministic Mermaid `flowchart TD` call-graph wiki from `(caller, callee, file_path)` edges.
pub fn synthesize_openwiki_markdown(
    project_name: &str,
    edges: &[(String, String, String)],
) -> String {
    let mut by_file: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for (caller, callee, file_path) in edges {
        let norm = normalize_path(file_path);
        by_file
            .entry(norm)
            .or_default()
            .insert((caller.clone(), callee.clone()));
    }

    let mut out = String::new();
    out.push_str(&format!("# OpenWiki Architecture — {project_name}\n\n"));
    out.push_str(&format!(
        "Indexed files: {} | Call edges: {}\n\n",
        by_file.len(),
        edges.len()
    ));
    out.push_str("```mermaid\nflowchart TD\n");

    let mut rendered = 0usize;
    for (_file, pairs) in &by_file {
        for (caller, callee) in pairs {
            if rendered >= 80 {
                break;
            }
            let c1 = sanitize_mermaid_id(caller);
            let c2 = sanitize_mermaid_id(callee);
            out.push_str(&format!("    {c1}[\"{caller}\"] --> {c2}[\"{callee}\"]\n"));
            rendered += 1;
        }
    }
    if rendered == 0 {
        out.push_str("    root[\"No call edges indexed\"]\n");
    }
    out.push_str("```\n\n## Module Index\n\n");
    for (file, pairs) in &by_file {
        out.push_str(&format!("- `{file}` ({} call edges)\n", pairs.len()));
    }
    out
}

/// Writes `<repo>/.toksi/ai-memory/wiki/openwiki.md` and synchronizes `PROJECT_MAP.md`.
pub fn write_openwiki_and_sync_project_map(
    repo_root: &Path,
    project_name: &str,
    edges: &[(String, String, String)],
) -> Result<String, String> {
    let wiki_dir = repo_root.join(".toksi").join("ai-memory").join("wiki");
    fs::create_dir_all(&wiki_dir).map_err(|e| format!("openwiki mkdir failed: {e}"))?;
    let markdown = synthesize_openwiki_markdown(project_name, edges);
    fs::write(wiki_dir.join("openwiki.md"), &markdown)
        .map_err(|e| format!("openwiki write failed: {e}"))?;
    let _ = sync_project_map_from_structure(Some(repo_root));
    Ok(markdown)
}

fn sanitize_mermaid_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "node".to_string()
    } else {
        out
    }
}
