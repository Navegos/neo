// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rules/cli.rs

use std::path::{Path, PathBuf};
use crate::rules::{
    inject_rule_at_path, inspect_agent_rules, patch_agents_md,
    patch_all_agent_configs, patch_claude_md, remove_rule_from_path,
    RuleKind,
};
use crate::util::colors::*;

pub fn print_rules_help() {
    println!(
        "tokless rules — Manage in-tree coding rules & skills (principles, caveman, ponytail)\n\n\
        Usage:\n\
          tokless rules list\n\
          tokless rules show <rule> [--cursor] [--body]\n\
          tokless rules patch [target_dir_or_file] [--rules <r1,r2>]\n\
          tokless rules inject <file> <rule>\n\
          tokless rules remove <file> <rule>\n\
          tokless rules status [target_dir]\n\n\
        Rules:\n\
          principles  Karpathy guidelines: think first, simplify, surgical changes, verify\n\
          caveman     ASD-STE100 primitive English compression guidelines\n\
          ponytail    Lazy senior developer build discipline ladder (the Ladder)\n"
    );
}

pub fn run_rules_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        print_rules_help();
        return 0;
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    match cmd {
        "help" | "-h" | "--help" => {
            print_rules_help();
            0
        }
        "list" | "ls" => {
            println!("In-tree coding rules and guidelines compiled into tokless binary:\n");
            for rule in RuleKind::all() {
                println!(
                    "  {} {}\n    {}\n    Cursor rule: {}\n",
                    C.cyan(rule.as_str()),
                    C.bold(&format!("({})", rule.title())),
                    rule.description(),
                    rule.cursor_filename()
                );
            }
            0
        }
        "show" | "view" | "cat" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless rules show <rule>' requires a rule name (principles, caveman, ponytail)");
                return 1;
            }
            let rule_name = &rest[0];
            let rule = match RuleKind::from_str(rule_name) {
                Some(r) => r,
                None => {
                    eprintln!("Error: unknown rule '{}'. Available: principles, caveman, ponytail", rule_name);
                    return 1;
                }
            };

            let is_cursor = rest.iter().any(|a| a == "--cursor" || a == "-c");
            let is_body = rest.iter().any(|a| a == "--body" || a == "-b");

            if is_cursor {
                print!("{}", rule.cursor_rule());
            } else if is_body {
                println!("{}", rule.section_body());
            } else {
                print!("{}", rule.markdown());
            }
            0
        }
        "patch" => {
            let mut target_dir = PathBuf::from(".");
            let mut selected_rules = RuleKind::all().to_vec();

            let mut i = 0;
            while i < rest.len() {
                let arg = &rest[i];
                if arg == "--rules" || arg == "-r" {
                    if i + 1 < rest.len() {
                        let list_str = &rest[i + 1];
                        selected_rules.clear();
                        for item in list_str.split(',') {
                            if let Some(r) = RuleKind::from_str(item.trim()) {
                                if !selected_rules.contains(&r) {
                                    selected_rules.push(r);
                                }
                            } else {
                                eprintln!("Warning: unknown rule '{}' skipped", item.trim());
                            }
                        }
                        i += 1;
                    }
                } else if !arg.starts_with('-') {
                    target_dir = PathBuf::from(arg);
                }
                i += 1;
            }

            if selected_rules.is_empty() {
                eprintln!("Error: no valid rules selected for patching");
                return 1;
            }

            if target_dir.is_file() {
                let fname = target_dir.file_name().and_then(|f| f.to_str()).unwrap_or("");
                if fname.eq_ignore_ascii_case("claude.md") {
                    match patch_claude_md(&target_dir, &selected_rules) {
                        Ok(modified) => {
                            println!("  {} Patched {} (modified: {})", C.green(Sym::CHECK), target_dir.display(), modified);
                            0
                        }
                        Err(e) => {
                            eprintln!("Error patching {}: {}", target_dir.display(), e);
                            1
                        }
                    }
                } else if fname.eq_ignore_ascii_case("agents.md") {
                    match patch_agents_md(&target_dir, &selected_rules) {
                        Ok(modified) => {
                            println!("  {} Patched {} (modified: {})", C.green(Sym::CHECK), target_dir.display(), modified);
                            0
                        }
                        Err(e) => {
                            eprintln!("Error patching {}: {}", target_dir.display(), e);
                            1
                        }
                    }
                } else {
                    let mut modified = false;
                    for &r in &selected_rules {
                        if inject_rule_at_path(&target_dir, r).unwrap_or(false) {
                            modified = true;
                        }
                    }
                    println!("  {} Patched {} (modified: {})", C.green(Sym::CHECK), target_dir.display(), modified);
                    0
                }
            } else {
                match patch_all_agent_configs(&target_dir, &selected_rules) {
                    Ok(report) => {
                        println!("=== Patched Agent Configurations in {} ===", target_dir.display());
                        println!("  CLAUDE.md: {}", if report.claude_patched { "updated" } else { "already up to date" });
                        println!("  AGENTS.md: {}", if report.agents_patched { "updated" } else { "already up to date" });
                        println!("  .cursor/rules/: {} rule files written", report.cursor_rules_written.len());
                        0
                    }
                    Err(e) => {
                        eprintln!("Error patching configurations in {}: {}", target_dir.display(), e);
                        1
                    }
                }
            }
        }
        "inject" => {
            if rest.len() < 2 {
                eprintln!("Error: 'tokless rules inject <file> <rule>' requires file path and rule name");
                return 1;
            }
            let file_path = Path::new(&rest[0]);
            let rule = match RuleKind::from_str(&rest[1]) {
                Some(r) => r,
                None => {
                    eprintln!("Error: unknown rule '{}'. Available: principles, caveman, ponytail", rest[1]);
                    return 1;
                }
            };
            match inject_rule_at_path(file_path, rule) {
                Ok(modified) => {
                    if modified {
                        println!("  {} Injected rule '{}' into {}", C.green(Sym::CHECK), rule.as_str(), file_path.display());
                    } else {
                        println!("  {} Rule '{}' already present in {}", C.cyan("i"), rule.as_str(), file_path.display());
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error injecting rule: {}", e);
                    1
                }
            }
        }
        "remove" => {
            if rest.len() < 2 {
                eprintln!("Error: 'tokless rules remove <file> <rule>' requires file path and rule name");
                return 1;
            }
            let file_path = Path::new(&rest[0]);
            let rule = match RuleKind::from_str(&rest[1]) {
                Some(r) => r,
                None => {
                    eprintln!("Error: unknown rule '{}'. Available: principles, caveman, ponytail", rest[1]);
                    return 1;
                }
            };
            match remove_rule_from_path(file_path, rule) {
                Ok(modified) => {
                    if modified {
                        println!("  {} Removed rule '{}' from {}", C.green(Sym::CHECK), rule.as_str(), file_path.display());
                    } else {
                        println!("  {} Rule '{}' was not present in {}", C.cyan("i"), rule.as_str(), file_path.display());
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error removing rule: {}", e);
                    1
                }
            }
        }
        "status" => {
            let target_dir = if rest.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(&rest[0])
            };

            let statuses = inspect_agent_rules(&target_dir);
            if statuses.is_empty() {
                println!("No agent configs (CLAUDE.md, AGENTS.md, .cursor/rules/) found in {}", target_dir.display());
            } else {
                println!("=== Rule Status in {} ===", target_dir.display());
                for st in statuses {
                    let rules_str = if st.present_rules.is_empty() {
                        "none".to_string()
                    } else {
                        st.present_rules.iter().map(|r| r.as_str()).collect::<Vec<_>>().join(", ")
                    };
                    println!("  {}: [{}]", st.target, rules_str);
                }
            }
            0
        }
        other => {
            eprintln!("Error: unknown subcommand '{}'", other);
            print_rules_help();
            1
        }
    }
}

