// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/security.rs
// created: 2026-10-27
// lastModified: 2026-11-08

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
        if Regex::new(pat).is_ok_and(|re| re.is_match(trimmed)) {
            return Some(format!("Command blocked by sandbox security policy: matches '{}'", pat));
        }
    }
    None
}

/// Check if a target file path targets sensitive credentials or keys.
pub fn check_path_deny(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    for pat in DENIED_PATH_PATTERNS {
        if Regex::new(pat).is_ok_and(|re| re.is_match(&path_str)) {
            return Some(format!("Access to sensitive file blocked by security policy: matches '{}'", pat));
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

/// D-R48-OWN-FILE-1 (#1326): Check if a target file is one of the agent's own instruction or skill files.
/// Skill references, rule files, and CLAUDE.md sit in the agent's config dir or shared ~/.agents/skills.
/// Instructions must reach the agent whole; markdown files in these locations are never denied.
pub fn is_own_file(agent: &str, target: &Path) -> bool {
    let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext != "md" && ext != "markdown" {
        return false;
    }

    let file_name = target.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
    if file_name == "claude.md" || file_name == "agents.md" || file_name == "gemini.md" {
        return true;
    }

    let target_norm = normalize_components(target).to_string_lossy().to_lowercase();
    let own_dirs = own_dirs_of(agent);
    for dir in own_dirs {
        let dir_norm = normalize_components(&dir).to_string_lossy().to_lowercase();
        if target_norm.starts_with(&dir_norm) {
            return true;
        }
    }
    false
}

/// The dirs this agent reads its own instruction and skill files from, plus shared skill dirs.
pub fn own_dirs_of(agent: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = dirs::home_dir().or_else(|| {
        std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok().map(PathBuf::from)
    });

    if let Some(h) = home {
        // Shared skill dir
        dirs.push(h.join(".agents").join("skills"));

        // Agent-specific config dir
        match agent {
            "claude-code" | "claude" => {
                if let Ok(c) = std::env::var("CLAUDE_CONFIG_DIR") {
                    dirs.push(PathBuf::from(c));
                } else {
                    dirs.push(h.join(".claude"));
                }
            }
            "codex" => dirs.push(h.join(".codex")),
            "gemini" | "agy" | "antigravity" => dirs.push(h.join(".gemini")),
            "copilot-cli" | "copilot" => dirs.push(h.join(".copilot")),
            "kiro" => dirs.push(h.join(".kiro")),
            "opencode" => dirs.push(h.join(".opencode")),
            "pi" => dirs.push(h.join(".pi")),
            "vscode" => {
                dirs.push(h.join(".vscode"));
                #[cfg(windows)]
                if let Ok(app_data) = std::env::var("APPDATA") {
                    dirs.push(PathBuf::from(app_data).join("Code").join("User"));
                }
            }
            _ => {
                dirs.push(h.join(format!(".{}", agent)));
            }
        }
    }

    // Also include workspace-relative skills and .agents/skills if available
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join(".agents").join("skills"));
        dirs.push(cwd.join("skills"));
    }

    dirs
}

/// D-R48-DENY-ROUTE-1 (#1326): Return the appropriate Read deny route message.
/// Inside root: points to ctx_execute(language, code, file=...).
/// Outside root: points to ranged Read or sed -n '1,200p'.
pub fn read_deny_message(path: &str, size: usize, inside_root: bool, prefix: &str) -> String {
    if inside_root {
        format!(
            "context-mode: {}-byte file, over the one-read limit. Run {}ctx_execute(language, code, file=\"{}\") and print only the answer.",
            size, prefix, path
        )
    } else {
        format!(
            "context-mode: {}-byte file, over the one-read limit. Read \"{}\" in parts with offset and limit, or sed -n '1,200p'.",
            size, path
        )
    }
}
