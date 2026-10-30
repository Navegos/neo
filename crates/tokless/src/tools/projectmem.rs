// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/projectmem.rs
// created: 2026-10-05
// lastModified: 2026-10-30

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::agents::{ide_project_root, omp_agent_dir_resolved, pi_agent_dir_resolved};
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::util::exec::{run, RunOptions};
use crate::util::instructions::{cursor_project_rule_content, install_cursor_project_rules};
use crate::util::jsonc::{stringify_json, try_parse_jsonc, OrderedMap, Value};
use crate::util::mcpspawn::{wrap_bounded_tool, McpSpawn};
use crate::util::paths::{
    claude_code_paths, cline_paths_resolved, codex_paths_resolved, copilot_paths_resolved,
    cursor_paths_resolved, ensure_dir, exists, home, kilo_paths_resolved,
    opencode_paths_resolved, read_file_safe, tokless_persisted_abs, which, write_file_atomic,
    IS_WIN,
};
use super::copilot_transaction::with_copilot_transaction;
use super::mcp_dispatch::{mcp_configure, mcp_has, mcp_remove};
use super::helpers::{
    allow_claude_mcp_server, cursor_mcp_path, disallow_claude_mcp_server, droid_mcp_path,
    grok_mcp_path, has_antigravity_mcp, has_array_mcp_entry, has_codex_mcp_entry, has_copilot_mcp,
    has_mcp_entry_at_file, has_owner, is_test, kilo_has_owner, kilo_remove_owner,
    kilo_write_owner, load_ordered, omp_mcp_path, pi_ensure_mcp_adapter, pi_mcp_adapter_ready,
    pi_mcp_path, pi_remove_mcp_adapter, remove_antigravity_mcp, remove_array_mcp_entry,
    remove_codex_mcp_entry, remove_copilot_mcp_safe, remove_hook_group_in_hooks_obj,
    remove_json_mcp_entry, remove_owner_safe, sync_copilot_ide_instructions_safe, tokless_cmd,
    upsert_antigravity_mcp, upsert_array_mcp_entry, upsert_codex_mcp_entry,
    upsert_copilot_mcp_safe, upsert_hook_group_in_hooks_obj, upsert_json_mcp_entry, write_owner,
};

pub const PROJECTMEM_MCP_INSTRUCTIONS: &str = "projectmem writer tools: log_issue, record_attempt, record_fix, add_decision, add_note.";

pub const PROJECTMEM_EVENT_LIMIT: usize = 25;
pub const PROJECTMEM_GOTCHA_LIMIT: usize = 15;
pub const PROJECTMEM_SECTION_LIMIT: usize = 2000;
pub const PROJECTMEM_SESSION_LIMIT: usize = 8000;

pub fn projectmem_ensure_installed(opts: &RunOpts) -> Result<bool, DynError> {
    if let Some(ref report) = opts.report {
        report("ready", 1.0);
    }
    Ok(true)
}

pub fn ensure_projectmem_project(root: &str) -> Result<(), String> {
    if root.is_empty() {
        return Ok(());
    }
    let pm_dir = Path::new(root).join(".projectmem");
    sweep_projectmem_temps(&pm_dir.to_string_lossy());
    if pm_dir.join("summary.md").exists() && pm_dir.join("PROJECT_MAP.md").exists() {
        projectmem_git_exclude(root);
        return Ok(());
    }
    let purpose = projectmem_seed_purpose(root);
    crate::projectmem::store::initialize(Some(Path::new(root)), Some(&purpose))?;
    projectmem_git_exclude(root);
    Ok(())
}

pub fn projectmem_seed_purpose(root: &str) -> String {
    let base = Path::new(root)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");
    for readme in ["README.md", "readme.md", "README.txt", "README"] {
        let p = Path::new(root).join(readme);
        if let Some(raw) = read_file_safe(&p) {
            for line in raw.lines() {
                let t = line.trim().trim_start_matches('#').trim();
                if !t.is_empty() && !t.starts_with("![") && !t.starts_with("[!") {
                    let truncated: String = t.chars().take(120).collect();
                    return truncated;
                }
            }
        }
    }
    format!("{} workspace", base)
}

pub fn sweep_projectmem_temps(dir: &str) {
    let p = Path::new(dir);
    if !p.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(p) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(".tmp") || name.ends_with(".tmp") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

pub fn projectmem_git_exclude(root: &str) {
    let git_dir = Path::new(root).join(".git");
    if !git_dir.is_dir() {
        return;
    }
    let info_dir = git_dir.join("info");
    let _ = ensure_dir(&info_dir);
    let exclude = info_dir.join("exclude");
    let existing = read_file_safe(&exclude).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == ".projectmem/" || l.trim() == ".projectmem") {
        return;
    }
    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(".projectmem/\n");
    let _ = write_file_atomic(&exclude, &updated);
}

pub fn projectmem_bin() -> String {
    let tok = tokless_persisted_abs();
    if !tok.is_empty() && Path::new(&tok).is_file() {
        return tok;
    }
    let p = which("tokless");
    if !p.is_empty() {
        return p;
    }
    let pj = which("pjm");
    if !pj.is_empty() {
        return pj;
    }
    let name = if IS_WIN { "pjm.exe" } else { "pjm" };
    let cand = Path::new(&home()).join(".local").join("bin").join(name);
    if cand.is_file() {
        return cand.to_string_lossy().to_string();
    }
    "tokless".to_string()
}

pub fn projectmem_mcp_bin() -> String {
    let tok = tokless_persisted_abs();
    if !tok.is_empty() && Path::new(&tok).is_file() {
        return tok;
    }
    let p = which("tokless");
    if !p.is_empty() {
        return p;
    }
    let pm = which("pjm-mcp");
    if !pm.is_empty() {
        return pm;
    }
    let name = if IS_WIN { "pjm-mcp.exe" } else { "pjm-mcp" };
    let cand = Path::new(&home()).join(".local").join("bin").join(name);
    if cand.is_file() {
        return cand.to_string_lossy().to_string();
    }
    "tokless".to_string()
}

pub fn projectmem_installed() -> bool {
    true
}

pub fn projectmem_installed_version() -> Option<String> {
    Some("0.3.4".to_string())
}

pub fn run_pjm(root: &str, args: &[&str]) -> String {
    let bin = projectmem_bin();
    if bin.is_empty() {
        return String::new();
    }
    let full_args: Vec<&str> = if bin.ends_with("tokless") || bin.ends_with("tokless.exe") {
        let mut v = vec!["mem"];
        v.extend_from_slice(args);
        v
    } else {
        args.to_vec()
    };
    let res = run(
        &bin,
        &full_args,
        RunOptions {
            capture: true,
            quiet: true,
            cwd: Some(root.to_string()),
            env: vec![("PROJECTMEM_ROOT".to_string(), root.to_string())],
            timeout: Some(Duration::from_secs(5)),
        },
    );
    if res.code == 0 {
        res.stdout.trim().to_string()
    } else {
        String::new()
    }
}

fn truncate_str(s: &str, limit: usize) -> String {
    if s.chars().count() <= limit {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(limit).collect();
        out.push_str("…");
        out
    }
}

pub fn projectmem_session_text(root: &str) -> String {
    let pm_dir = Path::new(root).join(".projectmem");
    if !pm_dir.is_dir() {
        return String::new();
    }
    let mut sections = Vec::new();

    if let Some(summary) = read_file_safe(pm_dir.join("summary.md")) {
        let t = summary.trim();
        if !t.is_empty() {
            sections.push(format!("## Summary\n{}", truncate_str(t, PROJECTMEM_SECTION_LIMIT)));
        }
    }
    if let Some(map_md) = read_file_safe(pm_dir.join("PROJECT_MAP.md")) {
        let t = map_md.trim();
        if !t.is_empty() {
            sections.push(format!("## Project Map\n{}", truncate_str(t, PROJECTMEM_SECTION_LIMIT)));
        }
    }
    if let Some(plan) = read_file_safe(pm_dir.join("plan.md")) {
        let t = plan.trim();
        if !t.is_empty() && t != "# Plan" {
            sections.push(format!("## Plan\n{}", truncate_str(t, PROJECTMEM_SECTION_LIMIT)));
        }
    }
    if let Some(events) = read_file_safe(pm_dir.join("events.jsonl")) {
        let lines: Vec<&str> = events.lines().filter(|l| !l.trim().is_empty()).collect();
        if !lines.is_empty() {
            let start = lines.len().saturating_sub(PROJECTMEM_EVENT_LIMIT);
            let recent = lines[start..].join("\n");
            sections.push(format!("## Recent Events\n{}", truncate_str(&recent, PROJECTMEM_SECTION_LIMIT)));
        }
    }
    let global_gotchas = Path::new(&home()).join(".projectmem").join("gotchas.jsonl");
    if let Some(gotchas) = read_file_safe(&global_gotchas) {
        let lines: Vec<&str> = gotchas.lines().filter(|l| !l.trim().is_empty()).collect();
        if !lines.is_empty() {
            let start = lines.len().saturating_sub(PROJECTMEM_GOTCHA_LIMIT);
            let recent = lines[start..].join("\n");
            sections.push(format!("## Global Gotchas\n{}", truncate_str(&recent, PROJECTMEM_SECTION_LIMIT)));
        }
    }

    if sections.is_empty() {
        return String::new();
    }
    let body = format!("# Project Memory (.projectmem)\n\n{}", sections.join("\n\n"));
    truncate_str(&body, PROJECTMEM_SESSION_LIMIT)
}

pub fn refresh_projectmem_rule_files(root: &str) {
    if root.is_empty() {
        return;
    }
    let digest = projectmem_session_text(root);
    if digest.is_empty() {
        return;
    }
    // Cursor project rule if .cursor/rules exists
    let cursor_rules = Path::new(root).join(".cursor").join("rules");
    if cursor_rules.is_dir() {
        let p = cursor_rules.join("projectmem-state.mdc");
        let content = format!(
            "---\ndescription: Live Project Memory state from .projectmem/\nalwaysApply: true\n---\n\n{}\n",
            digest
        );
        let _ = write_file_atomic(p, &content);
    }
    // Kilo project rule if .kilocode/rules exists
    let kilo_rules = Path::new(root).join(".kilocode").join("rules");
    if kilo_rules.is_dir() {
        let p = kilo_rules.join("projectmem-state.md");
        let _ = write_file_atomic(p, &format!("{}\n", digest));
    }
    // Cline project rule if .clinerules exists as directory
    let cline_rules = Path::new(root).join(".clinerules");
    if cline_rules.is_dir() {
        let p = cline_rules.join("projectmem-state.md");
        let _ = write_file_atomic(p, &format!("{}\n", digest));
    }
}

const OPENCODE_PROJECTMEM_PLUGIN: &str = r#"import { spawnSync } from "node:child_process"
const TOKLESS_BIN = %TOKLESS%

export const ToklessProjectmemPlugin = async () => ({
  "session.created": async () => {
    try {
      spawnSync(TOKLESS_BIN, ["projectmem-hook", "opencode"], { stdio: "ignore" })
    } catch {}
  },
})

export default ToklessProjectmemPlugin
"#;

const PI_PROJECTMEM_EXT: &str = r#"const TOKLESS_BIN = %TOKLESS%

export default function (pi: any) {
  pi.on("session_start", async () => {
    try {
      await pi.exec(TOKLESS_BIN, ["projectmem-hook", "pi"])
    } catch {}
  })
}
"#;

const OMP_PROJECTMEM_EXT: &str = r#"import { spawnSync } from "node:child_process"
const TOKLESS_BIN = %TOKLESS%

export default function (pi: any) {
  pi.on("session_start", async () => {
    try {
      spawnSync(TOKLESS_BIN, ["projectmem-hook", "omp"], { stdio: "ignore" })
    } catch {}
  })
}
"#;

pub fn write_projectmem_extension(agent: &str) -> bool {
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return false;
    }
    let quoted = format!("{:?}", exe);
    match agent {
        "opencode" => {
            let p = Path::new(&opencode_paths_resolved().plugins_dir).join("tokless-projectmem.ts");
            if let Some(parent) = p.parent() {
                let _ = ensure_dir(parent);
            }
            write_file_atomic(p, &OPENCODE_PROJECTMEM_PLUGIN.replace("%TOKLESS%", &quoted)).is_ok()
        }
        "pi" => {
            let p = Path::new(&pi_agent_dir_resolved()).join("extensions").join("projectmem.ts");
            if let Some(parent) = p.parent() {
                let _ = ensure_dir(parent);
            }
            write_file_atomic(p, &PI_PROJECTMEM_EXT.replace("%TOKLESS%", &quoted)).is_ok()
        }
        "omp" => {
            let p = Path::new(&omp_agent_dir_resolved())
                .join("extensions")
                .join("tokless-projectmem.ts");
            if let Some(parent) = p.parent() {
                let _ = ensure_dir(parent);
            }
            write_file_atomic(p, &OMP_PROJECTMEM_EXT.replace("%TOKLESS%", &quoted)).is_ok()
        }
        _ => true,
    }
}

pub fn remove_projectmem_extension(agent: &str) {
    let path = match agent {
        "opencode" => Path::new(&opencode_paths_resolved().plugins_dir).join("tokless-projectmem.ts"),
        "pi" => Path::new(&pi_agent_dir_resolved()).join("extensions").join("projectmem.ts"),
        "omp" => Path::new(&omp_agent_dir_resolved())
            .join("extensions")
            .join("tokless-projectmem.ts"),
        _ => return,
    };
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

pub fn projectmem_install_session_push(agent: &str) -> bool {
    match agent {
        "claude" => {
            let cmd = tokless_cmd(&["projectmem-hook", "claude"]);
            upsert_hook_group_in_hooks_obj(
                &claude_code_paths().settings,
                "SessionStart",
                "",
                &cmd,
                None,
            )
        }
        "codex" => {
            let cmd = tokless_cmd(&["projectmem-hook", "codex"]);
            let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
            upsert_hook_group_in_hooks_obj(&p, "SessionStart", "", &cmd, Some(10))
        }
        "antigravity" => {
            let cmd = tokless_cmd(&["projectmem-hook", "antigravity"]);
            let p = Path::new(&home()).join(".gemini").join("config").join("hooks.json").to_string_lossy().to_string();
            if let Some(parent) = Path::new(&p).parent() {
                let _ = ensure_dir(parent);
            }
            let mut cfg = load_ordered(&p);
            let mut pm_group = OrderedMap::new();
            let mut entry = OrderedMap::new();
            let mut hook_cfg = OrderedMap::new();
            hook_cfg.insert("type".into(), Value::String("command".into()));
            hook_cfg.insert("command".into(), Value::String(cmd));
            entry.insert("hooks".into(), Value::Array(vec![Value::Object(hook_cfg)]));
            pm_group.insert("SessionStart".into(), Value::Array(vec![Value::Object(entry)]));
            cfg.insert("projectmem".into(), Value::Object(pm_group));
            write_file_atomic(&p, &stringify_json(&Value::Object(cfg))).is_ok()
        }
        "copilot" => {
            let cmd = tokless_cmd(&["projectmem-hook", "copilot"]);
            if cmd.is_empty() {
                return false;
            }
            let p1 = Path::new(&copilot_paths_resolved().hooks_dir).join("tokless-projectmem.json");
            if let Some(parent) = p1.parent() {
                let _ = ensure_dir(parent);
            }
            let mut obj = OrderedMap::new();
            obj.insert("command".into(), Value::String(cmd));
            let json = stringify_json(&Value::Object(obj));
            let ok1 = write_file_atomic(&p1, &json).is_ok();
            let p2 = Path::new(&ide_project_root()).join(".github").join("hooks").join("tokless-projectmem.json");
            if let Some(parent) = p2.parent() {
                let _ = ensure_dir(parent);
            }
            let ok2 = write_file_atomic(&p2, &json).is_ok();
            ok1 && ok2
        }
        "droid" => {
            let cmd = tokless_cmd(&["projectmem-hook", "droid"]);
            let p = Path::new(&home()).join(".factory").join("hooks.json").to_string_lossy().to_string();
            upsert_hook_group_in_hooks_obj(&p, "SessionStart", "", &cmd, None)
        }
        "grok" => {
            let cmd = tokless_cmd(&["projectmem-hook", "grok"]);
            let dir = env::var("GROK_HOME")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| Path::new(&home()).join(".grok").to_string_lossy().to_string());
            let p = Path::new(&dir).join("hooks").join("tokless-projectmem.json");
            if let Some(parent) = p.parent() {
                let _ = ensure_dir(parent);
            }
            let mut obj = OrderedMap::new();
            obj.insert("command".into(), Value::String(cmd));
            write_file_atomic(p, &stringify_json(&Value::Object(obj))).is_ok()
        }
        "opencode" | "pi" | "omp" => write_projectmem_extension(agent),
        _ => true,
    }
}

pub fn projectmem_remove_session_push(agent: &str) {
    match agent {
        "claude" => {
            remove_hook_group_in_hooks_obj(&claude_code_paths().settings, "SessionStart", "projectmem-hook");
        }
        "codex" => {
            let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
            remove_hook_group_in_hooks_obj(&p, "SessionStart", "projectmem-hook");
        }
        "antigravity" => {
            let p = Path::new(&home()).join(".gemini").join("config").join("hooks.json").to_string_lossy().to_string();
            if let Some(raw) = read_file_safe(&p) {
                if let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) {
                    if cfg.remove("projectmem").is_some() {
                        if cfg.is_empty() {
                            let _ = fs::remove_file(&p);
                        } else {
                            let _ = write_file_atomic(&p, &stringify_json(&Value::Object(cfg)));
                        }
                    }
                }
            }
        }
        "copilot" => {
            let p1 = Path::new(&copilot_paths_resolved().hooks_dir).join("tokless-projectmem.json");
            if p1.exists() {
                let _ = fs::remove_file(p1);
            }
            let p2 = Path::new(&ide_project_root()).join(".github").join("hooks").join("tokless-projectmem.json");
            if p2.exists() {
                let _ = fs::remove_file(p2);
            }
        }
        "droid" => {
            let p = Path::new(&home()).join(".factory").join("hooks.json").to_string_lossy().to_string();
            remove_hook_group_in_hooks_obj(&p, "SessionStart", "projectmem-hook");
        }
        "grok" => {
            let dir = env::var("GROK_HOME")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| Path::new(&home()).join(".grok").to_string_lossy().to_string());
            let p = Path::new(&dir).join("hooks").join("tokless-projectmem.json");
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }
        "opencode" | "pi" | "omp" => remove_projectmem_extension(agent),
        _ => {}
    }
}

pub fn projectmem_configure_mcp(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let mcp_bin = projectmem_mcp_bin();
    let (bin, args) = if mcp_bin.ends_with("tokless") || mcp_bin.ends_with("tokless.exe") {
        (mcp_bin, vec!["mem".to_string(), "mcp".to_string()])
    } else if !mcp_bin.is_empty() {
        (mcp_bin, Vec::new())
    } else {
        ("tokless".to_string(), vec!["mem".to_string(), "mcp".to_string()])
    };
    let raw_spawn = McpSpawn {
        command: bin,
        args,
    };
    let spawn = wrap_bounded_tool("projectmem", raw_spawn);

    match agent {
        "claude" => {
            let p = claude_code_paths();
            if !upsert_json_mcp_entry(&p.global_json, "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for claude".into());
            }
            allow_claude_mcp_server("projectmem");
            Ok(true)
        }
        "opencode" => {
            let p = opencode_paths_resolved();
            if !upsert_array_mcp_entry(&p.config, "projectmem", &spawn) {
                return Err("failed to configure projectmem MCP for opencode".into());
            }
            Ok(true)
        }
        "codex" => {
            if !upsert_codex_mcp_entry("projectmem", &spawn) {
                return Err("failed to configure projectmem MCP for codex".into());
            }
            Ok(true)
        }
        "antigravity" => {
            if !upsert_antigravity_mcp("projectmem", &spawn) {
                return Err("failed to configure projectmem MCP for antigravity".into());
            }
            Ok(true)
        }
        "copilot" => {
            upsert_copilot_mcp_safe("projectmem", &spawn, &spawn)?;
            Ok(true)
        }
        "droid" => {
            if !upsert_json_mcp_entry(&droid_mcp_path(), "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for droid".into());
            }
            Ok(true)
        }
        "pi" => {
            pi_ensure_mcp_adapter(false);
            if !upsert_json_mcp_entry(&pi_mcp_path(), "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for pi".into());
            }
            Ok(true)
        }
        "omp" => {
            if !upsert_json_mcp_entry(&omp_mcp_path(), "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for omp".into());
            }
            Ok(true)
        }
        "grok" => {
            if !upsert_json_mcp_entry(&grok_mcp_path(), "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for grok".into());
            }
            Ok(true)
        }
        "cursor" => {
            if !upsert_json_mcp_entry(&cursor_mcp_path(), "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for cursor".into());
            }
            Ok(true)
        }
        "kilo" => {
            let p = kilo_paths_resolved();
            if !upsert_array_mcp_entry(&p.config, "projectmem", &spawn) {
                return Err("failed to configure projectmem MCP for kilo".into());
            }
            Ok(true)
        }
        "cline" => {
            let p = cline_paths_resolved();
            if !upsert_json_mcp_entry(&p.mcp_config, "mcpServers", "projectmem", &spawn, &[]) {
                return Err("failed to configure projectmem MCP for cline".into());
            }
            Ok(true)
        }
        "opencode2" | "amp" | "kimi" | "zcode" | "dsh" => {
            if !mcp_configure(agent, "projectmem") {
                return Err(format!("failed to configure projectmem MCP for {}", agent).into());
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn projectmem_verify(agent: &str) -> bool {
    match agent {
        "claude" => {
            has_mcp_entry_at_file(&claude_code_paths().global_json, "mcpServers", "projectmem", None)
                && has_owner("claude", "projectmem")
        }
        "opencode" => {
            has_array_mcp_entry(&opencode_paths_resolved().config, "projectmem", None)
                && has_owner("opencode", "projectmem")
        }
        "codex" => {
            has_codex_mcp_entry("projectmem", None) && has_owner("codex", "projectmem")
        }
        "antigravity" => {
            has_antigravity_mcp("projectmem", None) && has_owner("antigravity", "projectmem")
        }
        "copilot" => {
            has_copilot_mcp("projectmem", None, None) && has_owner("copilot", "projectmem")
        }
        "droid" => {
            has_mcp_entry_at_file(&droid_mcp_path(), "mcpServers", "projectmem", None)
                && has_owner("droid", "projectmem")
        }
        "pi" => {
            pi_mcp_adapter_ready()
                && has_mcp_entry_at_file(&pi_mcp_path(), "mcpServers", "projectmem", None)
                && has_owner("pi", "projectmem")
        }
        "omp" => {
            has_mcp_entry_at_file(&omp_mcp_path(), "mcpServers", "projectmem", None)
                && has_owner("omp", "projectmem")
        }
        "grok" => {
            has_mcp_entry_at_file(&grok_mcp_path(), "mcpServers", "projectmem", None)
                && has_owner("grok", "projectmem")
        }
        "cursor" => {
            let rules_p = Path::new(&cursor_paths_resolved().rules_dir)
                .join("projectmem.mdc")
                .to_string_lossy()
                .to_string();
            let rules_ok = if let Some(raw) = read_file_safe(&rules_p) {
                let expected = cursor_project_rule_content("projectmem");
                !expected.is_empty() && raw.replace("\r\n", "\n") == expected.replace("\r\n", "\n")
            } else {
                true
            };
            has_mcp_entry_at_file(&cursor_mcp_path(), "mcpServers", "projectmem", None) && rules_ok
        }
        "kilo" => {
            has_array_mcp_entry(&kilo_paths_resolved().config, "projectmem", None)
                && kilo_has_owner("projectmem")
        }
        "cline" => {
            has_mcp_entry_at_file(&cline_paths_resolved().mcp_config, "mcpServers", "projectmem", None)
                && has_owner("cline", "projectmem")
        }
        "opencode2" | "amp" | "kimi" | "zcode" | "dsh" => {
            mcp_has(agent, "projectmem", false) && has_owner(agent, "projectmem")
        }
        _ => false,
    }
}

pub fn projectmem_wire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if agent == "copilot" {
        return with_copilot_transaction(&[], || {
            projectmem_configure_mcp("copilot", opts)?;
            let _ = projectmem_install_session_push("copilot");
            if !write_owner("copilot", "projectmem") {
                return Err("failed to write copilot projectmem instructions".into());
            }
            sync_copilot_ide_instructions_safe()?;
            Ok(projectmem_verify("copilot"))
        });
    }
    projectmem_configure_mcp(agent, opts)?;
    let _ = projectmem_install_session_push(agent);
    match agent {
        "kilo" => {
            if !kilo_write_owner("projectmem") {
                return Err("failed to write kilo projectmem instructions".into());
            }
        }
        "cursor" => {
            let _ = install_cursor_project_rules(".", false);
        }
        _ => {
            if !write_owner(agent, "projectmem") {
                return Err(format!("failed to write projectmem instructions for {}", agent).into());
            }
        }
    }
    Ok(projectmem_verify(agent))
}

pub fn projectmem_unwire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    projectmem_remove_session_push(agent);
    match agent {
        "claude" => {
            remove_json_mcp_entry(&claude_code_paths().global_json, "mcpServers", "projectmem");
            disallow_claude_mcp_server("projectmem");
            remove_owner_safe("claude", "projectmem")?;
        }
        "opencode" => {
            remove_array_mcp_entry(&opencode_paths_resolved().config, "projectmem");
            remove_owner_safe("opencode", "projectmem")?;
        }
        "codex" => {
            remove_codex_mcp_entry("projectmem");
            remove_owner_safe("codex", "projectmem")?;
        }
        "antigravity" => {
            remove_antigravity_mcp("projectmem");
            remove_owner_safe("antigravity", "projectmem")?;
        }
        "copilot" => {
            return with_copilot_transaction(&[], || {
                remove_copilot_mcp_safe("projectmem")?;
                projectmem_remove_session_push("copilot");
                remove_owner_safe("copilot", "projectmem")?;
                sync_copilot_ide_instructions_safe()?;
                Ok(true)
            });
        }
        "droid" => {
            remove_json_mcp_entry(&droid_mcp_path(), "mcpServers", "projectmem");
            remove_owner_safe("droid", "projectmem")?;
        }
        "pi" => {
            remove_json_mcp_entry(&pi_mcp_path(), "mcpServers", "projectmem");
            pi_remove_mcp_adapter();
            remove_owner_safe("pi", "projectmem")?;
        }
        "omp" => {
            remove_json_mcp_entry(&omp_mcp_path(), "mcpServers", "projectmem");
            remove_owner_safe("omp", "projectmem")?;
        }
        "grok" => {
            remove_json_mcp_entry(&grok_mcp_path(), "mcpServers", "projectmem");
            remove_owner_safe("grok", "projectmem")?;
        }
        "cursor" => {
            remove_json_mcp_entry(&cursor_mcp_path(), "mcpServers", "projectmem");
            let p = Path::new(&cursor_paths_resolved().rules_dir).join("projectmem.mdc");
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }
        "kilo" => {
            remove_array_mcp_entry(&kilo_paths_resolved().config, "projectmem");
            kilo_remove_owner("projectmem");
        }
        "cline" => {
            remove_json_mcp_entry(&cline_paths_resolved().mcp_config, "mcpServers", "projectmem");
            remove_owner_safe("cline", "projectmem")?;
        }
        "opencode2" | "amp" | "kimi" | "zcode" | "dsh" => {
            mcp_remove(agent, "projectmem");
            remove_owner_safe(agent, "projectmem")?;
        }
        _ => {}
    }
    Ok(true)
}

pub fn projectmem_index_project(dir: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    ensure_projectmem_project(dir).map_err(|e| -> DynError { e.into() })?;
    refresh_projectmem_rule_files(dir);
    Ok(exists(Path::new(dir).join(".projectmem").join("summary.md")))
}

pub fn build_projectmem() -> ToolManifest {
    let mut tool = ToolManifest::new("projectmem", "Project Memory");
    tool.description = "Cross-session persistent project memory in .projectmem/".into();
    tool.homepage = "https://github.com/Navegos/projectmem".into();
    tool.install_hint = "Native embedded in tokless (tokless mem)".into();
    tool.channel = Channel::Binary;
    tool.install = Some(Arc::new(|opts| projectmem_ensure_installed(opts)));
    tool.index_ready = Some(Arc::new(|| {
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into());
        exists(Path::new(&cwd).join(".projectmem").join("summary.md"))
    }));
    tool.index_project = Some(Arc::new(|dir, opts| projectmem_index_project(dir, opts)));

    let agents = [
        "claude", "opencode", "opencode2", "codex", "antigravity", "copilot", "droid", "grok",
        "pi", "omp", "kilo", "cline", "cursor", "amp", "kimi", "zcode", "dsh",
    ];

    for agent_id in agents {
        let ag = agent_id.to_string();
        let ag_w = ag.clone();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(move |opts| projectmem_wire(&ag_w, opts)),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| projectmem_unwire(&ag_u, opts)),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || {
                if !is_test() && !projectmem_installed() {
                    return Some(false);
                }
                Some(projectmem_verify(&ag_v))
            }),
        );

        let ag_a = ag.clone();
        tool.wired_any_for.insert(
            ag.clone(),
            Arc::new(move || Some(mcp_has(&ag_a, "projectmem", true))),
        );
    }

    tool
}
