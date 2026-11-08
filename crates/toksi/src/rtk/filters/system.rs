// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/filters/system.rs
// created: 2026-10-27
// lastModified: 2026-11-08

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
    let bin = super::normalize_bin_name(parts.first().copied().unwrap_or(""));
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
        "ast-grep" | "sg" => filter_ast_grep(&clean, exit_code),
        "black" => filter_black(&clean, exit_code),
        "json" => filter_json(&clean, sub == "--keys-only" || sub == "schema"),
        "log" => filter_log(&clean),
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

/// Filter ast-grep search matches by grouping and capping lines per file.
pub fn filter_ast_grep(raw: &str, exit_code: i32) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    let mut file_matches: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    let mut non_match_lines = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() == 3 && parts[1].chars().all(|c| c.is_ascii_digit()) {
            let file = parts[0].to_string();
            let entry = format!("{}:{}", parts[1], parts[2]);
            file_matches.entry(file).or_default().push(entry);
        } else {
            non_match_lines.push(line.to_string());
        }
    }

    if file_matches.is_empty() {
        if exit_code != 0 {
            return format!("ast-grep: failed (exit code {})\n{}", exit_code, raw.trim());
        }
        return raw.to_string();
    }

    let mut out = Vec::new();
    for (file, entries) in file_matches {
        let count = entries.len();
        let display_limit = 5;
        for entry in entries.iter().take(display_limit) {
            out.push(format!("{}:{}", file, entry));
        }
        if count > display_limit {
            out.push(format!("  [... +{} more match line(s) in {}]", count - display_limit, file));
        }
    }
    out.join("\n")
}

/// Filter python black formatting output.
pub fn filter_black(raw: &str, exit_code: i32) -> String {
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains("reformatted") || trimmed.contains("unchanged") || trimmed.contains("would be reformatted") {
            summary = trimmed.to_string();
        }
    }

    if exit_code == 0 {
        if !summary.is_empty() {
            format!("black: {}", summary)
        } else {
            "black: all files formatted".to_string()
        }
    } else {
        format!("black: FAILED (exit code {})\n{}", exit_code, raw.trim())
    }
}

/// Filter JSON string into a compact representation or key schema.
pub fn filter_json(raw: &str, schema_only: bool) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Strip leading BOM if present
    let content = trimmed.strip_prefix('\u{feff}').unwrap_or(trimmed);
    let val: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };

    if schema_only {
        extract_json_schema(&val, 0, 5)
    } else {
        compact_json_value(&val, 0, 4)
    }
}

fn extract_json_schema(value: &serde_json::Value, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return "...".to_string();
    }
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "bool".to_string(),
        serde_json::Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "int".to_string()
            } else {
                "float".to_string()
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                format!("string[{}]", s.len())
            } else if s.is_empty() {
                "string".to_string()
            } else if s.starts_with("http://") || s.starts_with("https://") {
                "url".to_string()
            } else {
                "string".to_string()
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", extract_json_schema(&arr[0], depth + 1, max_depth))
            }
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                "{}".to_string()
            } else {
                let mut parts = Vec::new();
                for (i, (k, v)) in map.iter().enumerate() {
                    if i >= 16 {
                        parts.push(format!("...+{}", map.len() - i));
                        break;
                    }
                    parts.push(format!("{}:{}", k, extract_json_schema(v, depth + 1, max_depth)));
                }
                format!("{{{}}}", parts.join(","))
            }
        }
    }
}

fn compact_json_value(value: &serde_json::Value, depth: usize, max_depth: usize) -> String {
    let indent = "  ".repeat(depth);
    if depth > max_depth {
        return format!("{}...", indent);
    }
    match value {
        serde_json::Value::Null => format!("{}null", indent),
        serde_json::Value::Bool(b) => format!("{}{}", indent, b),
        serde_json::Value::Number(n) => format!("{}{}", indent, n),
        serde_json::Value::String(s) => {
            if s.len() > 80 {
                let end = s.char_indices().map(|(i, _)| i).nth(77).unwrap_or(s.len());
                format!("{}\"{}...\"", indent, &s[..end])
            } else {
                format!("{}\"{}\"", indent, s)
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                format!("{}[]", indent)
            } else if arr.len() > 5 {
                let first = compact_json_value(&arr[0], depth + 1, max_depth);
                format!("{}[{}, ... +{} more items]", indent, first.trim(), arr.len() - 1)
            } else {
                let items: Vec<String> = arr.iter().map(|v| compact_json_value(v, depth + 1, max_depth)).collect();
                let all_simple = arr.iter().all(|v| matches!(v, serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) | serde_json::Value::String(_)));
                if all_simple {
                    let inline: Vec<&str> = items.iter().map(|s| s.trim()).collect();
                    format!("{}[{}]", indent, inline.join(", "))
                } else {
                    let mut lines = vec![format!("{}[", indent)];
                    for item in &items {
                        lines.push(format!("{},", item));
                    }
                    lines.push(format!("{}]", indent));
                    lines.join("\n")
                }
            }
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                format!("{}{{}}", indent)
            } else {
                let mut lines = vec![format!("{}{{", indent)];
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                for (i, key) in keys.iter().enumerate() {
                    let val = &map[*key];
                    let is_simple = matches!(val, serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) | serde_json::Value::String(_));
                    if is_simple {
                        let val_str = compact_json_value(val, 0, max_depth);
                        lines.push(format!("{}  {}: {}", indent, key, val_str.trim()));
                    } else {
                        lines.push(format!("{}  {}:", indent, key));
                        lines.push(compact_json_value(val, depth + 1, max_depth));
                    }
                    if i >= 20 {
                        lines.push(format!("{}  ... +{} more keys", indent, keys.len() - i - 1));
                        break;
                    }
                }
                lines.push(format!("{}}}", indent));
                lines.join("\n")
            }
        }
    }
}

/// Filter repeated log lines into deduplicated counts and summary.
pub fn filter_log(raw: &str) -> String {
    use std::collections::HashMap;

    let mut error_counts: HashMap<String, usize> = HashMap::new();
    let mut warn_counts: HashMap<String, usize> = HashMap::new();
    let mut info_count = 0usize;
    let mut unique_errors: Vec<String> = Vec::new();
    let mut unique_warnings: Vec<String> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        let normalized = if let Some(idx) = trimmed.find(|c: char| c.is_alphabetic()) {
            &trimmed[idx..]
        } else {
            trimmed
        };

        if lower.contains("error") || lower.contains("fatal") || lower.contains("panic") || lower.contains("critical") {
            let count = error_counts.entry(normalized.to_string()).or_insert(0);
            if *count == 0 {
                unique_errors.push(trimmed.to_string());
            }
            *count += 1;
        } else if lower.contains("warn") {
            let count = warn_counts.entry(normalized.to_string()).or_insert(0);
            if *count == 0 {
                unique_warnings.push(trimmed.to_string());
            }
            *count += 1;
        } else {
            info_count += 1;
        }
    }

    let total_errors: usize = error_counts.values().sum();
    let total_warns: usize = warn_counts.values().sum();

    if total_errors == 0 && total_warns == 0 {
        return raw.lines().take(20).collect::<Vec<&str>>().join("\n");
    }

    let mut out = Vec::new();
    out.push(format!("Log Summary: {} errors ({} unique), {} warnings ({} unique), {} info lines",
        total_errors, error_counts.len(), total_warns, warn_counts.len(), info_count));

    if !unique_errors.is_empty() {
        out.push(String::new());
        out.push("[ERRORS]".to_string());
        let mut err_list: Vec<_> = error_counts.iter().collect();
        err_list.sort_by(|a, b| b.1.cmp(a.1));
        for (norm, count) in err_list.iter().take(15) {
            let orig = unique_errors.iter().find(|e| e.contains(norm.as_str())).map(|s| s.as_str()).unwrap_or(norm);
            let display_msg = if orig.len() > 120 {
                let end = orig.char_indices().map(|(i, _)| i).nth(117).unwrap_or(orig.len());
                format!("{}...", &orig[..end])
            } else {
                orig.to_string()
            };
            if **count > 1 {
                out.push(format!("  [x{}] {}", count, display_msg));
            } else {
                out.push(format!("  {}", display_msg));
            }
        }
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

    #[test]
    fn test_filter_ast_grep() {
        let raw = "src/lib.rs:10:pub fn foo() {}\nsrc/lib.rs:20:pub fn bar() {}\nsrc/lib.rs:30:pub fn baz() {}\nsrc/lib.rs:40:pub fn qux() {}\nsrc/lib.rs:50:pub fn abc() {}\nsrc/lib.rs:60:pub fn def() {}\n";
        let out = filter_ast_grep(raw, 0);
        assert!(out.contains("src/lib.rs:10:pub fn foo() {}"));
        assert!(out.contains("1 more match line(s) in src/lib.rs"));
    }

    #[test]
    fn test_filter_black() {
        let raw = "reformatted /path/to/script.py\nAll done! ✨ 🍰 ✨\n1 file reformatted, 3 files left unchanged.\n";
        let out = filter_black(raw, 0);
        assert_eq!(out, "black: 1 file reformatted, 3 files left unchanged.");
    }

    #[test]
    fn test_filter_json_compact() {
        let raw = r#"{"name": "toksi", "items": [1, 2, 3, 4, 5, 6, 7], "active": true}"#;
        let out = filter_json(raw, false);
        assert!(out.contains("toksi"));
        assert!(out.contains("+6 more items"));
    }

    #[test]
    fn test_filter_json_schema() {
        let raw = r#"{"name": "toksi", "id": 42, "tags": ["a", "b"]}"#;
        let out = filter_json(raw, true);
        assert!(out.contains("name:string"));
        assert!(out.contains("id:int"));
    }

    #[test]
    fn test_filter_log() {
        let raw = "2026-11-08 12:00:00 [ERROR] Connection refused to db\n2026-11-08 12:00:01 [ERROR] Connection refused to db\n2026-11-08 12:00:02 [WARN] Retrying...";
        let out = filter_log(raw);
        assert!(out.contains("2 errors"));
        assert!(out.contains("[x2]"));
    }
}
