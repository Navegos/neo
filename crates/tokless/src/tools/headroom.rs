// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/headroom.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::agents::opencode::open_code_proxy_satisfied;
use crate::agents::proxy_agent::{
    configure_proxy_agent, proxy_agent_applicable, proxy_agent_wired, proxy_endpoint_for,
     proxy_readiness_for, remove_proxy_agent, ProxyReadiness,
};
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::headroom::autostart::{
    disable_proxy_autostart, enable_proxy_autostart, proxy_autostart_configured, start_proxy,
    stop_proxy, ERR_PROXY_AUTOSTART_UNAVAILABLE,
};
use crate::headroom::proxy::{
    acquire_proxy_lifecycle_lock, proxy_running, stop_proxy_preserving_autostart,
};
use crate::util::headroom::{ensure_headroom, headroom_bin, headroom_installed};
use crate::util::headroom_proxy::{
    clear_headroom_proxy_runtime, clear_proxy_routing_preference, proxy_routing_enabled,
    proxy_routing_preference_set, read_proxy_runtime, save_headroom_proxy_runtime,
    set_proxy_routing_enabled,
};
use crate::util::paths::{is_win, prepend_process_path, temp_dir};
use super::helpers::is_test;

/// Reports agents with an applicable config that tokless can wire.
pub fn headroom_wired(id: &str) -> bool {
    !proxy_endpoint_for(id).is_empty() && proxy_agent_applicable(id)
}

pub fn headroom_ensure_installed(opts: &RunOpts) -> Result<bool, DynError> {
    if is_test() {
        let shim_dir = Path::new(&temp_dir()).join("tokless-test-headroom");
        let _ = fs::create_dir_all(&shim_dir);
        let shim_path = shim_dir.join(if is_win() { "headroom.bat" } else { "headroom" });
        let _ = fs::write(&shim_path, if is_win() { "@echo headroom 0.5.32" } else { "#!/bin/sh\necho headroom 0.5.32\n" });
        prepend_process_path(&shim_dir.to_string_lossy());
        return Ok(true);
    }

    if let Some(ref report) = opts.report {
        report("checking", 0.1);
    }
    if headroom_installed() && !opts.upgrade {
        if let Some(ref report) = opts.report {
            report("already installed", 1.0);
        }
        return Ok(true);
    }
    if opts.dry_run {
        return Ok(true);
    }
    if let Some(ref report) = opts.report {
        report(if opts.upgrade { "upgrading" } else { "installing" }, 0.3);
    }
    ensure_headroom(opts.upgrade).map_err(|e| -> DynError { e.into() })?;
    if let Some(ref report) = opts.report {
        report("ready", 1.0);
    }
    Ok(true)
}

pub fn headroom_wire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if !headroom_wired(agent) {
        return Ok(true);
    }
    if proxy_routing_preference_set() && !proxy_routing_enabled() {
        return Ok(true);
    }
    let _lifecycle_lock = acquire_proxy_lifecycle_lock()
        .map_err(|e| -> DynError { format!("proxy lifecycle lock: {}", e).into() })?;
    if proxy_routing_preference_set() && !proxy_routing_enabled() {
        return Ok(true);
    }
    if !is_test() && !headroom_installed() {
        return Err(format!("headroom is not installed at {}", headroom_bin()).into());
    }
    let preference_set = proxy_routing_preference_set();
    let preference_enabled = proxy_routing_enabled();
    let runtime_before = read_proxy_runtime();
    let restore_runtime = || -> Result<(), DynError> {
        if let Some(ref st) = runtime_before {
            save_headroom_proxy_runtime(st)
        } else {
            clear_headroom_proxy_runtime().map_err(|e| -> DynError { e.into() })
        }
    };

    let mut was_running = false;
    let mut autostart_before = false;
    if !is_test() {
        was_running = proxy_running();
        autostart_before = proxy_autostart_configured();
        if let Err(err) = start_proxy() {
            let mut msg = err.to_string();
            if let Err(r_err) = restore_runtime() {
                msg = format!("{}; {}", msg, r_err);
            }
            return Err(msg.into());
        }
        if !autostart_before {
            if let Err(err) = enable_proxy_autostart() {
                if !err.to_string().contains(ERR_PROXY_AUTOSTART_UNAVAILABLE) {
                    let mut msg = err.to_string();
                    if !was_running {
                        if let Err(stop_err) = stop_proxy() {
                            msg = format!("{}; proxy rollback: {}", msg, stop_err);
                        }
                    }
                    if let Err(r_err) = restore_runtime() {
                        msg = format!("{}; {}", msg, r_err);
                    }
                    return Err(msg.into());
                }
            }
        }
    }

    let was_wired = proxy_agent_wired(agent);
    let rollback = || -> Option<String> {
        let mut errs = Vec::new();
        if !was_wired && proxy_agent_wired(agent) && !remove_proxy_agent(agent) {
            errs.push(format!("{} proxy wiring rollback failed", agent));
        }
        if !is_test() {
            if !was_running {
                let stop_res = if autostart_before {
                    stop_proxy_preserving_autostart()
                } else {
                    stop_proxy()
                };
                if let Err(err) = stop_res {
                    errs.push(format!("proxy rollback: {}", err));
                }
            } else if !autostart_before {
                if let Err(err) = disable_proxy_autostart() {
                    errs.push(format!("autostart rollback: {}", err));
                }
                if let Err(err) = start_proxy() {
                    errs.push(format!("proxy restart rollback: {}", err));
                }
            }
        }
        if !preference_set {
            if let Err(err) = clear_proxy_routing_preference() {
                errs.push(format!("proxy preference rollback: {}", err));
            }
        } else if let Err(err) = set_proxy_routing_enabled(preference_enabled) {
            errs.push(format!("proxy preference rollback: {}", err));
        }
        if let Err(err) = restore_runtime() {
            errs.push(format!("proxy runtime rollback: {}", err));
        }
        if errs.is_empty() {
            None
        } else {
            Some(errs.join("; "))
        }
    };

    if !configure_proxy_agent(agent) {
        let mut msg = format!(
            "{} proxy wiring not applied (differing existing config value, or write failed)",
            agent
        );
        if let Some(rb) = rollback() {
            msg = format!("{}; {}", msg, rb);
        }
        return Err(msg.into());
    }
    if !proxy_routing_preference_set() {
        if let Err(err) = set_proxy_routing_enabled(true) {
            let mut msg = format!("headroom proxy preference: {}", err);
            if let Some(rb) = rollback() {
                msg = format!("{}; {}", msg, rb);
            }
            return Err(msg.into());
        }
    }
    if !headroom_verify(agent) {
        let mut msg = format!("{} proxy wiring verification failed", agent);
        if let Some(rb) = rollback() {
            msg = format!("{}; {}", msg, rb);
        }
        return Err(msg.into());
    }
    Ok(true)
}

pub fn headroom_unwire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if !headroom_wired(agent) {
        return Ok(false);
    }
    if !proxy_agent_wired(agent) {
        return Ok(false);
    }
    let _lifecycle_lock = acquire_proxy_lifecycle_lock()
        .map_err(|e| -> DynError { format!("proxy lifecycle lock: {}", e).into() })?;
    if !headroom_wired(agent) || !proxy_agent_wired(agent) {
        return Ok(false);
    }
    if !remove_proxy_agent(agent) {
        return Err(format!("{} proxy unwire failed — removal did not take effect", agent).into());
    }
    if proxy_agent_wired(agent) {
        return Err(format!("{} proxy unwire incomplete — managed wiring remains", agent).into());
    }
    Ok(true)
}

pub fn headroom_verify(agent: &str) -> bool {
    if !is_test() {
        return matches!(
            proxy_readiness_for(agent),
            ProxyReadiness::Ready
                | ProxyReadiness::Manual
                | ProxyReadiness::ByokExcluded
                | ProxyReadiness::RoutingDisabled
                | ProxyReadiness::NotApplicable
        );
    }
    // Test homes have no real headroom bin, so verify wiring only.
    if proxy_routing_preference_set() && !proxy_routing_enabled() {
        return true;
    }
    if !headroom_wired(agent) {
        return true;
    }
    if agent == "opencode" {
        return open_code_proxy_satisfied();
    }
    proxy_agent_wired(agent)
}

pub fn build_headroom() -> ToolManifest {
    let mut tool = ToolManifest::new("headroom", "Headroom");
    tool.description = "On-demand token compression proxy for large, self-contained text.".into();
    tool.homepage = "https://github.com/headroomlabs-ai/headroom".into();
    tool.install_hint = "Tokless-managed uv tool: headroom-ai[proxy] (Python 3.14).".into();
    tool.channel = Channel::UV;
    tool.install = Some(Arc::new(|opts| headroom_ensure_installed(opts)));

    let agents = [
        "claude",
        "opencode",
        "opencode2",
        "codex",
        "cursor",
        "antigravity",
        "copilot",
        "droid",
        "grok",
        "kilo",
        "omp",
        "pi",
        "cline",
        "kimi",
        "zcode",
        "dsh",
        "amp",
    ];

    for agent_id in agents {
        let ag = agent_id.to_string();
        let ag_w = ag.clone();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(move |opts| headroom_wire(&ag_w, opts)),
        );
        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| headroom_unwire(&ag_u, opts)),
        );
        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(headroom_verify(&ag_v))),
        );
        let ag_a = ag.clone();
        tool.wired_any_for.insert(
            ag,
            Arc::new(move || Some(headroom_wired(&ag_a))),
        );
    }

    tool
}
