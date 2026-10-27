// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/cli.rs
// created: 2026-10-27

use std::fs;

use super::heatmap::{compute_stats_for_root, format_score_text, generate_heatmap};
use super::mcp::run_mcp_server;
use super::precheck::{clear_snooze, run_precheck, set_snooze};
use super::schema::{resolve_event_ref, Event};
use super::store::{
    active_issue_marker, ai_instructions_path, append_event, clear_current_issue,
    current_issue_id, get_git_commit, initialize, issue_exists, latest_open_issue_within,
    next_issue_id, normalize_issue_id, plan_path, project_map_path, read_events, require_mem_dir,
    summary_path, write_current_issue,
};
use super::summary::regenerate_summary;

pub fn print_mem_help() {
    println!(
        "tokless mem — Project memory and failure prevention for humans and AI tools\n\n\
        Usage:\n\
          tokless mem init [--purpose <text>]\n\
          tokless mem log <text> [--at <file:line>]\n\
          tokless mem attempt <text> [--worked|--failed|--partial] [--at <file:line>] [--issue <id>] [--auto-issue]\n\
          tokless mem fix <text> [--at <file:line>] [--issue <id>]\n\
          tokless mem decision <text> [--at <file:line>] [--supersedes <id>]\n\
          tokless mem note <text> [--at <file:line>]\n\
          tokless mem precheck [--files <list>] [--working] [--level <info|warn|block>] [--snooze <duration>] [--unsnooze]\n\
          tokless mem show | summary\n\
          tokless mem map\n\
          tokless mem plan [idea]\n\
          tokless mem instructions\n\
          tokless mem regenerate\n\
          tokless mem stats | score [--format <text|json>]\n\
          tokless mem heatmap\n\
          tokless mem search <query> [--limit <n>]\n\
          tokless mem mcp\n"
    );
}

pub fn run_mem_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        print_mem_help();
        return 0;
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    match cmd {
        "help" | "-h" | "--help" => {
            print_mem_help();
            0
        }
        "mcp" => run_mcp_server(),
        "init" => {
            let mut purpose = None;
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--purpose" && i + 1 < rest.len() {
                    purpose = Some(rest[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            match initialize(None, purpose.as_deref()) {
                Ok(p) => {
                    println!("Initialized project memory at {}", p.display());
                    0
                }
                Err(e) => {
                    eprintln!("Error initializing project memory: {}", e);
                    1
                }
            }
        }
        "log" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless mem log' requires issue summary");
                return 1;
            }
            let mut text = String::new();
            let mut location = None;
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--at" && i + 1 < rest.len() {
                    location = Some(rest[i + 1].clone());
                    i += 2;
                } else if text.is_empty() {
                    text = rest[i].clone();
                    i += 1;
                } else {
                    text.push(' ');
                    text.push_str(&rest[i]);
                    i += 1;
                }
            }
            let events = match read_events(None) {
                Ok(evs) => evs,
                Err(e) => {
                    eprintln!("Error reading events: {}", e);
                    return 1;
                }
            };
            let issue_id = next_issue_id(&events);
            let mut event = Event::new("issue", &text);
            event.issue_id = Some(issue_id.clone());
            event.location = location;
            event.git_commit = get_git_commit(None);

            if let Err(e) = append_event(&mut event, None) {
                eprintln!("Error appending event: {}", e);
                return 1;
            }
            write_current_issue(&issue_id, None);
            let _ = regenerate_summary(None);
            println!("Logged issue #{}", issue_id);
            0
        }
        "attempt" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless mem attempt' requires attempt summary");
                return 1;
            }
            let mut text = String::new();
            let mut location = None;
            let mut outcome = "failed";
            let mut issue_arg = None;
            let mut auto_issue = false;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--worked" {
                    outcome = "worked";
                    i += 1;
                } else if rest[i] == "--failed" {
                    outcome = "failed";
                    i += 1;
                } else if rest[i] == "--partial" {
                    outcome = "partial";
                    i += 1;
                } else if rest[i] == "--auto-issue" {
                    auto_issue = true;
                    i += 1;
                } else if rest[i] == "--at" && i + 1 < rest.len() {
                    location = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--issue" && i + 1 < rest.len() {
                    issue_arg = Some(rest[i + 1].clone());
                    i += 2;
                } else if text.is_empty() {
                    text = rest[i].clone();
                    i += 1;
                } else {
                    text.push(' ');
                    text.push_str(&rest[i]);
                    i += 1;
                }
            }

            let events = match read_events(None) {
                Ok(evs) => evs,
                Err(e) => {
                    eprintln!("Error reading events: {}", e);
                    return 1;
                }
            };

            let mut issue_id: Option<String> = None;
            if let Some(arg) = issue_arg {
                if let Some(norm) = normalize_issue_id(Some(&arg)) {
                    if issue_exists(&events, &norm) {
                        issue_id = Some(norm);
                    } else {
                        eprintln!("Error: Issue #{} was not found.", arg);
                        return 1;
                    }
                }
            }

            if issue_id.is_none() {
                issue_id = active_issue_marker(&events, None)
                    .or_else(|| latest_open_issue_within(&events, 5));
            }

            if issue_id.is_none() {
                if !auto_issue {
                    eprintln!("Error: No active issue. Run `tokless mem log \"<summary>\"` first or pass --auto-issue.");
                    return 1;
                }
                let new_id = next_issue_id(&events);
                let mut auto_ev = Event::new("issue", &text);
                auto_ev.issue_id = Some(new_id.clone());
                auto_ev.location = location.clone();
                auto_ev.git_commit = get_git_commit(None);
                if let Err(e) = append_event(&mut auto_ev, None) {
                    eprintln!("Error creating parent issue: {}", e);
                    return 1;
                }
                write_current_issue(&new_id, None);
                issue_id = Some(new_id);
            }

            let iid = issue_id.unwrap();
            let mut event = Event::new("attempt", &text);
            event.issue_id = Some(iid.clone());
            event.outcome = Some(outcome.to_string());
            event.location = location;
            event.git_commit = get_git_commit(None);

            if let Err(e) = append_event(&mut event, None) {
                eprintln!("Error appending event: {}", e);
                return 1;
            }
            let _ = regenerate_summary(None);
            println!("Recorded {} attempt on #{}", outcome, iid);
            0
        }
        "fix" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless mem fix' requires fix summary");
                return 1;
            }
            let mut text = String::new();
            let mut location = None;
            let mut issue_arg = None;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--at" && i + 1 < rest.len() {
                    location = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--issue" && i + 1 < rest.len() {
                    issue_arg = Some(rest[i + 1].clone());
                    i += 2;
                } else if text.is_empty() {
                    text = rest[i].clone();
                    i += 1;
                } else {
                    text.push(' ');
                    text.push_str(&rest[i]);
                    i += 1;
                }
            }

            let events = match read_events(None) {
                Ok(evs) => evs,
                Err(e) => {
                    eprintln!("Error reading events: {}", e);
                    return 1;
                }
            };

            let issue_id = if let Some(arg) = issue_arg {
                match normalize_issue_id(Some(&arg)) {
                    Some(norm) if issue_exists(&events, &norm) => norm,
                    _ => {
                        eprintln!("Error: Issue #{} was not found.", arg);
                        return 1;
                    }
                }
            } else {
                match active_issue_marker(&events, None).or_else(|| current_issue_id(&events)) {
                    Some(id) => id,
                    None => {
                        eprintln!("Error: No open issue found. Run `tokless mem log <text>` first.");
                        return 1;
                    }
                }
            };

            let mut event = Event::new("fix", &text);
            event.issue_id = Some(issue_id.clone());
            event.location = location;
            event.git_commit = get_git_commit(None);

            if let Err(e) = append_event(&mut event, None) {
                eprintln!("Error appending event: {}", e);
                return 1;
            }

            if let Some(active) = active_issue_marker(&events, None) {
                if active == issue_id {
                    clear_current_issue(None);
                }
            }

            let _ = regenerate_summary(None);
            println!("Fixed issue #{}", issue_id);
            0
        }
        "decision" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless mem decision' requires summary");
                return 1;
            }
            let mut text = String::new();
            let mut location = None;
            let mut supersedes = None;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--at" && i + 1 < rest.len() {
                    location = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--supersedes" && i + 1 < rest.len() {
                    supersedes = Some(rest[i + 1].clone());
                    i += 2;
                } else if text.is_empty() {
                    text = rest[i].clone();
                    i += 1;
                } else {
                    text.push(' ');
                    text.push_str(&rest[i]);
                    i += 1;
                }
            }

            let events = match read_events(None) {
                Ok(evs) => evs,
                Err(e) => {
                    eprintln!("Error reading events: {}", e);
                    return 1;
                }
            };

            let mut superseded_id = None;
            if let Some(ref sup) = supersedes {
                match resolve_event_ref(&events, sup) {
                    Ok(old) => superseded_id = Some(old.id),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return 1;
                    }
                }
            }

            let mut event = Event::new("decision", &text);
            event.location = location;
            event.supersedes = superseded_id.clone();
            event.git_commit = get_git_commit(None);

            if let Err(e) = append_event(&mut event, None) {
                eprintln!("Error appending event: {}", e);
                return 1;
            }
            let _ = regenerate_summary(None);

            if let Some(sup) = superseded_id {
                println!("Recorded decision (supersedes {})", sup);
            } else {
                println!("Recorded decision");
            }
            0
        }
        "note" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless mem note' requires summary");
                return 1;
            }
            let mut text = String::new();
            let mut location = None;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--at" && i + 1 < rest.len() {
                    location = Some(rest[i + 1].clone());
                    i += 2;
                } else if text.is_empty() {
                    text = rest[i].clone();
                    i += 1;
                } else {
                    text.push(' ');
                    text.push_str(&rest[i]);
                    i += 1;
                }
            }

            let mut event = Event::new("note", &text);
            event.location = location;
            event.git_commit = get_git_commit(None);

            if let Err(e) = append_event(&mut event, None) {
                eprintln!("Error appending event: {}", e);
                return 1;
            }
            let _ = regenerate_summary(None);
            println!("Recorded note");
            0
        }
        "precheck" => {
            let mut working = false;
            let mut quiet = false;
            let mut level = None;
            let mut files = Vec::new();
            let mut snooze = None;
            let mut unsnooze = false;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--working" {
                    working = true;
                    i += 1;
                } else if rest[i] == "--quiet" {
                    quiet = true;
                    i += 1;
                } else if rest[i] == "--unsnooze" {
                    unsnooze = true;
                    i += 1;
                } else if rest[i] == "--snooze" && i + 1 < rest.len() {
                    snooze = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--level" && i + 1 < rest.len() {
                    level = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--files" {
                    i += 1;
                    while i < rest.len() && !rest[i].starts_with('-') {
                        files.push(rest[i].clone());
                        i += 1;
                    }
                } else {
                    files.push(rest[i].clone());
                    i += 1;
                }
            }

            if unsnooze {
                if clear_snooze(None) {
                    println!("projectmem: precheck warnings re-enabled.");
                } else {
                    println!("projectmem: no active snooze.");
                }
                return 0;
            }

            if let Some(dur) = snooze {
                match set_snooze(&dur, None) {
                    Ok(exp) => {
                        println!("projectmem: precheck warnings snoozed until {} (logged to memory).", exp.format("%H:%M UTC"));
                        return 0;
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return 1;
                    }
                }
            }

            let file_opt = if files.is_empty() { None } else { Some(files.as_slice()) };
            match run_precheck(file_opt, working, level.as_deref(), quiet, None) {
                Ok(w) => {
                    let has_block = w.iter().any(|item| item.severity == "block");
                    if has_block && level.as_deref() == Some("block") {
                        1
                    } else {
                        0
                    }
                }
                Err(e) => {
                    eprintln!("Error running precheck: {}", e);
                    1
                }
            }
        }
        "show" | "summary" => {
            let p = match summary_path(None) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return 1;
                }
            };
            if let Ok(content) = fs::read_to_string(p) {
                println!("{}", content);
                0
            } else {
                eprintln!("No summary found. Run `tokless mem init` first.");
                1
            }
        }
        "map" => {
            let p = match project_map_path(None) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return 1;
                }
            };
            if let Ok(content) = fs::read_to_string(p) {
                println!("{}", content);
                0
            } else {
                eprintln!("No project map found.");
                1
            }
        }
        "plan" => {
            let p = match plan_path(None) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return 1;
                }
            };
            if !rest.is_empty() {
                let idea = rest.join(" ");
                let mut existing = fs::read_to_string(&p).unwrap_or_default();
                let bullet = format!("- {}\n", idea);
                if let Some(pos) = existing.find("## Ideas\n") {
                    let insert_pos = pos + "## Ideas\n".len();
                    existing.insert_str(insert_pos, &bullet);
                } else {
                    existing.push_str("\n## Ideas\n");
                    existing.push_str(&bullet);
                }
                let _ = fs::write(&p, existing);
                println!("Appended idea to plan.md: {}", idea);
                return 0;
            }
            if let Ok(content) = fs::read_to_string(p) {
                println!("{}", content);
                0
            } else {
                eprintln!("No plan found.");
                1
            }
        }
        "instructions" => {
            let p = match ai_instructions_path(None) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return 1;
                }
            };
            if let Ok(content) = fs::read_to_string(p) {
                println!("{}", content);
                0
            } else {
                eprintln!("No instructions found. Run `tokless mem init` first.");
                1
            }
        }
        "regenerate" => {
            match regenerate_summary(None) {
                Ok(p) => {
                    println!("Regenerated summary at {}", p.display());
                    0
                }
                Err(e) => {
                    eprintln!("Error regenerating summary: {}", e);
                    1
                }
            }
        }
        "stats" | "score" => {
            let mut format_json = false;
            for arg in rest {
                if arg == "--json" || arg == "json" {
                    format_json = true;
                }
            }
            if rest.len() >= 2 && rest[0] == "--format" && rest[1] == "json" {
                format_json = true;
            }

            match compute_stats_for_root(None) {
                Ok(res) => {
                    if format_json {
                        if let Ok(json_str) = serde_json::to_string_pretty(&res) {
                            println!("{}", json_str);
                        }
                    } else {
                        print!("{}", format_score_text(&res));
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error computing score: {}", e);
                    1
                }
            }
        }
        "heatmap" => {
            let events = match read_events(None) {
                Ok(evs) => evs,
                Err(e) => {
                    eprintln!("Error reading events: {}", e);
                    return 1;
                }
            };
            let heatmap = generate_heatmap(&events);
            println!("========================================");
            println!("  PROJECTMEM FAILURE HEATMAP");
            println!("========================================");
            println!("  Score: {} ({}) | Tokens Saved: {}", heatmap.score.score, heatmap.score.grade, heatmap.score.tokens_saved);
            println!("----------------------------------------");
            println!("  Top Failure Hotspots:");
            if heatmap.top_hotspots.is_empty() {
                println!("    No failure hotspots recorded.");
            } else {
                for hs in heatmap.top_hotspots.iter().take(10) {
                    println!("    - {} ({} failed attempts, {} open issues, {} total events)", hs.file, hs.failed_attempts, hs.open_issues, hs.total_events);
                }
            }
            println!("----------------------------------------");
            println!("  Directory Clusters:");
            for (dir, count) in &heatmap.directory_clusters {
                println!("    - {}: {} events", dir, count);
            }
            println!("========================================");
            0
        }
        "search" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless mem search' requires query text");
                return 1;
            }
            let mut limit = 10;
            let mut query = String::new();
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--limit" && i + 1 < rest.len() {
                    limit = rest[i + 1].parse().unwrap_or(10);
                    i += 2;
                } else if query.is_empty() {
                    query = rest[i].clone();
                    i += 1;
                } else {
                    query.push(' ');
                    query.push_str(&rest[i]);
                    i += 1;
                }
            }

            let mem = match require_mem_dir(None) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return 1;
                }
            };
            let root = mem.parent().unwrap_or(&mem);
            let result = super::mcp::tool_search_events(root, &query, limit);
            println!("{}", result);
            0
        }
        _ => {
            eprintln!("Unknown command 'tokless mem {}'. Run 'tokless mem help' for usage.", cmd);
            1
        }
    }
}
