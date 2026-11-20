// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/patch.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Lark `ApplyPatchTool`, `HashlineEditTool`, unified `HookEventName` matrix,
//! and TOML/MD custom slash command loader.

#![deny(unsafe_code)]

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use crate::plugins::hooks::HookEventName;
use crate::util::paths::{normalize_path, write_file_atomic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchOp {
    AddFile { path: String, content: String },
    DeleteFile { path: String },
    UpdateFile { path: String, hunks: Vec<PatchHunk> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchHunk {
    pub removed: Vec<String>,
    pub added: Vec<String>,
}

/// Parses and applies Lark-style `*** Begin Patch` / `*** End Patch` blocks safely inside `workspace_root`.
pub struct ApplyPatchTool;

impl ApplyPatchTool {
    pub fn parse(patch_text: &str) -> Result<Vec<PatchOp>, String> {
        let lines: Vec<&str> = patch_text.lines().collect();
        let begin_idx = lines
            .iter()
            .position(|l| l.trim() == "*** Begin Patch")
            .ok_or_else(|| "Missing '*** Begin Patch' header".to_string())?;
        let end_idx = lines
            .iter()
            .rposition(|l| l.trim() == "*** End Patch")
            .ok_or_else(|| "Missing '*** End Patch' footer".to_string())?;
        if end_idx <= begin_idx {
            return Err("Invalid patch boundaries".to_string());
        }

        let mut ops = Vec::new();
        let mut i = begin_idx + 1;
        while i < end_idx {
            let line = lines[i].trim_end();
            if let Some(path) = line.strip_prefix("*** Add File: ") {
                i += 1;
                let mut content_lines = Vec::new();
                while i < end_idx && !lines[i].starts_with("*** ") {
                    let body = lines[i].strip_prefix('+').unwrap_or(lines[i]);
                    content_lines.push(body);
                    i += 1;
                }
                ops.push(PatchOp::AddFile {
                    path: path.trim().to_string(),
                    content: content_lines.join("\n"),
                });
            } else if let Some(path) = line.strip_prefix("*** Delete File: ") {
                ops.push(PatchOp::DeleteFile {
                    path: path.trim().to_string(),
                });
                i += 1;
            } else if let Some(path) = line.strip_prefix("*** Update File: ") {
                i += 1;
                let mut hunks = Vec::new();
                let mut removed = Vec::new();
                let mut added = Vec::new();
                while i < end_idx && !lines[i].starts_with("*** ") {
                    let l = lines[i];
                    if l.starts_with("@@") {
                        if !removed.is_empty() || !added.is_empty() {
                            hunks.push(PatchHunk {
                                removed: std::mem::take(&mut removed),
                                added: std::mem::take(&mut added),
                            });
                        }
                    } else if let Some(r) = l.strip_prefix('-') {
                        removed.push(r.to_string());
                    } else if let Some(a) = l.strip_prefix('+') {
                        added.push(a.to_string());
                    } else if let Some(c) = l.strip_prefix(' ') {
                        removed.push(c.to_string());
                        added.push(c.to_string());
                    }
                    i += 1;
                }
                if !removed.is_empty() || !added.is_empty() {
                    hunks.push(PatchHunk { removed, added });
                }
                ops.push(PatchOp::UpdateFile {
                    path: path.trim().to_string(),
                    hunks,
                });
            } else {
                i += 1;
            }
        }
        Ok(ops)
    }

    pub fn apply(workspace_root: &Path, patch_text: &str) -> Result<Vec<String>, String> {
        let ops = Self::parse(patch_text)?;
        let mut touched = Vec::new();
        for op in ops {
            match op {
                PatchOp::AddFile { path, content } => {
                    let full = resolve_safe_path(workspace_root, &path)?;
                    write_file_atomic(&full, &content)
                        .map_err(|e| format!("Failed to add '{}': {}", path, e))?;
                    touched.push(normalize_path(&path));
                }
                PatchOp::DeleteFile { path } => {
                    let full = resolve_safe_path(workspace_root, &path)?;
                    if full.exists() {
                        std::fs::remove_file(&full)
                            .map_err(|e| format!("Failed to delete '{}': {}", path, e))?;
                    }
                    touched.push(normalize_path(&path));
                }
                PatchOp::UpdateFile { path, hunks } => {
                    let full = resolve_safe_path(workspace_root, &path)?;
                    let mut text = std::fs::read_to_string(&full)
                        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
                    let had_trailing_nl = text.ends_with('\n');
                    for hunk in hunks {
                        let old_block = hunk.removed.join("\n");
                        let new_block = hunk.added.join("\n");
                        if !old_block.is_empty() {
                            if !text.contains(&old_block) {
                                return Err(format!("Hunk target not found in '{}'", path));
                            }
                            text = text.replacen(&old_block, &new_block, 1);
                        } else if !new_block.is_empty() {
                            if !text.is_empty() && !text.ends_with('\n') {
                                text.push('\n');
                            }
                            text.push_str(&new_block);
                        }
                    }
                    if had_trailing_nl && !text.ends_with('\n') {
                        text.push('\n');
                    }
                    write_file_atomic(&full, &text)
                        .map_err(|e| format!("Failed to write '{}': {}", path, e))?;
                    touched.push(normalize_path(&path));
                }
            }
        }
        Ok(touched)
    }
}

/// HashlineEditTool: line-hash anchored edits preventing stale-line corruption.
pub struct HashlineEditTool;

impl HashlineEditTool {
    pub fn line_hash(line: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(line.as_bytes());
        let out = hasher.finalize();
        format!("{:02x}{:02x}{:02x}{:02x}", out[0], out[1], out[2], out[3])
    }

    pub fn format_numbered(content: &str) -> Vec<String> {
        content
            .lines()
            .enumerate()
            .map(|(idx, line)| format!("{}:{}: {}", idx + 1, Self::line_hash(line), line))
            .collect()
    }

    pub fn replace_line(
        workspace_root: &Path,
        rel_path: &str,
        line_number_1_based: usize,
        expected_hash: &str,
        replacement: &str,
    ) -> Result<(), String> {
        if line_number_1_based == 0 {
            return Err("Line number must be >= 1".to_string());
        }
        let full = resolve_safe_path(workspace_root, rel_path)?;
        let raw = std::fs::read_to_string(&full)
            .map_err(|e| format!("Failed to read '{}': {}", rel_path, e))?;
        let had_trailing_nl = raw.ends_with('\n');
        let mut lines: Vec<String> = raw.lines().map(ToString::to_string).collect();
        let idx = line_number_1_based - 1;
        let current = lines
            .get(idx)
            .ok_or_else(|| format!("Line {} out of range (len={})", line_number_1_based, lines.len()))?;
        let actual_hash = Self::line_hash(current);
        if !actual_hash.eq_ignore_ascii_case(expected_hash.trim()) {
            return Err(format!(
                "Hash mismatch at line {}: expected '{}', actual '{}'",
                line_number_1_based, expected_hash, actual_hash
            ));
        }
        lines[idx] = replacement.to_string();
        let mut updated = lines.join("\n");
        if had_trailing_nl {
            updated.push('\n');
        }
        write_file_atomic(&full, &updated)
            .map_err(|e| format!("Failed to write '{}': {}", rel_path, e))
    }
}

fn resolve_safe_path(workspace_root: &Path, rel: &str) -> Result<PathBuf, String> {
    let p = Path::new(rel);
    if p.is_absolute() {
        return Err(format!("Absolute patch paths are forbidden: '{}'", rel));
    }
    for comp in p.components() {
        if matches!(comp, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err(format!("Path traversal prohibited: '{}'", rel));
        }
    }
    Ok(workspace_root.join(p))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSlashCommand {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub source_path: String,
}

#[derive(Debug, Deserialize)]
struct TomlSlashFile {
    #[serde(default)]
    description: String,
    #[serde(default)]
    prompt: String,
}

/// Loads custom TOML/Markdown slash commands from `<workspace>/.agents/commands` and `.toksi/commands`.
pub fn load_custom_slash_commands(workspace_root: &Path) -> Vec<CustomSlashCommand> {
    let dirs = [
        workspace_root.join(".agents").join("commands"),
        workspace_root.join(".toksi").join("commands"),
    ];
    let mut out = Vec::new();
    for dir in &dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Ok(raw) = std::fs::read_to_string(&path) else {
                continue;
            };
            if ext.eq_ignore_ascii_case("toml") {
                if let Ok(parsed) = toml::from_str::<TomlSlashFile>(&raw) {
                    out.push(CustomSlashCommand {
                        name: format!("/{}", stem.to_ascii_lowercase()),
                        description: parsed.description,
                        prompt: parsed.prompt,
                        source_path: normalize_path(&path.to_string_lossy()),
                    });
                }
            } else if ext.eq_ignore_ascii_case("md") {
                let desc = raw
                    .lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("Custom markdown slash command")
                    .trim_start_matches('#')
                    .trim()
                    .to_string();
                out.push(CustomSlashCommand {
                    name: format!("/{}", stem.to_ascii_lowercase()),
                    description: desc,
                    prompt: raw,
                    source_path: normalize_path(&path.to_string_lossy()),
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}
