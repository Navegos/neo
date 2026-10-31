// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/main.rs
// created: 2026-10-05
// lastModified: 2026-10-30

use std::collections::HashMap;
use std::env;
use std::process;

use tokless::agents;
use tokless::commands::{
    run_codegraph_index_hook, run_codex_perm_hook,
    run_cursor_project_rules_hook, run_daemon_cli, run_disable, run_doctor,
    run_grok_session_start_hook, run_index, run_info, run_init, run_mcp, run_prime,
    run_projectmem_hook, run_proxy_cli, run_rtk_cli, run_rtk_hook,
    run_rtk_hook_cline, run_rtk_hook_codex, run_rtk_hook_copilot, run_rtk_hook_cursor,
    run_rtk_hook_droid, run_rtk_hook_dsh, run_rtk_hook_grok, run_rtk_rewrite, run_self_update,
    run_top, run_tui_cli, run_uninstall, run_update, run_gain, InitOptions, PrimeOptions,
};

use tokless::core::{agent_ids, tool_ids};
use tokless::headroom::autostart::grok_oauth_proxy_owned;
use tokless::headroom::byok_gateway::run_byok_gateway_serve;
use tokless::headroom::proxy::{
    ensure_copilot_proxy_up, ensure_proxy_up, run_proxy_foreground, run_proxy_watchdog,
    start_proxy, stop_proxy,
};
use tokless::tools;
use tokless::util::colors::*;
use tokless::util::headroom_proxy::proxy_routing_enabled;
use tokless::util::logger::{set_verbose, L};
use tokless::util::paths::ensure_process_path;
use tokless::util::version::tokless_version;

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
    format!(
        "{} — token-saving for AI coding agents (Claude Code, OpenCode, Codex, Antigravity, GitHub Copilot, Cursor, Factory Droid, Grok, Pi, OMP, Kilo, Cline, Amp, Kimi, ZCode, DSH)\n\n\
        {}\n  {}              Install + wire everything (default; safe to re-run)\n\
          {}       Update the tokless CLI, then show version diff and upgrade tools\n\
          {}       Show what's wired up; warn about anything broken\n\
          {}         Display token savings dashboard across tools\n\
          {}          Live interactive terminal monitor for proxies and agents\n\
          {}          Full multi-tab interactive terminal control center\n\
          {}    Embedded web telemetry & control cockpit (--port, --open)\n\
          {}        Generate unified context primer for AI agent sessions\n\
          {}       Manage background proxy supervisor: start|stop|status\n\
          {}         Show how tokless was installed, plus paths and config locations\n\
          {}        Build per-project indexes (codegraph) in the current dir\n\
          {}    Remove everything tokless ever touched\n\
          {}          Manage project memory, failure prevention, and MCP server\n\
          {}          Isolated sandbox execution and SQLite FTS5 knowledge base\n\
          {}          Tree-sitter AST call-graph, blast-radius, and code explorer\n\
          {}          Execute command and apply token-reduction filters\n\
          {}      Manage the headroom HTTP proxy: up|down|status\n\n\
        {}\n  --agents <list>     Limit to a subset: claude,opencode,opencode2,codex,antigravity,copilot,cursor,droid,grok,pi,omp,kilo,cline,amp,kimi,zcode,dsh\n\
          --tools <list>      Limit to a subset: rtk,caveman,ponytail,codegraph,context-mode,headroom,projectmem\n\
          -j, --jobs <n>      Number of parallel threads for agent wiring\n\
          --dry-run           Show what would change without writing anything\n\
          --verbose           Show every step\n\n\
        {}",
        C.bold(&cy("tokless")),
        C.bold("Usage:"),
        cy("tokless"),
        cy("tokless update"),
        cy("tokless doctor"),
        cy("tokless gain"),
        cy("tokless top"),
        cy("tokless tui"),
        cy("tokless dashboard"),
        cy("tokless prime"),
        cy("tokless daemon"),
        cy("tokless info"),
        cy("tokless index"),
        cy("tokless uninstall"),
        cy("tokless mem"),
        cy("tokless ctx"),
        cy("tokless graph"),
        cy("tokless rtk"),
        cy("tokless proxy"),
        C.bold("Flags:"),
        C.gray("Docs: https://github.com/Navegos/neo/#tokless")
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
            || cmd == "agy-hook" || cmd == "cursor-hook" || cmd == "copilot-hook" => true,
        [a, b, ..] if a == "rtk" && b == "hook" => true,
        [a, b, ..] if a == "mem" && b == "mcp" => true,
        [a, b, ..] if a == "ctx" && b == "mcp" => true,
        [a, b, ..] if a == "graph" && b == "mcp" => true,
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
        return tokless::projectmem::run_mem_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "ctx" {
        return tokless::contextmode::run_ctx_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "graph" {
        return tokless::codegraph::run_graph_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "tui" {
        return run_tui_cli(&args[2..]);
    }
    if args.len() >= 2 && args[1] == "dashboard" {
        return tokless::dashboard::run_dashboard_cli(&args[2..]);
    }

    let p = parse_args(&args[1..]);
    if p.bools.get("verbose") == Some(&true) {
        set_verbose(true);
    }
    if p.bools.get("version") == Some(&true) || p.bools.get("v") == Some(&true) || p.cmd == "version" {
        println!("{}", tokless_version());
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
        "doctor" => run_doctor(p.bools.get("offline") == Some(&true), p.bools.get("json") == Some(&true)),
        "stats" => run_doctor(true, true),
        "gain" => run_gain(),
        "top" => run_top(p.bools.get("once") == Some(&true) || p.bools.get("1") == Some(&true)),
        "tui" => run_tui_cli(&args[2..]),
        "dashboard" => tokless::dashboard::run_dashboard_cli(&args[2..]),
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
        "mem" => tokless::projectmem::run_mem_cli(&args[2..]),
        "ctx" => tokless::contextmode::run_ctx_cli(&args[2..]),
        "graph" => tokless::codegraph::run_graph_cli(&args[2..]),
        "self-update" => run_self_update(p.bools.get("check") == Some(&true)),
        _ => {
            println!("{}", help_text());
            L.err(&format!("Unknown command: {}", command));
            1
        }
    }
}
