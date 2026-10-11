use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use serde::Deserialize;

use super::process_identity::{
    identify_process, kill_process, process_gone, spawn_detached, ProcessIdentityInfo,
};
use super::proxy::{
    acquire_proxy_start_lock, cleanup_unverified_child, resolve_headroom_bin,
    verify_identity_with_retry, write_proxy_ownership, ProxyOwnership, PROXY_POLL_INTERVAL,
    PROXY_PROBE_TIMEOUT, PROXY_READY_TIMEOUT, PROXY_STOP_TIMEOUT,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::paths::{ensure_dir, read_file_safe};

pub const DEFAULT_GROK_OAUTH_PROXY_PORT: u16 = 8788;

pub fn grok_oauth_proxy_port() -> u16 {
    for var in ["TOKLESS_GROK_PROXY_PORT", "TOKLESS_HEADROOM_GROK_PORT"] {
        if let Ok(raw) = std::env::var(var) {
            if let Ok(p) = raw.trim().parse::<u16>() {
                if p > 0 {
                    return p;
                }
            }
        }
    }
    DEFAULT_GROK_OAUTH_PROXY_PORT
}

pub fn grok_proxy_files() -> (String, String) {
    let root = headroom_paths_resolved().root;
    let pid_file = Path::new(&root)
        .join("grok-proxy.pid")
        .to_string_lossy()
        .to_string();
    let log_file = Path::new(&root)
        .join("grok-proxy.log")
        .to_string_lossy()
        .to_string();
    (pid_file, log_file)
}

pub fn grok_proxy_args(port: u16) -> Vec<String> {
    vec![
        "proxy".to_string(),
        "--port".to_string(),
        port.to_string(),
        "--no-cache".to_string(),
        "--protect-tool-results".to_string(),
        "Bash".to_string(),
        "--anthropic-api-url".to_string(),
        "https://api.anthropic.com".to_string(),
        "--openai-api-url".to_string(),
        "https://cli-chat-proxy.grok.com".to_string(),
    ]
}

pub fn grok_proxy_live_z(timeout: Duration) -> bool {
    let url = format!("http://127.0.0.1:{}/livez", grok_oauth_proxy_port());
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
    else {
        return false;
    };
    let Ok(resp) = client.get(&url).send() else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    #[derive(Deserialize)]
    struct LiveResp {
        #[serde(default)]
        service: String,
    }
    resp.json::<LiveResp>()
        .map(|l| l.service == "headroom-proxy")
        .unwrap_or(false)
}

pub fn grok_oauth_proxy_running() -> bool {
    grok_proxy_live_z(PROXY_PROBE_TIMEOUT)
}

pub fn grok_oauth_proxy_owned() -> bool {
    grok_oauth_proxy_running() && grok_ownership_valid()
}

pub fn grok_ownership_valid() -> bool {
    let (pid_file, _) = grok_proxy_files();
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

pub fn grok_rollback(
    pid: u32,
    pid_file: &str,
    cause: Box<dyn std::error::Error + Send + Sync>,
) -> Box<dyn std::error::Error + Send + Sync> {
    grok_rollback_with_identity(pid, pid_file, None, cause)
}

pub fn grok_rollback_with_identity(
    pid: u32,
    pid_file: &str,
    expected: Option<&ProcessIdentityInfo>,
    cause: Box<dyn std::error::Error + Send + Sync>,
) -> Box<dyn std::error::Error + Send + Sync> {
    let record_raw = read_file_safe(pid_file);
    if let Some(exp) = expected {
        if !exp.executable.is_empty() {
            match identify_process(pid) {
                Ok(current) if current.equal(exp) => {}
                _ => return cause,
            }
        }
    }
    if let Err(err) = kill_process(pid) {
        return format!("{}; grok rollback kill for pid {}: {}", cause, pid, err).into();
    }

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(pid) && !grok_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            if let Some(ref expected_raw) = record_raw {
                match read_file_safe(pid_file) {
                    Some(ref current) if current == expected_raw => {
                        if let Err(err) = std::fs::remove_file(pid_file) {
                            if err.kind() != std::io::ErrorKind::NotFound {
                                return format!("{}; grok rollback record removal: {}", cause, err)
                                    .into();
                            }
                        }
                    }
                    _ => {
                        return format!(
                            "{}; grok proxy ownership record changed during rollback — refusing to remove replacement record",
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

    format!("{}; grok rollback did not stop pid {}", cause, pid).into()
}

pub fn start_grok_oauth_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("grok proxy start: {}", e).into() })?;

    if grok_oauth_proxy_running() {
        if !grok_ownership_valid() {
            return Err(format!(
                "port {} held by an unverified process — refusing to attach grok proxy",
                grok_oauth_proxy_port()
            )
            .into());
        }
        return Ok(());
    }

    let bin = resolve_headroom_bin();
    if bin.is_empty() {
        return Err("headroom binary not found — install headroom first".into());
    }

    let port = grok_oauth_proxy_port();
    let args = grok_proxy_args(port);
    let (pid_file, log_file) = grok_proxy_files();
    if let Some(parent) = Path::new(&log_file).parent() {
        ensure_dir(parent)?;
    }

    let log = File::create(&log_file)?;
    let log_err = log.try_clone()?;

    let mut cmd = Command::new(&bin);
    cmd.args(&args);
    cmd.stdout(Stdio::from(log)).stderr(Stdio::from(log_err));

    let child = spawn_detached(&mut cmd)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("grok proxy failed to start: {}", e).into() })?;
    let pid = child.id();

    let identity = match verify_identity_with_retry(pid, &bin, &args) {
        Ok(id) => id,
        Err((partial, err)) => {
            let msg: Box<dyn std::error::Error + Send + Sync> = format!(
                "grok proxy startup identity unavailable; refusing to signal pid {}: {}",
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
        return Err(grok_rollback_with_identity(
            pid,
            &pid_file,
            Some(&identity),
            format!("grok proxy ownership record: {}", err).into(),
        ));
    }

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    while Instant::now() < deadline {
        match identify_process(pid) {
            Ok(current) if current.equal(&identity) => {}
            _ => {
                return Err(grok_rollback_with_identity(
                    pid,
                    &pid_file,
                    Some(&identity),
                    "grok proxy identity changed during startup".into(),
                ));
            }
        }
        if grok_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(grok_rollback_with_identity(
        pid,
        &pid_file,
        Some(&identity),
        format!(
            "grok proxy did not become ready within {:?} — see {}",
            PROXY_READY_TIMEOUT, log_file
        )
        .into(),
    ))
}

/// Removes an orphaned ownership record only when the recorded process is gone
/// and no healthy listener remains.
pub fn clear_stale_grok_proxy_record() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("grok proxy stale cleanup: {}", e).into() })?;

    let (pid_file, _) = grok_proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        return Ok(());
    };
    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return Err(format!(
            "invalid grok proxy ownership record {} — refusing to clear",
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
            "invalid grok proxy ownership record {} — refusing to clear",
            pid_file
        )
        .into());
    }
    if identify_process(record.pid).is_ok() {
        return Err(format!(
            "grok proxy pid {} is still identifiable — refusing to clear ownership record",
            record.pid
        )
        .into());
    }
    if !process_gone(record.pid) {
        return Err(format!(
            "grok proxy pid {} is still alive — refusing to clear ownership record",
            record.pid
        )
        .into());
    }
    if grok_proxy_live_z(PROXY_PROBE_TIMEOUT) {
        return Err(format!(
            "grok proxy pid {} is gone but a healthy listener remains — refusing to clear ownership record",
            record.pid
        )
        .into());
    }
    match read_file_safe(&pid_file) {
        Some(ref current) if current == &raw => {
            if let Err(err) = std::fs::remove_file(&pid_file) {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(format!(
                        "remove stale grok proxy ownership record {}: {}",
                        pid_file, err
                    )
                    .into());
                }
            }
            Ok(())
        }
        _ => Err(
            "grok proxy ownership record changed during stale cleanup — refusing to remove replacement record"
                .into(),
        ),
    }
}

pub fn stop_grok_oauth_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("grok proxy stop: {}", e).into() })?;

    let (pid_file, _) = grok_proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        if grok_oauth_proxy_running() {
            return Err(
                "Grok OAuth proxy listener is running without tokless ownership — refusing to stop"
                    .into(),
            );
        }
        return Ok(());
    };
    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return Err(format!(
            "invalid grok proxy ownership record {} — refusing to stop",
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
            "invalid grok proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    }

    match identify_process(record.pid) {
        Ok(identity) => {
            if !identity.matches_record(&record) {
                return Err(format!(
                    "grok proxy pid {} identity could not be verified — refusing to stop",
                    record.pid
                )
                .into());
            }
        }
        Err(_) => {
            if process_gone(record.pid) {
                if grok_proxy_live_z(PROXY_PROBE_TIMEOUT) {
                    return Err(format!(
                        "grok proxy pid {} is gone but a healthy listener remains — refusing to remove ownership record",
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
                            "grok proxy ownership record changed during stale cleanup — refusing to remove replacement record"
                                .into(),
                        );
                    }
                }
            }
            return Err(format!(
                "grok proxy pid {} identity could not be verified — refusing to stop",
                record.pid
            )
            .into());
        }
    }

    kill_process(record.pid)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("failed to stop grok proxy pid {}: {}", record.pid, e).into() })?;

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(record.pid) && !grok_proxy_live_z(PROXY_PROBE_TIMEOUT) {
            match read_file_safe(&pid_file) {
                Some(ref current) if current == &raw => {
                    std::fs::remove_file(&pid_file)?;
                    return Ok(());
                }
                _ => {
                    return Err(
                        "grok proxy ownership record changed during stop — refusing to remove replacement record"
                            .into(),
                    );
                }
            }
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(format!("Grok proxy did not stop (pid {})", record.pid).into())
}
