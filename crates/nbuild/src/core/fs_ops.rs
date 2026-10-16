// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/fs_ops.rs
// created: 2026-10-16

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsFilter {
    All,
    File,
    Dir,
}

pub struct FsOps;

impl FsOps {
    /// Recursively search for files and directories matching a pattern/substring.
    pub fn search<P: AsRef<Path>>(root: P, pattern: &str, filter: FsFilter) -> Result<Vec<PathBuf>> {
        let root = root.as_ref();
        if !root.exists() {
            bail!("Search root directory does not exist: {}", root.display());
        }

        let mut matches = Vec::new();
        let pattern_lower = pattern.to_lowercase();
        let has_wildcard = pattern.contains('*') || pattern.contains('?');

        Self::walk_search(root, &pattern_lower, has_wildcard, filter, &mut matches)?;
        Ok(matches)
    }

    fn walk_search(
        dir: &Path,
        pattern_lower: &str,
        has_wildcard: bool,
        filter: FsFilter,
        matches: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // Skip inaccessible dirs gracefully
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();

            let matched = if has_wildcard {
                Self::match_wildcard(&file_name, pattern_lower)
            } else {
                file_name.contains(pattern_lower)
            };

            if matched {
                let include = match filter {
                    FsFilter::All => true,
                    FsFilter::File => !is_dir,
                    FsFilter::Dir => is_dir,
                };
                if include {
                    matches.push(path.clone());
                }
            }

            if is_dir {
                let _ = Self::walk_search(&path, pattern_lower, has_wildcard, filter, matches);
            }
        }

        Ok(())
    }

    fn match_wildcard(text: &str, pattern: &str) -> bool {
        let t_chars: Vec<char> = text.chars().collect();
        let p_chars: Vec<char> = pattern.chars().collect();
        let mut t_idx = 0;
        let mut p_idx = 0;
        let mut star_idx = None;
        let mut match_idx = 0;

        while t_idx < t_chars.len() {
            if p_idx < p_chars.len() && (p_chars[p_idx] == '?' || p_chars[p_idx] == t_chars[t_idx]) {
                t_idx += 1;
                p_idx += 1;
            } else if p_idx < p_chars.len() && p_chars[p_idx] == '*' {
                star_idx = Some(p_idx);
                match_idx = t_idx;
                p_idx += 1;
            } else if let Some(si) = star_idx {
                p_idx = si + 1;
                match_idx += 1;
                t_idx = match_idx;
            } else {
                return false;
            }
        }

        while p_idx < p_chars.len() && p_chars[p_idx] == '*' {
            p_idx += 1;
        }

        p_idx == p_chars.len()
    }

    /// Create directory tree recursively.
    pub fn mkdir<P: AsRef<Path>>(dir: P) -> Result<()> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))
    }

    /// Touch or create an empty file, creating parent directories if needed.
    pub fn touch<P: AsRef<Path>>(file: P) -> Result<()> {
        let file = file.as_ref();
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        if !file.exists() {
            fs::write(file, b"")
                .with_context(|| format!("Failed to create file: {}", file.display()))?;
        }
        Ok(())
    }

    /// Copy a file or directory recursively.
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q, recursive: bool) -> Result<()> {
        let src = src.as_ref();
        let dest = dest.as_ref();

        if !src.exists() {
            bail!("Source path does not exist: {}", src.display());
        }

        if src.is_dir() {
            if !recursive {
                bail!("Source is a directory but recursive copy flag was not specified");
            }
            fs::create_dir_all(dest)?;
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let sub_src = entry.path();
                let sub_dest = dest.join(file_name);
                Self::copy(&sub_src, &sub_dest, true)?;
            }
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src, dest)
                .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;
        }

        Ok(())
    }

    /// Move file or directory from `src` to `dest`.
    pub fn move_path<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<()> {
        let src = src.as_ref();
        let dest = dest.as_ref();

        if !src.exists() {
            bail!("Source path does not exist: {}", src.display());
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Try direct atomic rename first
        if fs::rename(src, dest).is_err() {
            // Fallback: cross-filesystem copy and delete
            Self::copy(src, dest, true)?;
            Self::delete(src, true, true)?;
        }

        Ok(())
    }

    /// Rename file or directory within its directory.
    pub fn rename<P: AsRef<Path>>(src: P, new_name: &str) -> Result<PathBuf> {
        let src = src.as_ref();
        if !src.exists() {
            bail!("Source path does not exist: {}", src.display());
        }

        let parent = src.parent().unwrap_or_else(|| Path::new("."));
        let dest = parent.join(new_name);
        Self::move_path(src, &dest)?;
        Ok(dest)
    }

    /// Delete file or directory. Handles clearing read-only attributes on Windows.
    pub fn delete<P: AsRef<Path>>(path: P, recursive: bool, _force: bool) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() && !path.is_symlink() {
            return Ok(());
        }

        // Clear read-only attribute if file/dir has it
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                let _ = fs::set_permissions(path, perms);
            }
        }

        if path.is_dir() && !path.is_symlink() {
            if !recursive {
                bail!("Path is a directory; use recursive flag to delete: {}", path.display());
            }
            // Clear inner read-only attributes
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let _ = Self::delete(entry.path(), true, true);
            }
            fs::remove_dir(path)
                .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
        } else {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove file: {}", path.display()))?;
        }

        Ok(())
    }
}

