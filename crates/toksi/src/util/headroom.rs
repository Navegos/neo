// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/headroom.rs
// created: 2026-10-05
// lastModified: 2026-11-06

use std::path::Path;
use super::mcpspawn::{pick_mcp_spawn, McpSpawn};
use super::paths::{exists, toksi_data_dir};

pub struct HeadroomPaths {
    pub root: String,
    pub bin: String,
    pub tools: String,
}

pub fn headroom_paths_resolved() -> HeadroomPaths {
    let root = Path::new(&toksi_data_dir()).join("headroom");
    let bin = root.join("bin");
    let tools = root.join("tools");
    HeadroomPaths {
        tools: tools.to_string_lossy().to_string(),
        bin: bin.to_string_lossy().to_string(),
        root: root.to_string_lossy().to_string(),
    }
}

pub fn headroom_bin() -> String {
    let toksi = crate::util::paths::toksi_persisted_abs();
    if !toksi.is_empty() && exists(&toksi) {
        return toksi;
    }
    let t_abs = crate::util::paths::toksi_abs();
    if !t_abs.is_empty() && exists(&t_abs) {
        return t_abs;
    }
    if let Ok(cur) = std::env::current_exe() {
        let cur_str = cur.to_string_lossy().to_string();
        if exists(&cur_str) && !crate::util::paths::is_test_executable(&cur_str) {
            return cur_str;
        }
    }
    let w = crate::util::paths::which("toksi");
    if !w.is_empty() && exists(&w) {
        return w;
    }
    "toksi".to_string()
}

pub fn headroom_env() -> Vec<(String, String)> {
    Vec::new()
}

pub fn mcp_spawn_for(tool_id: &str) -> McpSpawn {
    pick_mcp_spawn(tool_id, &[])
}

pub fn headroom_installed() -> bool {
    // Native Rust headroom engine is fully embedded directly in toksi
    true
}

pub fn headroom_installed_version() -> Option<String> {
    Some(crate::util::version::toksi_version().to_string())
}

pub fn ensure_headroom(upgrade: bool) -> Result<bool, String> {
    crate::headroom::ensure_installed(crate::core::RunOpts {
        upgrade,
        ..Default::default()
    })
    .map_err(|e| e.to_string())
}
