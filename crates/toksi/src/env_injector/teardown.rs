// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/env_injector/teardown.rs
// created: 2026-10-31
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, Once};

use serde::{Deserialize, Serialize};

use crate::util::headroom::headroom_paths_resolved;
use crate::util::logger::L;
use crate::util::paths::{normalize_path, read_file_safe, write_file_atomic};
use crate::util::ports::PortsConfig;

use super::{
    EnvInjectorError, MANAGED_PROXY_KEYS, TOKSI_OWNED_MARKER, overlay_capture_originals,
    overlay_restore_or_clear,
};

static PANIC_HOOK_REGISTERED: Once = Once::new();
static SIGNAL_TASK_SPAWNED: AtomicBool = AtomicBool::new(false);
static ACTIVE_SNAPSHOT: LazyLock<Mutex<Option<EnvSnapshot>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvSnapshot {
    pub proxy_url: String,
    pub managed_keys: Vec<String>,
    pub overlay_originals: Vec<(String, Option<String>)>,
    pub windows_reg_originals: Vec<super::windows::WindowsRegEntrySnapshot>,
    pub unix_snapshot: super::unix::UnixEnvSnapshot,
}

fn snapshot_file_path() -> PathBuf {
    let raw = Path::new(&headroom_paths_resolved().root).join("env_snapshot.json");
    PathBuf::from(normalize_path(&raw.to_string_lossy()))
}

fn load_persisted_snapshot(path: &Path) -> Option<EnvSnapshot> {
    let raw = read_file_safe(path)?;
    serde_json::from_str::<EnvSnapshot>(&raw).ok()
}

fn save_persisted_snapshot(path: &Path, snapshot: &EnvSnapshot) {
    if let Ok(json) = serde_json::to_string(snapshot) {
        let _ = write_file_atomic(path, &json);
    }
}

fn remove_persisted_snapshot(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn already_owned_by_toksi() -> bool {
    #[cfg(windows)]
    {
        if super::windows::get_env_reg_value(TOKSI_OWNED_MARKER).as_deref() == Some("1") {
            return true;
        }
    }
    super::effective_env_var(TOKSI_OWNED_MARKER).as_deref() == Some("1")
}

/// Resolves the active Headroom proxy URL honoring `~/.toksi/config.toml` `[ports]`.
pub fn resolved_proxy_env_url() -> String {
    crate::util::headroom_proxy::headroom_proxy_url()
}

/// Computes the Headroom proxy URL from an explicit `PortsConfig` override.
pub fn proxy_url_from_ports(ports: &PortsConfig) -> String {
    format!("http://127.0.0.1:{}", ports.headroom_proxy)
}

/// Captures a snapshot of pre-injection environment values across OS stores and overlay.
pub fn capture_current_snapshot(proxy_url: &str, keys: &[&str]) -> EnvSnapshot {
    #[cfg(not(windows))]
    let unix_snapshot = {
        let conf_path = super::unix::get_environment_d_conf_path();
        super::unix::capture_unix_env_snapshot(keys, conf_path.as_deref(), cfg!(unix))
    };
    #[cfg(windows)]
    let unix_snapshot = super::unix::UnixEnvSnapshot::default();

    EnvSnapshot {
        proxy_url: proxy_url.to_string(),
        managed_keys: keys.iter().map(|&k| k.to_string()).collect(),
        overlay_originals: overlay_capture_originals(keys),
        windows_reg_originals: super::windows::capture_windows_reg_snapshot(keys),
        unix_snapshot,
    }
}

fn get_or_capture_snapshot(proxy_url: &str, keys: &[&str]) -> EnvSnapshot {
    if let Ok(guard) = ACTIVE_SNAPSHOT.lock()
        && let Some(ref existing) = *guard
    {
        return existing.clone();
    }

    let snap_path = snapshot_file_path();
    if already_owned_by_toksi()
        && let Some(disk_snap) = load_persisted_snapshot(&snap_path)
    {
        if let Ok(mut guard) = ACTIVE_SNAPSHOT.lock() {
            *guard = Some(disk_snap.clone());
        }
        return disk_snap;
    }

    let captured = capture_current_snapshot(proxy_url, keys);
    if let Ok(mut guard) = ACTIVE_SNAPSHOT.lock() {
        *guard = Some(captured.clone());
    }
    save_persisted_snapshot(&snap_path, &captured);
    captured
}

/// Registers safe panic-hook and Tokio signal teardown handlers (idempotent).
pub fn register_teardown_handlers() {
    PANIC_HOOK_REGISTERED.call_once(|| {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = teardown_proxy_env();
            prev_hook(info);
        }));
    });

    if let Ok(handle) = tokio::runtime::Handle::try_current()
        && !SIGNAL_TASK_SPAWNED.swap(true, Ordering::SeqCst)
    {
        handle.spawn(async move {
            let sig = wait_for_shutdown_signal().await;
            L.info(&format!("Signal {} captured; tearing down proxy environment", sig));
            let _ = teardown_proxy_env();
            SIGNAL_TASK_SPAWNED.store(false, Ordering::SeqCst);
        });
    }
}

/// Awaits termination signals (`Ctrl+C`, `SIGINT`, `SIGTERM`, `SIGHUP`, Windows console events)
/// using safe `tokio::signal` primitives.
pub async fn wait_for_shutdown_signal() -> &'static str {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigint = signal(SignalKind::interrupt()).ok();
        let mut sigterm = signal(SignalKind::terminate()).ok();
        let mut sighup = signal(SignalKind::hangup()).ok();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => "SIGINT",
            _ = async {
                if let Some(ref mut s) = sigint {
                    let _ = s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => "SIGINT",
            _ = async {
                if let Some(ref mut s) = sigterm {
                    let _ = s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => "SIGTERM",
            _ = async {
                if let Some(ref mut s) = sighup {
                    let _ = s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => "SIGHUP",
        }
    }

    #[cfg(windows)]
    {
        let mut ctrl_break = tokio::signal::windows::ctrl_break().ok();
        let mut ctrl_close = tokio::signal::windows::ctrl_close().ok();
        let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown().ok();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => "CTRL_C",
            _ = async {
                if let Some(ref mut s) = ctrl_break {
                    let _ = s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => "CTRL_BREAK",
            _ = async {
                if let Some(ref mut s) = ctrl_close {
                    let _ = s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => "CTRL_CLOSE",
            _ = async {
                if let Some(ref mut s) = ctrl_shutdown {
                    let _ = s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => "CTRL_SHUTDOWN",
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = tokio::signal::ctrl_c().await;
        "CTRL_C"
    }
}

/// Injects proxy environment variables for an explicit proxy URL after snapshotting originals.
pub fn inject_proxy_env_for_url(url: &str) -> Result<Vec<String>, EnvInjectorError> {
    let _snapshot = get_or_capture_snapshot(url, MANAGED_PROXY_KEYS);
    register_teardown_handlers();

    #[cfg(windows)]
    {
        super::windows::inject_windows_env(url)
    }

    #[cfg(not(windows))]
    {
        super::unix::inject_unix_env(url)
    }
}

/// Injects proxy environment variables using `~/.toksi/config.toml` `[ports]` overrides.
pub fn inject_proxy_env_for_ports(ports: &PortsConfig) -> Result<Vec<String>, EnvInjectorError> {
    let url = proxy_url_from_ports(ports);
    inject_proxy_env_for_url(&url)
}

/// Injects proxy environment variables using resolved runtime/config port settings.
pub fn inject_proxy_env() -> Result<Vec<String>, EnvInjectorError> {
    let url = resolved_proxy_env_url();
    inject_proxy_env_for_url(&url)
}

/// Convenience helper returning both the injected key list and an active RAII `TeardownGuard`.
pub fn inject_proxy_env_and_guard() -> Result<(Vec<String>, TeardownGuard), EnvInjectorError> {
    let keys = inject_proxy_env()?;
    let snap = ACTIVE_SNAPSHOT
        .lock()
        .ok()
        .and_then(|g| g.clone());
    let guard = match snap {
        Some(s) => TeardownGuard::with_snapshot(s),
        None => TeardownGuard::new(),
    };
    Ok((keys, guard))
}

fn teardown_with_snapshot(snapshot: Option<EnvSnapshot>) -> Result<Vec<String>, EnvInjectorError> {
    let snap_path = snapshot_file_path();
    let effective_snap = snapshot
        .or_else(|| {
            ACTIVE_SNAPSHOT
                .lock()
                .ok()
                .and_then(|mut g| g.take())
        })
        .or_else(|| load_persisted_snapshot(&snap_path));

    if let Ok(mut guard) = ACTIVE_SNAPSHOT.lock() {
        *guard = None;
    }

    let res = match effective_snap {
        Some(snap) => {
            #[cfg(windows)]
            let res = super::windows::restore_windows_reg_snapshot(&snap.windows_reg_originals);

            #[cfg(not(windows))]
            let res = {
                let conf_path = super::unix::get_environment_d_conf_path();
                let legacy_path = super::unix::get_legacy_environment_d_conf_path();
                super::unix::restore_unix_env_snapshot(
                    &snap.unix_snapshot,
                    conf_path.as_deref(),
                    legacy_path.as_deref(),
                    cfg!(unix),
                )
            };

            let key_refs: Vec<&str> = snap.managed_keys.iter().map(String::as_str).collect();
            overlay_restore_or_clear(&snap.overlay_originals, &key_refs);
            res
        }
        None => {
            #[cfg(windows)]
            let res = super::windows::teardown_windows_env();

            #[cfg(not(windows))]
            let res = super::unix::teardown_unix_env();

            overlay_restore_or_clear(&[], MANAGED_PROXY_KEYS);
            res
        }
    };

    remove_persisted_snapshot(&snap_path);
    res
}

/// Restores pre-injection environment snapshots (or deletes Toksi-injected keys) cleanly.
pub fn teardown_proxy_env() -> Result<Vec<String>, EnvInjectorError> {
    teardown_with_snapshot(None)
}

/// RAII guard guaranteeing non-destructive environment restoration on normal exit,
/// early returns, signal handling, or panics.
pub struct TeardownGuard {
    active: bool,
    snapshot: Option<EnvSnapshot>,
}

impl TeardownGuard {
    pub fn new() -> Self {
        let url = resolved_proxy_env_url();
        let snapshot = get_or_capture_snapshot(&url, MANAGED_PROXY_KEYS);
        Self {
            active: true,
            snapshot: Some(snapshot),
        }
    }

    pub fn with_snapshot(snapshot: EnvSnapshot) -> Self {
        Self {
            active: true,
            snapshot: Some(snapshot),
        }
    }

    pub fn snapshot(&self) -> Option<&EnvSnapshot> {
        self.snapshot.as_ref()
    }

    pub fn disarm(&mut self) {
        self.active = false;
        self.snapshot = None;
    }
}

impl Default for TeardownGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TeardownGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = teardown_with_snapshot(self.snapshot.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teardown_guard_lifecycle() {
        let snap = EnvSnapshot {
            proxy_url: "http://127.0.0.1:8787".to_string(),
            managed_keys: vec!["GOOGLE_GEMINI_BASE_URL".to_string()],
            ..Default::default()
        };
        let mut guard = TeardownGuard::with_snapshot(snap);
        assert!(guard.active);
        assert!(guard.snapshot().is_some());
        guard.disarm();
        assert!(!guard.active);
        assert!(guard.snapshot().is_none());
    }

    #[test]
    fn test_teardown_guard_drop_restores_overlay_snapshot() {
        let Ok(_lock) = crate::env_injector::TEST_ENV_LOCK.lock() else {
            return;
        };
        super::super::overlay_restore_or_clear(&[], MANAGED_PROXY_KEYS);

        let snap = EnvSnapshot {
            proxy_url: "http://127.0.0.1:19876".to_string(),
            managed_keys: vec!["GOOGLE_GEMINI_BASE_URL".to_string()],
            overlay_originals: vec![(
                "GOOGLE_GEMINI_BASE_URL".to_string(),
                Some("https://existing.example.com".to_string()),
            )],
            ..Default::default()
        };

        {
            let _guard = TeardownGuard::with_snapshot(snap);
            super::super::overlay_apply_vars(
                &[("GOOGLE_GEMINI_BASE_URL", "http://127.0.0.1:19876")],
                TOKSI_OWNED_MARKER,
                "GOOGLE_GEMINI_BASE_URL",
            );
            assert_eq!(
                super::super::effective_env_var("GOOGLE_GEMINI_BASE_URL").as_deref(),
                Some("http://127.0.0.1:19876")
            );
        }

        assert_eq!(
            super::super::effective_env_var("GOOGLE_GEMINI_BASE_URL").as_deref(),
            Some("https://existing.example.com")
        );
        super::super::overlay_restore_or_clear(&[], MANAGED_PROXY_KEYS);
    }

    #[test]
    fn test_proxy_url_from_ports_config_override() {
        let mut ports = PortsConfig::default();
        assert_eq!(proxy_url_from_ports(&ports), "http://127.0.0.1:8787");
        ports.headroom_proxy = 19787;
        assert_eq!(proxy_url_from_ports(&ports), "http://127.0.0.1:19787");
    }

    #[test]
    fn test_register_handlers_idempotent() {
        register_teardown_handlers();
        register_teardown_handlers();
    }

    #[test]
    fn test_inject_and_teardown_proxy_env() {
        let Ok(_lock) = crate::env_injector::TEST_ENV_LOCK.lock() else {
            return;
        };
        match inject_proxy_env() {
            Ok(injected) => {
                assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
                assert!(!injected.contains(&"CLOUD_CODE_URL".to_string()));
                let tear_res = teardown_proxy_env();
                assert!(tear_res.is_ok());
            }
            Err(e) => {
                assert!(e.is_access_denied(), "unexpected error: {e}");
                let _ = teardown_proxy_env();
            }
        }
    }
}
