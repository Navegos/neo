// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/disable.rs
// created: 2026-10-05
// lastModified: 2026-10-30

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::commands::helpers::*;
use crate::core::{get_agent, list_agents, list_tools, AgentManifest, RunOpts, ToolManifest};
use crate::headroom::autostart::{disable_proxy_autostart, stop_proxy};
use crate::tools::ensure_instruction_separators;
use crate::util::colors::*;
use crate::util::exec::{rtk_install_dirs, run_cmd};
use crate::util::headroom_proxy::set_proxy_routing_enabled;
use crate::util::logger::L;
use crate::util::paths::{is_win, resolve_npm_binary, resolve_rtk_bin, which};
use crate::util::progress::NewRootSectionProgress;
use crate::util::prompt::{is_interactive, multi_select, MultiSelectOption};

pub fn pick_agents(
    all_agents: &[Arc<AgentManifest>],
    requested: &Option<Vec<String>>,
    yes: bool,
) -> (Vec<String>, bool) {
    if let Some(req) = requested {
        return (req.clone(), false);
    }
    let detected: Vec<Arc<AgentManifest>> = std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(all_agents.len());
        for a in all_agents {
            handles.push(s.spawn(move || {
                if a.detect().installed {
                    Some(a.clone())
                } else {
                    None
                }
            }));
        }
        let mut res = Vec::new();
        for h in handles {
            if let Ok(Some(a)) = h.join() {
                res.push(a);
            }
        }
        res
    });
    if detected.is_empty() {
        return (all_agents.iter().map(|a| a.id.clone()).collect(), false);
    }
    if yes || !is_interactive() {
        return (detected.iter().map(|a| a.id.clone()).collect(), false);
    }
    let options: Vec<MultiSelectOption> = detected
        .iter()
        .map(|a| MultiSelectOption {
            value: a.id.clone(),
            label: a.label.clone(),
            hint: String::new(),
            selected: true,
            ..Default::default()
        })
        .collect();
    let sel = multi_select("Select agents to disable:", &options);
    if sel.is_empty() {
        (Vec::new(), true)
    } else {
        (sel, false)
    }
}

pub fn pick_tools(
    all_tools: &[Arc<ToolManifest>],
    requested: &Option<Vec<String>>,
    yes: bool,
) -> (Vec<Arc<ToolManifest>>, bool) {
    if let Some(req) = requested {
        let filtered = all_tools
            .iter()
            .filter(|t| req.contains(&t.id))
            .cloned()
            .collect();
        return (filtered, false);
    }
    if yes || !is_interactive() {
        return (all_tools.to_vec(), false);
    }
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
    let sel = multi_select("Select tools to disable:", &options);
    if sel.is_empty() {
        (Vec::new(), true)
    } else {
        let filtered = all_tools
            .iter()
            .filter(|t| sel.contains(&t.id))
            .cloned()
            .collect();
        (filtered, false)
    }
}

pub fn purge_binaries(tools: &[Arc<ToolManifest>], dry_run: bool) {
    let bar = NewRootSectionProgress::new("Binaries");
    let trackable: Vec<&Arc<ToolManifest>> =
        tools.iter().filter(|t| !t.instruction_only).collect();
    if trackable.is_empty() {
        return;
    }
    bar.start(trackable.len());
    for tool in trackable {
        bar.begin(&tool.label);
        if dry_run {
            bar.complete("would remove");
            continue;
        }
        match tool.id.as_str() {
            "rtk" => {
                let rtk = resolve_rtk_bin();
                if !rtk.is_empty() {
                    let _ = fs::remove_file(&rtk);
                }
                for d in rtk_install_dirs() {
                    let bin = Path::new(&d).join(if is_win() { "rtk.exe" } else { "rtk" });
                    if bin.exists() {
                        let _ = fs::remove_file(bin);
                    }
                }
                bar.complete("removed");
            }
            "codegraph" => {
                let npm = resolve_npm_binary();
                if !npm.is_empty() {
                    let _ = run_cmd(&npm, &["uninstall", "-g", "@colbymchenry/codegraph"], &[]);
                }
                bar.complete("removed");
            }
            "context-mode" => {
                let npm = resolve_npm_binary();
                if !npm.is_empty() {
                    let _ = run_cmd(&npm, &["uninstall", "-g", "context-mode"], &[]);
                }
                bar.complete("removed");
            }
            "headroom" => {
                let _ = disable_proxy_autostart();
                let _ = stop_proxy();
                let _ = set_proxy_routing_enabled(false);
                let uv = which("uv");
                if !uv.is_empty() {
                    let _ = run_cmd(&uv, &["tool", "uninstall", "headroom-ai"], &[]);
                }
                bar.complete("removed");
            }
            "projectmem" => {
                let uv = which("uv");
                if !uv.is_empty() {
                    let _ = run_cmd(&uv, &["tool", "uninstall", "projectmem"], &[]);
                }
                bar.complete("removed");
            }
            _ => {
                bar.complete("skipped");
            }
        }
    }
    bar.done("");
    bust_version_cache_file();
}

pub fn run_purge(
    opts: InitOptions,
    cmd_name: &str,
    subtitle: &str,
    past_label: &str,
    purge: bool,
) -> i32 {
    crate::util::logger::set_quiet(!opts.verbose);
    cmd_header(cmd_name, subtitle);

    let all_agents = list_agents();
    let (agent_ids, cancelled_a) = pick_agents(&all_agents, &opts.agents, opts.yes);
    if cancelled_a || agent_ids.is_empty() {
        L.raw(&format!("  {}", C.gray("Cancelled.")));
        return 0;
    }

    let all_tools = list_tools();
    let (tools, cancelled_t) = pick_tools(&all_tools, &opts.tools, opts.yes);
    if cancelled_t || tools.is_empty() {
        L.raw(&format!("  {}", C.gray("Cancelled.")));
        return 0;
    }

    let unwire_bar = NewRootSectionProgress::new("Agents");
    unwire_bar.start(agent_ids.len());

    let mut had_error = false;
    for agent_id in &agent_ids {
        let label = get_agent(agent_id)
            .map(|a| a.label.clone())
            .unwrap_or_else(|| agent_id.clone());
        unwire_bar.begin(&label);

        let mut agent_err: Option<String> = None;
        for tool in &tools {
            if let Some(ref unwire_fn) = tool.unwire_for.get(agent_id) {
                let ro = RunOpts {
                    dry_run: opts.dry_run,
                    agent: agent_id.clone(),
                    ..Default::default()
                };
                if let Err(e) = unwire_fn(&ro) {
                    agent_err = Some(e.to_string());
                    break;
                }
            }
        }
        if !opts.dry_run {
            ensure_instruction_separators(agent_id);
        }

        if let Some(err) = agent_err {
            had_error = true;
            unwire_bar.fail(&first_line(&err));
        } else {
            unwire_bar.complete(past_label);
        }
    }
    unwire_bar.done("");

    if purge {
        purge_binaries(&tools, opts.dry_run);
    }

    print_repo_footer(true);
    L.raw("");
    if had_error { 1 } else { 0 }
}

pub fn run_disable(opts: InitOptions) -> i32 {
    run_purge(opts, "disable", "unwire tools from agents", "Disabled", false)
}

pub fn run_uninstall(opts: InitOptions) -> i32 {
    run_purge(
        opts,
        "uninstall",
        "unwire tools and remove binaries",
        "Uninstalled",
        true,
    )
}
