use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use super::paths::{home, which, exists, IS_WIN};

#[derive(Debug, Clone, Default)]
pub struct ExecResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub capture: bool,
    pub quiet: bool,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub timeout: Option<Duration>,
}

pub fn run(cmd: &str, args: &[&str], opts: RunOptions) -> ExecResult {
    let mut command = Command::new(cmd);
    command.args(args);

    if let Some(ref cwd) = opts.cwd {
        command.current_dir(cwd);
    }
    for (k, v) in &opts.env {
        command.env(k, v);
    }

    if opts.capture {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    } else if opts.quiet {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    match command.output() {
        Ok(output) => ExecResult {
            code: output.status.code().unwrap_or(0),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(err) => ExecResult {
            code: 127,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

pub fn run_cmd(cmd: &str, args: &[&str], env: &[(&str, &str)]) -> ExecResult {
    let mut command = Command::new(cmd);
    command.args(args);
    for (k, v) in env {
        command.env(k, v);
    }
    match command.output() {
        Ok(output) => ExecResult {
            code: output.status.code().unwrap_or(0),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(err) => ExecResult {
            code: 127,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

pub fn prepend_process_path(dir: &str) {

    let sep = if IS_WIN { ";" } else { ":" };
    let cur = std::env::var("PATH").unwrap_or_default();
    for d in cur.split(sep) {
        if d == dir {
            return;
        }
    }
    unsafe {
        std::env::set_var("PATH", format!("{}{}{}", dir, sep, cur));
    }
}

pub fn find_binary(bin: &str, extra_dirs: &[&str]) -> String {
    let p = which(bin);
    if !p.is_empty() {
        return p;
    }
    let names: Vec<String> = if IS_WIN {
        vec![
            format!("{}.exe", bin),
            format!("{}.cmd", bin),
            format!("{}.bat", bin),
            bin.to_string(),
        ]
    } else {
        vec![bin.to_string()]
    };
    for &dir in extra_dirs {
        if dir.is_empty() {
            continue;
        }
        for n in &names {
            let p = Path::new(dir).join(n);
            if p.is_file() {
                prepend_process_path(dir);
                return p.to_string_lossy().to_string();
            }
        }
    }
    String::new()
}

pub fn which_any(bins: &[&str]) -> (String, String) {
    for &b in bins {
        let p = which(b);
        if !p.is_empty() {
            return (b.to_string(), p);
        }
    }
    (String::new(), String::new())
}

pub fn rtk_install_dirs() -> Vec<String> {
    if IS_WIN {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return vec![Path::new(&local).join("rtk").join("bin").to_string_lossy().to_string()];
        }
        return vec![];
    }
    vec![Path::new(&home()).join(".local").join("bin").to_string_lossy().to_string()]
}

pub fn binary_healthy(p: &str) -> bool {
    let r = run(p, &["--version"], RunOptions { capture: true, ..Default::default() });
    r.code == 0 && r.stdout.contains('.')
}

pub fn resolve_rtk_bin() -> String {
    let p = which("rtk");
    if !p.is_empty() && binary_healthy(&p) {
        return p;
    }
    for d in rtk_install_dirs() {
        if !d.is_empty() {
            prepend_process_path(&d);
        }
    }
    which("rtk")
}

fn node_bin_candidates() -> Vec<String> {
    let h = home();
    let mut dirs = Vec::new();
    let nvm_node = Path::new(&h).join(".nvm").join("versions").join("node");
    if let Ok(entries) = std::fs::read_dir(nvm_node) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                dirs.push(entry.path().join("bin").to_string_lossy().to_string());
            }
        }
    }
    let fnm_dir = Path::new(&h).join(".fnm").join("node-versions");
    if let Ok(entries) = std::fs::read_dir(fnm_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                dirs.push(entry.path().join("installation").join("bin").to_string_lossy().to_string());
            }
        }
    }
    if let Ok(d) = std::env::var("FNM_MULTISHELL_PATH") {
        if !d.is_empty() {
            dirs.push(d);
        }
    }
    dirs.push(Path::new(&h).join(".volta").join("bin").to_string_lossy().to_string());
    dirs.push(Path::new(&h).join(".asdf").join("shims").to_string_lossy().to_string());
    dirs.push("/opt/homebrew/bin".to_string());
    dirs.push("/usr/local/bin".to_string());
    dirs
}

pub fn resolve_node_binary() -> String {
    let p = which("node");
    if !p.is_empty() && binary_healthy(&p) {
        return p;
    }
    for d in node_bin_candidates() {
        if d.is_empty() || !exists(&d) {
            continue;
        }
        let exe = if IS_WIN { "node.exe" } else { "node" };
        let p = Path::new(&d).join(exe);
        if exists(&p) && binary_healthy(&p.to_string_lossy()) {
            prepend_process_path(&d);
            return p.to_string_lossy().to_string();
        }
    }
    String::new()
}

pub fn resolve_npm_binary() -> String {
    let p = which("npm");
    if !p.is_empty() {
        return p;
    }
    let node_path = resolve_node_binary();
    if node_path.is_empty() {
        return String::new();
    }
    if let Some(dir) = Path::new(&node_path).parent() {
        let exe = if IS_WIN { "npm.cmd" } else { "npm" };
        let p = dir.join(exe);
        if exists(&p) {
            prepend_process_path(&dir.to_string_lossy());
            return p.to_string_lossy().to_string();
        }
    }
    String::new()
}

fn codegraph_bin_candidates() -> Vec<String> {
    let h = home();
    vec![
        Path::new(&h).join(".local").join("bin").to_string_lossy().to_string(),
        Path::new(&h).join(".bun").join("bin").to_string_lossy().to_string(),
    ]
}

pub fn resolve_codegraph_bin() -> String {
    let p = which("codegraph");
    if !p.is_empty() && binary_healthy(&p) {
        return p;
    }
    for d in codegraph_bin_candidates() {
        if d.is_empty() || !exists(&d) {
            continue;
        }
        let name = if IS_WIN { "codegraph.cmd" } else { "codegraph" };
        let p = Path::new(&d).join(name);
        if exists(&p) && binary_healthy(&p.to_string_lossy()) {
            prepend_process_path(&d);
            return p.to_string_lossy().to_string();
        }
    }
    let node_path = resolve_node_binary();
    if !node_path.is_empty() {
        if let Some(dir) = Path::new(&node_path).parent() {
            let name = if IS_WIN { "codegraph.cmd" } else { "codegraph" };
            let p = dir.join(name);
            if exists(&p) && binary_healthy(&p.to_string_lossy()) {
                prepend_process_path(&dir.to_string_lossy());
                return p.to_string_lossy().to_string();
            }
        }
    }
    String::new()
}

pub fn resolve_bun_binary() -> String {
    let p = which("bun");
    if !p.is_empty() {
        return p;
    }
    let p = Path::new(&home()).join(".bun").join("bin").join("bun");
    if exists(&p) {
        if let Some(dir) = p.parent() {
            prepend_process_path(&dir.to_string_lossy());
        }
        return p.to_string_lossy().to_string();
    }
    String::new()
}
