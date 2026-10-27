// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/doctor.rs
// created: 2026-10-05
// lastModified: 2026-10-27

use std::env;
use crate::commands::doctor_probe::{format_runtime_issues, probe_agent_runtime, RuntimeIssue};
use crate::commands::doctor_stats::doctor_tool_stats;
use crate::commands::helpers::*;
use crate::core::{list_agents, list_tools, ToolManifest};
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::progress::*;
use crate::util::selection::is_tool_selected;
use crate::util::versions::*;

#[derive(Clone, serde::Serialize)]
struct AgentReport {
    id: String,
    label: String,
    installed: bool,
    wired: bool,
    missing: Vec<String>,
    tool_rows: Vec<String>,
    #[serde(skip_serializing)]
    runtime: Vec<RuntimeIssue>,
}

pub fn run_doctor(offline: bool, json_output: bool) -> i32 {
    let tools = list_tools();
    let all_agents = list_agents();
    let mut reports_opt: Vec<Option<AgentReport>> = vec![None; all_agents.len()];

    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(all_agents.len());
        for agent in &all_agents {
            let tools_ref = &tools;
            handles.push(s.spawn(move || {
                if !agent.detect().installed {
                    return None;
                }
                let missing = doctor_missing_tools(tools_ref, &agent.id);
                let runtime = probe_agent_runtime(&agent.id);
                let wired = missing.is_empty() && runtime.is_empty();
                let mut tool_rows = Vec::new();
                for t in tools_ref {
                    if t.instruction_only || !is_tool_selected(&t.id) {
                        continue;
                    }
                    if let Some(ref verify) = t.verify_for.get(&agent.id) {
                        let ok = verify().unwrap_or(false);
                        let mark = if ok { C.green(Sym::CHECK) } else { C.red("x") };
                        tool_rows.push(format!("  {} {}", mark, t.label));
                    }
                }
                Some(AgentReport {
                    id: agent.id.clone(),
                    label: agent.label.clone(),
                    installed: true,
                    wired,
                    missing,
                    tool_rows,
                    runtime,
                })
            }));
        }
        for (i, handle) in handles.into_iter().enumerate() {
            if let Ok(res) = handle.join() {
                reports_opt[i] = res;
            }
        }
    });

    let reports: Vec<AgentReport> = reports_opt.into_iter().flatten().collect();
    let cwd = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into());
    let tool_stats = doctor_tool_stats(&cwd, true);

    if json_output {
        #[derive(serde::Serialize)]
        struct DoctorJson<'a> {
            agents: &'a [AgentReport],
            stats: &'a std::collections::HashMap<String, Vec<String>>,
        }
        let doc = DoctorJson {
            agents: &reports,
            stats: &tool_stats,
        };
        if let Ok(out) = serde_json::to_string_pretty(&doc) {
            println!("{}", out);
            return 0;
        }
    }

    cmd_header("doctor", "quick health check");

    tree_top("Agents");
    if reports.is_empty() {
        tree_leaf(&C.dim("none detected on this machine"));
    } else {
        for r in &reports {
            tree_leaf(&doctor_summary_line(r));
            if !r.wired {
                for tr in &r.tool_rows {
                    tree_leaf(&C.dim(tr));
                }
            }
        }
    }
    tree_close();

    let port_rows = doctor_proxy_ports(&reports);
    if !port_rows.is_empty() {
        tree_corner("Ports");
        for p in port_rows {
            tree_leaf(&p);
        }
        tree_close();
    }

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

    tree_corner("Stats");
    if tool_stats.is_empty() {
        tree_leaf(&C.dim("no installed tools with local stats"));
    } else {
        let tool_ids = ["headroom", "projectmem", "rtk", "codegraph", "context-mode"];
        for id in tool_ids {
            if let Some(rows) = tool_stats.get(id) {
                tree_leaf(&format!("{}:", paint_name(id)));
                for row in rows {
                    tree_leaf(&format!("  {}", row));
                }
            }
        }
    }
    tree_close();

    let status_lines = doctor_status_lines(outdated, broken, installed_agents);
    tree_status(&status_lines);
    print_repo_footer(true);
    L.raw("");
    0
}

fn doctor_proxy_ports(reports: &[AgentReport]) -> Vec<String> {
    let mut ports = Vec::new();
    let main_port = crate::util::headroom_proxy::headroom_proxy_port();
    let main_running = crate::headroom::proxy_running();
    let main_status = if main_running {
        C.green("running")
    } else {
        C.gray("stopped")
    };
    ports.push(format!(
        "{} Headroom proxy (127.0.0.1:{}) â€” {}",
        if main_running { C.green(Sym::CHECK) } else { C.gray(Sym::BULLET) },
        main_port,
        main_status
    ));

    let byok_routes = crate::util::byok::read_byok_routes().unwrap_or_default();
    if !byok_routes.is_empty() {
        let gw_port = crate::util::byok::byok_gateway_port();
        ports.push(format!(
            "{} BYOK gateway (127.0.0.1:{}) â€” {}",
            C.green(Sym::CHECK),
            gw_port,
            C.green(&format!("{} routes active", byok_routes.len()))
        ));
    }

    for r in reports {
        if r.id == "copilot" && crate::agents::copilot::copilot_proxy_wired() {
            ports.push(format!(
                "{} Copilot proxy (127.0.0.1:{}) â€” {}",
                C.green(Sym::CHECK),
                crate::agents::copilot::copilot_proxy_port(),
                C.green("wired")
            ));
        }
        if r.id == "grok" && crate::agents::grok::grok_oauth_proxy_wired() {
            ports.push(format!(
                "{} Grok OAuth proxy (127.0.0.1:{}) â€” {}",
                C.green(Sym::CHECK),
                crate::util::grok::grok_oauth_proxy_port(),
                C.green("wired")
            ));
        }
    }
    ports
}

fn doctor_missing_tools(tools: &[std::sync::Arc<ToolManifest>], agent_id: &str) -> Vec<String> {
    let mut missing = Vec::new();
    for tool in tools {
        if tool.instruction_only || !is_tool_selected(&tool.id) {
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
