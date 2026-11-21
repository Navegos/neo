// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/cli.rs
// created: 2026-10-31
// lastModified: 2026-11-16

use std::path::{Path, PathBuf};
use crate::rules::{
    caveman_rule, inject_rule_at_path, inspect_agent_rules, patch_agents_md,
    patch_all_agent_configs, patch_claude_md, ponytail_rule, remove_rule_from_path,
    skill_validator, skills_repo, sync_all_rules_with_options, RuleKind, SyncComponents,
    SyncOptions,
};
use crate::util::colors::*;

pub fn print_rules_help() {
    println!(
        "toksi rules — Manage externalized coding rules & skills (principles, caveman, ponytail)\n\n\
        Usage:\n\
          toksi rules list\n\
          toksi rules show <rule|mode|spec> [--cursor] [--body]\n\
          toksi rules patch [target_dir_or_file] [--rules <r1,r2>]\n\
          toksi rules sync [target_dir] [--rules <r1,r2>] [--agents <a1,a2>] [--components <c1,c2>] [--workspace] [--global]\n\
          toksi rules grade [path]\n\
          toksi rules heal [path]\n\
          toksi rules catalog\n\
          toksi rules repo-sync\n\
          toksi rules audit-docs [repo_root]\n\
          toksi rules inject <file> <rule>\n\
          toksi rules remove <file> <rule>\n\
          toksi rules status [target_dir]\n\n\
        Rules:\n\
          principles  Karpathy guidelines: think first, simplify, surgical changes, verify\n\
          caveman     ASD-STE100 + Wave 17 Caveman v4 modes (ultracave, megacave, shrink, browse, .blocks)\n\
          ponytail    Lazy senior developer build discipline ladder v5 (lite/full/ultra/review)\n"
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
            println!("In-tree coding rules and guidelines compiled into toksi binary:\n");
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
                eprintln!("Error: 'toksi rules show <rule>' requires a rule, mode, or spec name");
                return 1;
            }
            let rule_name = &rest[0];
            if let Some(asset) = caveman_rule::lookup_asset(rule_name)
                .or_else(|| ponytail_rule::lookup_skill(rule_name))
                .filter(|_| RuleKind::from_str(rule_name).is_none())
            {
                print!("{}", asset);
                return 0;
            }
            let rule = match RuleKind::from_str(rule_name) {
                Some(r) => r,
                None => {
                    eprintln!("Error: unknown rule '{}'. Available: principles, caveman, ponytail, ultracave, megacave, shrink, browse, .blocks", rule_name);
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
        "sync" => {
            let mut target_dir = PathBuf::from(".");
            let mut selected_rules = RuleKind::all().to_vec();
            let mut include_global = false;
            let mut opts = SyncOptions::default();

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
                } else if arg == "--agents" || arg == "-a" {
                    if i + 1 < rest.len() {
                        let agents: Vec<String> = rest[i + 1]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        opts.selected_agents = Some(agents);
                        i += 1;
                    }
                } else if arg == "--components" || arg == "-c" {
                    if i + 1 < rest.len() {
                        match SyncComponents::from_csv(&rest[i + 1]) {
                            Ok(comp) => opts.components = comp,
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                return 1;
                            }
                        }
                        i += 1;
                    }
                } else if arg == "--workspace" || arg == "-w" {
                    opts.allow_workspace_install = true;
                } else if arg == "--global" || arg == "-g" {
                    include_global = true;
                } else if !arg.starts_with('-') {
                    target_dir = PathBuf::from(arg);
                }
                i += 1;
            }

            match sync_all_rules_with_options(&target_dir, include_global, &selected_rules, &opts) {
                Ok(report) => {
                    println!("=== Rules Synchronized ===");
                    println!("  Workspace: {}", report.workspace.display());
                    if !report.synced_agents.is_empty() {
                        println!("  Agents: {}", report.synced_agents.join(", "));
                    }
                    println!("  Updated: {}", report.updated_files.len());
                    for f in &report.updated_files {
                        println!("    {} {}", C.green(Sym::CHECK), f.display());
                    }
                    println!("  Up-to-date: {}", report.up_to_date_files.len());
                    if !report.errors.is_empty() {
                        println!("  Errors ({}):", report.errors.len());
                        for e in &report.errors {
                            println!("    {} {}", C.red("x"), e);
                        }
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error synchronizing rules: {}", e);
                    1
                }
            }
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
                        println!("  GEMINI.md: {}", if report.gemini_patched { "updated" } else { "already up to date" });
                        println!("  copilot-instructions: {}", if report.copilot_patched { "updated" } else { "already up to date" });
                        println!("  .clinerules: {}", if report.cline_patched { "updated" } else { "already up to date" });
                        println!("  CODEX.md: {}", if report.codex_patched { "updated" } else { "already up to date" });
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
                eprintln!("Error: 'toksi rules inject <file> <rule>' requires file path and rule name");
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
                eprintln!("Error: 'toksi rules remove <file> <rule>' requires file path and rule name");
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
                println!("No agent configs found in {}", target_dir.display());
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
        "grade" => {
            let target = if rest.is_empty() {
                skills_repo::SkillsRepoConfig::load().stage_agents_dir
            } else {
                PathBuf::from(&rest[0])
            };
            let report = if target.is_file() {
                match std::fs::read_to_string(&target) {
                    Ok(raw) => {
                        let single = skill_validator::analyze_content(
                            &target.to_string_lossy(),
                            &raw,
                            None,
                        );
                        let errors = single
                            .findings
                            .iter()
                            .filter(|f| f.severity == skill_validator::FindingSeverity::Error)
                            .count();
                        let warnings = single
                            .findings
                            .iter()
                            .filter(|f| f.severity == skill_validator::FindingSeverity::Warning)
                            .count();
                        let infos = single
                            .findings
                            .iter()
                            .filter(|f| f.severity == skill_validator::FindingSeverity::Info)
                            .count();
                        skill_validator::BatchValidationReport {
                            score: single.score,
                            grade: single.grade,
                            error_count: errors,
                            warning_count: warnings,
                            info_count: infos,
                            files: vec![single],
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading {}: {}", target.display(), e);
                        return 1;
                    }
                }
            } else {
                match skill_validator::analyze_directory(&target) {
                    Ok(rep) => rep,
                    Err(e) => {
                        eprintln!("Error grading {}: {}", target.display(), e);
                        return 1;
                    }
                }
            };
            println!(
                "=== SkillDoctor Grade: {} ({}/100) — {} files ({} err, {} warn, {} info) ===",
                report.grade,
                report.score,
                report.files.len(),
                report.error_count,
                report.warning_count,
                report.info_count
            );
            for f in &report.files {
                println!(
                    "  [{} {:>3}] {} (agentskills={}, enforce={}, vetted={})",
                    f.grade,
                    f.score,
                    f.file_path,
                    f.is_valid_agentskills,
                    f.has_enforce_guard,
                    f.vetted_for_install
                );
            }
            0
        }
        "heal" => {
            let target = if rest.is_empty() {
                skills_repo::SkillsRepoConfig::load().stage_agents_dir
            } else {
                PathBuf::from(&rest[0])
            };
            let mut healed_count = 0usize;
            let mut stack = vec![target.clone()];
            while let Some(cur) = stack.pop() {
                if cur.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&cur) {
                        for entry in entries.flatten() {
                            stack.push(entry.path());
                        }
                    }
                } else if cur.is_file() {
                    let lower = cur
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if lower == "skill.md" || lower.ends_with(".skill.md") {
                        if let Ok(raw) = std::fs::read_to_string(&cur) {
                            let parent_dir = cur
                                .parent()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .filter(|d| !d.eq_ignore_ascii_case("skills"));
                            let res = skill_validator::heal_skill_content(
                                &cur.to_string_lossy(),
                                &raw,
                                parent_dir,
                                true,
                            );
                            if res.changed {
                                if let Err(e) = std::fs::write(&cur, &res.output) {
                                    eprintln!("Error writing {}: {}", cur.display(), e);
                                    return 1;
                                }
                                healed_count += 1;
                                println!("  {} Healed {}: {}", C.green(Sym::CHECK), cur.display(), res.applied.join(", "));
                            }
                        }
                    }
                }
            }
            println!("Healed {} skill file(s).", healed_count);
            0
        }
        "catalog" => {
            let cfg = skills_repo::SkillsRepoConfig::load();
            match skills_repo::discover_vetted_catalog(&cfg) {
                Ok(cat) => {
                    println!(
                        "=== Vetted Skills Catalog: {} skills, {} collisions resolved, {} blocked ===",
                        cat.entries.len(),
                        cat.collisions.len(),
                        cat.blocked_skills.len()
                    );
                    for entry in &cat.entries {
                        println!(
                            "  [{} {:>3}] {} ({}) — triggers: {}",
                            entry.grade,
                            entry.score,
                            entry.name,
                            entry.origin.as_str(),
                            entry.triggers.join(", ")
                        );
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error building skills catalog: {}", e);
                    1
                }
            }
        }
        "repo-sync" => {
            let cfg = skills_repo::SkillsRepoConfig::load();
            match skills_repo::sync_external_skills_repo(&cfg) {
                Ok(rep) => {
                    println!(
                        "Synced {} external file(s) to {} (catalog={}, healed={}, collisions={}, blocked={})",
                        rep.synced_files.len(),
                        cfg.skills_dir.display(),
                        rep.catalog_count,
                        rep.healed_files.len(),
                        rep.collision_count,
                        rep.blocked_files.len()
                    );
                    0
                }
                Err(e) => {
                    eprintln!("Error syncing external skills repo: {}", e);
                    1
                }
            }
        }
        "audit-docs" => {
            let root = if rest.is_empty() {
                crate::projectmem::store::find_repo_root(None)
            } else {
                PathBuf::from(&rest[0])
            };
            match audit_docs(&root) {
                Ok(report) => {
                    if report.findings.is_empty() {
                        println!(
                            "{} Documentation audit clean: {} file(s) scanned, 0 stale .projectmem/ pre-read references.",
                            C.green(Sym::CHECK),
                            report.files_scanned
                        );
                        0
                    } else {
                        for f in &report.findings {
                            eprintln!("{} {}:{} -> {}", C.red(Sym::CROSS), f.file, f.line, f.snippet);
                        }
                        eprintln!(
                            "{} Found {} stale .projectmem/ pre-read reference(s) across {} file(s).",
                            C.red(Sym::CROSS),
                            report.findings.len(),
                            report.files_scanned
                        );
                        1
                    }
                }
                Err(e) => {
                    eprintln!("Error auditing docs: {}", e);
                    1
                }
            }
        }
        other => {
            eprintln!("Error: unknown subcommand '{}'", other);
            print_rules_help();
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocAuditFinding {
    pub file: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocAuditReport {
    pub files_scanned: usize,
    pub findings: Vec<DocAuditFinding>,
}

fn line_has_stale_projectmem_preread(line: &str) -> bool {
    const STALE_TARGETS: &[&str] = &[
        "plan.md",
        "PROJECT_MAP.md",
        "summary.md",
        "events.jsonl",
        "config.toml",
        "*.md",
        "plan_checkpoints",
        "issues/",
    ];
    let mut search_from = 0usize;
    while let Some(rel_idx) = line[search_from..].find(".projectmem/") {
        let idx = search_from + rel_idx;
        let preceded_by_toksi = idx >= 7 && &line[idx - 7..idx] == ".toksi/";
        if !preceded_by_toksi {
            let after = &line[idx + ".projectmem/".len()..];
            if STALE_TARGETS.iter().any(|t| after.starts_with(t)) {
                return true;
            }
        }
        search_from = idx + ".projectmem/".len();
    }
    false
}

/// Audit `.agents/brain/*.md`, `crates/toksi/{CONSOLIDATION_PLAN,PLAN,README}.md`,
/// and `.toksi/projectmem/*.md` for stale `.projectmem/` pre-read references.
pub fn audit_docs(repo_root: &Path) -> std::result::Result<DocAuditReport, String> {
    let mut targets: Vec<PathBuf> = Vec::new();

    let brain_dir = repo_root.join(".agents/brain");
    if brain_dir.is_dir() {
        let entries = std::fs::read_dir(&brain_dir)
            .map_err(|e| format!("failed to read {}: {}", brain_dir.display(), e))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
                targets.push(p);
            }
        }
    }

    for rel in [
        "crates/toksi/CONSOLIDATION_PLAN.md",
        "crates/toksi/PLAN.md",
        "crates/toksi/README.md",
    ] {
        let p = repo_root.join(rel);
        if p.is_file() {
            targets.push(p);
        }
    }

    let pm_dir = repo_root.join(".toksi/projectmem");
    if pm_dir.is_dir() {
        let entries = std::fs::read_dir(&pm_dir)
            .map_err(|e| format!("failed to read {}: {}", pm_dir.display(), e))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
                targets.push(p);
            }
        }
    }

    targets.sort();
    let mut findings = Vec::new();
    for path in &targets {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        for (idx, line) in content.lines().enumerate() {
            if line_has_stale_projectmem_preread(line) {
                findings.push(DocAuditFinding {
                    file: rel.clone(),
                    line: idx + 1,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }

    Ok(DocAuditReport {
        files_scanned: targets.len(),
        findings,
    })
}
