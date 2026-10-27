// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/prime.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::commands::doctor_stats::doctor_tool_stats;
use crate::headroom::proxy::proxy_running;
use crate::util::colors::*;
use crate::util::exec::run_cmd;
use crate::util::logger::L;
use crate::util::paths::is_win;

#[derive(Default, Clone, Debug)]
pub struct PrimeOptions {
    pub copy: bool,
    pub json: bool,
    pub output_file: Option<String>,
}

pub fn run_prime(opts: PrimeOptions) -> i32 {
    let cwd = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into());

    let (content, json_val) = generate_prime_context(&cwd);

    if opts.json {
        let out = serde_json::to_string_pretty(&json_val).unwrap_or_default();
        if let Some(ref path) = opts.output_file {
            if let Err(e) = fs::write(path, &out) {
                L.err(&format!("Failed writing to {}: {}", path, e));
                return 1;
            }
            L.raw(&format!("  {} Written JSON context to {}", C.green(Sym::CHECK), path));
        } else {
            println!("{}", out);
        }
        return 0;
    }

    if let Some(ref path) = opts.output_file {
        if let Err(e) = fs::write(path, &content) {
            L.err(&format!("Failed writing to {}: {}", path, e));
            return 1;
        }
        L.raw(&format!("  {} Written prime context to {}", C.green(Sym::CHECK), path));
        return 0;
    }

    if opts.copy {
        if copy_to_clipboard(&content) {
            L.raw(&format!("  {} Copied prime context to clipboard", C.green(Sym::CHECK)));
            return 0;
        } else {
            L.warn("Could not copy to clipboard; printing to stdout instead.");
        }
    }

    println!("{}", content);
    0
}

fn generate_prime_context(cwd: &str) -> (String, serde_json::Value) {
    let mut sections = Vec::new();

    sections.push("# Context Primer (Tokless)".to_string());
    sections.push(format!("Generated for workspace: `{}`", cwd));

    // Git branch / repo context
    let git_branch = run_cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"], &[]);
    let branch_str = if git_branch.code == 0 {
        git_branch.stdout.trim().to_string()
    } else {
        "unknown".to_string()
    };
    sections.push(format!("- **Git Branch**: `{}`", branch_str));

    // Headroom & Proxy status
    let proxy_active = proxy_running();
    let stats = doctor_tool_stats(cwd, true);
    let mut proxy_notes = vec![
        format!("- **Headroom Proxy**: {}", if proxy_active { "Active (127.0.0.1:8787)" } else { "Inactive" })
    ];
    if let Some(hr_rows) = stats.get("headroom") {
        for row in hr_rows {
            if row.contains("warmth:") || row.contains("lifetime:") {
                proxy_notes.push(format!("- **Headroom**: {}", row.trim()));
            }
        }
    }
    sections.push("\n## Proxy & Cache Performance".to_string());
    sections.extend(proxy_notes);

    // Project Memory
    let mut decisions = Vec::new();
    let mut gotchas = Vec::new();

    let events_path = Path::new(cwd).join(".projectmem").join("events.jsonl");
    if events_path.exists() {
        if let Ok(lines) = fs::read_to_string(&events_path) {
            for line in lines.lines().rev().take(50) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    let ev_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let summary = val.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                    if summary.is_empty() {
                        continue;
                    }
                    if ev_type == "decision" && decisions.len() < 5 {
                        decisions.push(format!("- {}", summary));
                    } else if (ev_type == "note" || summary.starts_with("gotcha:")) && gotchas.len() < 8 {
                        gotchas.push(format!("- {}", summary));
                    }
                }
            }
        }
    }

    sections.push("\n## Architectural Decisions & High-Priority Gotchas".to_string());
    if !decisions.is_empty() {
        sections.push("### Recent Decisions:".to_string());
        sections.extend(decisions.clone());
    }
    if !gotchas.is_empty() {
        sections.push("### Critical Gotchas:".to_string());
        sections.extend(gotchas.clone());
    }
    if decisions.is_empty() && gotchas.is_empty() {
        sections.push("- No recorded decisions or gotchas in `.projectmem/` yet.".to_string());
    }

    // CodeGraph Status
    let cg_dir = Path::new(cwd).join(".codegraph");
    if cg_dir.exists() {
        sections.push("\n## Code Index (CodeGraph)".to_string());
        sections.push("- Indexed database present in `.codegraph/`. Use CodeGraph MCP tool for blast radius and dependency flows.".to_string());
    }

    let full_content = sections.join("\n");
    let json_val = serde_json::json!({
        "workspace": cwd,
        "branch": branch_str,
        "proxy_active": proxy_active,
        "decisions": decisions,
        "gotchas": gotchas,
        "headroom_stats": stats.get("headroom").cloned().unwrap_or_default(),
    });

    (full_content, json_val)
}

fn copy_to_clipboard(text: &str) -> bool {
    if is_win() {
        if let Ok(mut child) = Command::new("clip").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }
        let res = run_cmd("powershell", &["-NoProfile", "-Command", "Set-Clipboard -Value $input"], &[]);
        res.code == 0
    } else if cfg!(target_os = "macos") {
        if let Ok(mut child) = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                return status.success();
            }
        }
        false
    } else {
        // Linux: wl-copy or xclip
        if let Ok(mut child) = Command::new("wl-copy").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }
        if let Ok(mut child) = Command::new("xclip").args(["-selection", "clipboard"]).stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                return status.success();
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_prime_context() {
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into());
        let (content, json_val) = generate_prime_context(&cwd);
        assert!(content.contains("# Context Primer (Tokless)"));
        assert!(content.contains("Proxy & Cache Performance"));
        assert_eq!(json_val["workspace"], cwd);
    }
}
