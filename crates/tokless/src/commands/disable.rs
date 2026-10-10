use std::fs;
use crate::commands::helpers::*;
use crate::core::{get_agent, list_agents, list_tools, RunOpts};
use crate::util::exec::run_cmd;
use crate::util::logger::L;
use crate::util::paths::{resolve_npm_binary, resolve_rtk_bin};
use crate::util::progress::NewRootSectionProgress;

pub fn run_disable(opts: InitOptions) -> i32 {
    disable_impl(opts, false, "Disabled")
}

pub fn run_uninstall(opts: InitOptions) -> i32 {
    disable_impl(opts, true, "Uninstalled")
}

fn disable_impl(opts: InitOptions, purge: bool, past_label: &str) -> i32 {
    crate::util::logger::set_quiet(!opts.verbose);

    let all_tools = list_tools();
    let tools = if let Some(ref selected) = opts.tools {
        all_tools.into_iter().filter(|t| selected.contains(&t.id)).collect()
    } else {
        all_tools
    };

    let all_agents = list_agents();
    let requested: Vec<String> = if let Some(ref req) = opts.agents {
        req.clone()
    } else {
        all_agents.iter().map(|a| a.id.clone()).collect()
    };

    let mut unwire_bar = NewRootSectionProgress::new("Agents");
    unwire_bar.start(requested.len());

    for agent_id in &requested {
        let label = get_agent(agent_id).map(|a| a.label.clone()).unwrap_or_else(|| agent_id.clone());
        unwire_bar.begin(&label);

        for tool in &tools {
            if let Some(ref unwire_fn) = tool.unwire_for.get(agent_id) {
                let ro = RunOpts {
                    dry_run: opts.dry_run,
                    agent: agent_id.clone(),
                    ..Default::default()
                };
                let _ = unwire_fn(&ro);
            }
        }
        unwire_bar.complete(past_label);
    }
    unwire_bar.done("");

    if purge && !opts.dry_run {
        let rtk = resolve_rtk_bin();
        if !rtk.is_empty() {
            let _ = fs::remove_file(&rtk);
        }
        let npm = resolve_npm_binary();
        if !npm.is_empty() {
            let _ = run_cmd(&npm, &["uninstall", "-g", "context-mode", "@colbymchenry/codegraph"], &[]);
        }
    }

    print_repo_footer(true);
    L.raw("");
    0
}
