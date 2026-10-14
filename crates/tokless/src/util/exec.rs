// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/exec.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::mcpspawn::codegraph_binary_healthy;
use super::paths::{exists, home, which, IS_WIN};

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
        if !cwd.is_empty() {
            command.current_dir(cwd);
        }
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
    } else {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());
    }

    if let Some(timeout) = opts.timeout {
        return run_with_timeout(command, opts.capture, timeout);
    }

    if opts.capture {
        match command.output() {
            Ok(output) => ExecResult {
                code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            },
            Err(err) => ExecResult {
                code: 127,
                stdout: String::new(),
                stderr: err.to_string(),
            },
        }
    } else {
        match command.status() {
            Ok(status) => ExecResult {
                code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
            },
            Err(err) => ExecResult {
                code: 127,
                stdout: String::new(),
                stderr: err.to_string(),
            },
        }
    }
}

fn run_with_timeout(mut command: Command, capture: bool, timeout: Duration) -> ExecResult {
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(err) => {
            return ExecResult {
                code: 127,
                stdout: String::new(),
                stderr: err.to_string(),
            }
        }
    };

    let out_handle = if capture {
        child.stdout.take().map(|mut out| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let _ = out.read_to_end(&mut buf);
                String::from_utf8_lossy(&buf).to_string()
            })
        })
    } else {
        None
    };

    let err_handle = if capture {
        child.stderr.take().map(|mut err| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let _ = err.read_to_end(&mut buf);
                String::from_utf8_lossy(&buf).to_string()
            })
        })
    } else {
        None
    };

    let start = Instant::now();
    let code = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.code().unwrap_or(1),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_process_tree(&mut child);
                    let _ = child.wait();
                    break 124;
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            Err(_) => break 127,
        }
    };

    let stdout = out_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();
    let stderr = err_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();

    ExecResult {
        code,
        stdout,
        stderr,
    }
}

fn kill_process_tree(child: &mut std::process::Child) {
    if IS_WIN {
        let pid = child.id().to_string();
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
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

pub fn find_binary<S: AsRef<str>>(bin: &str, extra_dirs: &[S]) -> String {
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
    for dir_item in extra_dirs {
        let dir = dir_item.as_ref();
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
            if !local.is_empty() {
                return vec![Path::new(&local)
                    .join("rtk")
                    .join("bin")
                    .to_string_lossy()
                    .to_string()];
            }
        }
        return vec![];
    }
    vec![Path::new(&home())
        .join(".local")
        .join("bin")
        .to_string_lossy()
        .to_string()]
}

pub fn binary_healthy(p: &str) -> bool {
    let r = run(
        p,
        &["--version"],
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
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

fn read_sorted_dirs_desc(parent: &Path, sub: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(parent) {
        let mut dir_paths: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();
        dir_paths.sort();
        dir_paths.reverse();
        for mut p in dir_paths {
            for s in sub {
                p = p.join(s);
            }
            out.push(p.to_string_lossy().to_string());
        }
    }
    out
}

fn node_bin_candidates() -> Vec<String> {
    let h = home();
    let mut dirs = Vec::new();
    let nvm_node = Path::new(&h).join(".nvm").join("versions").join("node");
    dirs.extend(read_sorted_dirs_desc(&nvm_node, &["bin"]));

    let fnm_dir = Path::new(&h).join(".fnm").join("node-versions");
    dirs.extend(read_sorted_dirs_desc(&fnm_dir, &["installation", "bin"]));

    if let Ok(d) = std::env::var("FNM_MULTISHELL_PATH") {
        if !d.is_empty() {
            dirs.push(d);
        }
    }
    dirs.push(Path::new(&h).join(".volta").join("bin").to_string_lossy().to_string());
    dirs.push(Path::new(&h).join(".asdf").join("shims").to_string_lossy().to_string());

    let n_dir = Path::new(&h).join(".n").join("versions").join("node");
    dirs.extend(read_sorted_dirs_desc(&n_dir, &["bin"]));

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
        Path::new(&h)
            .join(".local")
            .join("bin")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".bun")
            .join("bin")
            .to_string_lossy()
            .to_string(),
    ]
}

pub fn resolve_codegraph_bin() -> String {
    let p = which("codegraph");
    if !p.is_empty() && codegraph_binary_healthy(&p) {
        return p;
    }
    for d in codegraph_bin_candidates() {
        if d.is_empty() || !exists(&d) {
            continue;
        }
        let name = if IS_WIN { "codegraph.cmd" } else { "codegraph" };
        let p = Path::new(&d).join(name);
        if exists(&p) && codegraph_binary_healthy(&p.to_string_lossy()) {
            prepend_process_path(&d);
            return p.to_string_lossy().to_string();
        }
    }
    let node_path = resolve_node_binary();
    if !node_path.is_empty() {
        if let Some(dir) = Path::new(&node_path).parent() {
            let name = if IS_WIN { "codegraph.cmd" } else { "codegraph" };
            let p = dir.join(name);
            if exists(&p) && codegraph_binary_healthy(&p.to_string_lossy()) {
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
