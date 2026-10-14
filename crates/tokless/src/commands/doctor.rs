// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/doctor.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::env;
use crate::commands::doctor_probe::{format_runtime_issues, probe_agent_runtime, RuntimeIssue};
use crate::commands::helpers::*;
use crate::core::{list_agents, list_tools, ToolManifest};
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::progress::*;
use crate::util::versions::*;

struct AgentReport {
    label: String,
    installed: bool,
    wired: bool,
    missing: Vec<String>,
    runtime: Vec<RuntimeIssue>,
}

pub fn run_doctor(offline: bool) -> i32 {
    cmd_header("doctor", "quick health check");

    let tools = list_tools();
    let mut reports = Vec::new();

    for agent in list_agents() {
        if !agent.detect().installed {
            continue;
        }
        let missing = doctor_missing_tools(&tools, &agent.id);
        let runtime = probe_agent_runtime(&agent.id);
        let wired = missing.is_empty() && runtime.is_empty();
        reports.push(AgentReport {
            label: agent.label.clone(),
            installed: true,
            wired,
            missing,
            runtime,
        });
    }

    tree_top("Agents");
    if reports.is_empty() {
        tree_leaf(&C.dim("none detected on this machine"));
    } else {
        for r in &reports {
            tree_leaf(&doctor_summary_line(r));
        }
    }
    tree_close();

    let broken = reports.iter().filter(|r| !r.wired).count();
    let installed_agents = reports.len();

    let mut outdated = -1;
    if !offline && env::var("TOKLESS_TEST").as_deref() != Ok("1") {
        let v = gather_versions();
        outdated = count_outdated(&v) as i32;

        tree_corner("Tools");
        for tool in &tools {
            if tool.instruction_only {
                continue;
            }
            if let Some(info) = v.get(&tool.id) {
                tree_leaf(&tool_version_display_line(&tool.id, Some(info), false));
            }
        }
        tree_close();
    }

    let status_lines = doctor_status_lines(outdated, broken, installed_agents);
    tree_status(&status_lines);
    print_repo_footer(true);
    L.raw("");
    0
}

fn doctor_missing_tools(tools: &[std::sync::Arc<ToolManifest>], agent_id: &str) -> Vec<String> {
    let mut missing = Vec::new();
    for tool in tools {
        if tool.instruction_only {
            continue;
        }
        if let Some(ref verify) = tool.verify_for.get(agent_id) {
            if let Some(false) = verify() {
                missing.push(tool.label.clone());
            }
        }
    }
    missing
}

fn doctor_summary_line(r: &AgentReport) -> String {
    let name = paint_name(&pad_end(&r.label, 16));
    if !r.installed {
        return format!(
            "{}{} {}",
            C.gray(&format!("{} ", Sym::BULLET)),
            C.dim(&pad_end(&r.label, 16)),
            C.gray("not installed")
        );
    }
    if r.wired {
        format!("{} {} {}", C.green(Sym::CHECK), name, C.green("all tools wired"))
    } else if !r.runtime.is_empty() && r.missing.is_empty() {
        format!(
            "{} {} {}",
            C.yellow(Sym::WARN),
            name,
            C.yellow(&format!("runtime: {}", format_runtime_issues(&r.runtime)))
        )
    } else if !r.runtime.is_empty() {
        format!(
            "{} {} {}",
            C.yellow(Sym::WARN),
            name,
            C.yellow(&format!(
                "missing: {}; runtime: {}",
                r.missing.join(", "),
                format_runtime_issues(&r.runtime)
            ))
        )
    } else {
        format!(
            "{} {} {}",
            C.yellow(Sym::WARN),
            name,
            C.yellow(&format!("missing: {}", r.missing.join(", ")))
        )
    }
}

fn doctor_status_lines(outdated: i32, broken: usize, installed_agents: usize) -> Vec<String> {
    let tools_ok = outdated == 0;
    let agents_ok = broken == 0;
    let online = outdated >= 0;

    if online && tools_ok && agents_ok {
        return vec![status_ok("Everything up to date.")];
    }

    let mut lines = Vec::new();
    if online {
        if outdated > 0 {
            lines.push(status_kv(
                &C.yellow(Sym::WARN),
                "tools",
                &format!(
                    "{} — run {}",
                    C.yellow(&format!("{} available", plural(outdated as usize))),
                    paint_cmd("tokless update")
                ),
            ));
        } else {
            lines.push(status_kv(
                &C.green(Sym::CHECK),
                "tools",
                &C.green("all up to date"),
            ));
        }
    }

    if broken > 0 {
        lines.push(status_kv(
            &C.yellow(Sym::WARN),
            "agents",
            &format!(
                "{} — run {}",
                C.yellow(&format!("{} incomplete", broken)),
                paint_cmd("tokless")
            ),
        ));
    } else if installed_agents > 0 {
        lines.push(status_kv(
            &C.green(Sym::CHECK),
            "agents",
            &C.green("all wired"),
        ));
    } else {
        lines.push(status_kv(
            &C.gray(Sym::BULLET),
            "agents",
            &C.dim("none installed"),
        ));
    }
    lines
}
