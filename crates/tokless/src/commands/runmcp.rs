// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/runmcp.rs
// created: 2026-10-05
// lastModified: 2026-10-31

use std::env;
use std::io::{self, Read};
use std::path::Path;

use crate::commands::index::{
    find_project_dir, looks_like_project, resolve_hook_project_dir_from_input,
};
use crate::commands::mcp_proxy::run_mcp_proxy;
use crate::core::RunOpts;
use crate::headroom::autostart::ensure_proxy_up;
use crate::tools::codegraph::ensure_codegraph_index;
use crate::tools::projectmem::{
    ensure_projectmem_project, projectmem_session_text, refresh_projectmem_rule_files,
};
use crate::util::jsonc::{stringify_json, OrderedMap, Value};
use crate::util::paths::{ensure_process_path, prepend_process_path};

pub fn projectmem_root(workspace: &str) -> String {
    let base = if !workspace.is_empty() && workspace != "${workspaceFolder}" {
        workspace.to_string()
    } else {
        env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into())
    };
    find_project_dir(&base)
}

pub fn ensure_proxy_for_agent(agent: &str) {
    if !agent.is_empty() {
        let _ = ensure_proxy_up();
    }
}

pub fn codegraph_workspace(workspace: &str) -> String {
    let base = if !workspace.is_empty() && workspace != "${workspaceFolder}" {
        workspace.to_string()
    } else {
        env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into())
    };
    find_project_dir(&base)
}

pub fn codegraph_mcp_command(cmd_args: &[String]) -> bool {
    cmd_args
        .iter()
        .any(|a| a.to_lowercase().contains("codegraph"))
}

pub fn inject_codegraph_path(args: &[String], root: &str) -> Vec<String> {
    let mut out = args.to_vec();
    if root.is_empty() {
        return out;
    }
    if let Some(pos) = out.iter().position(|a| a == "serve") {
        let after = &out[pos + 1..];
        let has_positional = after.iter().any(|a| !a.starts_with('-'));
        if !has_positional {
            out.push(root.to_string());
        }
        return out;
    }
    if !out.iter().any(|a| a == "--path" || a.starts_with("--path=")) {
        out.push("--path".into());
        out.push(root.to_string());
    }
    out
}

pub fn run_mcp(argv: &[String]) -> i32 {
    let mut agent = String::new();
    let mut workspace = String::new();
    let mut context_mode = false;
    let mut tool = String::new();
    let mut rest_idx = 0;

    while rest_idx < argv.len() {
        let a = &argv[rest_idx];
        if a == "--agent" && rest_idx + 1 < argv.len() {
            agent = argv[rest_idx + 1].clone();
            rest_idx += 2;
        } else if a == "--workspace" && rest_idx + 1 < argv.len() {
            workspace = argv[rest_idx + 1].clone();
            rest_idx += 2;
        } else if a == "--tool" && rest_idx + 1 < argv.len() {
            tool = argv[rest_idx + 1].clone();
            rest_idx += 2;
        } else if a == "--context-mode" {
            context_mode = true;
            rest_idx += 1;
        } else {
            break;
        }
    }

    if context_mode && tool.is_empty() {
        tool = "context-mode".into();
    }
    if tool == "headroom" {
        return 2;
    }

    let cmd_args = &argv[rest_idx..];
    if cmd_args.is_empty() {
        return 1;
    }

    ensure_process_path();
    if cmd_args[0].contains(std::path::MAIN_SEPARATOR) || cmd_args[0].contains('/') {
        if let Some(parent) = Path::new(&cmd_args[0]).parent() {
            prepend_process_path(&parent.to_string_lossy());
        }
    }

    let actual_cmd_args: Vec<String> = cmd_args.to_vec();

    let is_codegraph = codegraph_mcp_command(&actual_cmd_args) || tool == "codegraph";
    if is_codegraph {
        let root = codegraph_workspace(&workspace);
        if looks_like_project(&root) {
            let _ = ensure_codegraph_index(&root, &RunOpts::default());
            let _ = ensure_projectmem_project(&root);
            refresh_projectmem_rule_files(&root);
        }
        unsafe {
            std::env::set_var("CODEGRAPH_PROJECT_DIR", &root);
        }
        return crate::codegraph::mcp::run_mcp_server();
    }

    let is_projectmem = actual_cmd_args.iter().any(|a| a.contains("projectmem") || a.contains("pjm-mcp")) || tool == "projectmem";
    if is_projectmem {
        let root = projectmem_root(&workspace);
        if looks_like_project(&root) {
            let _ = ensure_projectmem_project(&root);
            refresh_projectmem_rule_files(&root);
        }
        unsafe {
            std::env::set_var("PROJECTMEM_ROOT", &root);
        }
        return crate::projectmem::mcp::run_mcp_server();
    }

    let is_contextmode = actual_cmd_args.iter().any(|a| a.contains("context-mode") || a.contains("ctx-mcp")) || tool == "context-mode" || context_mode;
    if is_contextmode {
        let root = projectmem_root(&workspace);
        unsafe {
            std::env::set_var("CONTEXT_MODE_PROJECT_DIR", &root);
        }
        return crate::contextmode::mcp::run_mcp_server();
    }

    let is_icm = actual_cmd_args.iter().any(|a| a.contains("icm") || a.contains("icm-mcp") || a.contains("aimemory") || a.contains("handoff"))
        || tool == "icm" || tool == "aimemory" || tool == "handoff";
    if is_icm {
        let root = projectmem_root(&workspace);
        unsafe {
            std::env::set_var("PROJECTMEM_ROOT", &root);
        }
        return crate::icm::mcp::run_mcp_server();
    }

    let env_vars = Vec::new();
    ensure_proxy_for_agent(&agent);

    let bin_path = actual_cmd_args[0].clone();
    run_mcp_proxy(&agent, &bin_path, &actual_cmd_args, &env_vars, &tool)
}

pub fn run_projectmem_hook(agent: &str) -> i32 {
    ensure_process_path();
    let mut input = Vec::new();
    let _ = io::stdin().read_to_end(&mut input);
    let raw = String::from_utf8_lossy(&input);
    let root = resolve_hook_project_dir_from_input(&raw);
    if !looks_like_project(&root) {
        return 0;
    }
    let _ = ensure_projectmem_project(&root);
    refresh_projectmem_rule_files(&root);

    let ctx = projectmem_session_text(&root);
    if ctx.is_empty() {
        return 0;
    }

    match agent {
        "claude" | "codex" | "droid" | "grok" | "copilot" => {
            let mut hook_out = OrderedMap::new();
            hook_out.insert("hookEventName".into(), Value::String("SessionStart".into()));
            hook_out.insert("additionalContext".into(), Value::String(ctx));
            let mut resp = OrderedMap::new();
            resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
            println!("{}", stringify_json(&Value::Object(resp)));
        }
        "antigravity" | "agy" => {
            let mut resp = OrderedMap::new();
            resp.insert("additionalContext".into(), Value::String(ctx));
            println!("{}", stringify_json(&Value::Object(resp)));
        }
        _ => {}
    }
    0
}
