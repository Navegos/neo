use std::collections::HashMap;
use std::env;
use crate::commands::helpers::*;
use crate::commands::selfupdate::maybe_self_update;
use crate::core::{get_agent, list_agents, list_tools, RunOpts};
use crate::tools::projectmem::ensure_projectmem_project;
use crate::util::colors::*;
use crate::util::deps::ensure_deps;
use crate::util::logger::L;
use crate::util::paths::{self_heal_path, tokless_persisted_abs};
use crate::util::progress::NewRootSectionProgress;

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

    let all_tools = list_tools();
    let tools = if let Some(ref selected) = opts.tools {
        all_tools.into_iter().filter(|t| selected.contains(&t.id)).collect()
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
    let mut tool_bar = NewRootSectionProgress::new("Tools");
    tool_bar.start(install_tools.len());

    let mut install_logs = HashMap::new();
    for tool in &install_tools {
        tool_bar.begin(&tool.label);
        if tool_needs_node(tool) && !node_ok {
            tool_bar.fail("needs Node.js — https://nodejs.org/en/download");
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
                }
                Err(e) => {
                    let err_str = e.to_string();
                    tool_bar.fail(&first_line(&err_str));
                    install_logs.insert(tool.label.clone(), err_str);
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
    }

    let all_agents = list_agents();
    let mut installed_ids = HashMap::new();
    for a in &all_agents {
        let d = a.detect();
        if d.installed {
            installed_ids.insert(a.id.clone(), d.source);
        }
    }

    let requested: Vec<String> = if let Some(ref req) = opts.agents {
        req.clone()
    } else {
        // Auto-select detected agents that match at least one tool
        let mut auto = Vec::new();
        for a in &all_agents {
            if installed_ids.contains_key(&a.id) {
                auto.push(a.id.clone());
            }
        }
        if auto.is_empty() {
            crate::util::logger::set_quiet(false);
            L.raw("");
            L.raw(&format!("  {} No compatible agents detected — nothing to wire.", C.yellow(Sym::WARN)));
            L.raw(&format!("  {} {}", C.gray("Install an agent, then re-run tokless; or select one explicitly: "), C.cyan("tokless --agents claude,opencode,codex")));
            L.raw("");
            return 0;
        }
        auto
    };

    let mut wire_bar = NewRootSectionProgress::new("Agents");
    wire_bar.start(requested.len());

    for agent_id in &requested {
        let label = get_agent(agent_id).map(|a| a.label.clone()).unwrap_or_else(|| agent_id.clone());
        wire_bar.begin(&label);

        let mut wired_any = false;
        let mut wire_err: Option<String> = None;

        for tool in &tools {
            if let Some(ref wire_fn) = tool.wire_for.get(agent_id) {
                let ro = RunOpts {
                    dry_run: opts.dry_run,
                    agent: agent_id.clone(),
                    ..Default::default()
                };
                match wire_fn(&ro) {
                    Ok(true) => {
                        wired_any = true;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        wire_err = Some(e.to_string());
                        break;
                    }
                }
            }
        }

        if let Some(err) = wire_err {
            wire_bar.fail(&first_line(&err));
        } else if wired_any {
            wire_bar.complete("wired");
        } else {
            wire_bar.complete("up to date");
        }
    }
    wire_bar.done("");

    // Seed projectmem if project exists
    if let Ok(cwd) = env::current_dir() {
        let root = cwd.to_string_lossy().to_string();
        let _ = ensure_projectmem_project(&root);
    }

    print_repo_footer(true);
    L.raw("");
    0
}
