// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/security.rs
// created: 2026-10-27

use std::path::{Path, PathBuf};
use regex::Regex;

const DENIED_COMMAND_PATTERNS: &[&str] = &[
    r"(?i)\brm\s+(-[a-z]*r[a-z]*f[a-z]*|-rf|-fr)\s+([/~]|\*|/\*)",
    r"(?i)\bmkfs(\.[a-z0-9]+)?\b",
    r"(?i)\bdd\s+if=/dev/(zero|urandom)\s+of=/dev/[a-z]+",
    r":\(\)\s*\{\s*:\|:&\s*\};:",
    r"(?i)\b(shutdown|reboot|poweroff|init\s+0)\b",
    r"(?i)\bchmod\s+(-[a-z]*R[a-z]*\s+)?777\s+/",
];

const DENIED_PATH_PATTERNS: &[&str] = &[
    r"(?i)(^|[/\\])\.env(\.[a-z0-9._-]+)?$",
    r"(?i)(^|[/\\])id_(rsa|ed25519|ecdsa|dsa)(\.pub)?$",
    r"(?i)\.(pem|key|pfx|p12|keystore)$",
    r"(?i)(^|[/\\])\.aws[/\\]credentials$",
    r"(?i)(^|[/\\])\.ssh[/\\][^/\\]+$",
    r"(?i)(^|[/\\])credentials\.json$",
    r"(?i)(^|[/\\])service[-_]?account.*\.json$",
];

/// Check if a command matches dangerous or destructive command patterns.
pub fn check_command_deny(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();
    for pat in DENIED_COMMAND_PATTERNS {
        if let Ok(re) = Regex::new(pat) {
            if re.is_match(trimmed) {
                return Some(format!("Command blocked by sandbox security policy: matches '{}'", pat));
            }
        }
    }
    None
}

/// Check if a target file path targets sensitive credentials or keys.
pub fn check_path_deny(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    for pat in DENIED_PATH_PATTERNS {
        if let Ok(re) = Regex::new(pat) {
            if re.is_match(&path_str) {
                return Some(format!("Access to sensitive file blocked by security policy: matches '{}'", pat));
            }
        }
    }
    None
}

fn normalize_components(p: &Path) -> PathBuf {
    let s = p.to_string_lossy().replace('\\', "/");
    let clean = s.trim_start_matches("//?/").trim_start_matches("\\\\?\\");
    let mut parts = Vec::new();
    for seg in clean.split('/') {
        if seg == "." || seg.is_empty() {
            continue;
        } else if seg == ".." {
            parts.pop();
        } else {
            parts.push(seg);
        }
    }
    PathBuf::from(parts.join("/"))
}

/// Check if a file path is contained within a root project boundary.
pub fn check_boundary(target: &Path, root: &Path) -> Result<PathBuf, String> {
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root_norm = normalize_components(&root_canon);

    let full_target = if target.is_relative() {
        root.join(target)
    } else {
        target.to_path_buf()
    };

    let target_canon = full_target.canonicalize().unwrap_or_else(|_| full_target.clone());
    let target_norm = normalize_components(&target_canon);

    let target_str = target_norm.to_string_lossy().to_lowercase();
    let root_str = root_norm.to_string_lossy().to_lowercase();

    if target_str.starts_with(&root_str) {
        Ok(target_canon)
    } else {
        Err(format!(
            "Path boundary violation: '{}' is outside project root '{}'",
            target.display(),
            root.display()
        ))
    }
}
