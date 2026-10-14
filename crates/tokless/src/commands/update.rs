// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/update.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use crate::commands::helpers::*;
use crate::commands::init::run_init;
use crate::commands::selfupdate::maybe_self_update;
use crate::core::{list_agents, list_tools, RunOpts};
use crate::tools::{ensure_instruction_separators, has_owner, kilo_has_owner};
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::progress::*;
use crate::util::versions::gather_versions_force;

pub fn resync_instruction_wiring() {
    let tools = list_tools();
    let agents = list_agents();
    for agent in &agents {
        if !agent.detect().installed {
            continue;
        }
        let mut any = false;
        for tool in &tools {
            if !tool.instruction_only {
                continue;
            }
            let wired = if agent.id == "kilo" {
                kilo_has_owner(&tool.id)
            } else {
                has_owner(&agent.id, &tool.id)
            };
            if wired {
                if let Some(ref wire_fn) = tool.wire_for.get(&agent.id) {
                    let ro = RunOpts {
                        dry_run: false,
                        agent: agent.id.clone(),
                        ..Default::default()
                    };
                    let _ = wire_fn(&ro);
                    any = true;
                }
            }
        }
        if any {
            ensure_instruction_separators(&agent.id);
        }
    }
}

pub fn resync_wiring(changed_tools: &[String], opts: &InitOptions) -> i32 {
    if !opts.dry_run {
        resync_instruction_wiring();
    }
    let mut init_opts = opts.clone();
    init_opts.tools = Some(changed_tools.to_vec());
    init_opts.upgrade = true;
    init_opts.yes = true;
    run_init(init_opts)
}

pub fn run_update(opts: InitOptions) -> i32 {
    maybe_self_update(&opts);

    cmd_header("update", "refresh tools to latest");

    if opts.dry_run {
        L.raw(&format!(
            "  {} Dry run — probe only, no installs.",
            C.cyan(Sym::INFO)
        ));
        L.raw("");
    }

    let versions = gather_versions_force();
    let mut changed = Vec::new();

    tree_top("Versions");
    for t in list_tools() {
        if t.instruction_only {
            continue;
        }
        let info = versions.get(&t.id);
        if tool_version_outdated(info)
            || info.map(|i| i.installed.is_none() && i.latest.is_some()).unwrap_or(false)
        {
            changed.push(t.id.clone());
        }
        tree_leaf(&tool_version_display_line(&t.id, info, true));
    }
    tree_close();

    if opts.dry_run {
        if !changed.is_empty() {
            tree_status(&[status_info(&format!(
                "{} {}",
                C.gray("Would upgrade: "),
                C.bold(&paint_ver(&join_comma(&changed)))
            ))]);
        } else {
            tree_status(&[status_ok("Everything up to date.")]);
        }
        print_repo_footer(true);
        L.raw("");
        return 0;
    }

    if changed.is_empty() {
        resync_instruction_wiring();
        tree_status(&[status_ok("Everything up to date.")]);
        print_repo_footer(true);
        L.raw("");
        return 0;
    }

    resync_wiring(&changed, &opts)
}
