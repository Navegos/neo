// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase10_env_injector_verification.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::ffi::OsStr;
use std::process::Command;
use std::sync::Mutex;

static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

use toksi::env_injector::{
    apply_env_to_command, effective_env_var, inject_proxy_env, register_teardown_handlers,
    teardown_proxy_env, TeardownGuard,
};

#[test]
fn test_phase10_env_injection_and_teardown_cycle() {
    let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = teardown_proxy_env();

    let injected = match inject_proxy_env() {
        Ok(keys) => keys,
        Err(e) => panic!("environment injection should succeed: {}", e),
    };
    assert!(!injected.is_empty(), "should report injected keys");
    assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert!(
        !injected.contains(&"CLOUD_CODE_URL".to_string()),
        "CLOUD_CODE_URL must not be injected (bypass auth)"
    );
    assert!(injected.contains(&"TOKSI_OWNED".to_string()));

    assert_eq!(effective_env_var("TOKSI_OWNED").as_deref(), Some("1"));

    let mut cmd = Command::new("toksi");
    apply_env_to_command(&mut cmd);
    let envs: Vec<(&OsStr, Option<&OsStr>)> = cmd.get_envs().collect();
    assert!(envs.contains(&(OsStr::new("TOKSI_OWNED"), Some(OsStr::new("1")))));
    assert!(envs.contains(&(OsStr::new("CLOUD_CODE_URL"), None)));

    let purged = match teardown_proxy_env() {
        Ok(keys) => keys,
        Err(e) => panic!("environment teardown should succeed: {}", e),
    };
    assert!(!purged.is_empty(), "should report purged keys");
    assert!(purged.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert!(purged.contains(&"TOKSI_OWNED".to_string()));

    assert!(effective_env_var("TOKSI_OWNED").is_none());
}

#[test]
fn test_phase10_teardown_guard_raii() {
    let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = teardown_proxy_env();

    let _ = inject_proxy_env();
    assert_eq!(effective_env_var("TOKSI_OWNED").as_deref(), Some("1"));

    {
        let _guard = TeardownGuard::new();
    }

    assert!(effective_env_var("TOKSI_OWNED").is_none());
}

#[test]
fn test_phase10_teardown_guard_disarm() {
    let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = teardown_proxy_env();
    let _ = inject_proxy_env();
    assert_eq!(effective_env_var("TOKSI_OWNED").as_deref(), Some("1"));

    {
        let mut guard = TeardownGuard::new();
        guard.disarm();
    }

    assert_eq!(effective_env_var("TOKSI_OWNED").as_deref(), Some("1"));

    let _ = teardown_proxy_env();
}

#[test]
fn test_phase10_register_handlers_idempotency() {
    register_teardown_handlers();
    register_teardown_handlers();
    register_teardown_handlers();
}

#[cfg(windows)]
#[test]
fn test_phase10_windows_native_reg_and_broadcast() {
    use toksi::env_injector::windows::{
        broadcast_setting_change, delete_env_reg_value, get_env_reg_value, set_env_reg_value,
    };

    let test_key = "TOKSI_PHASE10_VERIFY_KEY";
    let test_val = "http://127.0.0.1:8787";

    let _ = delete_env_reg_value(test_key);
    assert!(get_env_reg_value(test_key).is_none());

    match set_env_reg_value(test_key, test_val) {
        Ok(_) => {
            assert_eq!(get_env_reg_value(test_key), Some(test_val.to_string()));
            let broadcast_ok = broadcast_setting_change();
            assert!(broadcast_ok || !broadcast_ok);
            let deleted = match delete_env_reg_value(test_key) {
                Ok(deleted) => deleted,
                Err(e) => panic!("deleting registry value must succeed: {}", e),
            };
            assert!(deleted);
            assert!(get_env_reg_value(test_key).is_none());
        }
        Err(e) => {
            assert!(
                e.is_access_denied() || e.to_string().contains("code 5"),
                "unexpected error: {}",
                e
            );
        }
    }
}

#[test]
fn test_phase10_unix_dropin_and_vars() {
    let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use toksi::env_injector::unix::{
        inject_unix_env_vars_in, read_environment_d_entries, teardown_unix_env_in,
    };

    let temp_dir = std::env::temp_dir().join(format!("toksi_p10_unix_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let conf_path = temp_dir.join("99-toksi.conf");

    let injected = match inject_unix_env_vars_in(
        &[("GOOGLE_GEMINI_BASE_URL", "http://127.0.0.1:8787")],
        Some(&conf_path),
        false,
    ) {
        Ok(keys) => keys,
        Err(e) => panic!("unix inject should succeed: {}", e),
    };
    assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert_eq!(
        effective_env_var("GOOGLE_GEMINI_BASE_URL").as_deref(),
        Some("http://127.0.0.1:8787")
    );
    let entries = read_environment_d_entries(&conf_path);
    assert_eq!(
        entries.get("GOOGLE_GEMINI_BASE_URL").map(String::as_str),
        Some("http://127.0.0.1:8787")
    );
    assert!(!entries.contains_key("CLOUD_CODE_URL"));

    let purged = match teardown_unix_env_in(Some(&conf_path), None, false) {
        Ok(keys) => keys,
        Err(e) => panic!("unix teardown should succeed: {}", e),
    };
    assert!(purged.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert!(effective_env_var("GOOGLE_GEMINI_BASE_URL").is_none());
    assert!(!conf_path.exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

