// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/auto_memory.rs
// created: 2026-11-04
// lastModified: 2026-11-08

use std::fs;
use std::path::{Path, PathBuf};
use super::fts::{ContentStore, SearchOptions, SearchResult};

/// Prefix attached to memory labels in search results (D-UNIFIED-MEMORY-1, D-MEMORY-SOURCE-1)
pub const MEMORY_LABEL_PREFIX: &str = "memory ";

/// Memory file relative path segments inside the worktree
const MEMORY_REL_PATHS: &[&[&str]] = &[
    &["CLAUDE.md"],
    &["CLAUDE.local.md"],
    &[".claude", "CLAUDE.md"],
    &["AGENTS.md"],
    &["GEMINI.md"],
    &["QWEN.md"],
    &[".github", "copilot-instructions.md"],
];

#[derive(Debug, Clone)]
pub struct MemoryFile {
    pub label: String,
    pub path: PathBuf,
    pub content: String,
}

/// Reads all available auto-memory files in the worktree.
/// Never throws; non-existent files or paths leaving the worktree boundary are safely ignored.
pub fn read_auto_memory(worktree: &Path) -> Vec<MemoryFile> {
    let mut files = Vec::new();
    let abs_worktree = match worktree.canonicalize() {
        Ok(p) => p,
        Err(_) => worktree.to_path_buf(),
    };

    // 1. Standard agent memory files
    for parts in MEMORY_REL_PATHS {
        let mut rel = PathBuf::new();
        for p in *parts {
            rel.push(p);
        }
        let full_path = abs_worktree.join(&rel);
        if let Ok(canon) = full_path.canonicalize() {
            if canon.is_file() && canon.starts_with(&abs_worktree) {
                if let Ok(content) = fs::read_to_string(&canon) {
                    let label = parts.join("/");
                    files.push(MemoryFile {
                        label,
                        path: canon,
                        content,
                    });
                }
            }
        }
    }

    // 2. Kiro steering files: .kiro/steering/*.md
    let steering_dir = abs_worktree.join(".kiro").join("steering");
    if steering_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&steering_dir) {
            let mut steering_entries: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().map(|ext| ext == "md").unwrap_or(false))
                .collect();
            steering_entries.sort();

            for path in steering_entries {
                if let Ok(canon) = path.canonicalize() {
                    if canon.starts_with(&abs_worktree) {
                        if let (Ok(content), Some(file_name)) = (fs::read_to_string(&canon), path.file_name().and_then(|n| n.to_str())) {
                            let label = format!(".kiro/steering/{}", file_name);
                            files.push(MemoryFile {
                                label,
                                path: canon,
                                content,
                            });
                        }
                    }
                }
            }
        }
    }

    files
}

/// Searches worktree auto-memory files via SQLite FTS5 BM25.
/// If `source_filter` is specified, matches against that specific memory file.
pub fn query_auto_memory(
    worktree: &Path,
    query: &str,
    limit: usize,
    source_filter: Option<&str>,
) -> Vec<SearchResult> {
    let memory_files = read_auto_memory(worktree);
    if memory_files.is_empty() {
        return Vec::new();
    }

    let filtered_files: Vec<&MemoryFile> = if let Some(sf) = source_filter {
        let normalized = sf
            .strip_prefix(MEMORY_LABEL_PREFIX)
            .unwrap_or(sf)
            .replace('\\', "/");
        memory_files
            .iter()
            .filter(|f| f.label == normalized || f.label.ends_with(&normalized))
            .collect()
    } else {
        memory_files.iter().collect()
    };

    if filtered_files.is_empty() {
        return Vec::new();
    }

    let mut mem_store = match ContentStore::open_in_memory() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    for mf in filtered_files {
        let label = format!("{}{}", MEMORY_LABEL_PREFIX, mf.label);
        let _ = mem_store.index(&mf.content, &label, Some("markdown"), None);
    }

    let opts = SearchOptions {
        query: query.to_string(),
        limit,
        source: None,
        content_type: None,
        session_id: None,
    };

    mem_store.search(&opts).unwrap_or_default()
}

