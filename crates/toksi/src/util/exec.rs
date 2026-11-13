// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/exec.rs
// created: 2026-10-05
// lastModified: 2026-11-13

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};

use super::paths::{home, which, IS_WIN};

static EXTRA_PATH_DIRS: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| RwLock::new(Vec::new()));

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
    let trimmed = dir.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Ok(mut guard) = EXTRA_PATH_DIRS.write() {
        if !guard.iter().any(|d| d == trimmed) {
            guard.insert(0, trimmed.to_string());
        }
    }
}

pub fn extra_process_path_dirs() -> Vec<String> {
    EXTRA_PATH_DIRS
        .read()
        .map(|g| g.clone())
        .unwrap_or_default()
}

pub fn find_binary<S: AsRef<str>>(bin: &str, extra_dirs: &[S]) -> String {
    let dirs: Vec<&Path> = extra_dirs
        .iter()
        .map(|s| Path::new(s.as_ref()))
        .collect();

    if let Some(p) = super::paths::find_real_binary(bin, &dirs) {
        if let Some(parent) = p.parent() {
            prepend_process_path(&parent.to_string_lossy());
        }
        return p.to_string_lossy().to_string();
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
    if std::env::var("TOKSI_TEST").as_deref() == Ok("1") {
        return true;
    }
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
