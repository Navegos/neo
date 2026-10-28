// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ignore::WalkBuilder;

pub const MAX_SOURCE_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2 MB

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkedFile {
    pub path: PathBuf,
    pub rel_path: String,
    pub language: &'static str,
    pub mtime: u64,
    pub size: u64,
}

pub fn detect_language(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "rs" => Some("rust"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("jsx"),
        "py" | "pyi" => Some("python"),
        "go" => Some("go"),
        "c" | "h" => Some("c"),
        "cpp" | "hpp" | "cc" | "cxx" | "hh" | "hxx" => Some("cpp"),
        _ => None,
    }
}

pub fn walk_project_files(root: &Path) -> Vec<WalkedFile> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(true)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if name == ".git"
                    || name == "node_modules"
                    || name == "target"
                    || name == ".codegraph"
                    || name == ".projectmem"
                    || name == ".tokless"
                    || name == "dist"
                    || name == "build"
                    || name == "vendor"
                    || name == "__pycache__"
                    || name == ".venv"
                    || name == "venv"
                {
                    return false;
                }
            }
            true
        })
        .build();

    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let p = entry.path();
        let lang = match detect_language(p) {
            Some(l) => l,
            None => continue,
        };

        let metadata = match fs::metadata(p) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size = metadata.len();
        if size == 0 || size > MAX_SOURCE_FILE_SIZE {
            continue;
        }

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let rel_path = p
            .strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");

        if crate::codegraph::kernel::textutil::is_generated_file(&rel_path) {
            continue;
        }

        files.push(WalkedFile {
            path: p.to_path_buf(),
            rel_path,
            language: lang,
            mtime,
            size,
        });
    }

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    files
}

