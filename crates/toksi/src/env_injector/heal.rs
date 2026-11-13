// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/env_injector/heal.rs
// created: 2026-10-12
// lastModified: 2026-11-13

//! Phase 09 & 11: Legacy Healing & Binary Un-Hijack Engine.
//!
//! Scans system PATH and known agent binary locations for legacy hijacked binaries:
//! - `agy.real.exe`, `agy.real`, `agy.cmd.real`, `antigravity.real*`
//! - `grok.real.exe`, `grok.real`, `grok.cmd.real`
//! - `copilot.real.exe`, `copilot.real`, `copilot.exe.toksi`, `copilot.digest`
//! - `kilo.real.exe`, `kilo.real`
//! - Hijacked symlinks and Windows junction points pointing to shims or missing `.real` binaries
//! - Legacy `.cmd`, `.ps1`, `.sh` wrapper shims containing `toksi:` / `tokless:` markers
//! - Legacy `toksi-codegraph-init.js` / `tokless-codegraph-init.js` auto-index shims
//! - Orphaned proxy state files (`antigravity-cli-proxy.json`, `grok-cli-proxy.json`, `copilot-cli-proxy.json`)
//!
//! Atomically restores `.real` binaries back to their canonical filenames and purges shims.

#![deny(unsafe_code)]

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::util::logger::L;
use crate::util::paths::{home, normalize_path, toksi_data_dir};

const IS_WIN: bool = cfg!(windows);

/// Domain errors for legacy un-hijack and healing operations.
#[derive(Debug, thiserror::Error)]
pub enum HealError {
    #[error("I/O error at '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to restore '{real}' to '{target}': {reason}")]
    RestoreFailed {
        real: String,
        target: String,
        reason: String,
    },
}

fn norm_path_buf(path: &Path) -> PathBuf {
    PathBuf::from(normalize_path(&path.to_string_lossy()))
}

fn norm_str(path: &Path) -> String {
    normalize_path(&path.to_string_lossy())
}

/// Complete report of all legacy hijacks healed during a pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HealingReport {
    /// List of `(stashed_real_path, restored_canonical_path)`
    pub restored_binaries: Vec<(PathBuf, PathBuf)>,
    /// List of deleted shim wrappers or hijacked symlinks/junctions
    pub removed_shims: Vec<PathBuf>,
    /// List of deleted orphaned state files
    pub removed_state_files: Vec<PathBuf>,
    /// Errors encountered during the healing process
    pub errors: Vec<String>,
}

impl HealingReport {
    pub fn is_clean(&self) -> bool {
        self.restored_binaries.is_empty()
            && self.removed_shims.is_empty()
            && self.removed_state_files.is_empty()
            && self.errors.is_empty()
    }

    pub fn total_healed(&self) -> usize {
        self.restored_binaries.len() + self.removed_shims.len() + self.removed_state_files.len()
    }
}

/// Known shim marker substrings injected by legacy Toksi / Tokless versions.
const LEGACY_SHIM_MARKERS: &[&str] = &[
    "toksi:antigravity-cli-proxy",
    "toksi:antigravity-launcher",
    "toksi:grok-cli-proxy",
    "toksi:copilot-shim",
    "toksi:kilo-shim",
    "tokless:antigravity-cli-proxy",
    "tokless:antigravity-launcher",
    "tokless:grok-cli-proxy",
    "tokless:copilot-shim",
    "tokless:kilo-shim",
    "TOKSI_ANTIGRAVITY_HEADROOM_MANAGED",
    "TOKLESS_ANTIGRAVITY_HEADROOM_MANAGED",
    "__grok-proxy-ensure",
    "__ grok-proxy-owned",
    "__grok-proxy-owned",
    "__copilot-cli-proxy",
    "__copilot-proxy",
    "__proxy-ensure",
    "agy.real",
    "grok.real",
    "copilot.real",
];

/// Orphaned JSON state files created by legacy shim installers in the data dir.
const LEGACY_STATE_FILES: &[&str] = &[
    "antigravity-cli-proxy.json",
    "grok-cli-proxy.json",
    "copilot-cli-proxy.json",
    "kilo-cli-proxy.json",
    "antigravity-shim-state.json",
    "grok-shim-state.json",
    "copilot-shim-state.json",
    "grok-mitm-ca.pem",
    "toksi-codegraph-init.js",
    "tokless-codegraph-init.js",
];

/// Legacy standalone script/plugin filenames that should always be treated as shims.
const LEGACY_SCRIPT_FILENAMES: &[&str] = &[
    "toksi-codegraph-init.js",
    "tokless-codegraph-init.js",
    "toksi-index.ts",
    "tokless-index.ts",
    "toksi-rtk-rewrite.sh",
    "tokless-rtk-rewrite.sh",
];

/// Known agent binary stems that may have hijacked symlinks or junctions.
const KNOWN_AGENT_STEMS: &[&str] = &[
    "agy",
    "antigravity",
    "grok",
    "copilot",
    "kilo",
    "cursor",
    "claude",
    "codex",
    "opencode",
    "droid",
];

/// Executes a full healing pass across all candidate binary directories and data directories.
pub fn heal_legacy_hijacks() -> HealingReport {
    let bin_dirs = candidate_bin_dirs();
    let data_dirs = candidate_data_dirs();
    heal_legacy_hijacks_in(&bin_dirs, &data_dirs)
}

/// Scans (dry-run) all candidate binary and data directories for legacy hijacks without modifying disk.
pub fn scan_legacy_hijacks() -> HealingReport {
    let bin_dirs = candidate_bin_dirs();
    let data_dirs = candidate_data_dirs();
    scan_legacy_hijacks_in(&bin_dirs, &data_dirs)
}

/// Scans specified binary and data directories without modifying disk (dry-run).
pub fn scan_legacy_hijacks_in(bin_dirs: &[PathBuf], data_dirs: &[PathBuf]) -> HealingReport {
    let mut report = HealingReport::default();
    let mut visited = HashSet::new();

    for dir in bin_dirs {
        let key = norm_str(dir);
        if key.is_empty() || !visited.insert(key) {
            continue;
        }
        if !dir.is_dir() {
            continue;
        }
        scan_and_heal_bin_dir(dir, true, &mut report);
    }

    for data_dir in data_dirs {
        if !data_dir.is_dir() {
            continue;
        }
        scan_and_heal_data_dir(data_dir, true, &mut report);
    }

    report
}

/// Executes a healing pass across the specified binary directories and data directories.
pub fn heal_legacy_hijacks_in(bin_dirs: &[PathBuf], data_dirs: &[PathBuf]) -> HealingReport {
    let mut report = HealingReport::default();
    let mut visited = HashSet::new();

    // 1. Scan and heal binary directories
    for dir in bin_dirs {
        let key = norm_str(dir);
        if key.is_empty() || !visited.insert(key) {
            continue;
        }
        if !dir.is_dir() {
            continue;
        }
        scan_and_heal_bin_dir(dir, false, &mut report);
    }

    // 2. Scan and remove orphaned proxy state files in data directories
    for data_dir in data_dirs {
        if !data_dir.is_dir() {
            continue;
        }
        scan_and_heal_data_dir(data_dir, false, &mut report);
    }

    report
}

/// Collects all directories on `PATH` plus known agent installation directories.
pub fn candidate_bin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(path_os) = env::var_os("PATH") {
        for p in env::split_paths(&path_os) {
            if !p.as_os_str().is_empty() {
                dirs.push(norm_path_buf(&p));
            }
        }
    }

    let h = home();
    if !h.is_empty() {
        let hp = Path::new(&h);
        dirs.push(norm_path_buf(&hp.join(".local").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".cargo").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".bun").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".toksi").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".tokless").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".antigravity").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".grok").join("bin")));
        dirs.push(norm_path_buf(&hp.join(".copilot").join("bin")));
        dirs.push(norm_path_buf(
            &hp.join(".config").join("opencode").join("plugins"),
        ));
    }

    if IS_WIN {
        if let Ok(local) = env::var("LOCALAPPDATA")
            && !local.is_empty()
        {
            let lp = Path::new(&local);
            dirs.push(norm_path_buf(&lp.join("agy").join("bin")));
            dirs.push(norm_path_buf(
                &lp.join("Programs")
                    .join("Antigravity")
                    .join("resources")
                    .join("app")
                    .join("bin"),
            ));
            dirs.push(norm_path_buf(
                &lp.join("Programs")
                    .join("Antigravity IDE")
                    .join("resources")
                    .join("app")
                    .join("bin"),
            ));
            dirs.push(norm_path_buf(&lp.join("Programs").join("toksi")));
            dirs.push(norm_path_buf(&lp.join("Programs").join("tokless")));
            dirs.push(norm_path_buf(&lp.join("Microsoft").join("WinGet").join("Links")));
        }
        if let Ok(appdata) = env::var("APPDATA")
            && !appdata.is_empty()
        {
            let ap = Path::new(&appdata);
            dirs.push(norm_path_buf(&ap.join("npm")));
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
    }

    dirs
}

/// Collects candidate Toksi and legacy Tokless data directories where shim state JSONs live.
pub fn candidate_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let td = toksi_data_dir();
    if !td.is_empty() {
        let p = norm_path_buf(Path::new(&td));
        if let Some(parent) = p.parent() {
            dirs.push(norm_path_buf(&parent.join("tokless")));
        }
        dirs.push(p);
    }
    let h = home();
    if !h.is_empty() {
        let hp = Path::new(&h);
        dirs.push(norm_path_buf(&hp.join(".toksi")));
        dirs.push(norm_path_buf(&hp.join(".tokless")));
        dirs.push(norm_path_buf(
            &hp.join(".local").join("share").join("toksi"),
        ));
        dirs.push(norm_path_buf(
            &hp.join(".local").join("share").join("tokless"),
        ));
        dirs.push(norm_path_buf(
            &hp.join(".config").join("opencode").join("plugins"),
        ));
    }
    if IS_WIN
        && let Ok(local) = env::var("LOCALAPPDATA")
        && !local.is_empty()
    {
        let lp = Path::new(&local);
        dirs.push(norm_path_buf(&lp.join("toksi")));
        dirs.push(norm_path_buf(&lp.join("tokless")));
    }
    dirs
}

fn scan_and_heal_data_dir(data_dir: &Path, dry_run: bool, report: &mut HealingReport) {
    // 1. Check known state files directly
    for state_name in LEGACY_STATE_FILES {
        let state_path = data_dir.join(state_name);
        if state_path.is_file() {
            // Also inspect state file for recorded `bin` and `real` paths in case they are outside standard PATH
            if let Ok(raw) = fs::read_to_string(&state_path) {
                heal_from_state_json(&raw, dry_run, report);
            }
            if dry_run {
                report.removed_state_files.push(norm_path_buf(&state_path));
            } else {
                match fs::remove_file(&state_path) {
                    Ok(()) => {
                        L.debug(&format!(
                            "Removed orphaned legacy proxy state file: {}",
                            norm_str(&state_path)
                        ));
                        report.removed_state_files.push(norm_path_buf(&state_path));
                    }
                    Err(e) => report.errors.push(format!(
                        "Failed to remove state file {}: {e}",
                        norm_str(&state_path)
                    )),
                }
            }
        }
    }
}

fn heal_from_state_json(raw: &str, dry_run: bool, report: &mut HealingReport) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) else {
        return;
    };
    let Some(obj) = val.as_object() else {
        return;
    };
    let bin_str = obj
        .get("bin")
        .or_else(|| obj.get("Bin"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let real_str = obj
        .get("real")
        .or_else(|| obj.get("Real"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !real_str.is_empty() {
        let real_path = norm_path_buf(Path::new(real_str));
        if real_path.is_file() {
            let target_path = if !bin_str.is_empty() {
                derive_canonical_from_shim_and_real(Path::new(bin_str), &real_path)
            } else {
                derive_canonical_path(&real_path).unwrap_or_else(|| real_path.clone())
            };
            if target_path != real_path {
                if dry_run {
                    report
                        .restored_binaries
                        .push((real_path, norm_path_buf(&target_path)));
                } else if let Err(e) = restore_real_binary(&real_path, &target_path) {
                    report.errors.push(e.to_string());
                } else {
                    report
                        .restored_binaries
                        .push((real_path, norm_path_buf(&target_path)));
                }
            }
        }
    }

    if !bin_str.is_empty() {
        let bin_path = norm_path_buf(Path::new(bin_str));
        if is_toksi_shim_or_sidecar(&bin_path) || is_hijacked_symlink_or_junction(&bin_path) {
            if dry_run {
                report.removed_shims.push(bin_path);
            } else if remove_shim_or_link(&bin_path).is_ok() {
                report.removed_shims.push(bin_path);
            }
        }
    }
}

fn derive_canonical_from_shim_and_real(shim: &Path, real: &Path) -> PathBuf {
    if let Some(derived) = derive_canonical_path(real) {
        return norm_path_buf(&derived);
    }
    norm_path_buf(shim)
}

fn scan_and_heal_bin_dir(dir: &Path, dry_run: bool, report: &mut HealingReport) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut real_files = Vec::new();
    let mut candidate_shims = Vec::new();
    let mut candidate_links = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if is_hijacked_symlink_or_junction(&path) {
            candidate_links.push(path);
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if is_real_backup_filename(name) {
            real_files.push(path);
        } else if is_candidate_shim_or_sidecar_filename(name) {
            candidate_shims.push(path);
        }
    }

    // Pass 0: Remove hijacked symlinks or Windows junction points first so canonical paths are free
    for link_path in candidate_links {
        let norm_link = norm_path_buf(&link_path);
        if dry_run {
            report.removed_shims.push(norm_link);
        } else {
            match remove_shim_or_link(&link_path) {
                Ok(()) => {
                    L.debug(&format!(
                        "Removed hijacked symlink or junction point: {}",
                        norm_str(&link_path)
                    ));
                    report.removed_shims.push(norm_link);
                }
                Err(e) => report.errors.push(e.to_string()),
            }
        }
    }

    // Pass 1: Restore all `.real` binaries to their canonical filenames
    for real_path in real_files {
        let Some(canonical_path) = derive_canonical_path(&real_path) else {
            continue;
        };
        let norm_real = norm_path_buf(&real_path);
        let norm_canonical = norm_path_buf(&canonical_path);
        if dry_run {
            report.restored_binaries.push((norm_real, norm_canonical));
            continue;
        }
        // If canonical_path currently exists and is a shim or hijacked link, remove it first
        if (canonical_path.exists() || fs::symlink_metadata(&canonical_path).is_ok())
            && (is_toksi_shim_or_sidecar(&canonical_path)
                || is_hijacked_symlink_or_junction(&canonical_path))
            && remove_shim_or_link(&canonical_path).is_ok()
        {
            report.removed_shims.push(norm_canonical.clone());
        }

        match restore_real_binary(&real_path, &canonical_path) {
            Ok(()) => {
                report.restored_binaries.push((norm_real, norm_canonical));
            }
            Err(e) => {
                report.errors.push(e.to_string());
            }
        }
    }

    // Pass 2: Clean up any remaining shims, `.toksi` markers, and `.digest` sidecars
    for shim_path in candidate_shims {
        if !shim_path.exists() {
            // Already removed or overwritten during Pass 1
            continue;
        }
        if is_toksi_shim_or_sidecar(&shim_path) {
            let norm_shim = norm_path_buf(&shim_path);
            if dry_run {
                report.removed_shims.push(norm_shim);
            } else {
                match fs::remove_file(&shim_path) {
                    Ok(()) => {
                        L.debug(&format!(
                            "Removed legacy hijack shim or sidecar: {}",
                            norm_str(&shim_path)
                        ));
                        report.removed_shims.push(norm_shim);
                    }
                    Err(e) => report.errors.push(format!(
                        "Failed to remove shim {}: {e}",
                        norm_str(&shim_path)
                    )),
                }
            }
        }
    }
}

/// Checks whether `name` is a `.real` backup created by legacy binary hijacking.
///
/// Supported patterns:
/// - `<stem>.real.exe` (e.g., `agy.real.exe`, `grok.real.exe`, `copilot.real.exe`)
/// - `<stem>.cmd.real`, `<stem>.ps1.real`, `<stem>.bat.real`, `<stem>.sh.real`
/// - `<stem>.real` (e.g., `agy.real`, `grok.real`, `copilot.real`, `kilo.real`)
pub fn is_real_backup_filename(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.ends_with(".real.exe") && lower.len() > ".real.exe".len() {
        return true;
    }
    if lower.ends_with(".real") && lower.len() > ".real".len() {
        return true;
    }
    false
}

/// Derives the canonical binary path from a `.real` backup path.
///
/// Examples:
/// - `/path/agy.real.exe` -> `/path/agy.exe`
/// - `/path/agy.cmd.real` -> `/path/agy.cmd`
/// - `/path/agy.real` -> `/path/agy.exe` (on Windows) or `/path/agy` (on Unix)
pub fn derive_canonical_path(real_path: &Path) -> Option<PathBuf> {
    let parent = real_path.parent()?;
    let name = real_path.file_name()?.to_str()?;
    let lower = name.to_lowercase();

    if lower.ends_with(".real.exe") {
        let stem = &name[..name.len() - ".real.exe".len()];
        if stem.is_empty() {
            return None;
        }
        return Some(norm_path_buf(&parent.join(format!("{stem}.exe"))));
    }

    if lower.ends_with(".real") {
        let base = &name[..name.len() - ".real".len()];
        if base.is_empty() {
            return None;
        }
        // If base already has an extension like `.cmd`, `.bat`, `.ps1`, `.sh`, `.exe`, keep it
        let base_lower = base.to_lowercase();
        if base_lower.ends_with(".cmd")
            || base_lower.ends_with(".bat")
            || base_lower.ends_with(".ps1")
            || base_lower.ends_with(".sh")
            || base_lower.ends_with(".exe")
        {
            return Some(norm_path_buf(&parent.join(base)));
        }
        // Otherwise on Windows restore `<base>.exe` unless `<base>` without extension is a script
        if IS_WIN {
            return Some(norm_path_buf(&parent.join(format!("{base}.exe"))));
        }
        return Some(norm_path_buf(&parent.join(base)));
    }

    None
}

/// Atomically restores `real_path` to `canonical_path`, preserving executable permissions.
fn restore_real_binary(real_path: &Path, canonical_path: &Path) -> Result<(), HealError> {
    if !real_path.exists() {
        return Ok(());
    }

    // If canonical_path exists and is identical in size & content to real_path, just remove real_path
    if canonical_path.is_file()
        && !is_toksi_shim_or_sidecar(canonical_path)
        && files_identical(real_path, canonical_path)
    {
        fs::remove_file(real_path).map_err(|source| HealError::Io {
            path: norm_str(real_path),
            source,
        })?;
        return Ok(());
    }

    // Remove existing shim or broken target at canonical_path before rename
    if canonical_path.exists() || fs::symlink_metadata(canonical_path).is_ok() {
        remove_if_shim_or_broken_at_target(canonical_path)?;
    }

    ensure_executable_permissions(real_path);

    // Attempt atomic rename first
    match fs::rename(real_path, canonical_path) {
        Ok(()) => {
            ensure_executable_permissions(canonical_path);
            L.debug(&format!(
                "Restored hijacked binary from .real backup: {} -> {}",
                norm_str(real_path),
                norm_str(canonical_path)
            ));
            Ok(())
        }
        Err(_) => {
            // Fallback: copy via temporary file in target directory, then atomic rename
            let tmp_path = canonical_path.with_extension("toksi-restore-tmp");
            fs::copy(real_path, &tmp_path).map_err(|e| HealError::RestoreFailed {
                real: norm_str(real_path),
                target: norm_str(canonical_path),
                reason: format!("copy to temp failed: {e}"),
            })?;
            ensure_executable_permissions(&tmp_path);
            if let Err(e) = fs::rename(&tmp_path, canonical_path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(HealError::RestoreFailed {
                    real: norm_str(real_path),
                    target: norm_str(canonical_path),
                    reason: format!("atomic rename from temp failed: {e}"),
                });
            }
            let _ = fs::remove_file(real_path);
            L.debug(&format!(
                "Restored hijacked binary from .real backup via atomic copy: {} -> {}",
                norm_str(real_path),
                norm_str(canonical_path)
            ));
            Ok(())
        }
    }
}

fn ensure_executable_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            let mode = perms.mode();
            if mode & 0o111 == 0 {
                perms.set_mode(mode | 0o755);
                let _ = fs::set_permissions(path, perms);
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

fn remove_if_shim_or_broken_at_target(target: &Path) -> Result<(), HealError> {
    if is_symlink_or_reparse_point(target) {
        return remove_shim_or_link(target);
    }
    if is_toksi_shim_or_sidecar(target) {
        return fs::remove_file(target).map_err(|source| HealError::Io {
            path: norm_str(target),
            source,
        });
    }
    // When a `.real` file explicitly exists alongside `target`, check if `target` is a small script or 0-byte file
    if let Ok(meta) = fs::metadata(target) {
        if meta.len() == 0 {
            return fs::remove_file(target).map_err(|source| HealError::Io {
                path: norm_str(target),
                source,
            });
        }
        // Legacy Go shim or batch/shell wrapper is typically < 64KB text
        if meta.len() < 65536
            && let Ok(text) = fs::read_to_string(target)
        {
            let lower = text.to_lowercase();
            if lower.contains("toksi")
                || lower.contains("tokless")
                || lower.starts_with("#!")
                || lower.starts_with("@echo")
            {
                return fs::remove_file(target).map_err(|source| HealError::Io {
                    path: norm_str(target),
                    source,
                });
            }
        }
    }
    Ok(())
}

fn files_identical(a: &Path, b: &Path) -> bool {
    let (Ok(ma), Ok(mb)) = (fs::metadata(a), fs::metadata(b)) else {
        return false;
    };
    if ma.len() != mb.len() {
        return false;
    }
    let (Ok(ba), Ok(bb)) = (fs::read(a), fs::read(b)) else {
        return false;
    };
    ba == bb
}

fn is_candidate_shim_or_sidecar_filename(name: &str) -> bool {
    let lower = name.to_lowercase();
    if LEGACY_SCRIPT_FILENAMES.contains(&lower.as_str()) {
        return true;
    }
    if lower.ends_with(".toksi")
        || lower.ends_with(".tokless")
        || lower.ends_with(".digest")
    {
        return true;
    }
    // Check known hijacked binary names and scripts
    let prefixes = [
        "agy",
        "antigravity",
        "grok",
        "copilot",
        "kilo",
    ];
    prefixes.iter().any(|p| {
        lower == *p
            || lower == format!("{p}.cmd")
            || lower == format!("{p}.bat")
            || lower == format!("{p}.ps1")
            || lower == format!("{p}.sh")
            || lower == format!("{p}.exe")
    })
}

/// Checks if `path` is a symlink or a Windows junction/reparse point.
pub fn is_symlink_or_reparse_point(path: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    if meta.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        if (meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
            return true;
        }
    }
    false
}

/// Returns `true` if `path` is a hijacked symlink or Windows junction point created by legacy shims.
///
/// Never flags valid user/system symlinks unless they point to a Toksi/Tokless shim, `.real` backup,
/// or are broken links shadowing a sibling `.real` binary.
pub fn is_hijacked_symlink_or_junction(path: &Path) -> bool {
    if !is_symlink_or_reparse_point(path) {
        return false;
    }

    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let lower_name = name.to_lowercase();

    if lower_name.ends_with(".toksi")
        || lower_name.ends_with(".tokless")
        || lower_name.ends_with(".digest")
        || lower_name.ends_with(".real")
        || lower_name.ends_with(".real.exe")
    {
        return true;
    }

    let stem = Path::new(&lower_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&lower_name);

    // If a sibling `.real` or `.real.exe` backup exists in the same directory for this binary stem,
    // then this symlink/junction is hijacking the canonical binary name.
    if let Some(parent) = path.parent() {
        let sibling_real = parent.join(format!("{stem}.real"));
        let sibling_real_exe = parent.join(format!("{stem}.real.exe"));
        let sibling_exact_real = parent.join(format!("{name}.real"));
        if sibling_real.is_file() || sibling_real_exe.is_file() || sibling_exact_real.is_file() {
            return true;
        }
    }

    if let Ok(target) = fs::read_link(path) {
        let target_norm = norm_str(&target).to_lowercase();
        if target_norm.contains("tokless")
            || target_norm.contains(".real")
            || target_norm.contains("-cli-proxy")
            || target_norm.contains("toksi-shim")
            || target_norm.contains("tokless-shim")
        {
            return true;
        }
        let resolved = if target.is_relative() {
            path.parent().map(|p| p.join(&target)).unwrap_or(target)
        } else {
            target
        };
        if resolved.is_file() && is_toksi_shim_or_sidecar(&resolved) {
            return true;
        }
        if !resolved.exists()
            && KNOWN_AGENT_STEMS.contains(&stem)
            && (target_norm.contains("toksi") || target_norm.contains("shim") || target_norm.contains("proxy"))
        {
            return true;
        }
    }

    false
}

fn remove_shim_or_link(path: &Path) -> Result<(), HealError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(file_err) => {
            if fs::remove_dir(path).is_ok() {
                Ok(())
            } else {
                Err(HealError::Io {
                    path: norm_str(path),
                    source: file_err,
                })
            }
        }
    }
}

/// Determines whether `path` is a legacy Toksi/Tokless shim script or sidecar marker file.
///
/// Strictly verifies content markers before flagging any script so legitimate agent scripts
/// (such as npm's `agy.cmd` or `copilot.cmd`) are NEVER deleted.
pub fn is_toksi_shim_or_sidecar(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let lower_name = name.to_lowercase();

    if LEGACY_SCRIPT_FILENAMES.contains(&lower_name.as_str()) {
        return true;
    }

    // Sidecar files created by copilot/kilo legacy hijacker
    if lower_name.ends_with(".toksi") || lower_name.ends_with(".tokless") {
        return true;
    }
    if lower_name.ends_with(".digest")
        && (lower_name.starts_with("copilot")
            || lower_name.starts_with("agy")
            || lower_name.starts_with("grok")
            || lower_name.starts_with("kilo"))
    {
        return true;
    }

    // Only inspect small files (< 128 KB) as text shims
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if meta.len() > 131_072 {
        return false;
    }

    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };

    LEGACY_SHIM_MARKERS
        .iter()
        .any(|marker| content.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn make_temp_dir(prefix: &str) -> Result<PathBuf, std::io::Error> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let p = env::temp_dir().join(format!("toksi-heal-test-{prefix}-{nanos}"));
        fs::create_dir_all(&p)?;
        Ok(p)
    }

    #[test]
    fn test_derive_canonical_path() {
        let dir = Path::new("/tmp/bin");
        assert_eq!(
            derive_canonical_path(&dir.join("agy.real.exe")),
            Some(norm_path_buf(&dir.join("agy.exe")))
        );
        assert_eq!(
            derive_canonical_path(&dir.join("grok.cmd.real")),
            Some(norm_path_buf(&dir.join("grok.cmd")))
        );
        if IS_WIN {
            assert_eq!(
                derive_canonical_path(&dir.join("copilot.real")),
                Some(norm_path_buf(&dir.join("copilot.exe")))
            );
        } else {
            assert_eq!(
                derive_canonical_path(&dir.join("copilot.real")),
                Some(norm_path_buf(&dir.join("copilot")))
            );
        }
    }

    #[test]
    fn test_heal_legacy_hijacks_restores_real_and_purges_shims() -> TestResult {
        let bin_dir = make_temp_dir("bin")?;
        let data_dir = make_temp_dir("data")?;

        // 1. Create a fake `agy.real.exe` and a fake `agy.cmd` shim
        let real_agy = bin_dir.join("agy.real.exe");
        let shim_agy = bin_dir.join("agy.cmd");
        fs::write(&real_agy, b"REAL_BINARY_BYTES")?;
        fs::write(
            &shim_agy,
            "@echo off\r\nrem toksi:antigravity-cli-proxy\r\n\"agy.real.exe\" %*\r\n",
        )?;

        // 2. Create a fake `grok.real.exe` and `grok.ps1` shim
        let real_grok = bin_dir.join("grok.real.exe");
        let shim_grok = bin_dir.join("grok.ps1");
        fs::write(&real_grok, b"REAL_GROK_BYTES")?;
        fs::write(
            &shim_grok,
            "# toksi:grok-cli-proxy\n& \"grok.real.exe\" @args\n",
        )?;

        // 3. Create copilot sidecar files and legacy codegraph init script
        let copilot_digest = bin_dir.join("copilot.exe.digest");
        let copilot_toksi = bin_dir.join("copilot.exe.toksi");
        let legacy_cg_init = bin_dir.join("toksi-codegraph-init.js");
        fs::write(&copilot_digest, "abc")?;
        fs::write(&copilot_toksi, "xyz")?;
        fs::write(&legacy_cg_init, "// legacy init")?;

        // 4. Create orphaned state files in data_dir
        let state_agy = data_dir.join("antigravity-cli-proxy.json");
        let state_grok = data_dir.join("grok-cli-proxy.json");
        fs::write(&state_agy, "{}")?;
        fs::write(&state_grok, "{}")?;

        // Run healer
        let report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);

        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
        assert_eq!(report.restored_binaries.len(), 2);
        for (real_p, restored_p) in &report.restored_binaries {
            assert!(!real_p.to_string_lossy().contains('\\'));
            assert!(!restored_p.to_string_lossy().contains('\\'));
        }

        let canonical_agy = bin_dir.join("agy.exe");
        let canonical_grok = bin_dir.join("grok.exe");
        assert!(canonical_agy.exists());
        assert_eq!(fs::read(&canonical_agy)?, b"REAL_BINARY_BYTES");
        assert!(!real_agy.exists());
        assert!(!shim_agy.exists());

        assert!(canonical_grok.exists());
        assert_eq!(fs::read(&canonical_grok)?, b"REAL_GROK_BYTES");
        assert!(!real_grok.exists());
        assert!(!shim_grok.exists());

        assert!(!copilot_digest.exists());
        assert!(!copilot_toksi.exists());
        assert!(!legacy_cg_init.exists());
        assert!(!state_agy.exists());
        assert!(!state_grok.exists());

        // Cleanup
        let _ = fs::remove_dir_all(bin_dir);
        let _ = fs::remove_dir_all(data_dir);
        Ok(())
    }

    #[test]
    fn test_legitimate_scripts_are_never_deleted() -> TestResult {
        let bin_dir = make_temp_dir("legit")?;
        let data_dir = make_temp_dir("legit-data")?;

        // Create a legitimate npm wrapper for agy.cmd (without any toksi markers)
        let legit_agy = bin_dir.join("agy.cmd");
        let legit_content = "@IF EXIST \"%~dp0\\node.exe\" (\r\n  \"%~dp0\\node.exe\" \"%~dp0\\node_modules\\agy\\bin.js\" %*\r\n)\r\n";
        fs::write(&legit_agy, legit_content)?;

        let report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);
        assert!(report.is_clean());
        assert!(legit_agy.exists(), "Legitimate agy.cmd must not be deleted!");

        let _ = fs::remove_dir_all(bin_dir);
        let _ = fs::remove_dir_all(data_dir);
        Ok(())
    }
}
