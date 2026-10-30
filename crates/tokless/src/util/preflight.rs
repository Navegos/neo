// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/preflight.rs
// created: 2026-10-05
// lastModified: 2026-10-30

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::exec::{run, RunOptions};
use super::logger::L;
use super::paths::{exists, which, IS_WIN};

static NODE_AGE_CHECKED: AtomicBool = AtomicBool::new(false);

pub fn node_age_already_checked() -> bool {
    NODE_AGE_CHECKED.load(Ordering::Relaxed)
}

pub fn node_major() -> i32 {
    let node_bin = which("node");
    if node_bin.is_empty() {
        return 0;
    }
    let r = run(
        &node_bin,
        &["--version"],
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
    let v = r.stdout.trim().trim_start_matches('v');
    let maj = v.split('.').next().unwrap_or(v);
    maj.parse::<i32>().unwrap_or(0)
}

pub fn ensure_deps(need_node: bool, need_git: bool, min_node: i32) -> (bool, bool) {
    let mut node_ok = !need_node || node_tools_ready();
    let git_ok = !need_git || !which("git").is_empty();

    if node_ok && git_ok {
        if need_node && min_node > 0 {
            let maj = node_major();
            if maj > 0 && maj < min_node {
                NODE_AGE_CHECKED.store(true, Ordering::Relaxed);
                L.warn(&format!(
                    "Node.js v{} is too old for some tools (need v{}+).",
                    maj, min_node
                ));
                L.sub("Please update Node.js on system PATH: https://nodejs.org/en/download");
                node_ok = false;
            }
        }
        return (node_ok, git_ok);
    }

    let mut missing = Vec::new();
    if !node_ok {
        missing.push("Node.js/npm");
    }
    if !git_ok {
        missing.push("git");
    }
    L.warn(&format!("Missing optional dependencies on PATH: {}", missing.join(", ")));
    L.info("Tokless uses existing system runtimes only. Install via system package manager or official site:");
    if !node_ok {
        L.sub("Node.js: https://nodejs.org/en/download");
    }
    if !git_ok {
        L.sub("Git: https://git-scm.com/downloads");
    }
    (node_ok, git_ok)
}

pub fn node_tools_ready() -> bool {
    let npm = which("npm");
    let npx = which("npx");
    if npm.is_empty() || npx.is_empty() {
        return false;
    }
    if is_wsl() && (is_windows_mount(&npm) || is_windows_mount(&npx)) {
        L.warn(&format!(
            "Found Windows Node ({}) in WSL — a Linux Node install is needed here.",
            npm
        ));
        return false;
    }
    true
}

pub fn is_wsl() -> bool {
    !IS_WIN
        && (!std::env::var("WSL_DISTRO_NAME").unwrap_or_default().is_empty()
            || !std::env::var("WSL_INTEROP").unwrap_or_default().is_empty())
}

pub fn windows_home_from_wsl() -> String {
    if !is_wsl() {
        return String::new();
    }
    let r = run(
        "cmd.exe",
        &["/c", "echo %USERPROFILE%"],
        RunOptions {
            capture: true,
            quiet: true,
            ..Default::default()
        },
    );
    let win = r.stdout.trim();
    if r.code != 0 || win.is_empty() || win.contains("%USERPROFILE%") {
        return String::new();
    }
    let p = run(
        "wslpath",
        &["-u", win],
        RunOptions {
            capture: true,
            quiet: true,
            ..Default::default()
        },
    );
    let out = p.stdout.trim();
    if p.code != 0 || out.is_empty() {
        return String::new();
    }
    out.to_string()
}

pub fn windows_cursor_home_from_wsl() -> String {
    let h = windows_home_from_wsl();
    if h.is_empty() {
        return String::new();
    }
    for sub in ["cursor", "Cursor"] {
        let p = Path::new(&h)
            .join("AppData")
            .join("Local")
            .join("Programs")
            .join(sub)
            .join("Cursor.exe");
        if exists(&p) {
            return h;
        }
    }
    String::new()
}

pub fn is_windows_mount(p: &str) -> bool {
    let b = p.as_bytes();
    b.len() > 6 && p.starts_with("/mnt/") && b[6] == b'/' && b[5].is_ascii_lowercase()
}
