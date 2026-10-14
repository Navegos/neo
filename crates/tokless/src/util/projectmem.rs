// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/projectmem.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::path::Path;
use std::time::Duration;
use super::exec::{run, RunOptions};
use super::paths::{home, which, IS_WIN};
use super::uv::ensure_uv;

pub fn projectmem_bin() -> String {
    let p = which("pjm");
    if !p.is_empty() {
        return p;
    }
    let name = if IS_WIN { "pjm.exe" } else { "pjm" };
    let cand = Path::new(&home()).join(".local").join("bin").join(name);
    if cand.is_file() {
        return cand.to_string_lossy().to_string();
    }
    String::new()
}

pub fn projectmem_mcp_bin() -> String {
    let p = which("pjm-mcp");
    if !p.is_empty() {
        return p;
    }
    let name = if IS_WIN { "pjm-mcp.exe" } else { "pjm-mcp" };
    let cand = Path::new(&home()).join(".local").join("bin").join(name);
    if cand.is_file() {
        return cand.to_string_lossy().to_string();
    }
    String::new()
}

pub fn projectmem_installed() -> bool {
    !projectmem_mcp_bin().is_empty()
}

pub fn ensure_projectmem(upgrade: bool) -> Result<bool, String> {
    if projectmem_installed() && !upgrade {
        return Ok(true);
    }
    let uv = ensure_uv().map_err(|e| e.to_string())?;
    let args: &[&str] = if upgrade {
        &["tool", "install", "--force", "--upgrade", "projectmem"]
    } else {
        &["tool", "install", "--force", "projectmem"]
    };
    let res = run(
        &uv,
        args,
        RunOptions {
            capture: true,
            timeout: Some(Duration::from_secs(180)),
            ..Default::default()
        },
    );
    if res.code != 0 {
        let msg = if !res.stderr.trim().is_empty() {
            res.stderr.trim().to_string()
        } else {
            res.stdout.trim().to_string()
        };
        return Err(format!("uv tool install projectmem failed: {}", msg));
    }
    Ok(true)
}

pub fn projectmem_installed_version() -> Option<String> {
    let uv = which("uv");
    if uv.is_empty() {
        return None;
    }
    let res = run(
        &uv,
        &["tool", "list"],
        RunOptions {
            capture: true,
            timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    );
    if res.code != 0 {
        return None;
    }
    for line in res.stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 && parts[0] == "projectmem" {
            return Some(parts[1].trim_start_matches('v').to_string());
        }
    }
    None
}
