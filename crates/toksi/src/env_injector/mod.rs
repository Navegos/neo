// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/env_injector/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;
use std::sync::{LazyLock, RwLock};

use crate::util::paths::normalize_path;

pub mod heal;
pub mod teardown;
pub mod unix;
pub mod windows;

pub use heal::{
    HealError, HealingReport, heal_legacy_hijacks, heal_legacy_hijacks_in,
    is_hijacked_symlink_or_junction, is_symlink_or_reparse_point, is_toksi_shim_or_sidecar,
    scan_legacy_hijacks, scan_legacy_hijacks_in,
};
pub use teardown::{
    EnvSnapshot, TeardownGuard, inject_proxy_env, inject_proxy_env_and_guard,
    inject_proxy_env_for_ports, inject_proxy_env_for_url, register_teardown_handlers,
    resolved_proxy_env_url, teardown_proxy_env, wait_for_shutdown_signal,
};

pub const TOKSI_OWNED_MARKER: &str = "TOKSI_OWNED";
pub const TOKSI_OWNED_KEYS: &str = "TOKSI_OWNED_KEYS";
pub const TOKLESS_OWNED_MARKER: &str = "TOKLESS_OWNED";
pub const TOKLESS_OWNED_KEYS: &str = "TOKLESS_OWNED_KEYS";
pub const CLOUD_CODE_URL_KEY: &str = "CLOUD_CODE_URL";

/// Default proxy environment variables managed non-destructively during daemon lifetime.
///
/// `CLOUD_CODE_URL` is intentionally excluded so Google auth/quota checks bypass proxy.
pub const MANAGED_PROXY_KEYS: &[&str] = &[
    "GOOGLE_GEMINI_BASE_URL",
    "ANTHROPIC_BASE_URL",
];

#[derive(Debug, thiserror::Error)]
pub enum EnvInjectorError {
    #[error("System environment FFI error: {0}")]
    Sys(#[from] toksi_sys::SysError),

    #[error("I/O error at '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Command '{command}' failed: {reason}")]
    CommandFailed { command: String, reason: String },

    #[error("Refusing to inject auth-bypass key '{0}'")]
    ForbiddenKey(String),

    #[error("Unsupported platform for {0}")]
    UnsupportedPlatform(&'static str),
}

impl EnvInjectorError {
    pub fn is_access_denied(&self) -> bool {
        matches!(self, Self::Sys(e) if e.is_access_denied())
    }
}

#[derive(Debug, Default)]
struct EnvOverlay {
    injected: BTreeMap<String, String>,
    removed: BTreeSet<String>,
}

static PROCESS_ENV_OVERLAY: LazyLock<RwLock<EnvOverlay>> =
    LazyLock::new(|| RwLock::new(EnvOverlay::default()));

/// Returns `true` if `key` must never be injected because it breaks direct auth/quota checks.
pub fn is_forbidden_auth_bypass_key(key: &str) -> bool {
    key.trim().eq_ignore_ascii_case(CLOUD_CODE_URL_KEY)
}

/// Normalizes path-like environment values to forward slashes (`/`), leaving URLs intact.
pub fn sanitize_env_value(key: &str, value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.contains("://") {
        return trimmed.to_string();
    }
    let upper = key.trim().to_ascii_uppercase();
    if upper.ends_with("_PATH")
        || upper.ends_with("_DIR")
        || upper.ends_with("_FILE")
        || upper.ends_with("_CONF")
        || upper.ends_with("_HOME")
    {
        normalize_path(trimmed)
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn overlay_capture_originals(keys: &[&str]) -> Vec<(String, Option<String>)> {
    let Ok(guard) = PROCESS_ENV_OVERLAY.read() else {
        return Vec::new();
    };
    keys.iter()
        .filter(|&&k| !is_forbidden_auth_bypass_key(k))
        .map(|&k| (k.to_string(), guard.injected.get(k).cloned()))
        .collect()
}

pub(crate) fn overlay_apply_vars(vars: &[(&str, &str)], marker: &str, keys_csv: &str) {
    if let Ok(mut guard) = PROCESS_ENV_OVERLAY.write() {
        for &(key, val) in vars {
            if is_forbidden_auth_bypass_key(key) {
                continue;
            }
            let sanitized = sanitize_env_value(key, val);
            guard.removed.remove(key);
            guard.injected.insert(key.to_string(), sanitized);
        }
        guard.injected.remove(CLOUD_CODE_URL_KEY);
        guard.removed.insert(CLOUD_CODE_URL_KEY.to_string());
        guard.removed.remove(marker);
        guard.injected.insert(marker.to_string(), "1".to_string());
        guard
            .injected
            .insert(TOKSI_OWNED_KEYS.to_string(), keys_csv.to_string());
        guard.removed.remove(TOKSI_OWNED_KEYS);
    }
}

pub(crate) fn overlay_restore_or_clear(
    originals: &[(String, Option<String>)],
    keys_to_clear: &[&str],
) {
    if let Ok(mut guard) = PROCESS_ENV_OVERLAY.write() {
        for &key in keys_to_clear {
            guard.injected.remove(key);
            guard.removed.insert(key.to_string());
        }
        for (key, orig) in originals {
            if is_forbidden_auth_bypass_key(key) {
                continue;
            }
            match orig {
                Some(val) => {
                    guard.removed.remove(key);
                    guard.injected.insert(key.clone(), val.clone());
                }
                None => {
                    guard.injected.remove(key);
                    guard.removed.insert(key.clone());
                }
            }
        }
        guard.injected.remove(CLOUD_CODE_URL_KEY);
        guard.removed.insert(CLOUD_CODE_URL_KEY.to_string());
        guard.injected.remove(TOKSI_OWNED_MARKER);
        guard.injected.remove(TOKSI_OWNED_KEYS);
        guard.injected.remove(TOKLESS_OWNED_MARKER);
        guard.injected.remove(TOKLESS_OWNED_KEYS);
        guard.removed.insert(TOKSI_OWNED_MARKER.to_string());
        guard.removed.insert(TOKSI_OWNED_KEYS.to_string());
        guard.removed.insert(TOKLESS_OWNED_MARKER.to_string());
        guard.removed.insert(TOKLESS_OWNED_KEYS.to_string());
    }
}

/// Applies non-destructive Toksi proxy environment overrides to a child `Command`
/// using safe `Command::env()` and `Command::env_remove()` without mutating process globals.
pub fn apply_env_to_command(cmd: &mut Command) {
    if let Ok(guard) = PROCESS_ENV_OVERLAY.read() {
        for key in &guard.removed {
            cmd.env_remove(key);
        }
        for (key, val) in &guard.injected {
            if !is_forbidden_auth_bypass_key(key) {
                cmd.env(key, val);
            }
        }
    }
    cmd.env_remove(CLOUD_CODE_URL_KEY);
}

/// Returns a snapshot of currently active in-memory Toksi environment overrides.
pub fn active_env_overrides() -> BTreeMap<String, String> {
    PROCESS_ENV_OVERLAY
        .read()
        .map(|g| g.injected.clone())
        .unwrap_or_default()
}

/// Reads an environment variable respecting active Toksi overlays first, then OS process env.
pub fn effective_env_var(key: &str) -> Option<String> {
    if let Ok(guard) = PROCESS_ENV_OVERLAY.read() {
        if guard.removed.contains(key) {
            return None;
        }
        if let Some(val) = guard.injected.get(key) {
            return Some(val.clone());
        }
    }
    std::env::var(key).ok()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EnvironmentStatus {
    pub binaries_clean: bool,
    pub hijacked_binaries: Vec<String>,
    pub shims_detected: Vec<String>,
    pub state_files_detected: Vec<String>,
    pub env_vars_healthy: bool,
    pub active_proxy_vars: Vec<String>,
    pub details: Vec<String>,
}

pub fn audit_environment_status() -> EnvironmentStatus {
    let scan = scan_legacy_hijacks();
    let binaries_clean = scan.is_clean();
    let hijacked_binaries: Vec<String> = scan
        .restored_binaries
        .iter()
        .map(|(s, _)| normalize_path(&s.to_string_lossy()))
        .collect();
    let shims_detected: Vec<String> = scan
        .removed_shims
        .iter()
        .map(|p| normalize_path(&p.to_string_lossy()))
        .collect();
    let state_files_detected: Vec<String> = scan
        .removed_state_files
        .iter()
        .map(|p| normalize_path(&p.to_string_lossy()))
        .collect();

    let mut active_proxy_vars = Vec::new();
    let mut details = Vec::new();
    let mut env_vars_healthy = true;

    #[cfg(windows)]
    {
        use windows::{get_env_reg_value, MANAGED_PROXY_KEYS, TOKSI_OWNED_MARKER, TOKLESS_OWNED_MARKER};
        if let Some(val) = get_env_reg_value(CLOUD_CODE_URL_KEY) {
            env_vars_healthy = false;
            details.push(format!(
                "CLOUD_CODE_URL detected in registry: {} (bypasses direct auth/quota)",
                val
            ));
        }
        let is_owned = get_env_reg_value(TOKSI_OWNED_MARKER).as_deref() == Some("1")
            || get_env_reg_value(TOKLESS_OWNED_MARKER).as_deref() == Some("1");
        if is_owned {
            active_proxy_vars.push(TOKSI_OWNED_MARKER.to_string());
            for &key in MANAGED_PROXY_KEYS {
                if let Some(val) = get_env_reg_value(key) {
                    active_proxy_vars.push(format!("{}={}", key, val));
                }
            }
            details.push("Windows HKCU\\Environment has active Toksi proxy configuration".to_string());
        } else {
            for &key in MANAGED_PROXY_KEYS {
                if let Some(val) = get_env_reg_value(key)
                    && (val.contains("127.0.0.1") || val.contains("localhost"))
                {
                    env_vars_healthy = false;
                    details.push(format!(
                        "Stale registry value without TOKSI_OWNED: {}={}",
                        key, val
                    ));
                }
            }
            if env_vars_healthy {
                details.push("Windows HKCU\\Environment clean (no proxy override active)".to_string());
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(val) = std::env::var(CLOUD_CODE_URL_KEY) {
            env_vars_healthy = false;
            details.push(format!(
                "CLOUD_CODE_URL detected in session: {} (bypasses direct auth/quota)",
                val
            ));
        }
        let env_d_entries = unix::get_environment_d_conf_path()
            .filter(|p| p.is_file())
            .map(|p| unix::read_environment_d_entries(&p))
            .unwrap_or_default();
        if let Some(val) = env_d_entries.get(CLOUD_CODE_URL_KEY) {
            env_vars_healthy = false;
            details.push(format!(
                "CLOUD_CODE_URL detected in environment.d: {} (bypasses direct auth/quota)",
                val
            ));
        }

        let overlay = active_env_overrides();
        for key in ["GOOGLE_GEMINI_BASE_URL", "ANTHROPIC_BASE_URL", TOKSI_OWNED_MARKER] {
            if let Some(val) = overlay
                .get(key)
                .cloned()
                .or_else(|| env_d_entries.get(key).cloned())
                .or_else(|| std::env::var(key).ok())
            {
                active_proxy_vars.push(format!("{}={}", key, val));
            }
        }
        if env_vars_healthy {
            if active_proxy_vars.is_empty() {
                details.push("Unix session environment clean".to_string());
            } else {
                details.push("Unix session environment has active Toksi proxy configuration".to_string());
            }
        }
    }

    EnvironmentStatus {
        binaries_clean,
        hijacked_binaries,
        shims_detected,
        state_files_detected,
        env_vars_healthy,
        active_proxy_vars,
        details,
    }
}

#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_env_to_command_forbids_cloud_code_url() {
        let Ok(_lock) = TEST_ENV_LOCK.lock() else {
            return;
        };
        overlay_apply_vars(
            &[
                ("GOOGLE_GEMINI_BASE_URL", "http://127.0.0.1:8787"),
                ("CLOUD_CODE_URL", "http://127.0.0.1:8787"),
                ("TOKSI_TEST_DIR", r"C:\Users\dev\.toksi"),
            ],
            TOKSI_OWNED_MARKER,
            "GOOGLE_GEMINI_BASE_URL,TOKSI_TEST_DIR",
        );

        let mut cmd = Command::new("cargo");
        apply_env_to_command(&mut cmd);

        let envs: BTreeMap<String, Option<String>> = cmd
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().to_string(),
                    v.map(|val| val.to_string_lossy().to_string()),
                )
            })
            .collect();

        assert_eq!(
            envs.get("GOOGLE_GEMINI_BASE_URL").and_then(|v| v.as_deref()),
            Some("http://127.0.0.1:8787")
        );
        assert_eq!(
            envs.get("TOKSI_TEST_DIR").and_then(|v| v.as_deref()),
            Some("C:/Users/dev/.toksi")
        );
        assert_eq!(envs.get(CLOUD_CODE_URL_KEY), Some(&None));

        overlay_restore_or_clear(&[], &["GOOGLE_GEMINI_BASE_URL", "TOKSI_TEST_DIR"]);
        assert!(active_env_overrides().is_empty());
    }
}
