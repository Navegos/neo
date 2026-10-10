use super::paths::{which, IS_WIN};
use super::exec::{run, RunOptions};
use super::npminstall::node_major;
use super::logger::L;
use super::prompt::confirm;

pub fn node_tools_ready() -> bool {
    let npm = which("npm");
    let npx = which("npx");
    !npm.is_empty() && !npx.is_empty()
}

pub fn ensure_deps(need_node: bool, need_git: bool, min_node: i32) -> (bool, bool) {
    let mut node_ok = !need_node || node_tools_ready();
    let mut git_ok = !need_git || !which("git").is_empty();

    if node_ok && git_ok {
        if need_node && min_node > 0 {
            let maj = node_major();
            if maj > 0 && maj < min_node {
                L.warn(&format!("Node.js v{} is too old for some tools (need v{}+).", maj, min_node));
                if confirm("Upgrade Node.js now? (y/n)", true) {
                    if install_node() && node_tools_ready() && node_major() >= min_node {
                        L.ok("Node.js upgraded.");
                    } else {
                        L.err(&format!("Node upgrade didn't reach v{}. Some tools may fail to install.", min_node));
                    }
                } else {
                    L.sub("Skipping. Update Node: https://nodejs.org/en/download");
                }
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
    L.warn(&format!("Missing: {}", missing.join(", ")));
    if !confirm("Install now?", true) {
        L.info("Skipping. Node: https://nodejs.org/en/download · git: https://git-scm.com/downloads");
        return (node_ok, git_ok);
    }
    if !node_ok {
        if install_node() && node_tools_ready() {
            L.ok("Node.js installed.");
            node_ok = true;
        } else {
            L.err("Node install didn't complete. Manual: https://nodejs.org/en/download");
        }
    }
    if !git_ok {
        if install_git() && !which("git").is_empty() {
            L.ok("Git installed.");
            git_ok = true;
        } else {
            L.err("Git install didn't complete. Manual: https://git-scm.com/downloads");
        }
    }
    (node_ok, git_ok)
}

pub fn install_node() -> bool {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return true;
    }
    if IS_WIN {
        if !which("winget").is_empty() {
            let r = run("winget", &["install", "-e", "--id", "OpenJS.NodeJS.LTS", "--accept-source-agreements", "--accept-package-agreements", "--silent"], RunOptions::default());
            if r.code == 0 && node_tools_ready() {
                return true;
            }
        }
        return false;
    }
    false
}

pub fn install_git() -> bool {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return true;
    }
    if IS_WIN && !which("winget").is_empty() {
        let r = run("winget", &["install", "-e", "--id", "Git.Git", "--accept-source-agreements", "--accept-package-agreements", "--silent"], RunOptions::default());
        return r.code == 0 && !which("git").is_empty();
    }
    false
}

