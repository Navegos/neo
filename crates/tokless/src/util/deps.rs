// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/deps.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use super::paths::which;
use super::logger::L;
pub use super::preflight::ensure_deps;

#[derive(Debug, Clone, Default)]
pub struct RuntimeReport {
    pub node: String,
    pub npm: String,
    pub npx: String,
    pub bun: String,
    pub bunx: String,
    pub cargo: String,
    pub python: String,
}

pub fn detect_runtimes() -> RuntimeReport {
    let mut py = which("python3");
    if py.is_empty() {
        py = which("python");
    }
    RuntimeReport {
        node: which("node"),
        npm: which("npm"),
        npx: which("npx"),
        bun: which("bun"),
        bunx: which("bunx"),
        cargo: which("cargo"),
        python: py,
    }
}

pub fn pick_js_runner() -> (String, Vec<String>, bool) {
    if !which("npx").is_empty() {
        return ("npx".to_string(), vec!["-y".to_string()], true);
    }
    if !which("bunx").is_empty() {
        return ("bunx".to_string(), vec![], true);
    }
    (String::new(), vec![], false)
}

pub fn report_runtimes(r: &RuntimeReport) {
    let fmt_line = |name: &str, p: &str| {
        if !p.is_empty() {
            L.sub(&format!("{}: {}", name, p));
        } else {
            L.sub(&format!("{}: not found", name));
        }
    };
    fmt_line("node", &r.node);
    let npmnpx = if !r.npm.is_empty() { &r.npm } else { &r.npx };
    fmt_line("npm/npx", npmnpx);
    let bunbunx = if !r.bun.is_empty() { &r.bun } else { &r.bunx };
    fmt_line("bun/bunx", bunbunx);
    fmt_line("cargo", &r.cargo);
    fmt_line("python", &r.python);
}
