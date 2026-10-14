// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/info.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::env;
use crate::commands::helpers::*;
use crate::core::{list_agents, list_tools};
use crate::util::colors::*;
use crate::util::install::install_info;
use crate::util::logger::L;
use crate::util::paths::*;
use crate::util::progress::*;
use crate::util::version::*;
use crate::util::versions::*;

pub fn run_info() -> i32 {
    cmd_header("info", "install + paths");

    let (rec, exact) = install_info();
    tree_top("Tokless");
    tree_leaf(&format!("{}{}", paint_key(&pad_end("version", 10)), C.bold(&paint_ver(&tokless_version()))));
    tree_leaf(&format!("{}{}", paint_key(&pad_end("binary", 10)), paint_path(&tilde_path(&tokless_abs()))));
    tree_leaf(&format!("{}{}", paint_key(&pad_end("install", 10)), format_install_method(&rec, exact)));
    tree_leaf(&format!("{}{}", paint_key(&pad_end("platform", 10)), C.blue(&format!("{}/{}", env::consts::OS, env::consts::ARCH))));
    tree_close();

    tree_corner("Agents");
    let mut n_agents = 0;
    for agent in list_agents() {
        if !agent.detect().installed {
            continue;
        }
        n_agents += 1;
        let dir = if let Some(ref cf) = agent.config_dir {
            tilde_path(&cf())
        } else {
            String::new()
        };
        let path_str = if !dir.is_empty() { paint_path(&dir) } else { C.dim("—") };
        tree_leaf(&format!("{} {} {}", C.green(Sym::CHECK), paint_name(&pad_end(&agent.label, 16)), path_str));
    }
    if n_agents == 0 {
        tree_leaf(&C.dim("none detected on this machine"));
    }
    tree_close();

    tree_corner("Tools");
    for tool in list_tools() {
        if tool.instruction_only {
            continue;
        }
        if let Some(v) = installed_version_for(&tool.id) {
            let mut row = format!("{}{}", paint_name(&pad_end(&tool.id, 14)), paint_ver(&pad_end(&format!("v{}", v), 10)));
            if let Some(p) = installed_path_for(&tool.id) {
                row.push_str(&paint_path(&tilde_path(&p)));
            }
            tree_leaf(&format!("{} {}", C.green(Sym::CHECK), row));
        } else {
            tree_leaf(&format!("{} {}{}", C.gray(&format!("{} ", Sym::BULLET)), C.dim(&pad_end(&tool.id, 14)), C.gray("not installed")));
        }
    }
    tree_close();

    tree_corner("State");
    for (label, path) in [
        ("install marker", install_marker_path()),
        ("version cache", version_cache_path()),
    ] {
        if exists(&path) {
            tree_leaf(&format!("{} {} {}", C.green(Sym::CHECK), paint_name(&pad_end(label, 16)), paint_path(&tilde_path(&path))));
        } else {
            tree_leaf(&format!("{} {} {}{}", C.gray(&format!("{} ", Sym::BULLET)), C.dim(&pad_end(label, 16)), paint_path(&tilde_path(&path)), C.yellow(" missing")));
        }
    }
    tree_close();

    tree_status(&[status_kv(
        &C.cyan(Sym::INFO),
        "tip",
        &format!("{} {}{}", C.gray("run "), paint_cmd("tokless doctor"), C.gray(" for health + updates")),
    )]);

    print_repo_footer(true);
    L.raw("");
    0
}

fn format_install_method(rec: &InstallRecord, exact: bool) -> String {
    let method = if rec.method.is_empty() { "unknown" } else { &rec.method };
    if !exact {
        return format!("{}{}", C.yellow(method), C.gray("  (guessed from path)"));
    }
    let mut out = C.green(&C.bold(method));
    if !rec.at.is_empty() {
        let date_part = if let Some(idx) = rec.at.find('T') {
            &rec.at[..idx]
        } else {
            &rec.at
        };
        out.push(' ');
        out.push_str(&paint_ver(date_part));
    }
    out
}

fn tilde_path(p: &str) -> String {
    if p.is_empty() {
        return String::new();
    }
    let h = home();
    if !h.is_empty() && p.starts_with(&h) {
        format!("~{}", &p[h.len()..])
    } else {
        p.to_string()
    }
}
