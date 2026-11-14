// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/agent.rs
// created: 2026-11-09
// lastModified: 2026-11-14

#![deny(unsafe_code)]

use crate::agents::manager::{AgentHealthReport, AgentManager, AgentMismatchReport};
use crate::util::colors::C;
use crate::util::paths::normalize_path;

pub fn run_agent_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        print_agent_help();
        return 0;
    }

    let subcmd = args[0].as_str();
    let rest = &args[1..];

    match subcmd {
        "install" => handle_install(rest),
        "health" => handle_health(rest),
        "mismatch" => handle_mismatch(rest),
        "upgrade" => handle_upgrade(rest),
        "fix" => handle_fix(rest),
        "restore" => handle_restore(rest),
        "remove" => handle_remove(rest),
        "edit" => handle_edit(rest),
        "list" => handle_list(),
        "-h" | "--help" | "help" => {
            print_agent_help();
            0
        }
        unknown => {
            eprintln!("Unknown agent subcommand: '{}'", unknown);
            print_agent_help();
            1
        }
    }
}

fn print_agent_help() {
    println!("{}", C.bold(&C.cyan("toksi agent — Agent Configuration & Lifecycle Management")));
    println!("Manage vendor and open agent directory protocols and harnesses.\n");
    println!("{}", C.bold("Usage:"));
    println!("  toksi agent <command> [agent] [flags]\n");
    println!("{}", C.bold("Commands:"));
    println!("  install <agent>     Install agent harness (default: user home; --workspace for local repo)");
    println!("  health [agent]      Audit configuration health, missing tools, and hook state");
    println!("  mismatch [agent]    Check differences between user and workspace configurations");
    println!("  upgrade [agent]     Upgrade agent hooks and sync newest rules/skills");
    println!("  fix [agent]         Repair configuration regressions and purge legacy runners");
    println!("  restore <agent>     Restore pristine default configuration or existing backup");
    println!("  remove <agent>      Cleanly remove Toksi configuration from agent directory");
    println!("  edit <agent>        Print primary configuration path for direct editing");
    println!("  list                List all supported agents and directory layouts\n");
    println!("{}", C.bold("Supported Agents:"));
    println!("  openagents, antigravity, copilot, claude, codex, cursor, grok, droid, cline, opencode...\n");
    println!("{}", C.bold("Flags:"));
    println!("  --workspace, -w     Operate strictly on current workspace directory instead of user home");
    println!("  --json              Output raw JSON report");
}

fn handle_list() -> i32 {
    let agents = AgentManager::all_managed_agents();
    println!("{}", C.bold("Supported Agents and Directory Layouts:"));
    for id in agents {
        let user_dir = AgentManager::user_config_dir(&id);
        let ws_dir = AgentManager::workspace_config_dir(&id, std::path::Path::new("."));
        println!(
            "  * {:<18} User: {:<32} Workspace: {}",
            C.cyan(&id),
            normalize_path(&user_dir.to_string_lossy()),
            normalize_path(&ws_dir.to_string_lossy())
        );
    }
    0
}

fn handle_install(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Error: Missing required agent argument. Example: 'toksi agent install antigravity'");
        return 1;
    }

    let agent_id = &args[0];
    let workspace = args.iter().any(|a| a == "--workspace" || a == "-w");

    match AgentManager::install(agent_id, workspace) {
        Ok(res) => {
            println!(
                "{} Successfully installed configuration for {}: {}",
                C.green("✓"),
                C.cyan(&res.agent),
                res.target_path
            );
            for d in res.details {
                println!("  - {}", d);
            }
            0
        }
        Err(e) => {
            eprintln!("{} Installation failed for {}: {}", C.red("✗"), agent_id, e);
            1
        }
    }
}

fn handle_health(args: &[String]) -> i32 {
    let json_output = args.iter().any(|a| a == "--json");
    let target_agents = if !args.is_empty() && !args[0].starts_with('-') {
        vec![args[0].clone()]
    } else {
        AgentManager::all_managed_agents()
    };

    let mut reports: Vec<AgentHealthReport> = Vec::new();
    for agent_id in target_agents {
        match AgentManager::health(&agent_id) {
            Ok(rep) => reports.push(rep),
            Err(e) => eprintln!("Failed to check health for {}: {}", agent_id, e),
        }
    }

    if json_output {
        if let Ok(json_str) = serde_json::to_string_pretty(&reports) {
            println!("{}", json_str);
            return 0;
        }
    }

    println!("{}", C.bold("Agent Configuration Health Audit:"));
    for rep in reports {
        let status_mark = if rep.installed && rep.mcp_configured {
            C.green("✓ OK")
        } else if rep.installed {
            C.yellow("! WARN")
        } else {
            C.gray("○ NOT INSTALLED")
        };

        println!(
            "  [{}] {:<18} ({}) -> {}",
            status_mark,
            C.cyan(&rep.id),
            rep.label,
            rep.config_dir
        );

        if !rep.issues.is_empty() {
            for issue in rep.issues {
                println!("      {}", C.yellow(&format!("↳ {}", issue)));
            }
        }
    }

    0
}

fn handle_mismatch(args: &[String]) -> i32 {
    let json_output = args.iter().any(|a| a == "--json");
    let target_agents = if !args.is_empty() && !args[0].starts_with('-') {
        vec![args[0].clone()]
    } else {
        AgentManager::all_managed_agents()
    };

    let mut reports: Vec<AgentMismatchReport> = Vec::new();
    for agent_id in target_agents {
        match AgentManager::mismatch(&agent_id) {
            Ok(rep) => reports.push(rep),
            Err(e) => eprintln!("Failed to inspect mismatch for {}: {}", agent_id, e),
        }
    }

    if json_output {
        if let Ok(json_str) = serde_json::to_string_pretty(&reports) {
            println!("{}", json_str);
            return 0;
        }
    }

    println!("{}", C.bold("Agent Configuration Mismatch Analysis:"));
    for rep in reports {
        let has_mismatch = !rep.missing_hooks.is_empty() || rep.outdated_rules;
        let mark = if has_mismatch { C.yellow("!") } else { C.green("✓") };
        println!(
            "  [{}] {:<14} User: {} | Workspace: {}",
            mark,
            C.cyan(&rep.id),
            rep.user_dir,
            rep.workspace_dir.as_deref().unwrap_or("none")
        );
        if !rep.missing_hooks.is_empty() {
            println!("      Missing hooks: {}", rep.missing_hooks.join(", "));
        }
        if rep.outdated_rules {
            println!("      Outdated rules or skills in user directory");
        }
    }

    0
}

fn handle_upgrade(args: &[String]) -> i32 {
    let target_agents = if !args.is_empty() && !args[0].starts_with('-') {
        vec![args[0].clone()]
    } else {
        AgentManager::all_managed_agents()
    };

    for agent_id in target_agents {
        match AgentManager::upgrade(&agent_id) {
            Ok(res) => {
                println!("{} Upgraded {}: {}", C.green("✓"), C.cyan(&agent_id), res.target_path);
                for d in res.details {
                    println!("  - {}", d);
                }
            }
            Err(e) => eprintln!("{} Upgrade failed for {}: {}", C.red("✗"), agent_id, e),
        }
    }
    0
}

fn handle_fix(args: &[String]) -> i32 {
    let workspace = args.iter().any(|a| a == "--workspace" || a == "-w");
    let ws_root = if workspace {
        Some(std::path::PathBuf::from("."))
    } else {
        None
    };
    let target_agents = if !args.is_empty() && !args[0].starts_with('-') {
        vec![args[0].clone()]
    } else {
        AgentManager::all_managed_agents()
    };

    for agent_id in target_agents {
        match AgentManager::fix_with_workspace(&agent_id, ws_root.as_deref()) {
            Ok(res) => {
                println!("{} Repaired {}: {}", C.green("✓"), C.cyan(&agent_id), normalize_path(&res.target_path));
                for d in res.details {
                    println!("  - {}", d);
                }
            }
            Err(e) => eprintln!("{} Repair failed for {}: {}", C.red("✗"), agent_id, e),
        }
    }
    0
}

fn handle_restore(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Error: Missing required agent argument. Example: 'toksi agent restore antigravity'");
        return 1;
    }
    let agent_id = &args[0];
    match AgentManager::restore(agent_id) {
        Ok(res) => {
            println!("{} Restored {}: {}", C.green("✓"), C.cyan(agent_id), res.target_path);
            for d in res.details {
                println!("  - {}", d);
            }
            0
        }
        Err(e) => {
            eprintln!("{} Restore failed for {}: {}", C.red("✗"), agent_id, e);
            1
        }
    }
}

fn handle_remove(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Error: Missing required agent argument. Example: 'toksi agent remove copilot'");
        return 1;
    }
    let agent_id = &args[0];
    let workspace = args.iter().any(|a| a == "--workspace" || a == "-w");
    match AgentManager::remove(agent_id, workspace) {
        Ok(res) => {
            println!("{} Removed {}: {}", C.green("✓"), C.cyan(agent_id), res.target_path);
            for d in res.details {
                println!("  - {}", d);
            }
            0
        }
        Err(e) => {
            eprintln!("{} Remove failed for {}: {}", C.red("✗"), agent_id, e);
            1
        }
    }
}

fn handle_edit(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Error: Missing required agent argument. Example: 'toksi agent edit claude'");
        return 1;
    }
    let agent_id = &args[0];
    match AgentManager::edit(agent_id) {
        Ok(path) => {
            println!("Configuration file for {}:\n  {}", C.cyan(agent_id), path);
            0
        }
        Err(e) => {
            eprintln!("{} Failed to locate config for {}: {}", C.red("✗"), agent_id, e);
            1
        }
    }
}
