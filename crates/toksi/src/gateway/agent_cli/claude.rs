// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/claude.rs
// created: 2026-11-09

use std::process::Child;
use std::time::Duration;

pub fn read_claude_session() -> Option<String> {
    crate::gateway::agent_cli::claude_code::read_claude_code_session()
}

/// Spawns the local Claude Code CLI bridge process in detached background mode.
pub fn launch_claude_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "claude.exe" } else { "claude" };
    let port_str = port.to_string();
    super::launch_detached_bridge(bin_name, &["serve", "--port", &port_str], "Claude")
}

/// Probes Claude CLI bridge endpoint reachability.
pub fn verify_claude_health(port: u16, timeout: Duration) -> bool {
    super::probe_local_livez(port, timeout)
}
