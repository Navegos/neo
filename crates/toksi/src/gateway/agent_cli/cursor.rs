// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/cursor.rs
// created: 2026-10-31
// lastModified: 2026-11-18

use std::process::Child;
use std::time::Duration;

pub fn read_cursor_session() -> Option<String> {
    let config_dir = dirs::config_dir()?;
    let path = config_dir.join("Cursor").join("User").join("globalStorage").join("state.vscdb");
    if path.exists() {
        return Some("cursor-local-bridge-active".to_string());
    }
    None
}

/// Spawns the local Cursor IDE RPC bridge process in detached background mode.
pub fn launch_cursor_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "cursor.exe" } else { "cursor" };
    let port_str = port.to_string();
    super::launch_detached_bridge(bin_name, &["--rpc-port", &port_str], "Cursor")
}

/// Probes Cursor RPC bridge reachability.
pub fn verify_cursor_health(port: u16, timeout: Duration) -> bool {
    super::probe_local_livez(port, timeout)
}
