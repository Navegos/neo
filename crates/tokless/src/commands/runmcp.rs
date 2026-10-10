use std::env;
use std::path::Path;
use crate::commands::mcp_proxy::run_mcp_proxy;
use crate::headroom::autostart::ensure_proxy_up;
use crate::tools::codegraph::ensure_codegraph_index;
use crate::tools::projectmem::{ensure_projectmem_project, refresh_projectmem_rule_files};
use crate::util::logger::L;
use crate::util::paths::{ensure_process_path, prepend_process_path, which};

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
    if cmd_args[0].contains(std::path::MAIN_SEPARATOR) {
        if let Some(parent) = Path::new(&cmd_args[0]).parent() {
            prepend_process_path(&parent.to_string_lossy());
        }
    }

    let mut actual_cmd_args: Vec<String> = cmd_args.to_vec();

    // Check if codegraph
    let is_codegraph = actual_cmd_args[0].contains("codegraph") || tool == "codegraph";
    if is_codegraph {
        let root = if !workspace.is_empty() && workspace != "${workspaceFolder}" {
            workspace.clone()
        } else {
            env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into())
        };
        let _ = ensure_codegraph_index(&root, &crate::core::RunOpts::default());
        // inject --path if not present
        if !actual_cmd_args.iter().any(|a| a == "--path" || a.starts_with("--path=")) {
            actual_cmd_args.push("--path".into());
            actual_cmd_args.push(root);
        }
    }

    let mut env_vars = Vec::new();
    if tool == "projectmem" {
        if which("pjm-mcp").is_empty() {
            L.err("projectmem: pjm-mcp not found; install with: uv tool install projectmem");
            return 1;
        }
        let root = if !workspace.is_empty() && workspace != "${workspaceFolder}" {
            workspace.clone()
        } else {
            env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into())
        };
        let _ = ensure_projectmem_project(&root);
        refresh_projectmem_rule_files(&root);
        env_vars.push(("PROJECTMEM_ROOT".into(), root));
    }

    if !agent.is_empty() {
        let _ = ensure_proxy_up();
    }

    let bin_path = actual_cmd_args[0].clone();
    run_mcp_proxy(&agent, &bin_path, &actual_cmd_args, &env_vars, &tool)
}

