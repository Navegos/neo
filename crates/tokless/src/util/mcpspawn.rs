// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/mcpspawn.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, UNIX_EPOCH};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::exec::{resolve_codegraph_bin, run, RunOptions};
use super::paths::{tokless_persisted_abs, which, IS_WIN};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct McpSpawn {
    pub command: String,
    pub args: Vec<String>,
}

fn spawn_command(bin: &str, resolved: &str) -> String {
    if !resolved.is_empty() {
        resolved.to_string()
    } else {
        bin.to_string()
    }
}

fn wrap_cmd_shim(s: McpSpawn) -> McpSpawn {
    if !IS_WIN {
        return s;
    }
    let mut p = s.command.clone();
    if !Path::new(&p).is_absolute() {
        p = which(&s.command);
    }
    let ext = Path::new(&p)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext == "cmd" || ext == "bat" {
        let mut new_args = vec!["/c".to_string(), p];
        new_args.extend(s.args);
        return McpSpawn {
            command: "cmd".to_string(),
            args: new_args,
        };
    }
    s
}

pub fn tokless_run_mcp_command() -> String {
    let self_exe = tokless_persisted_abs();
    if !self_exe.is_empty() {
        self_exe
    } else {
        "tokless".to_string()
    }
}

pub fn wrap_auto_index(agent: &str, s: McpSpawn) -> McpSpawn {
    let self_cmd = tokless_run_mcp_command();
    let mut args = vec![
        "run-mcp".to_string(),
        "--agent".to_string(),
        agent.to_string(),
        s.command,
    ];
    args.extend(s.args);
    McpSpawn {
        command: self_cmd,
        args,
    }
}

pub fn wrap_context_mode(s: McpSpawn) -> McpSpawn {
    let self_cmd = tokless_run_mcp_command();
    let mut args = vec![
        "run-mcp".to_string(),
        "--context-mode".to_string(),
        s.command,
    ];
    args.extend(s.args);
    McpSpawn {
        command: self_cmd,
        args,
    }
}

pub fn wrap_bounded_tool(tool_id: &str, s: McpSpawn) -> McpSpawn {
    let self_cmd = tokless_run_mcp_command();
    let mut args = vec![
        "run-mcp".to_string(),
        "--tool".to_string(),
        tool_id.to_string(),
        s.command,
    ];
    args.extend(s.args);
    McpSpawn {
        command: self_cmd,
        args,
    }
}

pub fn pick_codegraph_spawn(extra_args: &[&str]) -> Option<McpSpawn> {
    let p = resolve_codegraph_bin();
    if !p.is_empty() {
        let spawn = McpSpawn {
            command: spawn_command("codegraph", &p),
            args: extra_args.iter().map(|s| s.to_string()).collect(),
        };
        return Some(wrap_cmd_shim(spawn));
    }
    None
}

pub fn pick_mcp_spawn(bin: &str, extra_args: &[&str]) -> McpSpawn {
    let extra_vec: Vec<String> = extra_args.iter().map(|s| s.to_string()).collect();

    if bin == "codegraph" {
        if let Some(spawn) = pick_codegraph_spawn(extra_args) {
            return spawn;
        }
    } else if bin == "projectmem" {
        let mut spawn = wrap_cmd_shim(McpSpawn {
            command: "pjm-mcp".to_string(),
            args: extra_vec.clone(),
        });
        let p = which("pjm-mcp");
        if !p.is_empty() {
            spawn = wrap_cmd_shim(McpSpawn {
                command: spawn_command("pjm-mcp", &p),
                args: extra_vec,
            });
        }
        return wrap_bounded_tool("projectmem", spawn);
    } else {
        let p = which(bin);
        if !p.is_empty() {
            let spawn = wrap_cmd_shim(McpSpawn {
                command: spawn_command(bin, &p),
                args: extra_vec,
            });
            if bin == "context-mode" {
                return wrap_context_mode(spawn);
            }
            return spawn;
        }
    }

    let pkg = match bin {
        "context-mode" => "context-mode",
        "codegraph" => "@colbymchenry/codegraph",
        _ => bin,
    };

    let mut args = vec!["--no-install".to_string(), pkg.to_string()];
    args.extend(extra_vec);
    let mut cmd = "npx".to_string();
    let p = which("npx");
    if !p.is_empty() {
        cmd = spawn_command("npx", &p);
    }
    let spawn = wrap_cmd_shim(McpSpawn { command: cmd, args });
    if bin == "context-mode" {
        return wrap_context_mode(spawn);
    }
    spawn
}

pub fn mcp_spawn_for(tool_id: &str) -> McpSpawn {
    pick_mcp_spawn(tool_id, &[])
}

#[derive(Debug, Clone, Copy)]
struct CodegraphHealthEntry {
    size: u64,
    modified_ns: u128,
    ok: bool,
}

static CODEGRAPH_HEALTH: LazyLock<Mutex<HashMap<String, CodegraphHealthEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn semver_like_matches(s: &str) -> bool {
    Regex::new(r"\d+\.\d+(\.\d+)?")
        .map(|re| re.is_match(s))
        .unwrap_or(false)
}

fn codegraph_probe_command(p: &str) -> (String, Vec<String>) {
    if IS_WIN {
        let ext = Path::new(p)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "cmd" || ext == "bat" {
            return ("cmd".to_string(), vec!["/c".to_string(), p.to_string()]);
        }
    }
    (p.to_string(), Vec::new())
}

pub fn codegraph_binary_healthy(p: &str) -> bool {
    let meta = std::fs::metadata(p).ok();
    let stat_key = meta.as_ref().and_then(|m| {
        let mod_ns = m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
        Some((m.len(), mod_ns))
    });

    if let Some((size, mod_ns)) = stat_key {
        if let Ok(guard) = CODEGRAPH_HEALTH.lock() {
            if let Some(cached) = guard.get(p) {
                if cached.size == size && cached.modified_ns == mod_ns {
                    return cached.ok;
                }
            }
        }
    }

    let (cmd, mut args) = codegraph_probe_command(p);
    args.push("--version".to_string());
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let version = run(
        &cmd,
        &arg_refs,
        RunOptions {
            capture: true,
            timeout: Some(Duration::from_secs(8)),
            ..Default::default()
        },
    );
    let ok = version.code == 0 && semver_like_matches(&version.stdout);

    if let Some((size, modified_ns)) = stat_key {
        if let Ok(mut guard) = CODEGRAPH_HEALTH.lock() {
            guard.insert(
                p.to_string(),
                CodegraphHealthEntry {
                    size,
                    modified_ns,
                    ok,
                },
            );
        }
    }
    ok
}

pub fn codegraph_spawn_healthy(command: &str, prefix_args: &[String]) -> bool {
    if command.is_empty() {
        return false;
    }
    let mut args = prefix_args.to_vec();
    args.push("--version".to_string());
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let version = run(
        command,
        &arg_refs,
        RunOptions {
            capture: true,
            timeout: Some(Duration::from_secs(8)),
            ..Default::default()
        },
    );
    if version.code != 0 || !semver_like_matches(&version.stdout) {
        return false;
    }
    codegraph_mcp_tools_healthy(command, prefix_args)
}

pub fn codegraph_mcp_tools_healthy(command: &str, prefix_args: &[String]) -> bool {
    codegraph_mcp_tools_healthy_mode(command, prefix_args, false)
        || codegraph_mcp_tools_healthy_mode(command, prefix_args, true)
}

pub fn codegraph_mcp_tools_healthy_mode(
    command: &str,
    prefix_args: &[String],
    framed: bool,
) -> bool {
    let mut args = prefix_args.to_vec();
    args.push("serve".to_string());
    args.push("--mcp".to_string());

    let mut child = match Command::new(command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut stdin = match child.stdin.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
    };
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
    };

    let requests = [
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "tokless", "version": "0" }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    ];

    for req in &requests {
        if write_mcp_probe_request(&mut stdin, req, framed).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut rd = BufReader::new(stdout);
        loop {
            let msg = match read_mcp_probe_response(&mut rd, framed) {
                Ok(m) => m,
                Err(_) => {
                    let _ = tx.send(false);
                    return;
                }
            };
            #[derive(Deserialize)]
            struct ProbeResp {
                #[serde(default)]
                id: i64,
                #[serde(default)]
                result: ProbeResult,
            }
            #[derive(Deserialize, Default)]
            struct ProbeResult {
                #[serde(default)]
                tools: Vec<ProbeTool>,
            }
            #[derive(Deserialize)]
            struct ProbeTool {
                #[serde(default)]
                name: String,
            }
            if let Ok(resp) = serde_json::from_slice::<ProbeResp>(&msg) {
                if resp.id == 2 {
                    let found = resp
                        .result
                        .tools
                        .iter()
                        .any(|t| t.name == "codegraph_explore");
                    let _ = tx.send(found);
                    return;
                }
            }
        }
    });

    let healthy = rx
        .recv_timeout(Duration::from_secs(8))
        .unwrap_or(false);
    let _ = child.kill();
    let _ = child.wait();
    healthy
}

pub fn write_mcp_probe_request<W: Write>(
    w: &mut W,
    req: &serde_json::Value,
    framed: bool,
) -> io::Result<()> {
    let b = serde_json::to_vec(req).unwrap_or_default();
    if framed {
        let header = format!("Content-Length: {}\r\n\r\n", b.len());
        w.write_all(header.as_bytes())?;
        w.write_all(&b)?;
    } else {
        w.write_all(&b)?;
        w.write_all(b"\n")?;
    }
    w.flush()
}

pub fn read_mcp_probe_response<R: BufRead>(rd: &mut R, framed: bool) -> io::Result<Vec<u8>> {
    if !framed {
        let mut line = Vec::new();
        let n = rd.read_until(b'\n', &mut line)?;
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "eof"));
        }
        return Ok(line);
    }
    let mut length: usize = 0;
    loop {
        let mut line = String::new();
        let n = rd.read_line(&mut line)?;
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "eof"));
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            if k.trim().eq_ignore_ascii_case("Content-Length") {
                length = v.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }
    if length == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "missing Content-Length",
        ));
    }
    let mut buf = vec![0u8; length];
    rd.read_exact(&mut buf)?;
    Ok(buf)
}
