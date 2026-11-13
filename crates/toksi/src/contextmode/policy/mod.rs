// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/policy/mod.rs
// created: 2026-11-13

use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;
use super::security::{check_command_deny, is_own_file, read_deny_message};

pub const OUTPUT_CAP: usize = 16384;
pub const USER_DENY_REASON_PREFIX: &str = "context-mode user policy blocked: ";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Hint { text: String, once_key: String },
    Deny { reason: String, rewritten: Option<String> },
    Replace { output: String },
}

#[derive(Debug, Clone, Default)]
pub struct ToolCallInput {
    pub name: String,
    pub canonical_name: String,
    pub command: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub raw_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CanonicalHookEvent {
    pub event: String,
    pub agent: String,
    pub session_id: String,
    pub cwd: String,
    pub prompt: Option<String>,
    pub tool: Option<ToolCallInput>,
    pub turn: Option<usize>,
    pub is_subagent: bool,
    pub agent_type: Option<String>,
}

/// Evaluates PreToolUse policy according to master context-mode specifications.
/// Rule evaluation order:
/// 1. User Deny patterns (glob / regex rules)
/// 2. WebFetch tool redirection
/// 3. Shell / Bash command inspection
/// 4. Read tool file-size limit inspection
pub fn decide(
    ev: &CanonicalHookEvent,
    agent: &str,
    user_deny: &[String],
    project_root: &Path,
) -> Decision {
    let tool = match ev.tool.as_ref() {
        Some(t) => t,
        None => return Decision::Allow,
    };

    let canonical = tool.canonical_name.as_str();

    // 1. User Deny check
    if let Some(hit) = check_user_deny(tool, &ev.cwd, user_deny) {
        return Decision::Deny {
            reason: format!("{}{}", USER_DENY_REASON_PREFIX, hit),
            rewritten: None,
        };
    }

    // 2. WebFetch check
    if canonical == "WebFetch" || tool.name == "web_fetch" {
        if let Some(ref url) = tool.url {
            if !is_never_denied_host(url) {
                let msg = format!(
                    "context-mode: raw WebFetch of '{}' is blocked.\nRun ctx_index(url=\"{}\") then ctx_search(query=...) to read cleanly.",
                    url, url
                );
                return Decision::Deny {
                    reason: msg,
                    rewritten: None,
                };
            }
        }
    }

    // 3. Shell / Bash check
    if canonical == "Bash" || canonical == "shell" || tool.name == "run_shell_command" || tool.name == "execute_command" {
        if let Some(ref cmd) = tool.command {
            // Check dangerous/destructive patterns
            if let Some(denied) = check_command_deny(cmd) {
                return Decision::Deny {
                    reason: denied,
                    rewritten: None,
                };
            }

            // Check inline HTTP fetches
            if has_inline_http(cmd) {
                let msg = "context-mode: direct inline curl/wget/http fetch in shell blocked.\nRun ctx_fetch_and_index(url=\"...\") and ctx_search() instead.";
                return Decision::Deny {
                    reason: msg.to_string(),
                    rewritten: None,
                };
            }

            // Check single-file shell dumps (e.g. cat <large_file>)
            if let Some(target_file) = extract_cat_or_dump_target(cmd) {
                let p = Path::new(&target_file);
                let abs = if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    Path::new(&ev.cwd).join(p)
                };
                if let Ok(meta) = std::fs::metadata(&abs) {
                    if meta.is_file() && meta.len() as usize > OUTPUT_CAP && !is_own_file(agent, &abs) {
                        let inside = is_inside_root(&abs, project_root);
                        let reason = read_deny_message(&abs.to_string_lossy(), meta.len() as usize, inside, "");
                        return Decision::Deny {
                            reason,
                            rewritten: None,
                        };
                    }
                }
            }
        }
    }

    // 4. Read check
    if canonical == "Read" || tool.name == "read_file" || tool.name == "view" {
        if let Some(ref file_str) = tool.file_path {
            let p = Path::new(file_str);
            let abs = if p.is_absolute() {
                p.to_path_buf()
            } else {
                Path::new(&ev.cwd).join(p)
            };

            // Rule files & agent skills always bypass deny
            if is_own_file(agent, &abs) {
                return Decision::Allow;
            }

            if let Ok(meta) = std::fs::metadata(&abs) {
                if meta.is_file() && meta.len() as usize > OUTPUT_CAP {
                    let inside = is_inside_root(&abs, project_root);
                    let reason = read_deny_message(&abs.to_string_lossy(), meta.len() as usize, inside, "");
                    return Decision::Deny {
                        reason,
                        rewritten: None,
                    };
                }
            }
        }
    }

    Decision::Allow
}

fn is_never_denied_host(url_str: &str) -> bool {
    if let Ok(parsed) = url::Url::parse(url_str) {
        if let Some(host) = parsed.host_str() {
            let h = host.to_lowercase();
            return h == "localhost" || h == "127.0.0.1" || h.ends_with(".local");
        }
    }
    false
}

fn has_inline_http(cmd: &str) -> bool {
    let lower = cmd.to_lowercase();
    (lower.contains("curl ") || lower.contains("wget "))
        && (lower.contains("http://") || lower.contains("https://"))
}

fn extract_cat_or_dump_target(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    if (parts[0] == "cat" || parts[0] == "type") && parts.len() == 2 && !parts[1].starts_with('-') {
        return Some(parts[1].to_string());
    }
    None
}

fn is_inside_root(path: &Path, root: &Path) -> bool {
    let canon_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let canon_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let norm_path = normalize_path(&canon_path.to_string_lossy()).to_lowercase();
    let norm_root = normalize_path(&canon_root.to_string_lossy()).to_lowercase();
    norm_path.starts_with(&norm_root)
}

fn check_user_deny(tool: &ToolCallInput, cwd: &str, user_deny: &[String]) -> Option<String> {
    if user_deny.is_empty() {
        return None;
    }

    let cmd_target = tool.command.as_deref().unwrap_or("");
    let file_target = tool.file_path.as_deref().map(|p| {
        let pb = Path::new(cwd).join(p);
        normalize_path(&pb.to_string_lossy())
    }).unwrap_or_default();

    for rule in user_deny {
        let pat = rule.trim();
        if pat.is_empty() {
            continue;
        }

        // Try exact substring or glob
        if !cmd_target.is_empty() && (cmd_target.contains(pat) || matches_pattern(cmd_target, pat)) {
            return Some(pat.to_string());
        }
        if !file_target.is_empty() && (file_target.contains(pat) || matches_pattern(&file_target, pat)) {
            return Some(pat.to_string());
        }
    }

    None
}

fn matches_pattern(text: &str, pat: &str) -> bool {
    if pat.contains('*') || pat.contains('?') {
        let regex_str = format!("(?i)^{}$", pat.replace('.', r"\.").replace('*', ".*").replace('?', "."));
        Regex::new(&regex_str).is_ok_and(|re| re.is_match(text))
    } else {
        text.eq_ignore_ascii_case(pat)
    }
}
