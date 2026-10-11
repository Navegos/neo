use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::exec::{prepend_process_path, run, RunOptions};
use super::gitwin::install_git_windows_zip;
use super::logger::L;
use super::nodelinux::install_node_unix_tarball;
use super::nodewin::install_node_windows_zip;
use super::npminstall::node_major;
use super::paths::{exists, which, IS_WIN};
use super::prompt::confirm;

static NODE_AGE_CHECKED: AtomicBool = AtomicBool::new(false);

pub fn node_age_already_checked() -> bool {
    NODE_AGE_CHECKED.load(Ordering::Relaxed)
}

pub fn ensure_deps(need_node: bool, need_git: bool, min_node: i32) -> (bool, bool) {
    let mut node_ok = !need_node || node_tools_ready();
    let mut git_ok = !need_git || !which("git").is_empty();

    if node_ok && git_ok {
        if need_node && min_node > 0 {
            let maj = node_major();
            if maj > 0 && maj < min_node {
                NODE_AGE_CHECKED.store(true, Ordering::Relaxed);
                L.warn(&format!(
                    "Node.js v{} is too old for some tools (need v{}+).",
                    maj, min_node
                ));
                if confirm("Upgrade Node.js now? (y/n)", true) {
                    if install_node() && node_tools_ready() && node_major() >= min_node {
                        L.ok("Node.js upgraded.");
                    } else {
                        L.err(&format!(
                            "Node upgrade didn't reach v{}. Some tools may fail to install.",
                            min_node
                        ));
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

pub fn install_git() -> bool {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return true;
    }
    if IS_WIN {
        return install_git_windows_zip();
    }
    L.info("Install git with your package manager (apt/dnf/brew), then re-run: tokless");
    false
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

pub fn install_node() -> bool {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return true;
    }
    if IS_WIN {
        install_node_windows()
    } else {
        install_node_unix()
    }
}

pub fn install_node_for_tools() -> bool {
    install_node()
}

pub fn install_node_unix() -> bool {
    install_node_unix_tarball()
}

pub fn install_node_windows() -> bool {
    if !which("winget").is_empty() {
        let r = run(
            "winget",
            &[
                "install",
                "-e",
                "--id",
                "OpenJS.NodeJS.LTS",
                "--accept-source-agreements",
                "--accept-package-agreements",
                "--silent",
            ],
            RunOptions::default(),
        );
        if r.code == 0 {
            let pf = std::env::var("ProgramFiles")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| r"C:\Program Files".to_string());
            let local_app = std::env::var("LOCALAPPDATA").unwrap_or_default();
            for d in [
                Path::new(&pf).join("nodejs"),
                Path::new(&local_app).join("Programs").join("nodejs"),
            ] {
                if exists(d.join("npm.cmd")) {
                    prepend_process_path(&d.to_string_lossy());
                }
            }
            if !which("npm").is_empty() && !which("npx").is_empty() {
                return true;
            }
        }
        L.warn("winget install didn't complete — falling back to direct download from nodejs.org");
    }
    install_node_windows_zip()
}

pub fn install_cargo() -> bool {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return true;
    }
    if !confirm(
        "RTK needs Rust (cargo) to build for your platform. Install it now?",
        true,
    ) {
        return false;
    }
    if IS_WIN {
        let ps = "$ErrorActionPreference='Stop'; $u='https://win.rustup.rs/x86_64'; $o=\"$env:TEMP\\rustup-init.exe\"; Invoke-WebRequest -UseBasicParsing -Uri $u -OutFile $o; & $o -y --default-toolchain stable --profile minimal";
        return run(
            "powershell",
            &["-NoProfile", "-Command", ps],
            RunOptions::default(),
        )
        .code
            == 0;
    }
    if which("curl").is_empty() && which("wget").is_empty() {
        return false;
    }
    let fetcher = if !which("curl").is_empty() {
        "curl --proto '=https' --tlsv1.2 -sSf"
    } else {
        "wget -qO-"
    };
    let cmd = format!(
        "{} https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path",
        fetcher
    );
    let r = run("sh", &["-c", &cmd], RunOptions::default());
    if r.code != 0 {
        return false;
    }
    let home_env = std::env::var("HOME").unwrap_or_default();
    let cur_path = std::env::var("PATH").unwrap_or_default();
    unsafe {
        std::env::set_var("PATH", format!("{}/.cargo/bin:{}", home_env, cur_path));
    }
    !which("cargo").is_empty()
}
