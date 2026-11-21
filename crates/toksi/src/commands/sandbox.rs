// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/sandbox.rs
// created: 2026-11-21

//! CLI handler for `toksi sandbox` — Docker Sandboxes (`sbx` / `sandboxd`) 29-verb parity
//! unified across 3-OS native, `sandbox-runtime`, `OpenShell`, `E2B`, and `sbx` backends.

#![deny(unsafe_code)]

use std::path::PathBuf;

use crate::agent_runtime::sandbox::{
    global_sandbox_manager, SandboxBackend, SkillsMountMode, SBX_BUILT_IN_AGENTS, SBX_VERBS,
};
use crate::config::load_toksi_toml_config;

fn flag_val<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].as_str());
        }
        if let Some(rest) = args[i].strip_prefix(&format!("{flag}=")) {
            return Some(rest);
        }
        i += 1;
    }
    None
}

pub fn run_sandbox_cli(args: &[String]) -> i32 {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        println!(
            "Usage: toksi sandbox <verb> [args...]\n\
             Verbs (29-verb sbx parity): {}\n\
             Agents: {}\n\
             Flags: --backend <auto|native|sandbox-runtime|openshell|e2b|sbx> --allow-network <pat> --skills <readonly|readwrite|off>",
            SBX_VERBS.join(" | "),
            SBX_BUILT_IN_AGENTS.join(" | ")
        );
        return 0;
    }

    let verb = args[0].as_str();
    if !SBX_VERBS.contains(&verb) {
        eprintln!("Unknown sandbox verb '{verb}'. Supported: {}", SBX_VERBS.join(", "));
        return 2;
    }

    let mgr = global_sandbox_manager();
    let cfg = load_toksi_toml_config();
    let backend = flag_val(args, "--backend")
        .map(SandboxBackend::from_str_loose)
        .unwrap_or_else(|| SandboxBackend::from_str_loose(&cfg.security.sandbox_backend));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut policy = mgr.policy();
    if let Some(net) = flag_val(args, "--allow-network") {
        policy.allow_network = !net.is_empty();
        policy.allowed_network_patterns.push(net.to_string());
    }
    if let Some(sk) = flag_val(args, "--skills") {
        policy.skills_mode = SkillsMountMode::from_str_loose(sk);
    }

    match verb {
        "ls" => {
            let list = mgr.list();
            println!("{}", serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".to_string()));
            0
        }
        "create" => {
            let agent = args.get(1).map(String::as_str).unwrap_or("shell");
            let name = flag_val(args, "--name").unwrap_or(agent);
            let inst = mgr.create_with_options(name, backend, &cwd, agent, policy);
            println!("{}", serde_json::to_string_pretty(&inst).unwrap_or_default());
            0
        }
        "run" | "exec" => {
            let target = args.get(1).map(String::as_str).unwrap_or("");
            if target.is_empty() {
                eprintln!("Usage: toksi sandbox {verb} <program|sandbox> [args...]");
                return 1;
            }
            let cmd_args: Vec<String> = args.iter().skip(2).cloned().collect();
            match mgr.run_command(target, &cmd_args, &cwd, Some(&policy)) {
                Ok(out) => {
                    if !out.stdout.is_empty() {
                        print!("{}", out.stdout);
                    }
                    if !out.stderr.is_empty() {
                        eprint!("{}", out.stderr);
                    }
                    out.exit_code
                }
                Err(e) => {
                    eprintln!("Sandbox {verb} error: {e}");
                    1
                }
            }
        }
        "inspect" => {
            let id = args.get(1).map(String::as_str).unwrap_or("default-sbx");
            match mgr.inspect(id) {
                Some(inst) => {
                    println!("{}", serde_json::to_string_pretty(&inst).unwrap_or_default());
                    0
                }
                None => {
                    eprintln!("Sandbox '{id}' not found.");
                    1
                }
            }
        }
        "stop" => {
            let id = args.get(1).map(String::as_str).unwrap_or("default-sbx");
            match mgr.stop(id) {
                Ok(inst) => {
                    println!("Stopped '{}' (status={})", inst.id, inst.status);
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            }
        }
        "rm" => {
            let id = args.get(1).map(String::as_str).unwrap_or("default-sbx");
            match mgr.remove(id) {
                Ok(()) => {
                    println!("Removed '{id}'.");
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            }
        }
        "prune" => {
            let removed = mgr.prune_stopped();
            println!("Pruned {removed} stopped sandbox(es).");
            0
        }
        "secret" => {
            let sub = args.get(1).map(String::as_str).unwrap_or("ls");
            match sub {
                "set" if args.len() >= 4 => {
                    mgr.set_secret(&args[2], &args[3]);
                    println!("Stored proxy-injected secret '{}'.", args[2]);
                    0
                }
                _ => {
                    println!("Proxy-injected secret keys: {:?}", mgr.list_secret_keys());
                    0
                }
            }
        }
        "policy" => {
            println!("{}", serde_json::to_string_pretty(&policy).unwrap_or_default());
            0
        }
        _ => {
            let res = mgr.dispatch_sbx_verb(verb, &args[1..]);
            println!("{res}");
            0
        }
    }
}
