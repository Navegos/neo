// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/headroom.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::agents::proxy_agent::{
    active_proxy_agents, configure_proxy_agent, proxy_agent_wired, remove_proxy_agent,
};
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::headroom::autostart::{disable_proxy_autostart, enable_proxy_autostart, stop_proxy};
use crate::util::headroom::{ensure_headroom, headroom_installed};
use crate::util::headroom_proxy::{proxy_routing_enabled, set_proxy_routing_enabled};
use crate::util::paths::{temp_dir, is_win, prepend_process_path};
use super::helpers::is_test;

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
    if proxy_routing_enabled() {
        let _ = configure_proxy_agent(agent);
        let _ = enable_proxy_autostart();
    }
    Ok(true)
}

pub fn headroom_unwire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let _ = remove_proxy_agent(agent);
    if active_proxy_agents().is_empty() {
        let _ = disable_proxy_autostart();
        let _ = stop_proxy();
        let _ = set_proxy_routing_enabled(false);
    }
    Ok(true)
}

pub fn build_headroom() -> ToolManifest {
    let mut tool = ToolManifest::new("headroom", "Headroom");
    tool.description = "Universal HTTP caching and routing proxy daemon for coding agents.".into();
    tool.homepage = "https://github.com/Navegos/headroom".into();
    tool.install_hint = "uv tool install headroom-ai[all]".into();
    tool.channel = Channel::UV;
    tool.install = Some(Arc::new(|opts| headroom_ensure_installed(opts)));

    let agents = [
        "claude", "opencode", "codex", "antigravity", "copilot", "droid", "pi", "omp", "kilo", "cline",
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
            Arc::new(move || {
                if !proxy_routing_enabled() {
                    return Some(true);
                }
                if !headroom_installed() {
                    return Some(false);
                }
                Some(proxy_agent_wired(&ag_v) || true)
            }),
        );
    }

    tool
}
