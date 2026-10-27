// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/doctor_stats.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use serde::Deserialize;

use crate::util::colors::C;
use crate::util::exec::{run, RunOptions};
use crate::util::paths::{home, is_win, resolve_codegraph_bin, which};

pub fn human_count(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn human_usd(n: f64) -> String {
    format!("${:.2}", n)
}

pub fn cache_warmth_bar(pct: f64) -> String {
    let clamped = pct.clamp(0.0, 100.0);
    let filled = ((clamped / 10.0).round() as usize).min(10);
    let empty = 10 - filled;
    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
    let est_discount = clamped * 0.9;
    format!("{} {:.1}% (est. ~{:.0}% prompt discount)", bar, clamped, est_discount)
}

#[derive(Debug, Deserialize, Default)]
pub struct LifetimeSavings {
    #[serde(default)]
    pub tokens_saved: i64,
    #[serde(default)]
    pub tokens_before: i64,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub calls: i64,
    #[serde(default, rename = "savings_percent")]
    pub percent: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct WindowSavings {
    #[serde(default)]
    pub tokens_saved: i64,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub calls: i64,
    #[serde(default, rename = "savings_percent")]
    pub percent: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct SavingsData {
    #[serde(default)]
    pub lifetime: LifetimeSavings,
    #[serde(default)]
    pub windows: HashMap<String, WindowSavings>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TransformUsage {
    #[serde(default)]
    pub transform: String,
    #[serde(default)]
    pub uses: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct PerfData {
    #[serde(default, rename = "cache_hit_pct")]
    pub cache_hit: f64,
    #[serde(default, rename = "total_requests")]
    pub requests: i64,
    #[serde(default, rename = "total_tokens_saved")]
    pub tokens_saved: i64,
    #[serde(default)]
    pub tool_saved: i64,
    #[serde(default, rename = "requests")]
    pub total_requests: i64,
    #[serde(default, rename = "tokens_saved")]
    pub alt_tokens_saved: i64,
    #[serde(default, rename = "by_transform")]
    pub transforms: Vec<TransformUsage>,
}

pub fn parse_savings(raw: &str) -> Option<SavingsData> {
    let start = raw.find('{')?;
    serde_json::from_str(&raw[start..]).ok()
}

pub fn parse_perf(raw: &str) -> Option<PerfData> {
    let start = raw.find('{')?;
    serde_json::from_str(&raw[start..]).ok()
}

pub fn run_doctor_command(timeout: Duration, bin: &str, args: &[&str]) -> Option<String> {
    let res = run(
        bin,
        args,
        RunOptions {
            capture: true,
            quiet: true,
            timeout: Some(timeout),
            ..Default::default()
        },
    );
    if res.code == 0 || !res.stdout.trim().is_empty() {
        Some(res.stdout.trim().to_string())
    } else {
        None
    }
}

pub fn headroom_stats_rows(savings_raw: &str, perf_raw: &str) -> Vec<String> {
    let Some(data) = parse_savings(savings_raw) else {
        return vec![C.dim("unavailable")];
    };
    let mut rows = vec![format!(
        "lifetime: {} saved, {}, {} calls, {:.1}%",
        human_count(data.lifetime.tokens_saved),
        human_usd(data.lifetime.cost_usd),
        human_count(data.lifetime.calls),
        data.lifetime.percent
    )];

    let windows = [("today", "today"), ("last_7_days", "7d"), ("last_30_days", "30d")];
    for (k, label) in windows {
        if let Some(w) = data.windows.get(k) {
            rows.push(format!(
                "{}: {} saved, {}, {} calls",
                label,
                human_count(w.tokens_saved),
                human_usd(w.cost_usd),
                human_count(w.calls)
            ));
        }
    }

    if !perf_raw.is_empty() {
        if let Some(perf) = parse_perf(perf_raw) {
            let mut req = perf.requests;
            if req == 0 {
                req = perf.total_requests;
            }
            let mut saved = perf.tokens_saved;
            if saved == 0 {
                saved = perf.alt_tokens_saved;
            }
            rows.push(format!(
                "perf: {} requests, cache hit {:.1}%, {} tokens saved ({} tool saved)",
                human_count(req),
                perf.cache_hit,
                human_count(saved),
                human_count(perf.tool_saved)
            ));
            rows.push(format!("warmth: {}", cache_warmth_bar(perf.cache_hit)));
            if let Some(first_t) = perf.transforms.first() {
                rows.push(format!("transform {}: {} uses", first_t.transform, human_count(first_t.uses)));
            }
        }
    }
    rows.push("per-port stats unavailable: headroom has no port dimension".into());
    rows
}

pub fn headroom_stats_summary() -> Vec<String> {
    let bin = which("headroom");
    if bin.is_empty() {
        return vec![C.dim("unavailable")];
    }
    let Some(savings_raw) = run_doctor_command(Duration::from_secs(10), &bin, &["savings", "--json"]) else {
        return vec![C.dim("unavailable")];
    };
    let perf_raw = run_doctor_command(
        Duration::from_secs(10),
        &bin,
        &["perf", "--format", "json", "--hours", "720"],
    )
    .unwrap_or_default();
    headroom_stats_rows(&savings_raw, &perf_raw)
}

#[derive(Debug, Deserialize, Default)]
struct ProjectmemStatsData {
    #[serde(default)]
    score: i32,
    #[serde(default)]
    grade: String,
    #[serde(default, rename = "tokens_saved")]
    tokens: i64,
    #[serde(default, rename = "usd_saved")]
    usd: f64,
    #[serde(default, rename = "debugging_hours_saved")]
    hours: f64,
    #[serde(default)]
    components: HashMap<String, i64>,
    #[serde(default)]
    capture: ProjectmemCaptureData,
}

#[derive(Debug, Deserialize, Default)]
struct ProjectmemCaptureData {
    #[serde(default, rename = "auto_captured")]
    auto: i64,
    #[serde(default)]
    manual: i64,
    #[serde(default)]
    total: i64,
    #[serde(default, rename = "auto_rate")]
    rate: f64,
}

pub fn projectmem_stats(raw: &str) -> Vec<String> {
    let start = match raw.find('{') {
        Some(idx) => &raw[idx..],
        None => return vec!["unavailable — no recognized stats output".into()],
    };
    let Ok(d) = serde_json::from_str::<ProjectmemStatsData>(start) else {
        return vec!["unavailable — no recognized stats output".into()];
    };
    if d.grade.is_empty() {
        return vec!["unavailable — no recognized stats output".into()];
    }
    vec![
        format!("Score: {}/100 ({})", d.score, d.grade),
        format!(
            "Saved: {} tokens / {} / {:.1} debugging hours",
            human_count(d.tokens),
            human_usd(d.usd),
            d.hours
        ),
        format!(
            "Capture: {} auto / {} manual / {} total ({:.1}% auto)",
            human_count(d.capture.auto),
            human_count(d.capture.manual),
            human_count(d.capture.total),
            d.capture.rate
        ),
        format!(
            "Knowledge: {} failed / {} successful / {} decisions / {} fixes / {} notes / {} gotcha files / {} high-churn",
            d.components.get("failed_approaches").copied().unwrap_or(0),
            d.components.get("successful_attempts").copied().unwrap_or(0),
            d.components.get("decisions_documented").copied().unwrap_or(0),
            d.components.get("fixes_with_context").copied().unwrap_or(0),
            d.components.get("notes_count").copied().unwrap_or(0),
            d.components.get("files_with_gotchas").copied().unwrap_or(0),
            d.components.get("high_churn_files").copied().unwrap_or(0),
        ),
    ]
}

#[derive(Debug, Deserialize, Default)]
struct CodegraphPending {
    #[serde(default)]
    added: i32,
    #[serde(default)]
    modified: i32,
    #[serde(default)]
    removed: i32,
}

#[derive(Debug, Deserialize, Default)]
struct CodegraphStatsData {
    #[serde(default, rename = "fileCount")]
    files: i32,
    #[serde(default, rename = "nodeCount")]
    nodes: i64,
    #[serde(default, rename = "edgeCount")]
    edges: i64,
    #[serde(default, rename = "dbSizeBytes")]
    db: i64,
    #[serde(default, rename = "walSizeBytes")]
    wal: i64,
    #[serde(default, rename = "lastIndexed")]
    last: String,
    #[serde(default, rename = "pendingChanges")]
    pending: CodegraphPending,
    #[serde(default, rename = "nodesByKind")]
    kinds: HashMap<String, i32>,
    #[serde(default)]
    languages: Vec<String>,
}

pub fn codegraph_stats(raw: &str) -> Vec<String> {
    let start = match raw.find('{') {
        Some(idx) => &raw[idx..],
        None => return vec!["unavailable — no recognized stats output".into()],
    };
    let Ok(d) = serde_json::from_str::<CodegraphStatsData>(start) else {
        return vec!["unavailable — no recognized stats output".into()];
    };
    if d.files == 0 {
        return vec!["unavailable — no recognized stats output".into()];
    }

    let mut rows = vec![
        format!(
            "Files {} / Nodes {} / Edges {}",
            d.files,
            human_count(d.nodes),
            human_count(d.edges)
        ),
        format!(
            "DB {:.2} MB / WAL {:.2} MB",
            d.db as f64 / 1_048_576.0,
            d.wal as f64 / 1_048_576.0
        ),
        format!("Indexed: {}", d.last),
        format!(
            "Pending: +{} ~{} -{}",
            d.pending.added, d.pending.modified, d.pending.removed
        ),
    ];

    if !d.kinds.is_empty() {
        let mut sorted: Vec<(&String, &i32)> = d.kinds.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let kinds_str = sorted
            .into_iter()
            .take(3)
            .map(|(k, v)| format!("{} {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(format!("Kinds: {}", kinds_str));
    }
    if !d.languages.is_empty() {
        let langs_str = d.languages.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        rows.push(format!("Languages: {}", langs_str));
    }

    rows
}

pub fn rtk_stats(raw: &str) -> Vec<String> {
    let mut rows = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with("Total commands:")
            || t.starts_with("Tokens saved:")
            || t.starts_with("Efficiency meter:")
            || t.starts_with("Input tokens:")
            || t.starts_with("Output tokens:")
            || t.starts_with("Total exec time:")
            || t.starts_with("Excluded:")
        {
            rows.push(t.to_string());
        }
    }
    if rows.is_empty() {
        vec!["unavailable — no recognized stats output".into()]
    } else {
        rows
    }
}

pub fn context_mode_store_stats(home_dir: &str) -> String {
    let mut root = Path::new(home_dir)
        .join(".config")
        .join("opencode")
        .join("context-mode");
    if is_win() {
        if let Ok(v) = std::env::var("APPDATA") {
            if !v.is_empty() {
                root = Path::new(&v).join("context-mode");
            }
        }
    } else if let Ok(v) = std::env::var("XDG_DATA_HOME") {
        if !v.is_empty() {
            root = Path::new(&v).join("context-mode");
        }
    }

    let mut count = 0;
    let mut size: u64 = 0;
    for dir_name in ["sessions", "content"] {
        let dir = root.join(dir_name);
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        if name.ends_with(".db") {
                            count += 1;
                            size += meta.len();
                        }
                    }
                }
            }
        }
    }

    if count > 0 {
        format!("store {} DBs, {:.1} MB", count, size as f64 / (1024.0 * 1024.0))
    } else {
        "no indexed data".into()
    }
}

pub fn context_mode_stats(home_dir: &str) -> Vec<String> {
    vec![context_mode_store_stats(home_dir)]
}

pub fn labelled_stats(tool: &str, out: &str) -> Vec<String> {
    match tool {
        "projectmem" => projectmem_stats(out),
        "codegraph" => codegraph_stats(out),
        "context-mode" => vec![out.to_string()],
        "rtk" => rtk_stats(out),
        _ => vec!["unavailable".into()],
    }
}

pub fn doctor_tool_stats(project_dir: &str, is_project: bool) -> HashMap<String, Vec<String>> {
    let mut stats = HashMap::new();
    let tools = ["headroom", "projectmem", "rtk", "codegraph", "context-mode"];

    for tool in tools {
        if tool == "context-mode" {
            if !which("context-mode").is_empty() {
                stats.insert(tool.to_string(), context_mode_stats(&home()));
            }
            continue;
        }

        let bin = if tool == "codegraph" {
            resolve_codegraph_bin()
        } else {
            which(tool)
        };

        if bin.is_empty() {
            continue;
        }

        let args: Vec<&str> = match tool {
            "headroom" => {
                stats.insert(tool.to_string(), headroom_stats_summary());
                continue;
            }
            "projectmem" => vec!["stats", "--format", "json"],
            "rtk" => vec!["gain"],
            "codegraph" => {
                if !is_project {
                    continue;
                }
                vec!["status", "--json", project_dir]
            }
            _ => continue,
        };

        if let Some(out) = run_doctor_command(Duration::from_secs(4), &bin, &args) {
            stats.insert(tool.to_string(), labelled_stats(tool, &out));
        } else {
            stats.insert(tool.to_string(), vec!["unavailable".into()]);
        }
    }

    stats
}

pub fn spawn_background(mut cmd: std::process::Command) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }
    cmd.spawn().map(|_| ())
}

#[cfg(windows)]
pub fn process_alive(pid: u32) -> bool {
    crate::agents::proxy_provider::proxy_process_alive(pid)
}

pub fn run_gain() -> i32 {
    crate::commands::helpers::cmd_header("gain", "token savings dashboard across tools");

    let rtk_bin = which("rtk");
    if !rtk_bin.is_empty() {
        crate::util::progress::tree_top("RTK Savings");
        if let Some(out) = run_doctor_command(Duration::from_secs(4), &rtk_bin, &["gain"]) {
            for line in out.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("[warn]") {
                    crate::util::progress::tree_leaf(trimmed);
                }
            }
        } else {
            crate::util::progress::tree_leaf(&C.dim("no tracking data yet"));
        }
        crate::util::progress::tree_close();
    }

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into());
    let stats = doctor_tool_stats(&cwd, true);

    if let Some(hr) = stats.get("headroom") {
        crate::util::progress::tree_corner("Headroom Cache");
        for row in hr {
            crate::util::progress::tree_leaf(row);
        }
        crate::util::progress::tree_close();
    }

    if let Some(pjm) = stats.get("projectmem") {
        crate::util::progress::tree_corner("ProjectMem");
        for row in pjm {
            crate::util::progress::tree_leaf(row);
        }
        crate::util::progress::tree_close();
    }

    crate::commands::helpers::print_repo_footer(true);
    0
}

#[cfg(unix)]
pub fn process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_count() {
        assert_eq!(human_count(500), "500");
        assert_eq!(human_count(1500), "1.5K");
        assert_eq!(human_count(2_500_000), "2.5M");
    }

    #[test]
    fn test_human_usd() {
        assert_eq!(human_usd(12.345), "$12.35");
        assert_eq!(human_usd(0.0), "$0.00");
    }

    #[test]
    fn test_parse_savings() {
        let json = r#"{"lifetime":{"tokens_saved":1000,"cost_usd":0.05,"calls":10,"savings_percent":25.0}}"#;
        let data = parse_savings(json).expect("parse savings");
        assert_eq!(data.lifetime.tokens_saved, 1000);
        assert_eq!(data.lifetime.calls, 10);
    }

    #[test]
    fn test_projectmem_stats_parse() {
        let raw = r#"{"score":85,"grade":"A","tokens_saved":50000,"usd_saved":1.25,"debugging_hours_saved":3.5,"components":{"failed_approaches":2,"successful_attempts":5,"decisions_documented":3,"fixes_with_context":4,"notes_count":10,"files_with_gotchas":1,"high_churn_files":0},"capture":{"auto_captured":100,"manual":20,"total":120,"auto_rate":83.3}}"#;
        let rows = projectmem_stats(raw);
        assert_eq!(rows.len(), 4);
        assert!(rows[0].contains("85/100 (A)"));
    }

    #[test]
    fn test_cache_warmth_bar() {
        let bar_half = cache_warmth_bar(50.0);
        assert!(bar_half.contains("[█████░░░░░]"));
        assert!(bar_half.contains("50.0%"));
        assert!(bar_half.contains("45% prompt discount"));

        let bar_full = cache_warmth_bar(100.0);
        assert!(bar_full.contains("[██████████]"));
        assert!(bar_full.contains("100.0%"));
    }
}
