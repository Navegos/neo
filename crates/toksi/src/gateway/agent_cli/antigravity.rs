// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/antigravity.rs
// created: 2026-11-09

use std::process::Child;
use std::time::Duration;

pub fn read_antigravity_session() -> Option<String> {
    crate::gateway::agent_cli::gemini_agy::read_gemini_adc_session()
}

/// Spawns the local Antigravity CLI bridge (agy.exe / agy) in detached background mode.
pub fn launch_antigravity_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "agy.exe" } else { "agy" };
    let port_str = port.to_string();
    super::launch_detached_bridge(bin_name, &["serve", "--port", &port_str], "Antigravity")
}

/// Health check: probes ConnectRPC / /livez endpoint for reachability.
pub fn verify_antigravity_health(port: u16, timeout: Duration) -> bool {
    super::probe_local_livez(port, timeout)
}
