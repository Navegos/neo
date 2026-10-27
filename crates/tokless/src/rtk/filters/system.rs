// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/filters/system.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use crate::rtk::ansi::strip_ansi;

/// Filter system tool outputs.
pub fn filter_system(cmd: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{}\n{}", stdout, stderr)
    };
    let clean = strip_ansi(&combined);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");
    let sub = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match bin {
        "docker" | "podman" => filter_docker(&clean, sub, exit_code),
        "kubectl" => filter_kubectl(&clean, sub, exit_code),
        "curl" | "wget" => filter_curl_wget(&clean),
        "ls" | "dir" => filter_ls(&clean),
        "tree" => filter_tree(&clean),
        "cat" | "bat" | "head" | "tail" => filter_file_read(&clean, 100),
        _ => clean,
    }
}

/// Filter `docker ps` and container outputs into compact listings.
pub fn filter_docker(raw: &str, subcommand: &str, exit_code: i32) -> String {
    if subcommand == "ps" {
        let mut out = Vec::new();
        let mut lines = raw.lines();
        if let Some(header) = lines.next() {
            out.push(header.to_string());
        }
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let fields: Vec<&str> = trimmed.split_whitespace().collect();
            if fields.len() >= 4 {
                // Short ID (12 chars), image, status, name
                let id = fields[0];
                let image = fields[1];
                let name = fields.last().copied().unwrap_or("");
                out.push(format!("{:<12}  {:<20}  {}", id, image, name));
            } else {
                out.push(trimmed.to_string());
            }
        }
        if out.len() <= 1 {
            return "docker ps: 0 running containers".to_string();
        }
        return out.join("\n");
    }

    if exit_code != 0 {
        raw.lines().take(20).collect::<Vec<&str>>().join("\n")
    } else {
        let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            format!("docker {}: ok", subcommand)
        } else {
            lines.into_iter().take(25).collect::<Vec<&str>>().join("\n")
        }
    }
}

/// Filter `kubectl get` outputs.
pub fn filter_kubectl(raw: &str, subcommand: &str, exit_code: i32) -> String {
    if subcommand == "get" {
        let mut lines = raw.lines();
        let header = lines.next().unwrap_or("");
        let mut entries = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                entries.push(trimmed);
            }
        }

        if entries.is_empty() {
            return "No resources found".to_string();
        }

        let mut out = vec![header.to_string()];
        out.extend(entries.into_iter().map(String::from));
        return out.join("\n");
    }

    if exit_code != 0 {
        raw.lines().take(20).collect::<Vec<&str>>().join("\n")
    } else {
        raw.lines().take(30).collect::<Vec<&str>>().join("\n")
    }
}

/// Filter `curl` and `wget` outputs:
/// Strips progress meters `[####...]`, percentages, ETA speedometers.
pub fn filter_curl_wget(raw: &str) -> String {
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("% Total")
            || trimmed.starts_with("Dload  Upload")
            || trimmed.contains("--:--:--")
            || (trimmed.starts_with('[') && trimmed.contains('%') && trimmed.contains("/s"))
            || trimmed.starts_with("Connecting to ")
            || trimmed.starts_with("Resolving ")
        {
            continue;
        }
        out.push(line);
    }
    out.join("\n").trim().to_string()
}

/// Filter `ls` outputs: compact directory and file listings.
pub fn filter_ls(raw: &str) -> String {
    let mut items: Vec<&str> = raw.split_whitespace().collect();
    if items.len() > 100 {
        let omitted = items.len() - 100;
        items.truncate(100);
        let mut res = items.join("  ");
        res.push_str(&format!("  [... +{} more files]", omitted));
        res
    } else {
        items.join("  ")
    }
}

/// Filter `tree` outputs: ignores noise dirs and caps depth.
pub fn filter_tree(raw: &str) -> String {
    let mut out = Vec::new();
    for line in raw.lines() {
        if line.contains("node_modules") || line.contains("target/") || line.contains(".git/") {
            continue;
        }
        out.push(line);
        if out.len() >= 80 {
            out.push("[... tree capped at 80 entries]");
            break;
        }
    }
    out.join("\n")
}

/// Smart file read output with line truncation.
pub fn filter_file_read(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }

    let mut out: Vec<String> = Vec::with_capacity(max_lines + 1);
    for line in lines.iter().take(max_lines) {
        out.push((*line).to_string());
    }
    out.push(format!("[... {} more lines]", lines.len() - max_lines));
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_curl_strips_meter() {
        let raw = "  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current\n                                 Dload  Upload   Total   Spent    Left  Speed\n100   123  100   123    0     0   1200      0 --:--:-- --:--:-- --:--:--  1205\n{\"status\":\"ok\"}\n";
        let out = filter_curl_wget(raw);
        assert_eq!(out, "{\"status\":\"ok\"}");
    }

    #[test]
    fn test_filter_file_read_capping() {
        let content = (0..50).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        let out = filter_file_read(&content, 10);
        assert!(out.contains("[... 40 more lines]"));
    }
}
