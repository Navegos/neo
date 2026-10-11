use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use crate::core::ToolManifest;
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::progress::{rule, tree_close, tree_corner, tree_footer, tree_leaf};
use crate::util::versions::{
    installed_version_for, latest_version_for, semver_compare, version_cache_path, VersionInfo,
};

pub const REPO_URL: &str = "https://github.com/Navegos/neo";
pub const ISSUES_URL: &str = "https://github.com/Navegos/neo/issues";

#[derive(Default, Clone, Debug)]
pub struct InitOptions {
    pub agents: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub agent: String,
    pub dry_run: bool,
    pub yes: bool,
    pub verbose: bool,
    pub upgrade: bool,
}

pub fn bust_version_cache_file() {
    let p = version_cache_path();
    if !p.is_empty() && Path::new(&p).exists() {
        let _ = fs::remove_file(&p);
    }
}

pub fn tool_version_note(tool: &ToolManifest) -> String {
    if let Some(v) = installed_version_for(&tool.id) {
        return format!("v{}", v);
    }
    if let Some(v) = latest_version_for(&tool.id) {
        return format!("v{}", v);
    }
    String::new()
}

pub fn tool_version_outdated(info: Option<&VersionInfo>) -> bool {
    if let Some(info) = info {
        if let (Some(inst), Some(lat)) = (&info.installed, &info.latest) {
            return semver_compare(inst, lat) < 0;
        }
    }
    false
}

pub fn tool_version_display_line(
    tool_id: &str,
    info: Option<&VersionInfo>,
    upgrade_mode: bool,
) -> String {
    let name = paint_name(&pad_end(tool_id, 14));
    let installed = if let Some(info) = info {
        if let Some(ref inst) = info.installed {
            paint_ver(&pad_end(&format!("v{}", inst), 10))
        } else {
            C.dim(&pad_end("not on PATH", 10))
        }
    } else {
        C.dim(&pad_end("not on PATH", 10))
    };

    let latest = if let Some(info) = info {
        if let Some(ref lat) = info.latest {
            paint_ver(&pad_end(&format!("v{}", lat), 10))
        } else {
            C.dim(&pad_end("?", 10))
        }
    } else {
        C.dim(&pad_end("?", 10))
    };

    let mut mark = C.gray(Sym::BULLET);
    let mut suffix = String::new();

    if let Some(info) = info {
        if let (Some(inst), Some(lat)) = (&info.installed, &info.latest) {
            if semver_compare(inst, lat) < 0 {
                mark = C.yellow("↑");
                if upgrade_mode {
                    suffix = C.yellow(" → upgrade");
                }
            } else {
                mark = C.green(Sym::CHECK);
                if upgrade_mode {
                    suffix = C.green(" (up to date)");
                }
            }
        } else if info.installed.is_none() && info.latest.is_some() {
            if upgrade_mode {
                mark = C.yellow("+");
                suffix = C.yellow(" → install");
            }
        } else if upgrade_mode {
            suffix = C.dim(" (latest unknown)");
        }
    } else if upgrade_mode {
        suffix = C.dim(" (latest unknown)");
    }

    format!("{} {} {} {} {}{}", mark, name, installed, paint_arrow(), latest, suffix)
}

pub fn tool_needs_node(tool: &ToolManifest) -> bool {
    if tool.instruction_only {
        return false;
    }
    tool.needs_node || tool.min_node_major > 0
}

pub fn first_line(s: &str) -> &str {
    if let Some(idx) = s.find('\n') {
        &s[..idx]
    } else {
        s
    }
}

pub fn plural(n: usize) -> String {
    if n == 1 {
        "1 tool".into()
    } else {
        format!("{} tool(s)", n)
    }
}

pub fn join_comma(ss: &[String]) -> String {
    ss.join(", ")
}

pub fn print_failure_detail(logs: &HashMap<String, String>) {
    for (label, out) in logs {
        let lines = last_non_empty_lines(out, 4);
        if lines.is_empty() {
            continue;
        }
        L.raw(&format!("      {}", C.gray(&format!("{}:", label))));
        for ln in lines {
            L.raw(&format!("        {}", C.gray(&ln)));
        }
    }
}

pub fn last_non_empty_lines(s: &str, n: usize) -> Vec<String> {
    let mut keep = Vec::new();
    for ln in s.split('\n') {
        let stripped = strip_ansi(ln.trim());
        if !stripped.is_empty() {
            keep.push(stripped);
        }
    }
    if keep.len() > n {
        keep.split_off(keep.len() - n)
    } else {
        keep
    }
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(&next) = chars.peek() {
                chars.next();
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        if c == '\r' {
            continue;
        }
        out.push(c);
    }
    out
}

pub use crate::util::progress::pad_end;

pub fn cmd_header(cmd: &str, subtitle: &str) {
    L.raw("");
    L.raw(&format!(
        "  {}  {}",
        C.bold(&C.cyan(&format!("tokless {}", cmd))),
        C.gray(subtitle)
    ));
    L.raw("");
}

pub fn tree_status(lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    tree_corner("Status");
    for line in lines {
        tree_leaf(line);
    }
    tree_close();
}

pub fn paint_ver(v: &str) -> String {
    C.soft(v)
}
pub fn paint_name(s: &str) -> String {
    C.bold(s)
}
pub fn paint_path(s: &str) -> String {
    C.gray(s)
}
pub fn paint_key(s: &str) -> String {
    C.cyan(s)
}
pub fn paint_cmd(s: &str) -> String {
    C.bold(&C.cyan(s))
}
pub fn paint_arrow() -> String {
    C.dim("→")
}

pub fn status_ok(msg: &str) -> String {
    format!("{} {}", C.green(Sym::CHECK), C.green(msg))
}

pub fn status_warn(msg: &str) -> String {
    format!("{} {}", C.yellow(Sym::WARN), C.yellow(msg))
}

pub fn status_info(msg: &str) -> String {
    format!("{} {}", C.cyan(Sym::INFO), msg)
}

pub fn status_kv(mark: &str, key: &str, value: &str) -> String {
    format!("{} {} {}", mark, paint_key(&pad_end(key, 8)), value)
}

pub fn print_repo_footer(tree: bool) {
    if env::var("TOKLESS_TEST").as_deref() == Ok("1") {
        return;
    }
    if tree {
        tree_footer(52);
        L.raw(&format!(
            "  {} {}",
            C.gray("If tokless helps, please star it here: "),
            C.cyan(REPO_URL)
        ));
        L.raw(&format!(
            "  {} {}",
            C.gray("If you hit any issue or have ideas, please raise it here: "),
            C.cyan(ISSUES_URL)
        ));
        return;
    }
    L.raw("");
    L.raw(&C.gray(&rule(52)));
    L.raw(&format!(
        "  {} {}",
        C.gray("If tokless helps, please star it here: "),
        C.cyan(REPO_URL)
    ));
    L.raw(&format!(
        "  {} {}",
        C.gray("If you hit any issue or have ideas, please raise it here: "),
        C.cyan(ISSUES_URL)
    ));
}

pub fn stdout_tty() -> bool {
    crate::util::isatty::stdout_is_tty()
}
