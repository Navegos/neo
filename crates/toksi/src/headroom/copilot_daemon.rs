// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/copilot_daemon.rs
// created: 2026-10-11
// lastModified: 2026-11-13

use std::fs::OpenOptions;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::process_identity::{
    identify_process, kill_process, process_gone, spawn_detached, ProcessIdentityInfo,
};
use super::proxy::{
    acquire_proxy_lifecycle_lock, acquire_proxy_start_lock, cleanup_unverified_child,
    resolve_headroom_bin, verify_identity_with_retry, write_proxy_ownership, ProxyOwnership,
    PROXY_POLL_INTERVAL, PROXY_PROBE_TIMEOUT, PROXY_READY_TIMEOUT, PROXY_STOP_TIMEOUT,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::proxy_routing_enabled;
use crate::util::logger::with_quiet;
use crate::util::paths::{ensure_dir, home, read_file_safe, IS_WIN};

pub const DEFAULT_COPILOT_PROXY_PORT: u16 = 8789;
pub const DEFAULT_COPILOT_CLI_PROXY_PORT: u16 = 8790;

pub fn copilot_proxy_port() -> u16 {
    crate::util::ports::resolved_ports().copilot_vscode_proxy
}

pub fn copilot_cli_proxy_port() -> u16 {
    crate::util::ports::resolved_ports().copilot_cli_proxy
}

pub fn validate_copilot_ports() -> Result<(), String> {
    let vscode_port = copilot_proxy_port();
    let cli_port = copilot_cli_proxy_port();
    if vscode_port == cli_port {
        return Err(format!(
            "Port collision detected: Copilot VS Code and CLI proxy cannot both use port {}",
            vscode_port
        ));
    }
    if vscode_port == 0 || cli_port == 0 {
        return Err("Copilot proxy ports must be greater than 0".to_string());
    }
    Ok(())
}

pub fn copilot_proxy_files() -> (String, String) {
    let root = headroom_paths_resolved().root;
    let pid_file = Path::new(&root)
        .join("copilot-vscode.pid")
        .to_string_lossy()
        .to_string();
    let log_file = Path::new(&root)
        .join("copilot-vscode.log")
        .to_string_lossy()
        .to_string();
    (pid_file, log_file)
}

pub fn copilot_cli_proxy_files() -> (String, String) {
    let root = headroom_paths_resolved().root;
    let pid_file = Path::new(&root)
        .join("copilot-cli.pid")
        .to_string_lossy()
        .to_string();
    let log_file = Path::new(&root)
        .join("copilot-cli.log")
        .to_string_lossy()
        .to_string();
    (pid_file, log_file)
}

pub fn copilot_proxy_args() -> Vec<String> {
    vec![
        "__copilot-proxy".to_string(),
        "--port".to_string(),
        copilot_proxy_port().to_string(),
    ]
}

pub fn copilot_cli_proxy_args() -> Vec<String> {
    vec![
        "__copilot-cli-proxy".to_string(),
        "--port".to_string(),
        copilot_cli_proxy_port().to_string(),
    ]
}

pub fn copilot_port_live_z(port: u16, timeout: Duration) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect(&addr) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    use std::io::{Read, Write};
    let req = format!("GET /livez HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 1024];
    let Ok(n) = stream.read(&mut buf) else {
        return false;
    };
    if n == 0 {
        return false;
    }
    let s = String::from_utf8_lossy(&buf[..n]);
    (s.starts_with("HTTP/1.1 200") || s.starts_with("HTTP/1.0 200"))
        && (s.contains("headroom-proxy")
            || s.contains("copilot-proxy")
            || s.contains("copilot-cli-proxy")
            || s.contains("\"status\":\"ok\""))
}

pub fn copilot_proxy_live_z(timeout: Duration) -> bool {
    copilot_port_live_z(copilot_proxy_port(), timeout)
}

pub fn copilot_cli_proxy_live_z(timeout: Duration) -> bool {
    copilot_port_live_z(copilot_cli_proxy_port(), timeout)
}

fn copilot_vscode_proxy_wired() -> bool {
    let h = home();
    let mut settings_paths = Vec::new();
    if IS_WIN {
        if let Ok(appdata) = std::env::var("APPDATA") {
            settings_paths.push(
                Path::new(&appdata)
                    .join("Code")
                    .join("User")
                    .join("settings.json"),
            );
        }
        settings_paths.push(
            Path::new(&h)
                .join("AppData")
                .join("Roaming")
                .join("Code")
                .join("User")
                .join("settings.json"),
        );
    } else if std::env::consts::OS == "macos" {
        settings_paths.push(
            Path::new(&h)
                .join("Library")
                .join("Application Support")
                .join("Code")
                .join("User")
                .join("settings.json"),
        );
    } else {
        settings_paths.push(
            Path::new(&h)
                .join(".config")
                .join("Code")
                .join("User")
                .join("settings.json"),
        );
    }
    let needle = format!(":{}", copilot_proxy_port());
    for p in settings_paths {
        if let Some(raw) = read_file_safe(&p) {
            if raw.contains(&needle) {
                return true;
            }
        }
    }
    false
}

/// Starts Headroom's OAuth-aware VS Code wrapper when needed.
/// Fails open because session hooks must not block Copilot startup.
/// Skips start when VS Code is not Toksi-wired so `proxy down` stays down.
pub fn ensure_copilot_proxy_up() {
    if !proxy_routing_enabled() {
        return;
    }
    let Ok(_lock) = acquire_proxy_lifecycle_lock() else {
        return;
    };
    if !copilot_vscode_proxy_wired() {
        return;
    }
    with_quiet(|| {
        let _ = start_copilot_proxy();
    });
}

pub fn start_copilot_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("copilot proxy start: {}", e).into() })?;

    if copilot_proxy_live_z(PROXY_PROBE_TIMEOUT) {
        if copilot_proxy_ownership_valid() {
            return Ok(());
        }
        return Err(format!(
            "port {} held by an unverified process — refusing to attach Copilot proxy",
            copilot_proxy_port()
        )
        .into());
    }

    let bin = resolve_headroom_bin();
    if bin.is_empty() {
        return Err("headroom binary not found — install headroom first".into());
    }

    let args = copilot_proxy_args();
    let (pid_file, log_file) = copilot_proxy_files();
    if let Some(parent) = Path::new(&log_file).parent() {
        ensure_dir(parent)?;
    }

    let log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_file)?;
    let log_err = log.try_clone()?;

    let mut cmd = Command::new(&bin);
    cmd.args(&args);
    cmd.stdout(Stdio::from(log)).stderr(Stdio::from(log_err));

    let child = spawn_detached(&mut cmd)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("copilot proxy failed to start: {}", e).into() })?;
    let pid = child.id();

    let identity = match verify_identity_with_retry(pid, &bin, &args) {
        Ok(id) => id,
        Err((partial, err)) => {
            let msg: Box<dyn std::error::Error + Send + Sync> = format!(
                "copilot proxy startup identity unavailable; refusing to signal pid {}: {}",
                pid, err
            )
            .into();
            if partial.executable.is_empty() {
                return Err(cleanup_unverified_child(pid, msg));
            }
            return Err(msg);
        }
    };

    if let Err(err) = write_proxy_ownership(
        &pid_file,
        &ProxyOwnership {
            pid,
            executable: identity.executable.clone(),
            args: identity.args.clone(),
            start_fingerprint: identity.start.clone(),
            allowed_base_urls: String::new(),
        },
    ) {
        return Err(rollback_copilot_proxy(
            pid,
            &pid_file,
            Some(&identity),
            format!("copilot proxy ownership record: {}", err).into(),
        ));
    }

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    while Instant::now() < deadline {
        match identify_process(pid) {
            Ok(current) if current.equal(&identity) => {}
            _ => {
                return Err(rollback_copilot_proxy(
                    pid,
                    &pid_file,
                    Some(&identity),
                    "copilot proxy identity changed during startup".into(),
                ));
            }
        }
        if copilot_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(rollback_copilot_proxy(
        pid,
        &pid_file,
        Some(&identity),
        format!(
            "copilot proxy did not become ready within {:?} — see {}",
            PROXY_READY_TIMEOUT, log_file
        )
        .into(),
    ))
}

pub fn rollback_copilot_proxy(
    pid: u32,
    pid_file: &str,
    expected: Option<&ProcessIdentityInfo>,
    cause: Box<dyn std::error::Error + Send + Sync>,
) -> Box<dyn std::error::Error + Send + Sync> {
    let mut owned = false;
    let record_raw = read_file_safe(pid_file);
    if let Some(ref raw) = record_raw {
        if let Ok(record) = serde_json::from_str::<ProxyOwnership>(raw) {
            if record.pid == pid {
                if let Ok(identity) = identify_process(pid) {
                    if identity.matches_record(&record) {
                        owned = true;
                    }
                }
            }
        }
    }

    let expected_empty = expected.map(|e| e.executable.is_empty()).unwrap_or(true);
    if owned || expected_empty {
        if let Some(exp) = expected {
            if !exp.executable.is_empty() {
                match identify_process(pid) {
                    Ok(current) if current.equal(exp) => {}
                    _ => return cause,
                }
            }
        }
        if let Err(err) = kill_process(pid) {
            return format!("{}; rollback kill for pid {}: {}", cause, pid, err).into();
        }
    }

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(pid) && !copilot_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            if let Some(ref expected_raw) = record_raw {
                match read_file_safe(pid_file) {
                    Some(ref current) if current == expected_raw => {
                        if let Err(err) = std::fs::remove_file(pid_file) {
                            if err.kind() != std::io::ErrorKind::NotFound {
                                return format!("{}; rollback record removal: {}", cause, err)
                                    .into();
                            }
                        }
                    }
                    _ => {
                        return format!(
                            "{}; copilot proxy ownership record changed during rollback — refusing to remove replacement record",
                            cause
                        )
                        .into();
                    }
                }
            }
            return cause;
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    format!("{}; copilot proxy rollback did not stop pid {}", cause, pid).into()
}

pub fn copilot_proxy_ownership_valid() -> bool {
    let (pid_file, _) = copilot_proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        return false;
    };
    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return false;
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.args.is_empty()
        || record.start_fingerprint.is_empty()
    {
        return false;
    }
    identify_process(record.pid)
        .map(|id| id.matches_record(&record))
        .unwrap_or(false)
}

pub fn copilot_proxy_running() -> bool {
    copilot_proxy_live_z(PROXY_PROBE_TIMEOUT)
}

pub fn copilot_proxy_owned() -> bool {
    copilot_proxy_running() && copilot_proxy_ownership_valid()
}

pub fn stop_copilot_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("copilot proxy stop: {}", e).into() })?;

    let (pid_file, _) = copilot_proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        if copilot_proxy_running() {
            return Err(
                "Copilot proxy listener is running without toksi ownership — refusing to stop"
                    .into(),
            );
        }
        return Ok(());
    };

    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return Err(format!(
            "invalid Copilot proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.args.is_empty()
        || record.start_fingerprint.is_empty()
    {
        return Err(format!(
            "invalid Copilot proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    }

    let valid_identity = identify_process(record.pid)
        .map(|id| id.matches_record(&record))
        .unwrap_or(false);
    if !valid_identity {
        if process_gone(record.pid) {
            if copilot_proxy_live_z(PROXY_PROBE_TIMEOUT) {
                return Err(format!(
                    "Copilot proxy pid {} is gone but a healthy listener remains — refusing to remove ownership record",
                    record.pid
                )
                .into());
            }
            match read_file_safe(&pid_file) {
                Some(ref current) if current == &raw => {
                    std::fs::remove_file(&pid_file)?;
                    return Ok(());
                }
                _ => {
                    return Err(
                        "Copilot proxy ownership record changed during stale cleanup — refusing to remove replacement record"
                            .into(),
                    );
                }
            }
        }
        return Err(format!(
            "Copilot proxy pid {} identity could not be verified — refusing to stop",
            record.pid
        )
        .into());
    }

    kill_process(record.pid)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("failed to stop Copilot proxy pid {}: {}", record.pid, e).into() })?;

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(record.pid) && !copilot_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            match read_file_safe(&pid_file) {
                Some(ref current) if current == &raw => {
                    std::fs::remove_file(&pid_file)?;
                    return Ok(());
                }
                _ => {
                    return Err(
                        "Copilot proxy ownership record changed during stop — refusing to remove replacement record"
                            .into(),
                    );
                }
            }
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(format!("Copilot proxy did not stop (pid {})", record.pid).into())
}

pub fn copilot_cli_proxy_running() -> bool {
    copilot_cli_proxy_live_z(PROXY_PROBE_TIMEOUT)
}

pub fn copilot_cli_proxy_ownership_valid() -> bool {
    let (pid_file, _) = copilot_cli_proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        return false;
    };
    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return false;
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.args.is_empty()
        || record.start_fingerprint.is_empty()
    {
        return false;
    }
    identify_process(record.pid)
        .map(|id| id.matches_record(&record))
        .unwrap_or(false)
}

pub fn copilot_cli_proxy_owned() -> bool {
    copilot_cli_proxy_running() && copilot_cli_proxy_ownership_valid()
}

pub fn start_copilot_cli_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("copilot cli proxy start: {}", e).into() })?;

    if copilot_cli_proxy_live_z(PROXY_PROBE_TIMEOUT) {
        if copilot_cli_proxy_ownership_valid() {
            return Ok(());
        }
        return Err(format!(
            "port {} held by an unverified process — refusing to attach Copilot CLI proxy",
            copilot_cli_proxy_port()
        )
        .into());
    }

    let bin = resolve_headroom_bin();
    if bin.is_empty() {
        return Err("headroom binary not found — install headroom first".into());
    }

    let args = copilot_cli_proxy_args();
    let (pid_file, log_file) = copilot_cli_proxy_files();
    if let Some(parent) = Path::new(&log_file).parent() {
        ensure_dir(parent)?;
    }

    let log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_file)?;
    let log_err = log.try_clone()?;

    let mut cmd = Command::new(&bin);
    cmd.args(&args);
    cmd.stdout(Stdio::from(log)).stderr(Stdio::from(log_err));

    let child = spawn_detached(&mut cmd)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("copilot cli proxy failed to start: {}", e).into() })?;
    let pid = child.id();

    let identity = match verify_identity_with_retry(pid, &bin, &args) {
        Ok(id) => id,
        Err((partial, err)) => {
            let msg: Box<dyn std::error::Error + Send + Sync> = format!(
                "copilot cli proxy startup identity unavailable; refusing to signal pid {}: {}",
                pid, err
            )
            .into();
            if partial.executable.is_empty() {
                return Err(cleanup_unverified_child(pid, msg));
            }
            return Err(msg);
        }
    };

    if let Err(err) = write_proxy_ownership(
        &pid_file,
        &ProxyOwnership {
            pid,
            executable: identity.executable.clone(),
            args: identity.args.clone(),
            start_fingerprint: identity.start.clone(),
            allowed_base_urls: String::new(),
        },
    ) {
        return Err(rollback_copilot_proxy(
            pid,
            &pid_file,
            Some(&identity),
            format!("copilot cli proxy ownership record: {}", err).into(),
        ));
    }

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    while Instant::now() < deadline {
        match identify_process(pid) {
            Ok(current) if current.equal(&identity) => {}
            _ => {
                return Err(rollback_copilot_proxy(
                    pid,
                    &pid_file,
                    Some(&identity),
                    "copilot cli proxy identity changed during startup".into(),
                ));
            }
        }
        if copilot_cli_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(rollback_copilot_proxy(
        pid,
        &pid_file,
        Some(&identity),
        format!(
            "copilot cli proxy did not become ready within {:?} — see {}",
            PROXY_READY_TIMEOUT, log_file
        )
        .into(),
    ))
}

pub fn stop_copilot_cli_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("copilot cli proxy stop: {}", e).into() })?;

    let (pid_file, _) = copilot_cli_proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        if copilot_cli_proxy_running() {
            return Err(
                "Copilot CLI proxy listener is running without toksi ownership — refusing to stop"
                    .into(),
            );
        }
        return Ok(());
    };

    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return Err(format!(
            "invalid Copilot CLI proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.args.is_empty()
        || record.start_fingerprint.is_empty()
    {
        return Err(format!(
            "invalid Copilot CLI proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    }

    let valid_identity = identify_process(record.pid)
        .map(|id| id.matches_record(&record))
        .unwrap_or(false);
    if !valid_identity {
        if process_gone(record.pid) {
            if copilot_cli_proxy_live_z(PROXY_PROBE_TIMEOUT) {
                return Err(format!(
                    "Copilot CLI proxy pid {} is gone but a healthy listener remains — refusing to remove ownership record",
                    record.pid
                )
                .into());
            }
            match read_file_safe(&pid_file) {
                Some(ref current) if current == &raw => {
                    std::fs::remove_file(&pid_file)?;
                    return Ok(());
                }
                _ => {
                    return Err(
                        "Copilot CLI proxy ownership record changed during stale cleanup — refusing to remove replacement record"
                            .into(),
                    );
                }
            }
        }
        return Err(format!(
            "Copilot CLI proxy pid {} identity could not be verified — refusing to stop",
            record.pid
        )
        .into());
    }

    kill_process(record.pid)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("failed to stop Copilot CLI proxy pid {}: {}", record.pid, e).into() })?;

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(record.pid) && !copilot_cli_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            match read_file_safe(&pid_file) {
                Some(ref current) if current == &raw => {
                    std::fs::remove_file(&pid_file)?;
                    return Ok(());
                }
                _ => {
                    return Err(
                        "Copilot CLI proxy ownership record changed during stop — refusing to remove replacement record"
                            .into(),
                    );
                }
            }
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(format!("Copilot CLI proxy did not stop (pid {})", record.pid).into())
}

/// Native in-process async proxy listener for Copilot endpoints (VS Code or CLI)
pub async fn run_native_copilot_proxy(
    port: u16,
    is_cli: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::TcpListener;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    let s_name = if is_cli {
        "copilot-cli-proxy"
    } else {
        "copilot-vscode-proxy"
    };

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let service = s_name.to_string();
                tokio::spawn(async move {
                    let _ = serve_copilot_connection(&mut stream, port, &service).await;
                });
            }
            Err(_) => break,
        }
    }
    Ok(())
}

async fn serve_copilot_connection(
    stream: &mut tokio::net::TcpStream,
    port: u16,
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let Some(req) = crate::headroom::byok_gateway::read_http_request(stream).await? else {
            return Ok(());
        };

        let is_close_requested = req.headers.iter().any(|(k, v)| {
            k.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close")
        });

        let (path, _, _) = crate::headroom::byok_gateway::split_uri_path_and_query(&req.raw_uri);

        if path == "/livez" || path == "/healthz" || path == "/readyz" {
            let body = format!(
                r#"{{"service":"headroom-proxy","subservice":"{}","port":{},"status":"ok"}}"#,
                service_name, port
            );
            let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
            if is_close_requested {
                headers.push(("Connection".to_string(), "close".to_string()));
            }
            crate::headroom::byok_gateway::write_http_response(
                stream,
                200,
                "OK",
                &headers,
                body.as_bytes(),
            )
            .await?;
            if is_close_requested {
                use tokio::io::AsyncWriteExt;
                let _ = stream.shutdown().await;
                break;
            }
            continue;
        }

        // Real OAuth/SSE upstream streaming via Headroom live_zone + auto-healing pool
        let upstream_base = if service_name.contains("cli") {
            std::env::var("TOKSI_COPILOT_CLI_UPSTREAM_URL")
                .unwrap_or_else(|_| "https://api.githubcopilot.com".to_string())
        } else {
            std::env::var("TOKSI_COPILOT_UPSTREAM_URL")
                .unwrap_or_else(|_| "https://api.individual.githubcopilot.com".to_string())
        };
        let mut compressed_body = if req.method.eq_ignore_ascii_case("POST")
            && !req.body.is_empty()
            && !crate::headroom::byok_gateway::is_proto_or_connect_request(&path, &req.headers)
        {
            crate::headroom::live_zone::compress_request_body_safe(&path, &req.body)
        } else {
            req.body.clone()
        };
        let (_, _, query) = crate::headroom::byok_gateway::split_uri_path_and_query(&req.raw_uri);
        if let Err(e) = crate::headroom::byok_gateway::forward_streaming_to_byok_target(
            &upstream_base,
            &req.method,
            &path,
            Some(&query),
            &req.headers,
            &compressed_body,
            stream,
        )
        .await
        {
            let msg = format!(r#"{{"service":"{}","error":"copilot upstream failed: {}"}}"#, service_name, e);
            crate::headroom::byok_gateway::write_http_response(
                stream,
                502,
                "Bad Gateway",
                &[("Content-Type".to_string(), "application/json".to_string())],
                msg.as_bytes(),
            )
            .await?;
            compressed_body.clear();
        }

        if is_close_requested {
            use tokio::io::AsyncWriteExt;
            let _ = stream.shutdown().await;
            break;
        }
    }
    Ok(())
}
