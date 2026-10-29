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
        "find" => filter_find(&clean, exit_code),
        "wc" => filter_wc(cmd, &clean, exit_code),
        "env" => filter_env(&clean),
        "psql" => filter_psql(&clean, exit_code),
        "aws" => filter_aws(cmd, &clean, exit_code),
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

/// Filter `find` command output, grouping by directory and dropping permission errors.
pub fn filter_find(raw: &str, _exit_code: i32) -> String {
    let mut files = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.contains("Permission denied") || trimmed.contains("Operation not permitted") {
            continue;
        }
        files.push(trimmed);
    }

    if files.is_empty() {
        return "No files found".to_string();
    }

    if files.len() <= 60 {
        return files.join("\n");
    }

    let mut out: Vec<String> = files.iter().take(60).map(|s| (*s).to_string()).collect();
    out.push(format!("... [{} more files omitted by rtk]", files.len() - 60));
    out.join("\n")
}

/// Filter `wc` word count outputs into compact notation.
pub fn filter_wc(cmd: &str, raw: &str, _exit_code: i32) -> String {
    let lines: Vec<&str> = raw.trim().lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let is_lines_only = cmd.contains("-l");
    let is_words_only = cmd.contains("-w");
    let is_bytes_only = cmd.contains("-c");

    if lines.len() == 1 {
        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if is_lines_only || is_words_only || is_bytes_only {
            return parts.first().copied().unwrap_or("0").to_string();
        }
        if parts.len() >= 3 {
            return format!("{}L {}W {}B", parts[0], parts[1], parts[2]);
        }
        return lines[0].trim().to_string();
    }

    let mut out = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let label = parts[3..].join(" ");
            out.push(format!("{:<6} {:<6} {:<8} {}", parts[0], parts[1], parts[2], label));
        } else {
            out.push(line.trim().to_string());
        }
    }
    out.join("\n")
}

/// Filter `env` command output, masking sensitive tokens and formatting compactly.
pub fn filter_env(raw: &str) -> String {
    let mut vars = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            let k_upper = k.to_uppercase();
            let val = if k_upper.contains("KEY")
                || k_upper.contains("SECRET")
                || k_upper.contains("TOKEN")
                || k_upper.contains("PASSWORD")
                || k_upper.contains("AUTH")
            {
                "***REDACTED***"
            } else if v.len() > 80 {
                &v[..80]
            } else {
                v
            };
            vars.push(format!("{}={}", k, val));
        } else {
            vars.push(trimmed.to_string());
        }
    }

    vars.sort();
    vars.join("\n")
}

/// Filter PostgreSQL `psql` outputs.
pub fn filter_psql(raw: &str, _exit_code: i32) -> String {
    let mut out = Vec::new();
    let mut data_rows = 0;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip separator lines like +---+---+ or ----+----
        if trimmed.chars().all(|c| c == '-' || c == '+' || c == '|' || c == ' ') {
            continue;
        }
        // Skip row count footer e.g. (10 rows)
        if trimmed.starts_with('(') && trimmed.ends_with("rows)") {
            continue;
        }

        if trimmed.contains('|') {
            data_rows += 1;
            if data_rows <= 50 {
                let cols: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
                out.push(cols.join("\t"));
            }
        } else {
            out.push(trimmed.to_string());
        }
    }

    if data_rows > 50 {
        out.push(format!("... [{} more rows omitted by rtk]", data_rows - 50));
    }

    out.join("\n")
}

/// Filter AWS CLI outputs.
pub fn filter_aws(_cmd: &str, raw: &str, _exit_code: i32) -> String {
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"NextToken\":") || trimmed.contains("\"PaginationToken\":") {
            continue;
        }
        out.push(line);
    }
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

    #[test]
    fn test_filter_psql() {
        let raw = " id | name  | role  \n----+-------+-------\n  1 | Alice | admin \n  2 | Bob   | user  \n(2 rows)\n";
        let out = filter_psql(raw, 0);
        assert_eq!(out, "id\tname\trole\n1\tAlice\tadmin\n2\tBob\tuser");
    }

    #[test]
    fn test_filter_wc() {
        let raw = "  12   34  567 foo.rs\n";
        let out = filter_wc("wc foo.rs", raw, 0);
        assert_eq!(out, "12L 34W 567B");

        let out_l = filter_wc("wc -l foo.rs", raw, 0);
        assert_eq!(out_l, "12");
    }
}
