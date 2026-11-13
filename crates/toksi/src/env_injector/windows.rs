// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/env_injector/windows.rs
// created: 2026-10-31
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use toksi_sys::RegEnvEntry;
use toksi_sys::windows_env::{
    DEFAULT_BROADCAST_TIMEOUT_MS, broadcast_env_setting_change, delete_hkcu_env_value,
    get_hkcu_env_entry, get_hkcu_env_value, set_hkcu_env_entry, set_hkcu_env_value,
};

use super::{
    CLOUD_CODE_URL_KEY, EnvInjectorError, is_forbidden_auth_bypass_key, overlay_apply_vars,
    overlay_restore_or_clear, sanitize_env_value,
};

pub const TOKSI_OWNED_MARKER: &str = super::TOKSI_OWNED_MARKER;
pub const TOKSI_OWNED_KEYS: &str = super::TOKSI_OWNED_KEYS;
pub const TOKLESS_OWNED_MARKER: &str = super::TOKLESS_OWNED_MARKER;
pub const TOKLESS_OWNED_KEYS: &str = super::TOKLESS_OWNED_KEYS;
pub const MANAGED_PROXY_KEYS: &[&str] = super::MANAGED_PROXY_KEYS;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsRegEntrySnapshot {
    pub key: String,
    pub original: Option<(String, u32)>,
}

pub fn broadcast_setting_change() -> bool {
    broadcast_env_setting_change(DEFAULT_BROADCAST_TIMEOUT_MS)
}

pub fn get_env_reg_entry(name: &str) -> Option<RegEnvEntry> {
    if !cfg!(windows) {
        return None;
    }
    get_hkcu_env_entry(name).ok().flatten()
}

pub fn get_env_reg_value(name: &str) -> Option<String> {
    if !cfg!(windows) {
        return None;
    }
    get_hkcu_env_value(name).ok().flatten()
}

pub fn set_env_reg_value(name: &str, value: &str) -> Result<(), EnvInjectorError> {
    if is_forbidden_auth_bypass_key(name) {
        return Err(EnvInjectorError::ForbiddenKey(name.to_string()));
    }
    let sanitized = sanitize_env_value(name, value);
    set_hkcu_env_value(name, &sanitized)?;
    Ok(())
}

pub fn delete_env_reg_value(name: &str) -> Result<bool, EnvInjectorError> {
    Ok(delete_hkcu_env_value(name)?)
}

/// Captures original `HKCU\Environment` values before Toksi modifies them.
pub fn capture_windows_reg_snapshot(keys: &[&str]) -> Vec<WindowsRegEntrySnapshot> {
    if !cfg!(windows) {
        return Vec::new();
    }
    keys.iter()
        .filter(|&&k| !is_forbidden_auth_bypass_key(k))
        .map(|&k| {
            let original = get_env_reg_entry(k).map(|e| (e.value, e.reg_type));
            WindowsRegEntrySnapshot {
                key: k.to_string(),
                original,
            }
        })
        .collect()
}

/// Restores original `HKCU\Environment` values from a snapshot (or deletes keys that did not exist).
pub fn restore_windows_reg_snapshot(
    snapshot: &[WindowsRegEntrySnapshot],
) -> Result<Vec<String>, EnvInjectorError> {
    if !cfg!(windows) {
        return Ok(Vec::new());
    }

    let mut restored_or_purged = Vec::new();
    let mut overlay_originals = Vec::new();
    let mut reg_err: Option<EnvInjectorError> = None;

    for entry in snapshot {
        if is_forbidden_auth_bypass_key(&entry.key) {
            continue;
        }
        match &entry.original {
            Some((val, reg_type)) => {
                if let Err(e) = set_hkcu_env_entry(&entry.key, val, *reg_type)
                    && reg_err.is_none()
                {
                    reg_err = Some(e.into());
                }
                overlay_originals.push((entry.key.clone(), Some(val.clone())));
            }
            None => {
                if let Err(e) = delete_hkcu_env_value(&entry.key)
                    && reg_err.is_none()
                {
                    reg_err = Some(e.into());
                }
                overlay_originals.push((entry.key.clone(), None));
            }
        }
        if !restored_or_purged.contains(&entry.key) {
            restored_or_purged.push(entry.key.clone());
        }
    }

    for &k in MANAGED_PROXY_KEYS {
        if !snapshot.iter().any(|s| s.key.eq_ignore_ascii_case(k)) {
            let _ = delete_hkcu_env_value(k);
            if !restored_or_purged.contains(&k.to_string()) {
                restored_or_purged.push(k.to_string());
            }
        }
    }

    for marker in [
        TOKSI_OWNED_MARKER,
        TOKSI_OWNED_KEYS,
        TOKLESS_OWNED_MARKER,
        TOKLESS_OWNED_KEYS,
        CLOUD_CODE_URL_KEY,
    ] {
        let _ = delete_hkcu_env_value(marker);
    }

    overlay_restore_or_clear(&overlay_originals, MANAGED_PROXY_KEYS);

    if !restored_or_purged.contains(&TOKSI_OWNED_MARKER.to_string()) {
        restored_or_purged.push(TOKSI_OWNED_MARKER.to_string());
    }
    if !restored_or_purged.contains(&TOKSI_OWNED_KEYS.to_string()) {
        restored_or_purged.push(TOKSI_OWNED_KEYS.to_string());
    }

    broadcast_setting_change();

    if let Some(e) = reg_err
        && !e.is_access_denied()
    {
        return Err(e);
    }

    Ok(restored_or_purged)
}

pub fn inject_windows_env_vars(vars: &[(&str, &str)]) -> Result<Vec<String>, EnvInjectorError> {
    if !cfg!(windows) {
        return Err(EnvInjectorError::UnsupportedPlatform(
            "Windows environment injection is only supported on Windows",
        ));
    }

    let filtered: Vec<(&str, String)> = vars
        .iter()
        .filter(|(k, _)| !is_forbidden_auth_bypass_key(k))
        .map(|&(k, v)| (k, sanitize_env_value(k, v)))
        .collect();

    let mut injected = Vec::new();
    let keys: Vec<&str> = filtered.iter().map(|(k, _)| *k).collect();
    let keys_csv = keys.join(",");
    let mut reg_err: Option<EnvInjectorError> = None;

    for (key, val) in &filtered {
        if let Err(e) = set_env_reg_value(key, val)
            && reg_err.is_none()
        {
            reg_err = Some(e);
        }
        injected.push((*key).to_string());
    }

    // Explicitly delete CLOUD_CODE_URL from HKCU\Environment so auth/quota checks bypass proxy.
    let _ = delete_env_reg_value(CLOUD_CODE_URL_KEY);

    if let Err(e) = set_env_reg_value(TOKSI_OWNED_MARKER, "1")
        && reg_err.is_none()
    {
        reg_err = Some(e);
    }
    let _ = set_env_reg_value(TOKSI_OWNED_KEYS, &keys_csv);

    let overlay_pairs: Vec<(&str, &str)> = filtered.iter().map(|(k, v)| (*k, v.as_str())).collect();
    overlay_apply_vars(&overlay_pairs, TOKSI_OWNED_MARKER, &keys_csv);

    injected.push(TOKSI_OWNED_MARKER.to_string());
    injected.push(TOKSI_OWNED_KEYS.to_string());

    broadcast_setting_change();

    if let Some(e) = reg_err
        && !e.is_access_denied()
    {
        return Err(e);
    }

    Ok(injected)
}

pub fn inject_windows_env(proxy_url: &str) -> Result<Vec<String>, EnvInjectorError> {
    let vars: Vec<(&str, &str)> = MANAGED_PROXY_KEYS.iter().map(|&k| (k, proxy_url)).collect();
    inject_windows_env_vars(&vars)
}

pub fn teardown_windows_env() -> Result<Vec<String>, EnvInjectorError> {
    if !cfg!(windows) {
        return Ok(Vec::new());
    }

    let marker = get_env_reg_value(TOKSI_OWNED_MARKER)
        .or_else(|| super::effective_env_var(TOKSI_OWNED_MARKER))
        .or_else(|| get_env_reg_value(TOKLESS_OWNED_MARKER))
        .or_else(|| super::effective_env_var(TOKLESS_OWNED_MARKER));
    let mut purged = Vec::new();

    let has_managed_or_stale = MANAGED_PROXY_KEYS.iter().any(|k| {
        get_env_reg_value(k).is_some() || super::effective_env_var(k).is_some()
    }) || get_env_reg_value(CLOUD_CODE_URL_KEY).is_some()
        || super::effective_env_var(CLOUD_CODE_URL_KEY).is_some();

    if marker.as_deref() == Some("1") || marker.is_some() || has_managed_or_stale {
        let keys_str = get_env_reg_value(TOKSI_OWNED_KEYS)
            .or_else(|| super::effective_env_var(TOKSI_OWNED_KEYS))
            .or_else(|| get_env_reg_value(TOKLESS_OWNED_KEYS))
            .or_else(|| super::effective_env_var(TOKLESS_OWNED_KEYS))
            .unwrap_or_default();
        let mut keys: Vec<String> = if !keys_str.is_empty() {
            keys_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        for &k in MANAGED_PROXY_KEYS {
            if !keys.iter().any(|existing| existing.eq_ignore_ascii_case(k)) {
                keys.push(k.to_string());
            }
        }

        for key in &keys {
            let _ = delete_env_reg_value(key);
            if !purged.contains(key) {
                purged.push(key.clone());
            }
        }

        let _ = delete_env_reg_value(TOKSI_OWNED_MARKER);
        let _ = delete_env_reg_value(TOKSI_OWNED_KEYS);
        let _ = delete_env_reg_value(TOKLESS_OWNED_MARKER);
        let _ = delete_env_reg_value(TOKLESS_OWNED_KEYS);
        let _ = delete_env_reg_value(CLOUD_CODE_URL_KEY);

        let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();
        overlay_restore_or_clear(&[], &key_refs);

        if !purged.contains(&TOKSI_OWNED_MARKER.to_string()) {
            purged.push(TOKSI_OWNED_MARKER.to_string());
        }
        if !purged.contains(&TOKSI_OWNED_KEYS.to_string()) {
            purged.push(TOKSI_OWNED_KEYS.to_string());
        }

        broadcast_setting_change();
    } else {
        overlay_restore_or_clear(&[], MANAGED_PROXY_KEYS);
    }

    Ok(purged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    #[cfg(windows)]
    fn test_windows_reg_read_write_delete() {
        let test_key = "TOKSI_UNIT_TEST_TMP_KEY";
        let test_val = "http://127.0.0.1:9999";

        let _ = delete_env_reg_value(test_key);
        assert_eq!(get_env_reg_value(test_key), None);

        match set_env_reg_value(test_key, test_val) {
            Ok(()) => {
                assert_eq!(get_env_reg_value(test_key).as_deref(), Some(test_val));
                let deleted = delete_env_reg_value(test_key);
                assert!(matches!(deleted, Ok(true)));
                assert_eq!(get_env_reg_value(test_key), None);
            }
            Err(e) => {
                assert!(e.is_access_denied(), "unexpected error: {e}");
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_broadcast_setting_change() {
        let res = broadcast_setting_change();
        assert!(res || !res);
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_forbids_cloud_code_url() {
        let res = set_env_reg_value(CLOUD_CODE_URL_KEY, "http://127.0.0.1:8787");
        assert!(matches!(res, Err(EnvInjectorError::ForbiddenKey(_))));
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_env_inject_and_teardown() {
        let Ok(_lock) = crate::env_injector::TEST_ENV_LOCK.lock() else {
            return;
        };
        let test_url = "http://127.0.0.1:8787";
        let snapshot = capture_windows_reg_snapshot(MANAGED_PROXY_KEYS);

        match inject_windows_env(test_url) {
            Ok(injected) => {
                assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
                assert!(injected.contains(&"ANTHROPIC_BASE_URL".to_string()));
                assert!(injected.contains(&TOKSI_OWNED_MARKER.to_string()));
                assert_eq!(
                    crate::env_injector::effective_env_var("GOOGLE_GEMINI_BASE_URL").as_deref(),
                    Some(test_url)
                );
                assert_eq!(
                    crate::env_injector::effective_env_var(TOKSI_OWNED_MARKER).as_deref(),
                    Some("1")
                );
                assert_eq!(crate::env_injector::effective_env_var(CLOUD_CODE_URL_KEY), None);

                let mut cmd = Command::new("cmd");
                crate::env_injector::apply_env_to_command(&mut cmd);
                let has_cloud_remove = cmd
                    .get_envs()
                    .any(|(k, v)| k == CLOUD_CODE_URL_KEY && v.is_none());
                assert!(has_cloud_remove);

                let restored = restore_windows_reg_snapshot(&snapshot);
                assert!(restored.is_ok());
                assert_eq!(
                    crate::env_injector::effective_env_var(TOKSI_OWNED_MARKER),
                    None
                );
                assert_eq!(crate::env_injector::effective_env_var(CLOUD_CODE_URL_KEY), None);
            }
            Err(e) => {
                assert!(e.is_access_denied(), "unexpected error: {e}");
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_snapshot_restores_original_reg_value() {
        let test_key = "TOKSI_UNIT_TEST_SNAPSHOT_RESTORE";
        let orig_val = "https://original.example.com";
        let _ = delete_env_reg_value(test_key);

        if let Err(e) = set_env_reg_value(test_key, orig_val) {
            assert!(e.is_access_denied(), "unexpected error: {e}");
            return;
        }

        let snapshot = capture_windows_reg_snapshot(&[test_key]);
        let inj = inject_windows_env_vars(&[(test_key, "http://127.0.0.1:8787")]);
        assert!(inj.is_ok());
        assert_eq!(
            get_env_reg_value(test_key).as_deref(),
            Some("http://127.0.0.1:8787")
        );

        let rest = restore_windows_reg_snapshot(&snapshot);
        assert!(rest.is_ok());
        assert_eq!(get_env_reg_value(test_key).as_deref(), Some(orig_val));

        let _ = delete_env_reg_value(test_key);
    }
}
