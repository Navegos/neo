// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/env_injector/heal.rs
// created: 2026-10-31

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::agents::antigravity::agy_known_bin_dirs;
use crate::util::paths::{self, expected_bin_dirs, grok_paths_resolved, tokless_data_dir};

/// Markers that identify legacy Tokless-generated shims and batch wrappers.
const SHIM_MARKERS: &[&str] = &[
    "tokless:antigravity-cli-proxy",
    "tokless:grok-cli-proxy",
    "tokless:copilot-shim",
    "__proxy-ensure",
    "__grok-proxy-ensure",
    "__copilot",
    "rem tokless:",
    "# tokless:copilot-shim",
];

/// Known legacy orphaned state file names in Tokless data directories.
const ORPHANED_STATE_FILES: &[&str] = &[
    "antigravity-cli-proxy.json",
    "grok-cli-proxy.json",
    "copilot-cli-proxy.json",
    "grok-mitm-ca.pem",
];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HealingReport {
    /// List of (stashed_path, restored_canonical_path)
    pub restored_binaries: Vec<(PathBuf, PathBuf)>,
    /// List of deleted shim wrappers (e.g. `agy.cmd`, `grok.cmd`, `copilot.cmd`, shell scripts)
    pub removed_shims: Vec<PathBuf>,
    /// List of deleted orphaned state files (e.g. `antigravity-cli-proxy.json`)
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

/// Automatically detect and heal all legacy binary hijacks, shims, and orphaned state files.
pub fn heal_legacy_hijacks() -> HealingReport {
    let scan_dirs = collect_binary_scan_dirs();
    let data_dirs = collect_data_dirs();
    heal_legacy_hijacks_in(&scan_dirs, &data_dirs)
}

/// Targeted healing routine operating within the provided binary and data directories.
pub fn heal_legacy_hijacks_in(scan_dirs: &[PathBuf], data_dirs: &[PathBuf]) -> HealingReport {
    let mut report = HealingReport::default();
    let mut visited_dirs = HashSet::new();

    // 1. Scan binary directories for .real / .real.exe and shims
    for dir in scan_dirs {
        if !dir.is_dir() {
            continue;
        }
        let canonical_dir = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if !visited_dirs.insert(canonical_dir.clone()) {
            continue;
        }

        heal_directory(&canonical_dir, &mut report);
    }

    // 2. Scan data directories for orphaned state files
    let mut visited_data_dirs = HashSet::new();
    for data_dir in data_dirs {
        if !data_dir.is_dir() {
            continue;
        }
        let canonical_data = data_dir.canonicalize().unwrap_or_else(|_| data_dir.clone());
        if !visited_data_dirs.insert(canonical_data.clone()) {
            continue;
        }

        heal_orphaned_state_files(&canonical_data, &mut report);
    }

    report
}

/// Collects standard directories where agent binaries or legacy shims could reside.
fn collect_binary_scan_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Antigravity known bin directories
    for dir_str in agy_known_bin_dirs() {
        if !dir_str.is_empty() {
            dirs.push(PathBuf::from(dir_str));
        }
    }

    // Grok install directories
    let grok_dir = grok_paths_resolved().install_dir;
    if !grok_dir.is_empty() {
        dirs.push(PathBuf::from(grok_dir));
    }

    // Expected bin dirs (e.g. ~/.local/bin, %LOCALAPPDATA%\...)
    for dir_str in expected_bin_dirs() {
        if !dir_str.is_empty() {
            dirs.push(PathBuf::from(dir_str));
        }
    }

    // Directories in PATH
    let path_var = env::var("PATH").or_else(|_| env::var("Path")).unwrap_or_default();
    let sep = if paths::is_win() { ';' } else { ':' };
    for part in path_var.split(sep) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            dirs.push(PathBuf::from(trimmed));
        }
    }

    dirs
}

/// Collects directories where Tokless or legacy proxies stored state files.
fn collect_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let data_dir = tokless_data_dir();
    if !data_dir.is_empty() {
        dirs.push(PathBuf::from(data_dir));
    }
    let home = paths::home();
    if !home.is_empty() {
        let home_path = PathBuf::from(&home);
        dirs.push(home_path.join(".tokless"));
        dirs.push(home_path.join(".antigravity"));
        dirs.push(home_path.join(".grok"));
    }
    dirs
}

/// Heals a single binary directory by restoring .real / .real.exe files and deleting shims.
fn heal_directory(dir: &Path, report: &mut HealingReport) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut files = Vec::new();
    for entry in entries.flatten() {
        files.push(entry.path());
    }

    // Pass 1: Identify and atomically restore stashed .real / .real.exe binaries
    for file in &files {
        if let Some((is_real, canonical_name)) = analyze_stashed_binary(file) {
            if is_real {
                let canonical_path = dir.join(&canonical_name);
                restore_canonical_binary(file, &canonical_path, report);
            }
        }
    }

    // Pass 2: Identify and remove orphaned shims, batch wrappers, and sidecars
    for file in &files {
        if is_tokless_shim_or_sidecar(file) {
            match fs::remove_file(file) {
                Ok(_) => report.removed_shims.push(file.clone()),
                Err(err) => {
                    report.errors.push(format!(
                        "failed to remove shim {}: {}",
                        file.display(),
                        err
                    ));
                }
            }
        }
    }
}

/// Analyzes whether a filename matches a stashed .real or .real.exe binary pattern.
/// Returns (is_real, canonical_name)
fn analyze_stashed_binary(path: &Path) -> Option<(bool, String)> {
    let file_name = path.file_name()?.to_str()?;
    let lower = file_name.to_ascii_lowercase();

    // Specific known agents:
    // agy.real.exe -> agy.exe
    if lower == "agy.real.exe" {
        return Some((true, "agy.exe".to_string()));
    }
    // agy.real -> agy (or agy.exe on Windows if applicable)
    if lower == "agy.real" {
        let canonical = if paths::is_win() { "agy.exe" } else { "agy" };
        return Some((true, canonical.to_string()));
    }
    // grok.real.exe -> grok.exe
    if lower == "grok.real.exe" {
        return Some((true, "grok.exe".to_string()));
    }
    // grok.real -> grok (or grok.exe on Windows)
    if lower == "grok.real" {
        let canonical = if paths::is_win() { "grok.exe" } else { "grok" };
        return Some((true, canonical.to_string()));
    }
    // copilot.real.exe -> copilot.exe
    if lower == "copilot.real.exe" {
        return Some((true, "copilot.exe".to_string()));
    }
    // copilot.real -> copilot (or copilot.exe on Windows)
    if lower == "copilot.real" {
        let canonical = if paths::is_win() { "copilot.exe" } else { "copilot" };
        return Some((true, canonical.to_string()));
    }
    // copilot.cmd.real -> copilot.cmd
    if lower == "copilot.cmd.real" {
        return Some((true, "copilot.cmd".to_string()));
    }

    // Generic pattern: <stem>.real.exe -> <stem>.exe
    if lower.ends_with(".real.exe") {
        let orig_name = &file_name[..file_name.len() - 9];
        return Some((true, format!("{orig_name}.exe")));
    }

    // Generic pattern: <stem>.real -> <stem>
    if lower.ends_with(".real") {
        let orig_name = &file_name[..file_name.len() - 5];
        let canonical = if paths::is_win() && !orig_name.contains('.') {
            format!("{orig_name}.exe")
        } else {
            orig_name.to_string()
        };
        return Some((true, canonical));
    }

    None
}

/// Atomically restores a stashed binary back to its canonical path.
fn restore_canonical_binary(
    stashed_path: &Path,
    canonical_path: &Path,
    report: &mut HealingReport,
) {
    if !stashed_path.exists() {
        return;
    }

    // If the canonical path already exists, check if it's a shim/wrapper or older placeholder
    if canonical_path.exists() {
        if is_tokless_shim_or_sidecar(canonical_path) || is_placeholder_or_wrapper(canonical_path) {
            // Remove the bogus wrapper so we can safely rename the real binary
            let _ = fs::remove_file(canonical_path);
        } else {
            // Canonical file is already present and not obviously a shim.
            // Move canonical to a timestamped backup before attempting rename.
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let backup_path = canonical_path.with_extension(format!("tokless-heal-bak-{nanos}"));
            if let Err(e) = fs::rename(canonical_path, &backup_path) {
                report.errors.push(format!(
                    "cannot backup existing {} before restoring {}: {}",
                    canonical_path.display(),
                    stashed_path.display(),
                    e
                ));
                return;
            }
        }
    }

    // Atomically rename stashed binary to canonical path
    match fs::rename(stashed_path, canonical_path) {
        Ok(_) => {
            report.restored_binaries.push((
                stashed_path.to_path_buf(),
                canonical_path.to_path_buf(),
            ));
        }
        Err(err) => {
            // If cross-device link or rename failed, try copy + remove
            if fs::copy(stashed_path, canonical_path).is_ok() {
                let _ = fs::remove_file(stashed_path);
                report.restored_binaries.push((
                    stashed_path.to_path_buf(),
                    canonical_path.to_path_buf(),
                ));
            } else {
                report.errors.push(format!(
                    "failed to restore stashed binary {} -> {}: {}",
                    stashed_path.display(),
                    canonical_path.display(),
                    err
                ));
            }
        }
    }
}

/// Checks whether a given path is a legacy Tokless shim, batch script wrapper, or sidecar.
pub fn is_tokless_shim_or_sidecar(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(s) => s.to_ascii_lowercase(),
        None => return false,
    };

    // Digest, marker, or intermediate removal files
    if file_name.ends_with(".tokless")
        || file_name.ends_with(".digest")
        || file_name.ends_with(".removing")
    {
        return true;
    }

    // Candidate wrapper filenames
    let is_candidate_wrapper = file_name == "agy.cmd"
        || file_name == "agy.bat"
        || file_name == "agy"
        || file_name == "grok.cmd"
        || file_name == "grok.bat"
        || file_name == "grok"
        || file_name == "copilot.cmd"
        || file_name == "copilot.bat"
        || file_name == "copilot";

    if !is_candidate_wrapper {
        return false;
    }

    // Inspect content for shim markers (only read first 16KB)
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    if bytes.len() > 128 * 1024 {
        // Real binaries won't be under 128KB batch scripts with markers
        return false;
    }

    let Ok(text) = std::str::from_utf8(&bytes[..bytes.len().min(16 * 1024)]) else {
        return false;
    };

    for marker in SHIM_MARKERS {
        if text.contains(marker) {
            return true;
        }
    }

    // Additional checks for batch wrappers setting REAL/URL
    if (file_name.ends_with(".cmd") || file_name.ends_with(".bat"))
        && text.starts_with("@echo off")
        && (text.contains("%REAL%") || text.contains("GOOGLE_GEMINI_BASE_URL") || text.contains("HEADROOM_WRAP_SILENT"))
    {
        return true;
    }

    // Shell script wrapper check
    if text.starts_with("#!/") && (text.contains("exec \"$REAL\"") || text.contains("tokless")) {
        return true;
    }

    false
}

/// Checks if a file looks like a dummy placeholder or minimal wrapper rather than a real executable.
fn is_placeholder_or_wrapper(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    // 0-byte file is definitely a placeholder
    if meta.len() == 0 {
        return true;
    }
    // Very small files (< 1KB) that are valid UTF-8 text containing batch or shell scripts
    if meta.len() < 1024 {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                if text.starts_with("@echo off") || text.starts_with("#!") {
                    return true;
                }
            }
        }
    }
    false
}

/// Heals orphaned state files in the given data directory.
fn heal_orphaned_state_files(data_dir: &Path, report: &mut HealingReport) {
    for state_name in ORPHANED_STATE_FILES {
        let state_path = data_dir.join(state_name);
        if state_path.is_file() {
            match fs::remove_file(&state_path) {
                Ok(_) => report.removed_state_files.push(state_path),
                Err(err) => {
                    report.errors.push(format!(
                        "failed to remove orphaned state file {}: {}",
                        state_path.display(),
                        err
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let p = env::temp_dir().join(format!("tokless-heal-test-{name}-{nanos}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn test_analyze_stashed_binary() {
        assert_eq!(
            analyze_stashed_binary(Path::new("agy.real.exe")),
            Some((true, "agy.exe".to_string()))
        );
        assert_eq!(
            analyze_stashed_binary(Path::new("grok.real.exe")),
            Some((true, "grok.exe".to_string()))
        );
        assert_eq!(
            analyze_stashed_binary(Path::new("copilot.cmd.real")),
            Some((true, "copilot.cmd".to_string()))
        );
        assert_eq!(analyze_stashed_binary(Path::new("normal.exe")), None);
    }

    #[test]
    fn test_heal_legacy_binary_restoration() {
        let bin_dir = create_temp_dir("bin");
        let data_dir = create_temp_dir("data");

        // Simulate legacy hijacked agy:
        // agy.real.exe is the real binary
        // agy.cmd is the wrapper shim
        let real_agy = bin_dir.join("agy.real.exe");
        let shim_agy = bin_dir.join("agy.cmd");
        fs::write(&real_agy, b"REAL_BINARY_BYTES").unwrap();
        fs::write(
            &shim_agy,
            "@echo off\r\nrem tokless:antigravity-cli-proxy\r\nset \"REAL=agy.real.exe\"\r\n\"%REAL%\" %*\r\n",
        )
        .unwrap();

        // Simulate legacy grok shim
        let real_grok = bin_dir.join("grok.real.exe");
        let shim_grok = bin_dir.join("grok.cmd");
        fs::write(&real_grok, b"REAL_GROK_BYTES").unwrap();
        fs::write(
            &shim_grok,
            "@echo off\r\nrem tokless:grok-cli-proxy\r\nset \"REAL=grok.real.exe\"\r\n\"%REAL%\" %*\r\n",
        )
        .unwrap();

        // Simulate legacy copilot sidecars
        let copilot_digest = bin_dir.join("copilot.cmd.digest");
        let copilot_tokless = bin_dir.join("copilot.real.tokless");
        fs::write(&copilot_digest, "abc").unwrap();
        fs::write(&copilot_tokless, "xyz").unwrap();

        // Simulate orphaned state files in data dir
        let state_agy = data_dir.join("antigravity-cli-proxy.json");
        let state_grok = data_dir.join("grok-cli-proxy.json");
        fs::write(&state_agy, "{}").unwrap();
        fs::write(&state_grok, "{}").unwrap();

        let report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);

        assert!(!report.is_clean());
        assert_eq!(report.errors.len(), 0);

        // Verify binaries were restored
        let canonical_agy = bin_dir.join("agy.exe");
        let canonical_grok = bin_dir.join("grok.exe");
        assert!(canonical_agy.exists());
        assert_eq!(fs::read(&canonical_agy).unwrap(), b"REAL_BINARY_BYTES");
        assert!(!real_agy.exists());

        assert!(canonical_grok.exists());
        assert_eq!(fs::read(&canonical_grok).unwrap(), b"REAL_GROK_BYTES");
        assert!(!real_grok.exists());

        // Verify shims and sidecars deleted
        assert!(!shim_agy.exists());
        assert!(!shim_grok.exists());
        assert!(!copilot_digest.exists());
        assert!(!copilot_tokless.exists());

        // Verify orphaned state files deleted
        assert!(!state_agy.exists());
        assert!(!state_grok.exists());

        // Verify subsequent run is clean (idempotent)
        let report2 = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);
        assert!(report2.is_clean());
        assert_eq!(report2.total_healed(), 0);

        let _ = fs::remove_dir_all(bin_dir);
        let _ = fs::remove_dir_all(data_dir);
    }
}
