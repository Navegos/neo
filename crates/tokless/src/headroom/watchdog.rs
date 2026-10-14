// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/watchdog.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use super::process_identity::kill_process;
use super::proxy::{
    acquire_proxy_start_lock, proxy_args_from_runtime, proxy_args_match_recorded, proxy_daemon_env,
    proxy_file_matches, proxy_files, proxy_owned_process_live, proxy_port_from_runtime,
    proxy_supervised_args_match, proxy_supervised_file, resolve_headroom_bin,
    verify_identity_with_retry, write_proxy_ownership, write_proxy_supervised_state,
    ProxyOwnership,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::proxy_routing_enabled;
use crate::util::paths::{ensure_dir, read_file_safe, write_file_atomic, IS_WIN};

pub fn proxy_stop_request_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("proxy.stop.request")
        .to_string_lossy()
        .to_string()
}

pub fn request_proxy_stop() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !IS_WIN {
        return Ok(());
    }
    ensure_dir(&headroom_paths_resolved().root)?;
    write_file_atomic(proxy_stop_request_file(), "")?;
    Ok(())
}

pub fn clear_proxy_stop_request() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !IS_WIN {
        return Ok(());
    }
    let path = proxy_stop_request_file();
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn proxy_stop_requested() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if !IS_WIN {
        return Ok(false);
    }
    match std::fs::metadata(proxy_stop_request_file()) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

pub fn run_proxy_watchdog() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !IS_WIN {
        return Err("proxy watchdog is only supported on Windows".into());
    }
    if !proxy_routing_enabled() {
        return Ok(());
    }
    let bin = resolve_headroom_bin();
    if bin.is_empty() {
        return Err("headroom binary not found".into());
    }
    if proxy_stop_requested()? {
        return Ok(());
    }
    let mut attempts = 0usize;
    loop {
        let port = proxy_port_from_runtime();
        let args = proxy_args_from_runtime(port);
        if let Err(err) = run_headroom_supervised(&bin, &args) {
            if proxy_stop_requested()? {
                return Ok(());
            }
            if attempts >= 4 {
                return Err(format!(
                    "headroom watchdog failed after {} attempts: {}",
                    attempts + 1,
                    err
                )
                .into());
            }
            attempts += 1;
            std::thread::sleep(Duration::from_secs(1));
            continue;
        }
        if proxy_stop_requested()? {
            return Ok(());
        }
        attempts += 1;
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn run_headroom_supervised(
    bin: &str,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lock = acquire_proxy_start_lock()?;
    if proxy_stop_requested()? {
        return Ok(());
    }
    let port = proxy_port_from_runtime();
    if proxy_owned_process_live()
        && (proxy_args_match_recorded(port) || proxy_supervised_args_match(port))
    {
        return Ok(());
    }

    let mut cmd = Command::new(bin);
    cmd.args(args);
    cmd.env_clear();
    for (k, v) in proxy_daemon_env() {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;
    let pid = child.id();

    let identity = match verify_identity_with_retry(pid, bin, args) {
        Ok(id) => id,
        Err((partial, err)) => {
            if !partial.executable.is_empty() {
                return Err(err);
            }
            let _ = cleanup_supervised_process(Some(&mut child), "");
            return Err(err);
        }
    };

    let (pid_file, _) = proxy_files();
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
        let _ = cleanup_supervised_process(Some(&mut child), "");
        return Err(err);
    }

    if let Err(err) = write_proxy_supervised_state(pid, bin, args, &[], &identity.start) {
        let _ = cleanup_supervised_process(Some(&mut child), &pid_file);
        return Err(err);
    }

    match proxy_stop_requested() {
        Ok(true) => {
            return cleanup_supervised_process(Some(&mut child), &pid_file);
        }
        Ok(false) => {}
        Err(err) => {
            let _ = cleanup_supervised_process(Some(&mut child), &pid_file);
            return Err(err);
        }
    }

    let pid_raw = read_file_safe(&pid_file);
    let supervised_path = proxy_supervised_file();
    let supervised_raw = read_file_safe(&supervised_path);

    drop(lock);

    let wait_res = child.wait();
    let cleanup_res = cleanup_supervised_process_after_wait(
        &pid_file,
        pid_raw.as_deref(),
        supervised_raw.as_deref(),
    );

    if let Err(w_err) = wait_res {
        return Err(w_err.into());
    }
    cleanup_res
}

pub fn cleanup_supervised_process(
    child: Option<&mut Child>,
    pid_file: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut first_err: Option<Box<dyn std::error::Error + Send + Sync>> = None;
    if let Some(c) = child {
        let pid = c.id();
        if let Err(e) = c.kill() {
            let _ = kill_process(pid);
            if first_err.is_none() {
                first_err = Some(e.into());
            }
        }
        if let Err(e) = c.wait() {
            if first_err.is_none() {
                first_err = Some(e.into());
            }
        }
    }
    if !pid_file.is_empty() {
        if let Err(e) = remove_supervised_records(pid_file) {
            if first_err.is_none() {
                first_err = Some(e);
            }
        }
    }
    if let Some(err) = first_err {
        return Err(err);
    }
    Ok(())
}

pub fn cleanup_supervised_process_after_wait(
    pid_file: &str,
    pid_raw: Option<&str>,
    supervised_raw: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()?;
    if !proxy_file_matches(pid_file, pid_raw)
        || !proxy_file_matches(&proxy_supervised_file(), supervised_raw)
    {
        return Err(
            "supervised proxy record changed during watchdog cleanup — refusing to remove replacement record"
                .into(),
        );
    }
    remove_supervised_records(pid_file)
}

pub fn remove_supervised_records(
    pid_file: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pid_exists = read_file_safe(pid_file).is_some();
    let supervised_path = proxy_supervised_file();
    let supervised_exists = read_file_safe(&supervised_path).is_some();
    let mut first_err: Option<Box<dyn std::error::Error + Send + Sync>> = None;
    if pid_exists {
        if let Err(e) = std::fs::remove_file(pid_file) {
            if e.kind() != std::io::ErrorKind::NotFound {
                first_err = Some(e.into());
            }
        }
    }
    if supervised_exists {
        if let Err(e) = std::fs::remove_file(&supervised_path) {
            if e.kind() != std::io::ErrorKind::NotFound && first_err.is_none() {
                first_err = Some(e.into());
            }
        }
    }
    if let Some(err) = first_err {
        return Err(err);
    }
    Ok(())
}
