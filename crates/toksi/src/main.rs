// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/main.rs
// created: 2026-10-05
// lastModified: 2026-11-09

use std::collections::HashMap;
use std::env;
use std::process;

use toksi::agents;
use toksi::commands::{
    run_codegraph_index_hook, run_codex_perm_hook, run_ctx_hook,
    run_cursor_project_rules_hook, run_daemon_cli, run_disable, run_doctor,
    run_grok_session_start_hook, run_icm_hook, run_index, run_info, run_init, run_mcp, run_prime,
    run_projectmem_hook, run_proxy_cli, run_rtk_cli, run_rtk_hook,
    run_rtk_hook_cline, run_rtk_hook_codex, run_rtk_hook_copilot, run_rtk_hook_cursor,
    run_rtk_hook_droid, run_rtk_hook_dsh, run_rtk_hook_grok, run_rtk_rewrite, run_self_update,
    run_top, run_tui_cli, run_uninstall, run_update, run_gain, InitOptions, PrimeOptions,
};

use toksi::core::{agent_ids, tool_ids};
use toksi::headroom::autostart::grok_oauth_proxy_owned;
use toksi::headroom::byok_gateway::run_byok_gateway_serve;
use toksi::headroom::proxy::{
    ensure_copilot_proxy_up, ensure_proxy_up, run_proxy_foreground, run_proxy_watchdog,
    start_proxy, stop_proxy,
};
use toksi::tools;
use toksi::util::colors::*;
use toksi::util::headroom_proxy::proxy_routing_enabled;
use toksi::util::logger::{set_verbose, L};
use toksi::util::paths::ensure_process_path;
use toksi::util::version::toksi_version;

#[derive(Default)]
struct ParsedArgs {
    cmd: String,
    flags: HashMap<String, String>,
    bools: HashMap<String, bool>,
}

fn parse_args(argv: &[String]) -> ParsedArgs {
    let mut p = ParsedArgs::default();
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        if a.starts_with("--") {
            let key = &a[2..];
            if let Some(eq) = key.find('=') {
                p.flags.insert(key[..eq].to_string(), key[eq + 1..].to_string());
            } else if i + 1 < argv.len() && !argv[i + 1].starts_with('-') {
                p.flags.insert(key.to_string(), argv[i + 1].clone());
                i += 1;
            } else if key == "agents" || key == "tools" || key == "agent" {
                p.flags.insert(key.to_string(), String::new());
            } else {
                p.bools.insert(key.to_string(), true);
            }
        } else if a.starts_with("-j") {
            if a.len() > 2 {
                p.flags.insert("jobs".to_string(), a[2..].to_string());
            } else if i + 1 < argv.len() && !argv[i + 1].starts_with('-') {
                p.flags.insert("jobs".to_string(), argv[i + 1].clone());
                i += 1;
            } else {
                p.bools.insert("parallel".to_string(), true);
            }
        } else if a.starts_with('-') && a.len() == 2 {
            p.bools.insert(a[1..].to_string(), true);
        } else if p.cmd.is_empty() {
            p.cmd = a.clone();
        }
        i += 1;
    }
    p
}

fn help_text() -> String {
    let cy = |s: &str| C.cyan(s);
    let yl = |s: &str| C.yellow(s);
    let gr = |s: &str| C.gray(s);
    let bd = |s: &str| C.bold(s);

    format!(
        "{title} — High-Performance Headroom Proxy, Token Optimizer & Multi-Protocol AI Gateway\n\
        {tagline}\n\n\
        {sec_usage}\n\
          {cmd_init}              Install + wire all agents and tools (default; safe to re-run)\n\
          {cmd_update}            Update Toksi CLI, inspect version diff, and upgrade tools\n\
          {cmd_doctor}            Audit 17 agents, purge broken MCP runners, fix configuration regressions\n\
          {cmd_gain}              Display ground-truth token savings & ROI metrics across tools and models\n\
          {cmd_top}               Live interactive terminal monitor for proxies and active agent requests\n\
          {cmd_tui}               Full mouse-capable multi-tab interactive terminal control center\n\
          {cmd_dashboard}         Embedded web telemetry & control cockpit (--port, --open)\n\
          {cmd_prime}             Generate unified context primer for AI agent sessions\n\
          {cmd_daemon}            Unified 5-port background supervisor with auto-healing network watchdog\n\
          {cmd_info}              Show installation details, paths, config locations, and database status\n\
          {cmd_index}             Build per-project AST code graph index in current directory\n\
          {cmd_disable}           Disable agent hooks and proxy routing without removing files\n\
          {cmd_uninstall}         Remove everything Toksi ever touched, hooks, and environment variables\n\
          {cmd_rtk}               Execute command with token-reduction filters and ANSI stripping\n\
          {cmd_mem}               ProjectMem persistent memory, failure prevention, and MCP server\n\
          {cmd_ctx}               Context-mode isolated sandbox execution and SQLite FTS5 chunker\n\
          {cmd_graph}             CodeGraph Tree-sitter AST call-graph, blast-radius, and code explorer\n\
          {cmd_icm}               Instant Context Memory (ICM) SQLite FTS5 BM25 knowledge engine\n\
          {cmd_handoff}           Multi-agent claim-once context handoffs across 17+ agents\n\
          {cmd_rules}             Manage in-tree coding rules & skills (principles, caveman, ponytail)\n\
          {cmd_proxy}             Manage streaming Headroom and Copilot proxy supervisors: up|down|status\n\
          {cmd_auth}              OS Keyring credential manager: status|get|set|delete|sync\n\
          {cmd_db}                SQLite config database: status|export|import|backup|restore\n\
          {cmd_agent}             Agent directory lifecycle: install|health|mismatch|upgrade|fix|restore|remove|edit\n\
          {cmd_self_update}       In-place binary self-update to latest release\n\n\
        {sec_ports}\n\
          {p8787}           Streaming Headroom Proxy (Antigravity, Grok, and IDE base URL overrides)\n\
          {p8789}           GitHub Copilot VS Code Proxy (DEFAULT_COPILOT_PROXY_PORT)\n\
          {p8790}           GitHub Copilot CLI Proxy (DEFAULT_COPILOT_CLI_PROXY_PORT)\n\
          {p9099}           Enterprise AI Gateway & Drop-in SDKs (/v1/chat/completions, /v1/messages, /metrics)\n\
          {p9100}           Web Telemetry & Control Dashboard Cockpit (non-blocking background service)\n\n\
        {sec_agents}\n\
          claude, opencode, opencode2, codex, antigravity, copilot, cursor, droid, grok,\n\
          pi, omp, kilo, cline, amp, kimi, zcode, dsh, openagents\n\n\
        {sec_tools}\n\
          rtk, codegraph, context, projectmem, icm, aimemory, headroom\n\n\
        {sec_flags}\n\
          --agents <list>     Limit to a subset of agents (comma-separated)\n\
          --tools <list>      Limit to a subset of tools (comma-separated)\n\
          -j, --jobs <n>      Number of parallel threads for agent wiring\n\
          --dry-run           Show proposed changes without writing anything to disk\n\
          --verbose           Show detailed execution steps and debug information\n\
          -v, --version       Show version and build metadata\n\
          -h, --help          Show this comprehensive usage help\n\n\
        {sec_disciplines}\n\
          * Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced crate-wide.\n\
          * Zero Panics Policy: typed domain errors via thiserror; no panics in production.\n\
          * Universal Forward Slashes (/): normalized across Windows, Linux, and macOS.\n\
          * Ground-Truth Token Accounting: real tokens parsed from live SSE streams; 0 when idle.\n\
          * Network Reconnect Healing: auto-recovers from connection drops and VPN toggles.\n\n\
        {footer}",
        title = bd(&cy("toksi")),
        tagline = gr("Slashes token consumption by 30-70% for AI coding agents via prompt compression, AST indexing & memory"),
        sec_usage = bd("Commands:"),
        cmd_init = cy("toksi [init]"),
        cmd_update = cy("toksi update"),
        cmd_doctor = cy("toksi doctor"),
        cmd_gain = cy("toksi gain"),
        cmd_top = cy("toksi top"),
        cmd_tui = cy("toksi tui"),
        cmd_dashboard = cy("toksi dashboard"),
        cmd_prime = cy("toksi prime"),
        cmd_daemon = cy("toksi daemon"),
        cmd_info = cy("toksi info"),
        cmd_index = cy("toksi index"),
        cmd_disable = cy("toksi disable"),
        cmd_uninstall = cy("toksi uninstall"),
        cmd_rtk = cy("toksi rtk"),
        cmd_mem = cy("toksi mem"),
        cmd_ctx = cy("toksi ctx"),
        cmd_graph = cy("toksi graph"),
        cmd_icm = cy("toksi icm"),
        cmd_handoff = cy("toksi handoff"),
        cmd_rules = cy("toksi rules"),
        cmd_proxy = cy("toksi proxy"),
        cmd_auth = cy("toksi auth"),
        cmd_db = cy("toksi db"),
        cmd_agent = cy("toksi agent"),
        cmd_self_update = cy("toksi self-update"),
        sec_ports = bd("Dedicated 5-Port Network Topology:"),
        p8787 = yl("Port 8787"),
        p8789 = yl("Port 8789"),
        p8790 = yl("Port 8790"),
        p9099 = yl("Port 9099"),
        p9100 = yl("Port 9100"),
        sec_agents = bd("Supported Agents (17):"),
        sec_tools = bd("Managed Tools:"),
        sec_flags = bd("Flags & Options:"),
        sec_disciplines = bd("Disciplinary Standards & Architecture:"),
        footer = gr("Docs: https://github.com/Navegos/neo/#toksi")
    )
}

fn parse_list(raw: Option<&String>, allowed: &[String]) -> Result<Option<Vec<String>>, String> {
    let s = match raw {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(None),
    };
    let mut items = Vec::new();
    let mut invalid = Vec::new();
    for item in s.split(',') {
        let trimmed = item.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if allowed.contains(&trimmed) {
            items.push(trimmed);
        } else {
            invalid.push(trimmed);
        }
    }
    if !invalid.is_empty() {
        return Err(format!("Invalid value(s): {}. Allowed: {}", invalid.join(", "), allowed.join(", ")));
    }
    if items.is_empty() {
        return Err("value must not be empty".into());
    }
    Ok(Some(items))
}

fn is_session_boot_arg(args: &[String]) -> bool {
    match args {
        [cmd, ..] if cmd == "run-mcp" || cmd == "rtk-hook" || cmd == "grok-hook" || cmd == "codex-perm"
            || cmd == "agy-hook" || cmd == "cursor-hook" || cmd == "copilot-hook"
            || cmd == "projectmem-hook" || cmd == "ctx-hook" || cmd == "context-hook" || cmd == "context-mode-hook"
            || cmd == "icm-hook" => true,
        [a, b, ..] if (a == "rtk" || a == "ctx" || a == "context" || a == "context-mode" || a == "icm") && b == "hook" => true,
        [a, b, ..] if a == "mem" && b == "mcp" => true,
        [a, b, ..] if (a == "ctx" || a == "context" || a == "context-mode") && b == "mcp" => true,
        [a, b, ..] if a == "graph" && b == "mcp" => true,
        [a, b, ..] if (a == "icm" || a == "handoff" || a == "aimemory") && b == "mcp" => true,
        _ => false,
    }
}

fn main() {
    let code = run();
    process::exit(code);
}

fn run() -> i32 {
    agents::register();
    tools::register();
    ensure_process_path();

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "__proxy-ensure" => {
                if !proxy_routing_enabled() {
                    return 1;
                }
                if let Err(e) = start_proxy() {
                    L.err(&e.to_string());
                    return 1;
                }
                return 0;
            }
            "__byok-gateway-serve" => {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(r) => r,
                    Err(e) => {
                        L.err(&e.to_string());
                        return 1;
                    }
                };
                if let Err(e) = rt.block_on(run_byok_gateway_serve()) {
                    L.err(&e.to_string());
                    return 1;
                }
                return 0;
            }
            "__grok-proxy-owned" => {
                return if grok_oauth_proxy_owned() { 0 } else { 1 };
            }
            "__proxy-run" => {
                if let Err(e) = run_proxy_foreground() {
                    L.err(&e.to_string());
                    return 1;
                }
                return 0;
            }
            "__proxy-watch" => {
                if let Err(e) = run_proxy_watchdog() {
                    L.err(&e.to_string());
                    return 1;
                }
                return 0;
            }
            "__proxy-stop" => {
                if let Err(e) = stop_proxy() {
                    L.err(&e.to_string());
                    return 1;
                }
                return 0;
            }
            "__copilot" => {
                return 0;
            }
            _ => {}
        }
    }

    if is_session_boot_arg(&args[1..]) {
        if args.len() >= 4 && args[1] == "copilot-hook" && args[2] == "codegraph-index" && args[3] == "--vscode" {
            let _ = ensure_copilot_proxy_up();
        } else {
            let _ = ensure_proxy_up();
        }
    }

    if args.len() >= 3 && args[1] == "run-mcp" {
        return run_mcp(&args[2..]);
    }
    if args.len() >= 3 && args[1] == "rtk-rewrite" {
        return run_rtk_rewrite();
    }
    if args.len() >= 3 && args[1] == "rtk-hook" {
        return match args[2].as_str() {
            "agy" => run_rtk_hook(),
            "codex" | "claude" | "omp" | "zcode" => run_rtk_hook_codex(),
            "dsh" | "cmd" | "kimi" => run_rtk_hook_dsh(),
            "copilot" => run_rtk_hook_copilot(),
            "droid" => run_rtk_hook_droid(),
            "cline" => run_rtk_hook_cline(),
            "grok" => run_rtk_hook_grok(),
            "cursor" => run_rtk_hook_cursor(),
            _ => 1,
        };
    }
    if args.len() >= 3 && args[1] == "projectmem-hook" {
        return run_projectmem_hook(&args[2]);
    }
    if args.len() >= 3 && args[1] == "icm-hook" {
        return run_icm_hook(&args[2]);
    }
    if args.len() >= 3 && args[1] == "icm" && args[2] == "hook" {
        let agent = args.get(3).map(|s| s.as_str()).unwrap_or("generic");
        return run_icm_hook(agent);
    }
    if args.len() >= 2 && (args[1] == "ctx-hook" || args[1] == "context-hook" || args[1] == "context-mode-hook") {
        return run_ctx_hook(&args[2..]);
    }
    if args.len() >= 3 && (args[1] == "ctx" || args[1] == "context" || args[1] == "context-mode") && args[2] == "hook" {
        return run_ctx_hook(&args[3..]);
    }
    if args.len() >= 3 && args[1] == "grok-hook" && args[2] == "session-start" {
        return run_grok_session_start_hook();
    }
    if args.len() >= 2 && args[1] == "rtk" {
        if args.len() >= 4 && args[2] == "hook" && args[3] == "cursor" {
            return run_rtk_hook_cursor();
        }
        return run_rtk_cli(&args[2..]);
    }
    if args.len() >= 3 && args[1] == "codex-perm" && args[2] == "codex" {
        return run_codex_perm_hook();
    }
    if args.len() >= 2 && args[1] == "codegraph-hook" {
        return run_codegraph_index_hook();
    }
    if args.len() >= 3 && args[1] == "codegraph" && args[2] == "hook" {
        return run_codegraph_index_hook();
    }
    if args.len() >= 3 && (args[1] == "agy-hook" || args[1] == "cursor-hook" || args[1] == "copilot-hook") && args[2] == "codegraph-index" {
        return run_codegraph_index_hook();
    }
    if args.len() >= 3 && args[1] == "cursor-hook" && args[2] == "project-rules" {
        return run_cursor_project_rules_hook();
    }
    if args.len() >= 2 && args[1] == "proxy" {
        return run_proxy_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "daemon" {
        return run_daemon_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "mem" {
        return toksi::projectmem::run_mem_cli(&args[2..]);
    }
    if args.len() >= 2 && (args[1] == "ctx" || args[1] == "context" || args[1] == "context-mode") {
        return toksi::contextmode::run_ctx_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "graph" {
        return toksi::codegraph::run_graph_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "icm" {
        return toksi::icm::run_icm_cli(&args[2..]);
    }
    if args.len() >= 2 && (args[1] == "handoff" || args[1] == "aimemory") {
        return toksi::aimemory::run_handoff_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "tui" {
        return run_tui_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "dashboard" {
        return toksi::dashboard::run_dashboard_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "rules" {
        return toksi::rules::run_rules_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "auth" {
        return toksi::commands::run_auth_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "db" {
        return toksi::commands::run_db_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "agent" {
        return toksi::commands::run_agent_cli(&args[2..]);
    }

    let p = parse_args(&args[1..]);
    if p.bools.get("verbose") == Some(&true) {
        set_verbose(true);
    }
    if p.bools.get("version") == Some(&true) || p.bools.get("v") == Some(&true) || p.cmd == "version" {
        println!("{}", toksi_version());
        return 0;
    }
    if p.bools.get("help") == Some(&true) || p.cmd == "help" {
        println!("{}", help_text());
        return 0;
    }

    let agent_raw = p.flags.get("agents");
    let tool_raw = p.flags.get("tools");
    let agent_list = match parse_list(agent_raw, &agent_ids()) {
        Ok(l) => l,
        Err(e) => {
            L.err(&e);
            return 2;
        }
    };
    let tool_list = match parse_list(tool_raw, &tool_ids()) {
        Ok(l) => l,
        Err(e) => {
            L.err(&e);
            return 2;
        }
    };

    let command = if p.cmd.is_empty() { "init" } else { p.cmd.as_str() };
    let jobs = p.flags.get("jobs").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let parallel = p.bools.get("parallel") == Some(&true) || jobs > 1;
    let opts = InitOptions {
        agents: agent_list,
        tools: tool_list,
        agent: p.flags.get("agent").map(|s| s.trim().to_lowercase()).unwrap_or_default(),
        yes: p.bools.get("yes") == Some(&true),
        dry_run: p.bools.get("dry-run") == Some(&true) || p.bools.get("dryrun") == Some(&true),
        verbose: p.bools.get("verbose") == Some(&true),
        jobs,
        parallel,
        ..Default::default()
    };

    match command {
        "init" => run_init(opts),
        "update" => run_update(opts),
        "doctor" => run_doctor(
            p.bools.get("offline") == Some(&true),
            p.bools.get("json") == Some(&true),
            p.bools.get("fix") == Some(&true),
        ),
        "stats" => run_doctor(true, true, false),
        "gain" => run_gain(),
        "top" => run_top(p.bools.get("once") == Some(&true) || p.bools.get("1") == Some(&true)),
        "tui" => run_tui_cli(&args[2..]),
        "dashboard" => toksi::dashboard::run_dashboard_cli(&args[2..]),
        "prime" => run_prime(PrimeOptions {
            copy: p.bools.get("copy") == Some(&true) || p.bools.get("c") == Some(&true),
            json: p.bools.get("json") == Some(&true),
            output_file: p.flags.get("output").or_else(|| p.flags.get("o")).cloned(),
        }),
        "daemon" => run_daemon_cli(&args[2..]),
        "info" => run_info(),
        "index" => run_index(opts, p.bools.get("auto") == Some(&true)),
        "disable" => run_disable(opts),
        "uninstall" => run_uninstall(opts),
        "rtk" => run_rtk_cli(&args[2..]),
        "mem" => toksi::projectmem::run_mem_cli(&args[2..]),
        "ctx" => toksi::contextmode::run_ctx_cli(&args[2..]),
        "graph" => toksi::codegraph::run_graph_cli(&args[2..]),
        "icm" => toksi::icm::run_icm_cli(&args[2..]),
        "handoff" | "aimemory" => toksi::aimemory::run_handoff_cli(&args[2..]),
        "rules" => toksi::rules::run_rules_cli(&args[2..]),
        "auth" => toksi::commands::run_auth_cli(&args[2..]),
        "db" => toksi::commands::run_db_cli(&args[2..]),
        "agent" => toksi::commands::run_agent_cli(&args[2..]),
        "self-update" => run_self_update(p.bools.get("check") == Some(&true)),
        _ => {
            println!("{}", help_text());
            L.err(&format!("Unknown command: {}", command));
            1
        }
    }
}
