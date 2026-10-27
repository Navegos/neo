// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/projectmem.rs
// created: 2026-10-05
// lastModified: 2026-10-27

use std::path::Path;
use super::paths::{home, tokless_persisted_abs, which, IS_WIN};

pub fn projectmem_bin() -> String {
    let tok = tokless_persisted_abs();
    if !tok.is_empty() && Path::new(&tok).is_file() {
        return tok;
    }
    let p = which("tokless");
    if !p.is_empty() {
        return p;
    }
    let pj = which("pjm");
    if !pj.is_empty() {
        return pj;
    }
    let name = if IS_WIN { "pjm.exe" } else { "pjm" };
    let cand = Path::new(&home()).join(".local").join("bin").join(name);
    if cand.is_file() {
        return cand.to_string_lossy().to_string();
    }
    "tokless".to_string()
}

pub fn projectmem_mcp_bin() -> String {
    let tok = tokless_persisted_abs();
    if !tok.is_empty() && Path::new(&tok).is_file() {
        return tok;
    }
    let p = which("tokless");
    if !p.is_empty() {
        return p;
    }
    let pm = which("pjm-mcp");
    if !pm.is_empty() {
        return pm;
    }
    let name = if IS_WIN { "pjm-mcp.exe" } else { "pjm-mcp" };
    let cand = Path::new(&home()).join(".local").join("bin").join(name);
    if cand.is_file() {
        return cand.to_string_lossy().to_string();
    }
    "tokless".to_string()
}

pub fn projectmem_installed() -> bool {
    true
}

pub fn ensure_projectmem(_upgrade: bool) -> Result<bool, String> {
    Ok(true)
}

pub fn projectmem_installed_version() -> Option<String> {
    Some("0.3.4".to_string())
}
