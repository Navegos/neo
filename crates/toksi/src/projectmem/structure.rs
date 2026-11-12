// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/structure.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::error::Result;
use super::store::{atomic_write_file, find_repo_root, require_mem_dir, STRUCTURE_FILE};
use crate::util::paths::normalize_path;

pub const MAX_FILES: usize = 5000;

pub const IGNORE_DIRS: &[&str] = &[
    ".git", ".hg", ".svn", ".projectmem", ".toksi", ".idea", ".vscode",
    ".pytest_cache", ".mypy_cache", ".ruff_cache", "__pycache__",
    "node_modules", "venv", "env", "virtualenv", ".venv", ".tox", ".eggs",
    "build", "dist", "target", ".gradle", "htmlcov", "coverage",
    ".next", ".nuxt", ".cache", "site-packages",
];

pub const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "jsx", "ts", "tsx", "go", "rb", "java",
    "c", "cpp", "h", "hpp", "cs", "php", "swift", "kt",
    "html", "css", "scss", "md", "json", "toml", "yaml", "yml",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<StructureNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureRelationship {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureStats {
    pub files: usize,
    pub dirs: usize,
    pub relationships: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureResult {
    pub generated: String,
    pub root: String,
    pub stats: StructureStats,
    pub tree: StructureNode,
    pub files: Vec<String>,
    pub relationships: Vec<StructureRelationship>,
}

fn walk_dir(
    dir: &Path,
    rel_prefix: &str,
    collected_files: &mut Vec<String>,
    dir_count: &mut usize,
) -> StructureNode {
    let mut children = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        let mut entry_vec = Vec::new();
        for e in entries.flatten() {
            entry_vec.push(e);
        }
        entry_vec.sort_by_key(|e| e.file_name());

        for entry in entry_vec {
            if collected_files.len() >= MAX_FILES {
                break;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with('.') && file_name != ".gitignore" {
                continue;
            }

            let path = entry.path();
            if path.is_dir() {
                if IGNORE_DIRS.contains(&file_name.as_str()) {
                    continue;
                }
                *dir_count += 1;
                let sub_rel = if rel_prefix.is_empty() {
                    format!("{}/", file_name)
                } else {
                    format!("{}{}/", rel_prefix, file_name)
                };
                let sub_node = walk_dir(&path, &sub_rel, collected_files, dir_count);
                if !sub_node.children.is_empty() {
                    children.push(sub_node);
                }
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if CODE_EXTENSIONS.contains(&ext.as_str()) {
                    let rel_path = if rel_prefix.is_empty() {
                        file_name.clone()
                    } else {
                        format!("{}{}", rel_prefix, file_name)
                    };
                    collected_files.push(normalize_path(&rel_path));
                    children.push(StructureNode {
                        name: file_name,
                        path: normalize_path(&rel_path),
                        node_type: "file".to_string(),
                        children: Vec::new(),
                    });
                }
            }
        }
    }

    let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("root");
    StructureNode {
        name: dir_name.to_string(),
        path: if rel_prefix.is_empty() { "/".to_string() } else { normalize_path(rel_prefix) },
        node_type: "dir".to_string(),
        children,
    }
}

fn extract_relationships(root: &Path, files: &[String]) -> Vec<StructureRelationship> {
    let mut relationships = Vec::new();
    let file_set: HashSet<String> = files.iter().cloned().collect();

    let re_rust_mod = Regex::new(r"(?:pub\s+)?mod\s+([a-zA-Z0-9_]+);").ok();
    let re_py_import = Regex::new(r"(?:from\s+([a-zA-Z0-9_.]+)\s+import|import\s+([a-zA-Z0-9_.]+))").ok();

    for f in files {
        let p = root.join(f);
        if !p.exists() || !p.is_file() {
            continue;
        }
        let content = match fs::read_to_string(&p) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if f.ends_with(".rs") {
            if let Some(ref re) = re_rust_mod {
                for cap in re.captures_iter(&content) {
                    if let Some(m) = cap.get(1) {
                        let mod_name = m.as_str();
                        let parent = Path::new(f).parent().unwrap_or(Path::new(""));
                        let candidate1 = normalize_path(&parent.join(format!("{}.rs", mod_name)).to_string_lossy());
                        let candidate2 = normalize_path(&parent.join(mod_name).join("mod.rs").to_string_lossy());

                        if file_set.contains(&candidate1) {
                            relationships.push(StructureRelationship {
                                source: f.clone(),
                                target: candidate1,
                                kind: "mod".to_string(),
                            });
                        } else if file_set.contains(&candidate2) {
                            relationships.push(StructureRelationship {
                                source: f.clone(),
                                target: candidate2,
                                kind: "mod".to_string(),
                            });
                        }
                    }
                }
            }
        } else if f.ends_with(".py") {
            if let Some(ref re) = re_py_import {
                for cap in re.captures_iter(&content) {
                    let mod_path = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str()).unwrap_or("");
                    let rel_candidate = normalize_path(&format!("{}.py", mod_path.replace('.', "/")));
                    if file_set.contains(&rel_candidate) && rel_candidate != *f {
                        relationships.push(StructureRelationship {
                            source: f.clone(),
                            target: rel_candidate,
                            kind: "imports".to_string(),
                        });
                    }
                }
            }
        }
    }

    relationships
}

pub fn build_structure(root: Option<&Path>) -> Result<StructureResult> {
    let repo_root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| find_repo_root(root));
    let mut files = Vec::new();
    let mut dir_count = 0;

    let tree = walk_dir(&repo_root, "", &mut files, &mut dir_count);
    let relationships = extract_relationships(&repo_root, &files);

    let proj_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    Ok(StructureResult {
        generated: Utc::now().to_rfc3339(),
        root: proj_name.to_string(),
        stats: StructureStats {
            files: files.len(),
            dirs: dir_count,
            relationships: relationships.len(),
        },
        tree,
        files,
        relationships,
    })
}

pub fn generate_and_save_structure(root: Option<&Path>) -> Result<StructureResult> {
    let structure = build_structure(root)?;
    let mem_dir = require_mem_dir(root)?;
    let path = mem_dir.join(STRUCTURE_FILE);
    let json = serde_json::to_string_pretty(&structure)?;
    atomic_write_file(&path, &json)?;
    Ok(structure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_structure() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub mod foo;\n").unwrap();
        fs::write(root.join("src/foo.rs"), "pub fn bar() {}\n").unwrap();

        let structure = build_structure(Some(root)).unwrap();
        assert_eq!(structure.stats.files, 2);
        assert_eq!(structure.relationships.len(), 1);
        assert_eq!(structure.relationships[0].kind, "mod");
    }
}
