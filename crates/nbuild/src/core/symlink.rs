// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/symlink.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkStatus {
    Created,
    AlreadyValid,
    Updated,
    Conflict(String),
}

/// Robust cross-platform link engine supporting Windows Junctions, File Symlinks, and Hardlinks
pub struct LinkEngine;

impl LinkEngine {
    /// Link a directory from `src` to `dest`.
    /// On Windows: Uses directory junction or symlink.
    /// On Unix: Uses symlink.
    pub fn link_dir<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<LinkStatus> {
        let src = src.as_ref();
        let dest = dest.as_ref();

        if !src.exists() {
            anyhow::bail!("Source directory does not exist: {}", src.display());
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if dest.exists() || dest.is_symlink() {
            if let Ok(target) = fs::read_link(dest) {
                if target == src || target == fs::canonicalize(src).unwrap_or_else(|_| src.to_path_buf()) {
                    return Ok(LinkStatus::AlreadyValid);
                }
            }
            // Remove old link/dir if stale
            let _ = fs::remove_dir(dest).or_else(|_| fs::remove_file(dest));
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            if let Err(e) = symlink_dir(src, dest) {
                // If symlink failed (privilege not held), create junction using mklink /J
                let output = std::process::Command::new("cmd")
                    .args(["/C", "mklink", "/J", dest.to_str().unwrap(), src.to_str().unwrap()])
                    .output();
                match output {
                    Ok(out) if out.status.success() => Ok(LinkStatus::Created),
                    _ => anyhow::bail!("Failed to create directory junction on Windows: {e}"),
                }
            } else {
                Ok(LinkStatus::Created)
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(src, dest).with_context(|| format!("Failed to symlink dir {} to {}", src.display(), dest.display()))?;
            Ok(LinkStatus::Created)
        }
    }

    /// Link a file from `src` to `dest`.
    /// On Windows: Tries symlink_file -> falls back to hard_link -> falls back to copy.
    /// On Unix: Uses symlink.
    pub fn link_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q, conflict_guard: bool) -> Result<LinkStatus> {
        let src = src.as_ref();
        let dest = dest.as_ref();

        if !src.exists() {
            anyhow::bail!("Source file does not exist: {}", src.display());
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if dest.exists() || dest.is_symlink() {
            if let Ok(target) = fs::read_link(dest) {
                if target == src || target == fs::canonicalize(src).unwrap_or_else(|_| src.to_path_buf()) {
                    return Ok(LinkStatus::AlreadyValid);
                }
            }

            if conflict_guard {
                // Check if existing file is different
                if let (Ok(meta1), Ok(meta2)) = (fs::metadata(src), fs::metadata(dest)) {
                    if meta1.len() != meta2.len() {
                        return Ok(LinkStatus::Conflict(format!(
                            "Collision at destination: {} already exists with different size",
                            dest.display()
                        )));
                    }
                }
            }

            // Remove existing destination link/file
            let _ = fs::remove_file(dest);
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            // 1. Try file symlink (requires Dev Mode or Admin)
            if symlink_file(src, dest).is_ok() {
                return Ok(LinkStatus::Created);
            }

            // 2. Fallback to NTFS Hardlink
            if fs::hard_link(src, dest).is_ok() {
                return Ok(LinkStatus::Created);
            }

            // 3. Fallback to copy if cross-volume or permission denied
            fs::copy(src, dest)?;
            Ok(LinkStatus::Created)
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(src, dest).with_context(|| format!("Failed to symlink {} to {}", src.display(), dest.display()))?;
            Ok(LinkStatus::Created)
        }
    }

    /// Recursively links all files from `src_dir` into `dest_dir` with relative preservation
    pub fn link_tree<P: AsRef<Path>, Q: AsRef<Path>>(src_dir: P, dest_dir: Q, conflict_guard: bool) -> Result<Vec<(PathBuf, LinkStatus)>> {
        let src_dir = src_dir.as_ref();
        let dest_dir = dest_dir.as_ref();
        let mut results = Vec::new();

        if !src_dir.exists() {
            return Ok(results);
        }

        let walker = walkdir(src_dir)?;
        for entry in walker {
            let rel = entry.strip_prefix(src_dir)?;
            let target = dest_dir.join(rel);

            if entry.is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                let status = Self::link_file(&entry, &target, conflict_guard)?;
                results.push((target, status));
            }
        }

        Ok(results)
    }
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.push(path.clone());
                files.extend(walkdir(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}
