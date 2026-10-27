// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/init.rs
// created: 2026-10-05
// lastModified: 2026-10-27

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::agents::{install_cursor_project_rules_hook, snapshot_cursor_project_rules_hook};
use crate::commands::helpers::*;
use crate::commands::index::{find_project_dir, looks_like_project};
use crate::commands::selfupdate::maybe_self_update;
use crate::core::{get_agent, list_agents, list_tools, AgentManifest, RunOpts, ToolManifest};
use crate::tools::ensure_instruction_separators;
use crate::tools::projectmem::{ensure_projectmem_project, refresh_projectmem_rule_files};
use crate::util::colors::*;
use crate::util::deps::ensure_deps;
use crate::util::instructions::install_cursor_project_rules;
use crate::util::logger::L;
use crate::util::paths::{self_heal_path, tokless_persisted_abs};
use crate::util::selection::save_selection;
use crate::util::progress::{tree_close, tree_corner, tree_leaf, NewRootSectionProgress};
use crate::util::prompt::{is_interactive, multi_select, MultiSelectOption};

pub fn run_init(mut opts: InitOptions) -> i32 {
    crate::util::logger::set_quiet(!opts.verbose);

    if env::var("TOKLESS_INSTALLER_RUN").as_deref() == Ok("1") {
        opts.upgrade = true;
    } else {
        maybe_self_update(&opts);
        L.raw("");
    }

    if !opts.dry_run && tokless_persisted_abs().is_empty() {
        crate::util::logger::set_quiet(false);
        L.err("cannot wire agents: no stable tokless executable found; install tokless or put an absolute tokless binary on PATH");
        return 1;
    }

    let all_agents = list_agents();
    let installed_ids: HashMap<String, String> = std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(all_agents.len());
        for a in &all_agents {
            handles.push(s.spawn(move || {
                let d = a.detect();
                if d.installed {
                    Some((a.id.clone(), d.source))
                } else {
                    None
                }
            }));
        }
        let mut map = HashMap::with_capacity(all_agents.len());
        for h in handles {
            if let Ok(Some((id, src))) = h.join() {
                map.insert(id, src);
            }
        }
        map
    });

    let requested: Vec<String> = if let Some(ref req) = opts.agents {
        req.clone()
    } else {
        let mut auto = Vec::new();
        for a in &all_agents {
            if installed_ids.contains_key(&a.id) {
                auto.push(a.id.clone());
            }
        }
        if auto.is_empty() {
            crate::util::logger::set_quiet(false);
            L.raw("");
            L.raw(&format!(
                "  {} No compatible agents detected — nothing to wire.",
                C.yellow(Sym::WARN)
            ));
            L.raw(&format!(
                "  {} {}",
                C.gray("Install an agent, then re-run tokless; or select one explicitly: "),
                C.cyan("tokless --agents claude,opencode,codex")
            ));
            L.raw("");
            return 0;
        }
        if !opts.yes && is_interactive() && env::var("TOKLESS_INSTALLER_RUN").as_deref() != Ok("1") {
            let options: Vec<MultiSelectOption> = all_agents
                .iter()
                .filter(|a| installed_ids.contains_key(&a.id))
                .map(|a| MultiSelectOption {
                    value: a.id.clone(),
                    label: a.label.clone(),
                    hint: installed_ids.get(&a.id).cloned().unwrap_or_default(),
                    selected: true,
                    ..Default::default()
                })
                .collect();
            let sel = multi_select("Select agents to equip:", &options);
            if !sel.is_empty() {
                sel
            } else {
                L.raw(&format!("  {}", C.gray("Cancelled.")));
                return 0;
            }
        } else {
            auto
        }
    };

    let all_tools = list_tools();
    let mut selection_mode = "full";
    let tools: Vec<Arc<ToolManifest>> = if let Some(ref selected) = opts.tools {
        selection_mode = "selected";
        all_tools
            .into_iter()
            .filter(|t| selected.contains(&t.id))
            .collect()
    } else if !opts.yes && is_interactive() && env::var("TOKLESS_INSTALLER_RUN").as_deref() != Ok("1") {
        let options: Vec<MultiSelectOption> = all_tools
            .iter()
            .map(|t| MultiSelectOption {
                value: t.id.clone(),
                label: t.label.clone(),
                hint: t.description.clone(),
                selected: true,
                ..Default::default()
            })
            .collect();
        let sel = multi_select("Select tools to enable:", &options);
        if !sel.is_empty() {
            if sel.len() != all_tools.len() {
                selection_mode = "selected";
            }
            all_tools
                .into_iter()
                .filter(|t| sel.contains(&t.id))
                .collect()
        } else {
            L.raw(&format!("  {}", C.gray("Cancelled.")));
            return 0;
        }
    } else if let Some(saved) = crate::util::selection::load_selection() {
        if saved.mode == "selected" && !saved.tools.is_empty() {
            selection_mode = "selected";
            all_tools
                .into_iter()
                .filter(|t| saved.tools.contains(&t.id))
                .collect()
        } else {
            all_tools
        }
    } else {
        all_tools
    };

    let mut need_node = false;
    let mut need_git = false;
    let mut min_node = 0;
    for t in &tools {
        need_node = need_node || tool_needs_node(t);
        need_git = need_git || t.needs_git;
        if t.min_node_major > min_node {
            min_node = t.min_node_major;
        }
    }

    let (node_ok, _git_ok) = if !opts.dry_run {
        ensure_deps(need_node, need_git, min_node)
    } else {
        (true, true)
    };

    let install_tools: Vec<_> = tools.iter().filter(|t| !t.instruction_only).collect();
    let tool_bar = NewRootSectionProgress::new("Tools");
    tool_bar.start(install_tools.len());

    let mut install_logs = HashMap::new();
    let mut failed_tools = HashMap::new();
    for tool in &install_tools {
        tool_bar.begin(&tool.label);
        if tool_needs_node(tool) && !node_ok {
            tool_bar.fail("needs Node.js — https://nodejs.org/en/download");
            failed_tools.insert(tool.id.clone(), true);
            continue;
        }
        let ro = RunOpts {
            dry_run: opts.dry_run,
            upgrade: opts.upgrade,
            agent: opts.agent.clone(),
            ..Default::default()
        };
        if let Some(ref install_fn) = tool.install {
            match install_fn(&ro) {
                Ok(true) => {
                    tool_bar.complete(&tool_version_note(tool));
                }
                Ok(false) => {
                    tool_bar.fail("install failed");
                    install_logs.insert(tool.label.clone(), "install returned false".into());
                    failed_tools.insert(tool.id.clone(), true);
                }
                Err(e) => {
                    let err_str = e.to_string();
                    tool_bar.fail(first_line(&err_str));
                    install_logs.insert(tool.label.clone(), err_str);
                    failed_tools.insert(tool.id.clone(), true);
                }
            }
        } else {
            tool_bar.complete(&tool_version_note(tool));
        }
    }
    tool_bar.done("");
    print_failure_detail(&install_logs);

    if !opts.dry_run {
        self_heal_path();
        bust_version_cache_file();
    }

    let wire_bar = NewRootSectionProgress::new("Agents");
    wire_bar.start(requested.len());

    let mut equipped_agents = Vec::new();
    let mut wire_failures = 0usize;

    let wire_single_agent = |agent_id: &str| -> (String, String, bool, Option<String>) {
        let label = get_agent(agent_id)
            .map(|a| a.label.clone())
            .unwrap_or_else(|| agent_id.to_string());

        let cursor_snap = if agent_id == "cursor" && !opts.dry_run {
            snapshot_cursor_project_rules_hook().ok()
        } else {
            None
        };

        let mut wired_any = false;
        let mut wire_err: Option<String> = None;
        let mut newly_wired: Vec<Arc<ToolManifest>> = Vec::new();

        for tool in &tools {
            if failed_tools.contains_key(&tool.id) {
                continue;
            }
            if let Some(ref wire_fn) = tool.wire_for.get(agent_id) {
                let ro = RunOpts {
                    dry_run: opts.dry_run,
                    agent: agent_id.to_string(),
                    ..Default::default()
                };
                match wire_fn(&ro) {
                    Ok(true) => {
                        wired_any = true;
                        newly_wired.push(tool.clone());
                    }
                    Ok(false) => {}
                    Err(e) => {
                        wire_err = Some(e.to_string());
                        break;
                    }
                }
            }
        }

        if agent_id == "cursor" && wire_err.is_none() && !opts.dry_run {
            let _ = install_cursor_project_rules(".", false);
            let _ = install_cursor_project_rules_hook();
        }

        if let Some(ref err) = wire_err {
            if !opts.dry_run {
                for t in newly_wired.into_iter().rev() {
                    if let Some(ref unwire_fn) = t.unwire_for.get(agent_id) {
                        let ro = RunOpts {
                            dry_run: false,
                            agent: agent_id.to_string(),
                            ..Default::default()
                        };
                        let _ = unwire_fn(&ro);
                    }
                }
                if let Some(snap) = cursor_snap {
                    let _ = snap.restore();
                }
            }
            (agent_id.to_string(), label, wired_any, Some(err.clone()))
        } else {
            if !opts.dry_run {
                ensure_instruction_separators(agent_id);
                let all_tools_ref = list_tools();
                for t in &all_tools_ref {
                    if !tools.iter().any(|st| st.id == t.id) && !t.instruction_only {
                        if let Some(ref unwire_fn) = t.unwire_for.get(agent_id) {
                            let ro = RunOpts {
                                dry_run: false,
                                agent: agent_id.to_string(),
                                ..Default::default()
                            };
                            let _ = unwire_fn(&ro);
                        }
                    }
                }
            }
            (agent_id.to_string(), label, wired_any, None)
        }
    };

    let results: Vec<(String, String, bool, Option<String>)> = if opts.parallel || opts.jobs > 1 {
        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(requested.len());
            for agent_id in &requested {
                handles.push(s.spawn(|| wire_single_agent(agent_id)));
            }
            let mut res = Vec::with_capacity(requested.len());
            for h in handles {
                if let Ok(r) = h.join() {
                    res.push(r);
                }
            }
            res
        })
    } else {
        requested.iter().map(|id| wire_single_agent(id)).collect()
    };

    for (agent_id, label, wired_any, wire_err) in results {
        wire_bar.begin(&label);
        if let Some(err) = wire_err {
            wire_failures += 1;
            wire_bar.fail(first_line(&err));
        } else {
            if let Some(ag) = get_agent(&agent_id) {
                equipped_agents.push(ag);
            }
            if wired_any {
                wire_bar.complete("wired");
            } else {
                wire_bar.complete("up to date");
            }
        }
    }
    wire_bar.done("");

    if !opts.dry_run && tools.iter().any(|t| t.id == "projectmem") {
        projectmem_seed_project(&opts);
    }

    if !equipped_agents.is_empty() && !opts.upgrade {
        print_equipped_agent_tree(&equipped_agents, &tools);
    }

    print_repo_footer(true);
    L.raw("");
    if wire_failures > 0 || !failed_tools.is_empty() {
        1
    } else {
        save_selection_if_needed(selection_mode, &tools, opts.dry_run);
        0
    }
}

pub fn save_selection_if_needed(mode: &str, tools: &[Arc<ToolManifest>], dry_run: bool) -> String {
    if dry_run {
        return mode.to_string();
    }
    let all_tools = list_tools();
    let (actual_mode, tool_ids) = if mode == "full" || tools.len() == all_tools.len() {
        ("full", Vec::new())
    } else {
        ("selected", tools.iter().map(|t| t.id.clone()).collect())
    };
    if let Err(e) = save_selection(actual_mode, &tool_ids) {
        L.err(&format!("cannot persist tool selection: {}", e));
    }
    actual_mode.to_string()
}

pub fn init_agent_fully_wired(agent_id: &str, tools: &[Arc<ToolManifest>]) -> bool {
    for tool in tools {
        if let Some(ref verify_fn) = tool.verify_for.get(agent_id) {
            if verify_fn() == Some(false) {
                return false;
            }
        }
    }
    true
}

pub fn projectmem_seed_project(opts: &InitOptions) {
    if opts.dry_run {
        return;
    }
    if let Ok(cwd) = env::current_dir() {
        let cwd_str = cwd.to_string_lossy().to_string();
        let root = find_project_dir(&cwd_str);
        if looks_like_project(&root) {
            let _ = ensure_projectmem_project(&root);
            refresh_projectmem_rule_files(&root);
        }
    }
}

pub fn print_equipped_agent_tree(
    agents: &[Arc<AgentManifest>],
    tools: &[Arc<ToolManifest>],
) {
    if agents.is_empty() {
        return;
    }
    let tool_names: Vec<String> = tools.iter().map(|t| t.id.clone()).collect();
    let summary = join_comma(&tool_names);
    tree_corner("Equipped");
    for a in agents {
        tree_leaf(&format!(
            "{} {} {}",
            C.green(Sym::CHECK),
            paint_name(&pad_end(&a.label, 16)),
            C.gray(&summary)
        ));
    }
    tree_close();
}
