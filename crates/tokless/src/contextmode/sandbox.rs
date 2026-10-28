// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/sandbox.rs
// created: 2026-10-27

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::util::paths::{is_win, which};
use super::filter::{process_sandbox_output, DEFAULT_MAX_BYTES, DEFAULT_MAX_LINES};
use super::security::{check_command_deny, check_path_deny};
use super::stats::record_sandbox_bytes;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const MAX_RAW_OUTPUT_BYTES: usize = 50 * 1024 * 1024; // 50MB limit before OOM kill

#[derive(Debug, Clone)]
pub struct ExecRequest {
    pub language: String,
    pub code: String,
    pub cwd: Option<PathBuf>,
    pub timeout_secs: Option<u64>,
    pub max_lines: Option<usize>,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeMap {
    pub shell: String,
    pub python: Option<String>,
    pub node: Option<String>,
    pub bun: Option<String>,
    pub deno: Option<String>,
    pub rust: Option<String>,
}

pub fn detect_runtimes() -> RuntimeMap {
    let mut map = RuntimeMap::default();

    if is_win() {
        if !which("pwsh").is_empty() {
            map.shell = "pwsh".to_string();
        } else if !which("powershell").is_empty() {
            map.shell = "powershell".to_string();
        } else if !which("bash").is_empty() {
            map.shell = "bash".to_string();
        } else {
            map.shell = "cmd".to_string();
        }
    } else if !which("bash").is_empty() {
        map.shell = "bash".to_string();
    } else {
        map.shell = "sh".to_string();
    }

    // Python probe
    for py in ["python3", "python", "py"] {
        let p = which(py);
        if !p.is_empty() {
            map.python = Some(py.to_string());
            break;
        }
    }

    // JS/TS runtimes
    if !which("bun").is_empty() {
        map.bun = Some("bun".to_string());
    }
    if !which("node").is_empty() {
        map.node = Some("node".to_string());
    }
    if !which("deno").is_empty() {
        map.deno = Some("deno".to_string());
    }
    if !which("rustc").is_empty() {
        map.rust = Some("rustc".to_string());
    }

    map
}

pub struct PolyglotSandbox {
    runtimes: RuntimeMap,
    temp_dir: PathBuf,
}

impl Default for PolyglotSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl PolyglotSandbox {
    pub fn new() -> Self {
        let runtimes = detect_runtimes();
        let temp_dir = std::env::temp_dir().join("tokless-sandbox");
        let _ = fs::create_dir_all(&temp_dir);
        Self { runtimes, temp_dir }
    }

    pub fn execute(&self, req: &ExecRequest) -> Result<ExecResult, String> {
        let lang = req.language.to_lowercase();
        let timeout = Duration::from_secs(req.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));
        let max_lines = req.max_lines.unwrap_or(DEFAULT_MAX_LINES);
        let max_bytes = req.max_bytes.unwrap_or(DEFAULT_MAX_BYTES);

        // Security check
        if let Some(denied) = check_command_deny(&req.code) {
            return Ok(ExecResult {
                stdout: String::new(),
                stderr: denied,
                exit_code: 126,
                timed_out: false,
                duration_ms: 0,
            });
        }

        let script_file = self.prepare_script(&lang, &req.code)?;
        let (cmd, args) = self.build_invocation(&lang, &script_file)?;

        let mut command = Command::new(&cmd);
        command.args(&args);

        if let Some(ref cwd) = req.cwd {
            command.current_dir(cwd);
        }

        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let start_time = Instant::now();
        let mut child = command.spawn().map_err(|e| format!("Failed to spawn sandbox process '{}': {}", cmd, e))?;

        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();

        let (tx, rx) = mpsc::channel();

        // Background reader thread for stdout & stderr
        std::thread::spawn(move || {
            let mut out = Vec::new();
            let mut err = Vec::new();

            if let Some(mut h) = stdout_handle {
                let mut buf = [0u8; 8192];
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 || out.len() > MAX_RAW_OUTPUT_BYTES {
                        break;
                    }
                    out.extend_from_slice(&buf[..n]);
                }
            }

            if let Some(mut h) = stderr_handle {
                let mut buf = [0u8; 8192];
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 || err.len() > MAX_RAW_OUTPUT_BYTES {
                        break;
                    }
                    err.extend_from_slice(&buf[..n]);
                }
            }

            let _ = tx.send((out, err));
        });

        // Wait with timeout
        let mut timed_out = false;
        let mut exit_code = -1;

        let wait_start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_code = status.code().unwrap_or(-1);
                    break;
                }
                Ok(None) => {
                    if wait_start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        timed_out = true;
                        exit_code = 124; // Standard timeout exit code
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(_) => {
                    let _ = child.kill();
                    break;
                }
            }
        }

        let (raw_out, raw_err) = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Clean up temporary script file
        let _ = fs::remove_file(&script_file);

        let raw_stdout_str = String::from_utf8_lossy(&raw_out).to_string();
        let raw_stderr_str = String::from_utf8_lossy(&raw_err).to_string();

        let total_consumed = raw_stdout_str.len() + raw_stderr_str.len();
        record_sandbox_bytes(total_consumed);

        let filtered_stdout = process_sandbox_output(Some(&req.code), &raw_stdout_str, max_lines, max_bytes);
        let filtered_stderr = process_sandbox_output(None, &raw_stderr_str, max_lines, max_bytes);

        Ok(ExecResult {
            stdout: filtered_stdout,
            stderr: filtered_stderr,
            exit_code,
            timed_out,
            duration_ms,
        })
    }

    pub fn execute_file(
        &self,
        target_path: &Path,
        code: &str,
        language: Option<&str>,
        cwd: Option<&Path>,
        timeout_secs: Option<u64>,
    ) -> Result<ExecResult, String> {
        if let Some(denied) = check_path_deny(target_path) {
            return Ok(ExecResult {
                stdout: String::new(),
                stderr: denied,
                exit_code: 126,
                timed_out: false,
                duration_ms: 0,
            });
        }

        if !target_path.exists() {
            return Err(format!("Target file does not exist: {}", target_path.display()));
        }

        let lang = language
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.detect_language_for_file(target_path));

        let abs_path = target_path
            .canonicalize()
            .unwrap_or_else(|_| target_path.to_path_buf());
        let path_str = abs_path.to_string_lossy().to_string();

        // Polyglot file injection: provide TARGET_FILE and read contents into FILE_CONTENT / TARGET_PATH
        let wrapped_code = match lang.as_str() {
            "python" => format!(
                "import os, sys\nTARGET_PATH = r'{}'\nwith open(TARGET_PATH, 'r', encoding='utf-8', errors='ignore') as f:\n    FILE_CONTENT = f.read()\n\n{}",
                path_str, code
            ),
            "javascript" | "typescript" => format!(
                "const fs = require('fs');\nconst TARGET_PATH = '{}';\nconst FILE_CONTENT = fs.readFileSync(TARGET_PATH, 'utf8');\n\n{}",
                path_str.replace('\\', "\\\\"), code
            ),
            _ => format!(
                "export TARGET_PATH='{}'\n{}",
                path_str, code
            ),
        };

        self.execute(&ExecRequest {
            language: lang,
            code: wrapped_code,
            cwd: cwd.map(|p| p.to_path_buf()),
            timeout_secs,
            max_lines: None,
            max_bytes: None,
        })
    }

    fn detect_language_for_file(&self, path: &Path) -> String {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "py" => "python".to_string(),
            "js" | "mjs" | "cjs" => "javascript".to_string(),
            "ts" | "mts" | "cts" => "typescript".to_string(),
            "sh" | "bash" | "zsh" => "shell".to_string(),
            "ps1" => "shell".to_string(),
            _ => {
                if self.runtimes.python.is_some() {
                    "python".to_string()
                } else if self.runtimes.node.is_some() || self.runtimes.bun.is_some() {
                    "javascript".to_string()
                } else {
                    "shell".to_string()
                }
            }
        }
    }

    fn prepare_script(&self, language: &str, code: &str) -> Result<PathBuf, String> {
        let unique = format!("ctx-{}-{}.tmp", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), std::process::id());

        let (ext, content) = match language {
            "shell" | "bash" | "sh" => {
                if is_win() && (self.runtimes.shell.contains("powershell") || self.runtimes.shell.contains("pwsh")) {
                    let mut ps_content = String::from("\u{feff}[Console]::InputEncoding = [System.Text.UTF8Encoding]::new()\n[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()\n$OutputEncoding = [System.Text.UTF8Encoding]::new()\n");
                    ps_content.push_str(code);
                    ("ps1", ps_content)
                } else if is_win() && self.runtimes.shell == "cmd" {
                    ("cmd", code.to_string())
                } else {
                    ("sh", code.to_string())
                }
            }
            "python" | "py" => ("py", code.to_string()),
            "javascript" | "js" => ("js", code.to_string()),
            "typescript" | "ts" => ("ts", code.to_string()),
            "ruby" | "rb" => ("rb", code.to_string()),
            "rust" | "rs" => ("rs", code.to_string()),
            "go" => ("go", code.to_string()),
            _ => ("txt", code.to_string()),
        };

        let file_path = self.temp_dir.join(format!("{}.{}", unique, ext));
        fs::write(&file_path, content).map_err(|e| format!("Failed to write sandbox script: {}", e))?;
        Ok(file_path)
    }

    fn build_invocation(&self, language: &str, script_file: &Path) -> Result<(String, Vec<String>), String> {
        let script_str = script_file.to_string_lossy().to_string();

        match language {
            "shell" | "bash" | "sh" => {
                if self.runtimes.shell == "pwsh" || self.runtimes.shell == "powershell" {
                    Ok((
                        self.runtimes.shell.clone(),
                        vec![
                            "-NoProfile".to_string(),
                            "-NonInteractive".to_string(),
                            "-ExecutionPolicy".to_string(),
                            "Bypass".to_string(),
                            "-File".to_string(),
                            script_str,
                        ],
                    ))
                } else if self.runtimes.shell == "cmd" {
                    Ok(("cmd".to_string(), vec!["/c".to_string(), script_str]))
                } else {
                    Ok((self.runtimes.shell.clone(), vec![script_str]))
                }
            }
            "python" | "py" => {
                let py = self.runtimes.python.as_ref().ok_or("Python runtime not found in PATH")?;
                Ok((py.clone(), vec![script_str]))
            }
            "javascript" | "js" => {
                if let Some(ref bun) = self.runtimes.bun {
                    Ok((bun.clone(), vec!["run".to_string(), script_str]))
                } else if let Some(ref node) = self.runtimes.node {
                    Ok((node.clone(), vec![script_str]))
                } else if let Some(ref deno) = self.runtimes.deno {
                    Ok((deno.clone(), vec!["run".to_string(), "-A".to_string(), script_str]))
                } else {
                    Err("No JavaScript runtime (bun, node, deno) found in PATH".to_string())
                }
            }
            "typescript" | "ts" => {
                if let Some(ref bun) = self.runtimes.bun {
                    Ok((bun.clone(), vec!["run".to_string(), script_str]))
                } else if let Some(ref deno) = self.runtimes.deno {
                    Ok((deno.clone(), vec!["run".to_string(), "-A".to_string(), script_str]))
                } else if let Some(ref node) = self.runtimes.node {
                    Ok((node.clone(), vec![script_str]))
                } else {
                    Err("No TypeScript runtime (bun, deno, node) found in PATH".to_string())
                }
            }
            _ => {
                // Fall back to shell execution
                self.build_invocation("shell", script_file)
            }
        }
    }
}
