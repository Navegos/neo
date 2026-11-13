// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/cli.rs
// created: 2026-10-31
// lastModified: 2026-10-31

use crate::aimemory::default_handoffs_dir;
use crate::aimemory::handoff::{HandoffStore, NewHandoff};

pub fn print_handoff_help() {
    println!(
        "toksi handoff — Multi-agent claim-once context handoffs across 17+ agents\n\n\
        Usage:\n\
          toksi handoff offer <title> --task <task> --from <agent> [--to <agent>] [--files <list>] [--unresolved <list>] [--next <list>] [--notes <text>]\n\
          toksi handoff claim <id> --agent <agent>\n\
          toksi handoff resolve <id> --agent <agent> [--note <text>]\n\
          toksi handoff cancel <id> [--reason <text>]\n\
          toksi handoff list [--agent <agent>]\n\
          toksi handoff show <id>\n"
    );
}

pub fn run_handoff_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        print_handoff_help();
        return 0;
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    let store = HandoffStore::new(default_handoffs_dir());

    match cmd {
        "help" | "-h" | "--help" => {
            print_handoff_help();
            0
        }
        "list" => {
            let mut agent_filter = None;
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--agent" && i + 1 < rest.len() {
                    agent_filter = Some(rest[i + 1].as_str());
                    i += 2;
                } else {
                    i += 1;
                }
            }

            match store.list_active() {
                Ok(handoffs) => {
                    let filtered: Vec<_> = handoffs
                        .into_iter()
                        .filter(|h| {
                            if let Some(a) = agent_filter {
                                h.to_agent.as_deref() == Some(a) || h.to_agent.as_deref() == Some("any") || h.to_agent.is_none()
                            } else {
                                true
                            }
                        })
                        .collect();

                    println!("=== Active Handoffs ({}) ===", filtered.len());
                    for h in filtered {
                        println!(
                            "  [{}] {} (state: {}, from: {}, to: {})",
                            h.id,
                            h.title,
                            h.state.as_str(),
                            h.from_agent,
                            h.to_agent.as_deref().unwrap_or("any")
                        );
                        println!("      Task: {}", h.task);
                        if !h.active_files.is_empty() {
                            println!("      Files: {}", h.active_files.join(", "));
                        }
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error listing handoffs: {}", e);
                    1
                }
            }
        }
        "show" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi handoff show <id>");
                return 1;
            }
            let id = &rest[0];
            match store.get(id) {
                Ok(Some(h)) => {
                    println!("Handoff: {}", h.id);
                    println!("Title:   {}", h.title);
                    println!("Task:    {}", h.task);
                    println!("State:   {}", h.state.as_str());
                    println!("From:    {}", h.from_agent);
                    println!("To:      {}", h.to_agent.as_deref().unwrap_or("any"));
                    if let Some(ref c) = h.claimed_by {
                        println!("Claimed: {}", c);
                    }
                    if !h.active_files.is_empty() {
                        println!("Files:   {}", h.active_files.join(", "));
                    }
                    if !h.unresolved_issues.is_empty() {
                        println!("Open:    {}", h.unresolved_issues.join(", "));
                    }
                    if !h.next_steps.is_empty() {
                        println!("Next:    {}", h.next_steps.join(", "));
                    }
                    if let Some(ref notes) = h.context_notes {
                        println!("Notes:\n{}", notes);
                    }
                    0
                }
                Ok(None) => {
                    eprintln!("Handoff {} not found", id);
                    1
                }
                Err(e) => {
                    eprintln!("Error getting handoff: {}", e);
                    1
                }
            }
        }
        "offer" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi handoff offer <title> --task <task> --from <agent> [options]");
                return 1;
            }
            let mut title = String::new();
            let mut task = String::new();
            let mut from_agent = String::new();
            let mut to_agent = None;
            let mut active_files = Vec::new();
            let mut unresolved_issues = Vec::new();
            let mut next_steps = Vec::new();
            let mut context_notes = None;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--task" && i + 1 < rest.len() {
                    task = rest[i + 1].clone();
                    i += 2;
                } else if rest[i] == "--from" && i + 1 < rest.len() {
                    from_agent = rest[i + 1].clone();
                    i += 2;
                } else if rest[i] == "--to" && i + 1 < rest.len() {
                    to_agent = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--files" && i + 1 < rest.len() {
                    active_files = rest[i + 1].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    i += 2;
                } else if rest[i] == "--unresolved" && i + 1 < rest.len() {
                    unresolved_issues = rest[i + 1].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    i += 2;
                } else if rest[i] == "--next" && i + 1 < rest.len() {
                    next_steps = rest[i + 1].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    i += 2;
                } else if rest[i] == "--notes" && i + 1 < rest.len() {
                    context_notes = Some(rest[i + 1].clone());
                    i += 2;
                } else if title.is_empty() {
                    title = rest[i].clone();
                    i += 1;
                } else {
                    title.push(' ');
                    title.push_str(&rest[i]);
                    i += 1;
                }
            }

            if task.is_empty() || from_agent.is_empty() {
                eprintln!("Error: 'toksi handoff offer' requires <title>, --task <task>, and --from <agent>");
                return 1;
            }

            let new_h = NewHandoff {
                title,
                task,
                from_agent,
                to_agent,
                active_files,
                unresolved_issues,
                next_steps,
                context_notes,
            };

            match store.offer(new_h) {
                Ok(h) => {
                    println!("Offered handoff [{}] to {}", h.id, h.to_agent.as_deref().unwrap_or("any"));
                    0
                }
                Err(e) => {
                    eprintln!("Error offering handoff: {}", e);
                    1
                }
            }
        }
        "claim" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi handoff claim <id> --agent <agent>");
                return 1;
            }
            let id = &rest[0];
            let mut agent = String::new();
            let mut i = 1;
            while i < rest.len() {
                if rest[i] == "--agent" && i + 1 < rest.len() {
                    agent = rest[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if agent.is_empty() {
                eprintln!("Error: --agent <agent> required to claim handoff");
                return 1;
            }

            match store.claim(id, &agent) {
                Ok(h) => {
                    println!("Claimed handoff [{}] by agent '{}'", h.id, agent);
                    0
                }
                Err(e) => {
                    eprintln!("Error claiming handoff: {}", e);
                    1
                }
            }
        }
        "resolve" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi handoff resolve <id> --agent <agent> [--note <text>]");
                return 1;
            }
            let id = &rest[0];
            let mut agent = String::new();
            let mut note = None;
            let mut i = 1;
            while i < rest.len() {
                if rest[i] == "--agent" && i + 1 < rest.len() {
                    agent = rest[i + 1].clone();
                    i += 2;
                } else if rest[i] == "--note" && i + 1 < rest.len() {
                    note = Some(rest[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if agent.is_empty() {
                eprintln!("Error: --agent <agent> required to resolve handoff");
                return 1;
            }

            match store.resolve(id, &agent, note.as_deref()) {
                Ok(h) => {
                    println!("Resolved handoff [{}]", h.id);
                    0
                }
                Err(e) => {
                    eprintln!("Error resolving handoff: {}", e);
                    1
                }
            }
        }
        "cancel" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi handoff cancel <id> [--reason <text>]");
                return 1;
            }
            let id = &rest[0];
            let mut reason = None;
            let mut i = 1;
            while i < rest.len() {
                if rest[i] == "--reason" && i + 1 < rest.len() {
                    reason = Some(rest[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }

            match store.cancel(id, reason.as_deref()) {
                Ok(h) => {
                    println!("Cancelled handoff [{}]", h.id);
                    0
                }
                Err(e) => {
                    eprintln!("Error cancelling handoff: {}", e);
                    1
                }
            }
        }
        _ => {
            eprintln!("Unknown command 'toksi handoff {}'. Run 'toksi handoff help' for usage.", cmd);
            1
        }
    }
}

pub fn run_aimemory_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        println!("{}", crate::aimemory::slash::handle_slash_command("/ai-memory help", None));
        return 0;
    }
    if args[0] == "handoff" {
        return run_handoff_cli(&args[1..]);
    }
    let cmd = format!("/ai-memory {}", args.join(" "));
    let out = crate::aimemory::slash::handle_slash_command(&cmd, None);
    println!("{}", out);
    0
}

