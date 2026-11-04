// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/auth.rs
// created: 2026-11-04
// lastModified: 2026-11-04

use crate::keyring::{audit_keyring_statuses, current_backend_name, delete_credential, get_agent_token, get_credential, set_credential};
use crate::util::colors::C;

pub fn run_auth_cli(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");
    match sub {
        "status" => run_auth_status(),
        "get" => {
            if args.len() < 2 {
                eprintln!("Usage: toksi auth get <service|agent> [--account <name>] [--reveal]");
                return 1;
            }
            let service = &args[1];
            let mut account = "default";
            let mut reveal = false;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--account" && i + 1 < args.len() {
                    account = &args[i + 1];
                    i += 1;
                } else if args[i] == "--reveal" {
                    reveal = true;
                }
                i += 1;
            }
            run_auth_get(service, account, reveal)
        }
        "set" => {
            if args.len() < 3 {
                eprintln!("Usage: toksi auth set <service|agent> <secret> [--account <name>]");
                return 1;
            }
            let service = &args[1];
            let secret = &args[2];
            let mut account = "default";
            let mut i = 3;
            while i < args.len() {
                if args[i] == "--account" && i + 1 < args.len() {
                    account = &args[i + 1];
                    i += 1;
                }
                i += 1;
            }
            run_auth_set(service, account, secret)
        }
        "delete" | "rm" => {
            if args.len() < 2 {
                eprintln!("Usage: toksi auth delete <service|agent> [--account <name>]");
                return 1;
            }
            let service = &args[1];
            let mut account = "default";
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--account" && i + 1 < args.len() {
                    account = &args[i + 1];
                    i += 1;
                }
                i += 1;
            }
            run_auth_delete(service, account)
        }
        "sync" => run_auth_sync(),
        _ => {
            eprintln!("Unknown auth command: {}", sub);
            eprintln!("Usage: toksi auth [status|get|set|delete|sync]");
            1
        }
    }
}

fn run_auth_status() -> i32 {
    let backend = current_backend_name();
    println!("{}", C.bold(&format!("Toksi Secure Keyring Storage ({})", backend)));
    println!("{}", C.gray("----------------------------------------------------------------------"));
    println!("{:<14} {:<24} {:<10} {:<18} {:<10}", "Agent", "Target/Service", "Status", "Expires", "Preview");
    println!("{}", C.gray("----------------------------------------------------------------------"));

    let statuses = audit_keyring_statuses();
    for s in statuses {
        let status_str = if s.has_credential {
            if s.is_expired {
                C.yellow("Expired")
            } else {
                C.green("Configured")
            }
        } else {
            C.gray("Missing")
        };

        let exp_str = if let Some(exp) = s.expires_at {
            if let Some(dt) = chrono::DateTime::from_timestamp(exp, 0) {
                dt.format("%Y-%m-%d %H:%M").to_string()
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        };

        let preview = s.token_preview.unwrap_or_else(|| "-".to_string());
        println!("{:<14} {:<24} {:<10} {:<18} {:<10}", s.agent, s.service, status_str, exp_str, preview);
    }
    0
}

fn run_auth_get(service: &str, account: &str, reveal: bool) -> i32 {
    if let Some(env) = get_agent_token(service) {
        if reveal {
            println!("{}", env.access_token);
        } else {
            println!("Agent Token for '{}' (Backend: {})", service, current_backend_name());
            println!("  Type:    {}", env.token_type.as_deref().unwrap_or("Bearer"));
            println!("  Expired: {}", if env.is_expired(0) { C.red("Yes") } else { C.green("No") });
            if let Some(exp) = env.expires_at {
                if let Some(dt) = chrono::DateTime::from_timestamp(exp, 0) {
                    println!("  Expires: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                }
            }
            let tok = &env.access_token;
            let mask = if tok.len() > 10 {
                format!("{}...{}", &tok[..4], &tok[tok.len() - 4..])
            } else {
                "***".to_string()
            };
            println!("  Token:   {} (use --reveal to view full token)", mask);
        }
        return 0;
    }

    if let Some(secret) = get_credential(service, account) {
        if reveal {
            println!("{}", secret);
        } else {
            let mask = if secret.len() > 10 {
                format!("{}...{}", &secret[..4], &secret[secret.len() - 4..])
            } else {
                "***".to_string()
            };
            println!("Secret for '{}:{}': {} (use --reveal to view full token)", service, account, mask);
        }
        0
    } else {
        eprintln!("No credential found for service '{}' (account '{}')", service, account);
        1
    }
}

fn run_auth_set(service: &str, account: &str, secret: &str) -> i32 {
    if set_credential(service, account, secret) {
        println!("Successfully stored credential for '{}:{}' in {}", service, account, current_backend_name());
        0
    } else {
        eprintln!("Failed to store credential for '{}:{}'", service, account);
        1
    }
}

fn run_auth_delete(service: &str, account: &str) -> i32 {
    if delete_credential(service, account) {
        println!("Removed credential for '{}:{}' from {}", service, account, current_backend_name());
        0
    } else {
        eprintln!("Failed to remove credential for '{}:{}'", service, account);
        1
    }
}

fn run_auth_sync() -> i32 {
    println!("Scanning and syncing local agent session files to OS Keyring...");
    let mut count = 0;

    // 1. Antigravity / Gemini ADC
    if let Some(tok) = crate::gateway::agent_cli::read_gemini_adc_session() {
        if set_credential("gemini", "antigravity", &tok) {
            println!("  Synced Antigravity/Gemini token into Keyring");
            count += 1;
        }
    }

    // 2. Claude Code
    if let Some(tok) = crate::gateway::agent_cli::read_claude_code_session() {
        if set_credential("claude", "sessionKey", &tok) {
            println!("  Synced Claude Code token into Keyring");
            count += 1;
        }
    }

    // 3. GitHub Copilot
    if let Some(tok) = crate::gateway::agent_cli::read_copilot_session() {
        if set_credential("git:https://github.com", "oauth_token", &tok) {
            println!("  Synced Copilot token into Keyring");
            count += 1;
        }
    }

    println!("Sync complete. {} credential(s) verified in {}", count, current_backend_name());
    0
}
