// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/doctor_probe.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use regex::Regex;
use serde_json::Value;

use crate::agents::{omp_agent_dir_resolved, pi_agent_dir_resolved};
use crate::commands::mcp_proxy::resolve_mcp_command;
use crate::util::jsonc::try_parse_jsonc;
use crate::util::mcpspawn::McpSpawn;
use crate::util::paths::{
    antigravity_paths_resolved, claude_code_paths, cline_paths_resolved,
    codex_paths_resolved, copilot_paths_resolved, cursor_paths_resolved, home,
    kilo_paths_resolved, opencode_paths_resolved, read_file_safe,
    split_command, which, IS_WIN,
};

fn path_base_lowercase(s: &str) -> String {
    let replaced = s.replace('\\', "/");
    let p = Path::new(&replaced);
    p.file_name().unwrap_or_default().to_string_lossy().to_lowercase()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIssue {
    pub kind: String,
    pub detail: String,
}

pub struct RuntimeFile {
    pub path: String,
    pub markers: Vec<String>,
}

pub fn probe_agent_runtime(agent_id: &str) -> Vec<RuntimeIssue> {
    let mut out = Vec::new();
    for cmd in managed_hook_commands(agent_id) {
        if let Some(detail) = probe_hook_command(&cmd) {
            out.push(RuntimeIssue { kind: "hook".into(), detail });
        }
    }
    for file in managed_runtime_files(agent_id) {
        if let Some(detail) = probe_runtime_file(&file) {
            out.push(RuntimeIssue { kind: "hook".into(), detail });
        }
    }
    for spawn in managed_mcp_spawns(agent_id) {
        if let Some(detail) = probe_mcp_spawn(&spawn.command, &spawn.args) {
            out.push(RuntimeIssue { kind: "mcp".into(), detail });
        }
    }
    out
}

pub fn probe_runtime_file(file: &RuntimeFile) -> Option<String> {
    let raw = match read_file_safe(&file.path) {
        Some(r) => r,
        None => return None,
    };
    for marker in &file.markers {
        if !raw.contains(marker) {
            return Some(format!("RTK integration file invalid: {}", short_path(&file.path)));
        }
    }
    None
}

pub fn probe_hook_command(command: &str) -> Option<String> {
    let command = command.trim();
    if command.is_empty() {
        return Some("empty hook command".into());
    }
    let fields = split_command(command);
    if fields.is_empty() {
        return Some("empty hook command".into());
    }
    let exe = &fields[0];
    if is_tokless_managed_hook(&fields) && windows_bash_hostile_path(exe) {
        return Some(format!("backslash path breaks Git Bash hooks — re-run tokless ({})", short_path(exe)));
    }
    if let Err(_) = resolve_runnable(exe) {
        return Some(format!("hook binary not found: {}", short_path(exe)));
    }
    if !is_tokless_managed_hook(&fields) {
        return None;
    }
    live_hook(&fields)
}

pub fn windows_bash_hostile_path(exe: &str) -> bool {
    if !IS_WIN {
        return false;
    }
    if exe.contains('/') {
        return false;
    }
    if exe.len() >= 3 && exe.as_bytes()[1] == b':' && exe.as_bytes()[2] == b'\\' {
        return true;
    }
    exe.starts_with(r"\\")
}

pub fn is_tokless_managed_hook(fields: &[String]) -> bool {
    if fields.len() < 2 {
        return false;
    }
    let base = path_base_lowercase(&fields[0]);
    if base != "tokless" && base != "tokless.exe" {
        return false;
    }
    matches!(
        fields[1].as_str(),
        "rtk-hook" | "codex-perm" | "agy-hook" | "copilot-hook" | "index"
    )
}

fn run_output_with_timeout(cmd: &mut Command, timeout: Duration) -> io::Result<std::process::Output> {
    let mut child = cmd.spawn()?;
    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_) => return child.wait_with_output(),
            None => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "command timed out"));
                }
                std::thread::sleep(Duration::from_millis(30));
            }
        }
    }
}

pub fn live_hook(fields: &[String]) -> Option<String> {
    let exe = match resolve_runnable(&fields[0]) {
        Ok(e) => e,
        Err(_) => return Some(format!("hook not runnable: {}", short_path(&fields[0]))),
    };
    let mut cmd = Command::new(&exe);
    if fields.len() > 1 {
        cmd.args(&fields[1..]);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match run_output_with_timeout(&mut cmd, Duration::from_secs(3)) {
        Ok(out) => {
            if !out.status.success() {
                let msg = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
                if is_spawn_fail(&msg) {
                    return Some(format!("hook not runnable: {}", short_path(&fields[0])));
                }
            }
            None
        }
        Err(e) if e.kind() == io::ErrorKind::TimedOut => None,
        Err(_) => Some(format!("hook not runnable: {}", short_path(&fields[0]))),
    }
}

pub fn probe_mcp_spawn(command: &str, args: &[String]) -> Option<String> {
    let command = command.trim();
    if command.is_empty() {
        return Some("empty mcp command".into());
    }
    if let Err(_) = resolve_runnable(command) {
        return Some(format!("mcp binary not found: {}", short_path(command)));
    }
    let (inner_cmd, inner_args) = unwrap_run_mcp(command, args);
    if !is_cmd_name(&inner_cmd) {
        if let Err(_) = resolve_runnable(&inner_cmd) {
            if inner_cmd.contains('/') || inner_cmd.contains('\\') || inner_cmd.contains(':') {
                return Some(format!("mcp target not found: {}", short_path(&inner_cmd)));
            }
        }
    }
    let mut argv = vec![inner_cmd.clone()];
    argv.extend(inner_args);
    let mut path = inner_cmd;
    let lp = which(&path);
    if !lp.is_empty() {
        path = lp;
    }
    let (exe, norm_args) = resolve_mcp_command(&path, &argv);
    let base_path = Path::new(&path.replace('\\', "/")).file_name().unwrap_or_default().to_string_lossy().to_string();
    if is_cmd_name(&exe) || is_cmd_name(&base_path) {
        return probe_cmd_batch(&exe, &norm_args);
    }
    if let Err(_) = resolve_runnable(&exe) {
        return Some(format!("mcp binary not found: {}", short_path(&exe)));
    }
    if looks_like_codegraph(&exe, &norm_args) {
        return live_codegraph_version(&exe, &norm_args);
    }
    None
}

pub fn unwrap_run_mcp(command: &str, args: &[String]) -> (String, Vec<String>) {
    let base = path_base_lowercase(command);
    if base != "tokless" && base != "tokless.exe" {
        return (command.to_string(), args.to_vec());
    }
    if args.is_empty() || args[0] != "run-mcp" {
        return (command.to_string(), args.to_vec());
    }
    let mut current_args = args[1..].to_vec();
    while !current_args.is_empty() {
        match current_args[0].as_str() {
            "--agent" | "--workspace" | "--tool" => {
                if current_args.len() < 2 || current_args[1].is_empty() || current_args[1].starts_with("--") {
                    return (command.to_string(), args.to_vec());
                }
                current_args = current_args[2..].to_vec();
            }
            "--context-mode" => {
                current_args = current_args[1..].to_vec();
            }
            _ => {
                let first = current_args[0].clone();
                let rest = current_args[1..].to_vec();
                return (first, rest);
            }
        }
    }
    (command.to_string(), args.to_vec())
}

pub fn probe_cmd_batch(exe: &str, args: &[String]) -> Option<String> {
    if args.len() < 2 || !args[0].eq_ignore_ascii_case("/c") {
        return None;
    }
    let batch = &args[1];
    if !file_exists_any(batch) {
        return Some(format!("mcp batch not found: {}", short_path(batch)));
    }
    if !looks_like_codegraph(batch, &args[2..]) {
        return None;
    }
    live_cmd_batch_version(exe, batch)
}

pub fn live_cmd_batch_version(comspec: &str, batch: &str) -> Option<String> {
    let comspec = if comspec.is_empty() || is_cmd_name(comspec) {
        let p = which("cmd.exe");
        if !p.is_empty() {
            p
        } else {
            let p2 = which("cmd");
            if !p2.is_empty() { p2 } else { "cmd".to_string() }
        }
    } else {
        comspec.to_string()
    };
    let mut cmd = Command::new(&comspec);
    cmd.args(["/c", batch, "--version"]);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    if let Ok(output) = run_output_with_timeout(&mut cmd, Duration::from_secs(5)) {
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        if text.contains("is not recognized") {
            return Some(format!("mcp cmd /c path broken — re-run tokless ({})", short_path(batch)));
        }
        if !output.status.success() && is_spawn_fail(&text) {
            return Some(format!("mcp batch not runnable: {}", short_path(batch)));
        }
    }
    None
}

pub fn live_codegraph_version(exe: &str, _args: &[String]) -> Option<String> {
    let mut cmd = Command::new(exe);
    cmd.arg("--version");
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    if let Ok(output) = run_output_with_timeout(&mut cmd, Duration::from_secs(5)) {
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        if !output.status.success() && is_spawn_fail(&text) {
            return Some(format!("mcp not runnable: {}", short_path(exe)));
        }
    }
    None
}

pub fn looks_like_codegraph(path: &str, args: &[String]) -> bool {
    let check = |s: &str| {
        let mut base = path_base_lowercase(s);
        for ext in &[".cmd", ".bat", ".exe"] {
            if base.ends_with(ext) {
                base.truncate(base.len() - ext.len());
            }
        }
        base == "codegraph"
    };
    if check(path) {
        return true;
    }
    args.iter().any(|a| check(a))
}

pub fn resolve_runnable(exe: &str) -> Result<String, String> {
    look_exe(exe).map(|(p, _)| p)
}

pub fn look_exe(exe: &str) -> Result<(String, String), String> {
    if exe.is_empty() {
        return Err("empty".into());
    }
    if exe.contains('/') || exe.contains('\\') || Path::new(exe).is_absolute() || (exe.len() >= 2 && exe.as_bytes()[1] == b':') {
        for cand in path_candidates(exe) {
            let p = Path::new(&cand);
            if p.is_file() {
                return Ok((cand, "".into()));
            }
        }
        return Err("not found".into());
    }
    let p = which(exe);
    if !p.is_empty() {
        return Ok((p, "".into()));
    }
    Err("not on PATH".into())
}

pub fn path_candidates(exe: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    let mut add = |s: String| {
        if !s.is_empty() && seen.insert(s.clone()) {
            out.push(s);
        }
    };
    add(exe.to_string());
    if let Ok(clean) = Path::new(exe).canonicalize() {
        add(clean.to_string_lossy().to_string());
    }
    add(exe.replace('/', "\\"));
    add(exe.replace('\\', "/"));
    if IS_WIN {
        let forward = exe.replace('\\', "/");
        add(forward.replace('/', "\\"));
    }
    out
}

pub fn file_exists_any(p: &str) -> bool {
    for cand in path_candidates(p) {
        if Path::new(&cand).is_file() {
            return true;
        }
    }
    false
}

pub fn is_cmd_name(s: &str) -> bool {
    let base = path_base_lowercase(s);
    base == "cmd" || base == "cmd.exe"
}

pub fn is_spawn_fail(msg: &str) -> bool {
    let low = msg.to_lowercase();
    low.contains("executable file not found")
        || low.contains("no such file")
        || low.contains("cannot find the file")
        || low.contains("the system cannot find")
}

pub fn short_path(p: &str) -> String {
    if p.chars().count() <= 64 {
        return p.to_string();
    }
    let chars: Vec<char> = p.chars().collect();
    let prefix: String = chars[..28].iter().collect();
    let suffix: String = chars[chars.len() - 28..].iter().collect();
    format!("{}…{}", prefix, suffix)
}

pub fn format_runtime_issues(issues: &[RuntimeIssue]) -> String {
    if issues.is_empty() {
        return String::new();
    }
    let mut seen = std::collections::HashSet::new();
    let mut parts = Vec::new();
    for is in issues {
        if seen.insert(is.detail.clone()) {
            parts.push(is.detail.clone());
        }
    }
    parts.join(", ")
}

fn ide_root_for_doctor() -> String {
    env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
}

fn read_file_or_empty(p: &str) -> String {
    read_file_safe(p).unwrap_or_default()
}

fn extract_cline_hook_command(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("exec ") {
            if let Some(i) = line.find(" rtk-hook cline") {
                let exe = line["exec ".len()..i].trim().trim_matches(|c| c == '\'' || c == '"');
                if !exe.is_empty() {
                    return Some(format!("{} rtk-hook cline", exe));
                }
            }
        }
        const SPAWN_PREFIX: &str = "spawnSync(\"";
        if let Some(i) = line.find(SPAWN_PREFIX) {
            let start = i + SPAWN_PREFIX.len();
            if let Some(end) = line[start..].find("\", [\"rtk-hook\", \"cline\"]") {
                let exe = &line[start..start + end];
                if !exe.is_empty() {
                    return Some(format!("{} rtk-hook cline", exe));
                }
            }
        }
    }
    None
}

fn extract_json_commands(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    let re = Regex::new(r#""command"\s*:\s*"((?:\\.|[^"\\])*)""#).unwrap();
    let mut out = Vec::new();
    for cap in re.captures_iter(raw) {
        if let Some(m) = cap.get(1) {
            out.push(json_unescape(m.as_str()));
        }
    }
    out
}

fn json_unescape(s: &str) -> String {
    s.replace(r"\\", r"\").replace(r#"\""#, r#"""#).replace(r"\/", "/")
}

fn toml_unescape(s: &str) -> String {
    s.replace(r"\\", r"\").replace(r#"\""#, r#"""#)
}

fn managed_hook_commands(agent_id: &str) -> Vec<String> {
    let mut raws = Vec::new();
    let mut direct = Vec::new();
    match agent_id {
        "claude" => {
            raws.push(read_file_or_empty(&claude_code_paths().settings));
        }
        "codex" => {
            raws.push(read_file_or_empty(&format!("{}/hooks.json", codex_paths_resolved().dir)));
        }
        "antigravity" => {
            raws.push(read_file_or_empty(&format!("{}/.gemini/config/hooks.json", home())));
        }
        "copilot" => {
            let p = copilot_paths_resolved();
            raws.push(read_file_or_empty(&format!("{}/tokless-rtk.json", p.hooks_dir)));
            raws.push(read_file_or_empty(&format!("{}/tokless-codegraph-index.json", p.hooks_dir)));
            raws.push(read_file_or_empty(&format!("{}/.github/hooks/tokless-rtk.json", ide_root_for_doctor())));
            raws.push(read_file_or_empty(&format!("{}/.github/hooks/tokless-codegraph-index.json", ide_root_for_doctor())));
        }
        "droid" => {
            raws.push(read_file_or_empty(&format!("{}/.factory/hooks.json", home())));
        }
        "cursor" => {
            raws.push(read_file_or_empty(&format!("{}/hooks.json", cursor_paths_resolved().dir)));
        }
        "grok" => {
            let grok_dir = env::var("GROK_HOME").unwrap_or_else(|_| format!("{}/.grok", home()));
            raws.push(read_file_or_empty(&format!("{}/hooks/tokless-rtk.json", grok_dir)));
        }
        "cline" => {
            let mut path = format!("{}/PreToolUse", cline_paths_resolved().hooks_dir);
            if IS_WIN {
                path.push_str(".cjs");
            }
            let raw = read_file_or_empty(&path);
            if !raw.is_empty() {
                if let Some(cmd) = extract_cline_hook_command(&raw) {
                    direct.push(cmd);
                }
            }
        }
        _ => {}
    }

    let mut cmds = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in raws {
        direct.extend(extract_json_commands(&raw));
    }
    for c in direct {
        let fields = split_command(&c);
        if !is_tokless_managed_hook(&fields) || seen.contains(&c) {
            continue;
        }
        seen.insert(c.clone());
        cmds.push(c);
    }
    cmds
}

fn managed_runtime_files(agent_id: &str) -> Vec<RuntimeFile> {
    match agent_id {
        "opencode" => vec![RuntimeFile {
            path: format!("{}/tokless-rtk.ts", opencode_paths_resolved().plugins_dir),
            markers: vec!["TOKLESS_BIN".into(), "rtk-rewrite".into()],
        }],
        "pi" => vec![RuntimeFile {
            path: format!("{}/extensions/rtk.ts", pi_agent_dir_resolved()),
            markers: vec!["TOKLESS_BIN".into(), "rtk-rewrite".into()],
        }],
        "omp" => vec![RuntimeFile {
            path: format!("{}/extensions/tokless-rtk.ts", omp_agent_dir_resolved()),
            markers: vec!["TOKLESS".into(), "rtk-hook".into(), "omp".into()],
        }],
        "kilo" => vec![RuntimeFile {
            path: format!("{}/rtk.ts", kilo_paths_resolved().plugins_dir),
            markers: vec!["tokless-kilo-rtk-v1".into(), "rtk-rewrite".into()],
        }],
        "cline" => {
            let mut path = format!("{}/PreToolUse", cline_paths_resolved().hooks_dir);
            if IS_WIN {
                path.push_str(".cjs");
            }
            vec![RuntimeFile {
                path,
                markers: vec!["tokless-cline-rtk-v1".into(), "rtk-hook".into(), "cline".into()],
            }]
        }
        _ => Vec::new(),
    }
}

fn managed_mcp_spawns(agent_id: &str) -> Vec<McpSpawn> {
    let mut out = Vec::new();
    match agent_id {
        "claude" => {
            out.extend(mcp_from_command_args_file(&claude_code_paths().global_json, "mcpServers", "codegraph"));
        }
        "opencode" => {
            out.extend(mcp_from_command_array_file(&opencode_paths_resolved().config, "mcp", "codegraph"));
        }
        "codex" => {
            out.extend(mcp_from_codex_toml("codegraph"));
        }
        "antigravity" => {
            out.extend(mcp_from_command_args_file(&antigravity_paths_resolved().mcp_config_cli, "mcpServers", "codegraph"));
        }
        "copilot" => {
            let p = copilot_paths_resolved();
            out.extend(mcp_from_command_args_file(&p.mcp_config, "mcpServers", "codegraph"));
        }
        "droid" => {
            out.extend(mcp_from_command_args_file(&format!("{}/.factory/mcp.json", home()), "mcpServers", "codegraph"));
        }
        "pi" => {
            out.extend(mcp_from_command_args_file(&format!("{}/mcp.json", pi_agent_dir_resolved()), "mcpServers", "codegraph"));
        }
        "omp" => {
            out.extend(mcp_from_command_args_file(&format!("{}/mcp.json", omp_agent_dir_resolved()), "mcpServers", "codegraph"));
        }
        "cline" => {
            let p = cline_paths_resolved();
            for tool_id in &["codegraph", "context-mode"] {
                for spawn in mcp_from_command_args_file(&p.mcp_config, "mcpServers", tool_id) {
                    if is_managed_cline_mcp_spawn(tool_id, &spawn) {
                        out.push(spawn);
                    }
                }
            }
        }
        _ => {}
    }
    out
}

fn is_managed_cline_mcp_spawn(tool_id: &str, spawn: &McpSpawn) -> bool {
    let base = path_base_lowercase(&spawn.command);
    if base != "tokless" && base != "tokless.exe" || spawn.args.len() < 3 || spawn.args[0] != "run-mcp" {
        return false;
    }
    match tool_id {
        "context-mode" => {
            spawn.args.len() >= 3 && spawn.args[1] == "--context-mode" && is_cline_mcp_target(&spawn.args[2..], "context-mode")
        }
        "codegraph" => {
            spawn.args.len() >= 4 && spawn.args[1] == "--agent" && spawn.args[2] == "cline" && is_cline_mcp_target(&spawn.args[3..], "codegraph")
        }
        _ => false,
    }
}

fn is_cline_mcp_target(args: &[String], target: &str) -> bool {
    if args.is_empty() {
        return false;
    }
    let mut base = path_base_lowercase(&args[0]);
    for ext in &[".exe", ".cmd", ".bat"] {
        if base.ends_with(ext) {
            base.truncate(base.len() - ext.len());
        }
    }
    if base == target {
        return true;
    }
    if base == "npx" && args.len() >= 3 && args[1] == "--no-install" {
        return target == "context-mode" && args[2] == "context-mode"
            || target == "codegraph" && args[2] == "@colbymchenry/codegraph";
    }
    if base == "cmd" && args.len() >= 3 && args[1].eq_ignore_ascii_case("/c") {
        return is_cline_mcp_target(&args[2..], target);
    }
    false
}

fn mcp_from_command_args_file(path: &str, servers_key: &str, tool_id: &str) -> Vec<McpSpawn> {
    let raw = match read_file_safe(path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(o)) => o,
        _ => return Vec::new(),
    };
    let sv = match cfg.get(servers_key).and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return Vec::new(),
    };
    let ev = match sv.get(tool_id).and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return Vec::new(),
    };
    let cmd = match ev.get("command").and_then(|v| v.as_str()) {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => return Vec::new(),
    };
    let mut args = Vec::new();
    if let Some(Value::Array(arr)) = ev.get("args") {
        for a in arr {
            if let Some(s) = a.as_str() {
                args.push(s.to_string());
            }
        }
    }
    vec![McpSpawn { command: cmd, args }]
}

fn mcp_from_command_array_file(path: &str, mcp_key: &str, tool_id: &str) -> Vec<McpSpawn> {
    let raw = match read_file_safe(path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(o)) => o,
        _ => return Vec::new(),
    };
    let mm = match cfg.get(mcp_key).and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return Vec::new(),
    };
    let ev = match mm.get(tool_id).and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return Vec::new(),
    };
    let cv = match ev.get("command").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };
    let mut str_arr = Vec::new();
    for a in cv {
        if let Some(s) = a.as_str() {
            str_arr.push(s.to_string());
        }
    }
    if str_arr.is_empty() {
        return Vec::new();
    }
    vec![McpSpawn {
        command: str_arr[0].clone(),
        args: str_arr[1..].to_vec(),
    }]
}

fn mcp_from_codex_toml(tool_id: &str) -> Vec<McpSpawn> {
    let p = codex_paths_resolved();
    let raw = match read_file_safe(&p.config) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let header = format!("mcp_servers.{}", tool_id);
    let target = format!("[{}]", header);
    let start = match raw.find(&target) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let rest = &raw[start..];
    let end = rest[1..].find("\n[").map(|idx| idx + 1).unwrap_or(rest.len());
    let block = &rest[..end];

    let re_cmd = Regex::new(r#"(?m)^\s*command\s*=\s*"((?:\\.|[^"\\])*)""#).unwrap();
    let re_args = Regex::new(r#"(?m)^\s*args\s*=\s*\[([^\]]*)\]"#).unwrap();
    let re_str = Regex::new(r#""((?:\\.|[^"\\])*)""#).unwrap();

    let cmd = match re_cmd.captures(block).and_then(|c| c.get(1)) {
        Some(m) => toml_unescape(m.as_str()),
        None => return Vec::new(),
    };
    let mut args = Vec::new();
    if let Some(am) = re_args.captures(block).and_then(|c| c.get(1)) {
        for sm in re_str.captures_iter(am.as_str()) {
            if let Some(m) = sm.get(1) {
                args.push(toml_unescape(m.as_str()));
            }
        }
    }
    vec![McpSpawn { command: cmd, args }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_bash_hostile_path() {
        if IS_WIN {
            assert!(windows_bash_hostile_path(r"C:\Users\user\tokless.exe"));
            assert!(windows_bash_hostile_path(r"D:\tokless.exe"));
            assert!(windows_bash_hostile_path(r"\\server\share\tokless.exe"));
            assert!(!windows_bash_hostile_path("C:/Users/user/tokless.exe"));
            assert!(!windows_bash_hostile_path("tokless"));
        }
    }

    #[test]
    fn test_is_tokless_managed_hook() {
        assert!(is_tokless_managed_hook(&["tokless".into(), "rtk-hook".into()]));
        assert!(is_tokless_managed_hook(&["tokless.exe".into(), "codex-perm".into()]));
        assert!(is_tokless_managed_hook(&[r"C:\bin\tokless.exe".into(), "agy-hook".into()]));
        assert!(!is_tokless_managed_hook(&["tokless".into(), "unknown-cmd".into()]));
        assert!(!is_tokless_managed_hook(&["mywrapper".into(), "rtk-hook".into()]));
    }

    #[test]
    fn test_unwrap_run_mcp() {
        let (cmd, args) = unwrap_run_mcp(
            "tokless",
            &["run-mcp".into(), "--agent".into(), "claude".into(), "codegraph".into(), "serve".into()],
        );
        assert_eq!(cmd, "codegraph");
        assert_eq!(args, vec!["serve"]);

        let (cmd, args) = unwrap_run_mcp(
            "tokless",
            &["run-mcp".into(), "--context-mode".into(), "context-mode".into()],
        );
        assert_eq!(cmd, "context-mode");
        assert!(args.is_empty());
    }

    #[test]
    fn test_looks_like_codegraph() {
        assert!(looks_like_codegraph("codegraph", &[]));
        assert!(looks_like_codegraph("codegraph.cmd", &[]));
        assert!(looks_like_codegraph(r"C:\npm\codegraph.exe", &[]));
        assert!(looks_like_codegraph("cmd", &["/c".into(), "codegraph.cmd".into()]));
        assert!(!looks_like_codegraph("context-mode", &[]));
    }

    #[test]
    fn test_short_path() {
        let short = "C:/short/path";
        assert_eq!(short_path(short), short);

        let long = "C:/very/long/nested/path/that/exceeds/the/sixty/four/character/threshold/for/sure.exe";
        let res = short_path(long);
        assert!(res.contains('…'));
        assert_eq!(res.chars().count(), 57);
    }

    #[test]
    fn test_format_runtime_issues() {
        let issues = vec![
            RuntimeIssue { kind: "hook".into(), detail: "broken hook".into() },
            RuntimeIssue { kind: "mcp".into(), detail: "broken hook".into() },
            RuntimeIssue { kind: "mcp".into(), detail: "missing binary".into() },
        ];
        assert_eq!(format_runtime_issues(&issues), "broken hook, missing binary");
    }
}

