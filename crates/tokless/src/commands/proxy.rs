// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/proxy.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::HashSet;

use crate::agents::proxy_agent::{
    active_proxy_agents, configure_proxy_agent, detect_proxy, proxy_agent_applicable,
    proxy_agent_uses_headroom, proxy_spec_for, proxy_state_label, remove_proxy_agent, ProxyState,
};
use crate::agents::proxy_protocol::ProxyWireKind;
use crate::agents::{copilot_cli_proxy_port, copilot_proxy_port};
use crate::commands::helpers::*;
use crate::core::get_agent;
use crate::headroom::autostart::{
    disable_proxy_autostart, enable_proxy_autostart, proxy_autostart_enabled, start_proxy,
    stop_proxy,
};
use crate::headroom::proxy::{proxy_port, proxy_running};
use crate::util::colors::*;
use crate::util::headroom_proxy::set_proxy_routing_enabled;
use crate::util::logger::L;
use crate::util::progress::*;

pub fn proxy_agent_ids() -> Vec<String> {
    crate::agents::proxy_agent::proxy_agent_ids()
}

pub fn resolve_proxy_agents(requested: &Option<Vec<String>>, for_up: bool) -> (Vec<String>, Vec<String>) {
    let valid_ids = proxy_agent_ids();
    let valid_set: HashSet<&str> = valid_ids.iter().map(|s| s.as_str()).collect();

    if let Some(req) = requested {
        let mut selected = Vec::new();
        let mut unknown = Vec::new();
        for id in req {
            if valid_set.contains(id.as_str()) {
                selected.push(id.clone());
            } else {
                unknown.push(id.clone());
            }
        }
        return (selected, unknown);
    }

    let mut auto = Vec::new();
    for id in valid_ids {
        if !for_up || proxy_agent_applicable(&id) {
            auto.push(id);
        }
    }
    (auto, Vec::new())
}

pub fn validate_proxy_up_agents(targets: &[String], explicit: bool) -> Result<(), String> {
    for id in targets {
        if id == "grok" {
            return Err("grok does not support proxy routing".into());
        }
        if explicit && !proxy_agent_applicable(id) {
            return Err(format!("agent '{}' is not installed or configured on this machine", id));
        }
    }
    Ok(())
}

pub fn validate_proxy_lane_ports() -> Result<(), String> {
    let main_p = proxy_port();
    let cop_p = copilot_proxy_port();
    let cop_cli_p = copilot_cli_proxy_port();
    if main_p == cop_p || main_p == cop_cli_p || cop_p == cop_cli_p {
        return Err(format!(
            "conflicting proxy lane ports: headroom={}, copilot={}, copilot-cli={}",
            main_p, cop_p, cop_cli_p
        ));
    }
    Ok(())
}

pub fn proxy_instructions(agent_id: &str) -> Vec<String> {
    let port = proxy_port();
    match agent_id {
        "cursor" => vec![
            format!("Cursor settings → Models → Override OpenAI Base URL: http://127.0.0.1:{}/v1", port),
        ],
        _ => Vec::new(),
    }
}

pub fn run_proxy_up(opts: InitOptions) -> i32 {
    cmd_header("proxy up", "route agents through headroom");

    let explicit = opts.agents.is_some();
    let (targets, unknown) = resolve_proxy_agents(&opts.agents, true);
    if !unknown.is_empty() {
        L.err(&format!("unknown proxy agent(s): {}", join_comma(&unknown)));
        return 1;
    }
    if let Err(e) = validate_proxy_up_agents(&targets, explicit) {
        L.err(&e);
        return 1;
    }
    if let Err(e) = validate_proxy_lane_ports() {
        L.err(&e);
        return 1;
    }

    if opts.dry_run {
        L.raw(&format!(
            "  {} Dry run — would start headroom and configure agents.",
            C.cyan(Sym::INFO)
        ));
        L.raw("");
        return 0;
    }

    let needs_headroom = targets.iter().any(|id| proxy_agent_uses_headroom(id));
    if needs_headroom {
        if let Err(e) = start_proxy() {
            L.err(&format!("failed to start headroom proxy: {}", e));
            return 1;
        }
        let _ = enable_proxy_autostart();
    }
    let _ = set_proxy_routing_enabled(true);

    tree_top("Agents");
    let mut configured = Vec::new();
    let mut manual_notes = Vec::new();

    for id in &targets {
        let label = get_agent(id)
            .map(|a| a.label.clone())
            .unwrap_or_else(|| id.clone());
        if let Some(spec) = proxy_spec_for(id) {
            if spec.wire_kind == ProxyWireKind::Manual {
                tree_leaf(&format!(
                    "{} {} {}",
                    C.cyan(Sym::INFO),
                    paint_name(&pad_end(&label, 16)),
                    C.cyan("manual setup required")
                ));
                manual_notes.extend(proxy_instructions(id));
                continue;
            }
        }
        let ok = configure_proxy_agent(id);
        if ok {
            configured.push(id.clone());
            tree_leaf(&format!(
                "{} {} {}",
                C.green(Sym::CHECK),
                paint_name(&pad_end(&label, 16)),
                C.green("routed")
            ));
        } else {
            tree_leaf(&format!(
                "{} {} {}",
                C.yellow(Sym::WARN),
                paint_name(&pad_end(&label, 16)),
                C.yellow("could not route")
            ));
        }
    }
    tree_close();

    if !manual_notes.is_empty() {
        tree_corner("Manual Steps");
        for note in manual_notes {
            tree_leaf(&C.gray(&note));
        }
        tree_close();
    }

    tree_status(&[status_ok(&format!(
        "Headroom running on port {}.",
        proxy_port()
    ))]);
    print_repo_footer(true);
    L.raw("");
    0
}

pub fn run_proxy_down(opts: InitOptions) -> i32 {
    cmd_header("proxy down", "restore direct agent connections");

    let explicit = opts.agents.is_some();
    let (targets, unknown) = resolve_proxy_agents(&opts.agents, false);
    if !unknown.is_empty() {
        L.err(&format!("unknown proxy agent(s): {}", join_comma(&unknown)));
        return 1;
    }

    if opts.dry_run {
        L.raw(&format!(
            "  {} Dry run — would restore direct routing and stop headroom.",
            C.cyan(Sym::INFO)
        ));
        L.raw("");
        return 0;
    }

    tree_top("Agents");
    for id in &targets {
        let label = get_agent(id)
            .map(|a| a.label.clone())
            .unwrap_or_else(|| id.clone());
        let ok = remove_proxy_agent(id);
        let mark = if ok { C.green(Sym::CHECK) } else { C.gray(Sym::BULLET) };
        let status = if ok {
            C.green("restored")
        } else {
            C.gray("untouched")
        };
        tree_leaf(&format!("{} {} {}", mark, paint_name(&pad_end(&label, 16)), status));
    }
    tree_close();

    if !explicit || active_proxy_agents().is_empty() {
        let _ = disable_proxy_autostart();
        let _ = stop_proxy();
        let _ = set_proxy_routing_enabled(false);
        tree_status(&[status_ok("Headroom stopped and agent routing restored.")]);
    } else {
        tree_status(&[status_ok("Selected agent routing restored.")]);
    }

    print_repo_footer(true);
    L.raw("");
    0
}

pub fn run_proxy_status(opts: InitOptions) -> i32 {
    cmd_header("proxy status", "headroom proxy routing");

    let running = proxy_running();
    let autostart = proxy_autostart_enabled();

    tree_top("Daemon");
    let status_str = if running {
        format!("{} on port {}", C.green("running"), proxy_port())
    } else {
        C.dim("stopped")
    };
    tree_leaf(&format!("{}{}", paint_key(&pad_end("status", 12)), status_str));
    let autostart_str = if autostart {
        C.green("enabled")
    } else {
        C.dim("disabled")
    };
    tree_leaf(&format!(
        "{}{}",
        paint_key(&pad_end("autostart", 12)),
        autostart_str
    ));
    tree_close();

    let (targets, _) = resolve_proxy_agents(&opts.agents, false);
    tree_corner("Agents");
    for id in &targets {
        let label_name = get_agent(id)
            .map(|a| a.label.clone())
            .unwrap_or_else(|| id.clone());
        let det = detect_proxy(id);
        let state_label = proxy_state_label(det.state);
        let mark = match det.state {
            ProxyState::ProxyStateManaged
            | ProxyState::ProxyStateManagedBYOK
            | ProxyState::ProxyStateManagedOAuth => C.green(Sym::CHECK),
            ProxyState::ProxyStateNativeDirect
            | ProxyState::ProxyStateNativeUnconfigured
            | ProxyState::ProxyStateAbsent => C.gray(Sym::BULLET),
            _ => C.yellow(Sym::WARN),
        };
        let detail_suffix = if !det.detail.is_empty() {
            format!(" {}", C.gray(&format!("({})", det.detail)))
        } else {
            String::new()
        };
        tree_leaf(&format!(
            "{} {} {}{}",
            mark,
            paint_name(&pad_end(&label_name, 16)),
            state_label,
            detail_suffix
        ));
    }
    tree_close();

    print_repo_footer(true);
    L.raw("");
    0
}
