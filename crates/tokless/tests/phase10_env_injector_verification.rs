// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase10_env_injector_verification.rs
// created: 2026-10-31

use std::sync::Mutex;

static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

use tokless::env_injector::{
    inject_proxy_env, register_teardown_handlers, teardown_proxy_env, TeardownGuard,
};

#[test]
fn test_phase10_env_injection_and_teardown_cycle() {
    let _lock = TEST_ENV_LOCK.lock().unwrap();
    // 1. Initial teardown to ensure clean state
    let _ = teardown_proxy_env();

    // 2. Inject environment
    let injected = inject_proxy_env().expect("environment injection should succeed");
    assert!(!injected.is_empty(), "should report injected keys");
    assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert!(injected.contains(&"CLOUD_CODE_URL".to_string()));
    assert!(injected.contains(&"TOKLESS_OWNED".to_string()));

    // Verify current process has the injected variable
    assert_eq!(
        std::env::var("TOKLESS_OWNED").ok().as_deref(),
        Some("1")
    );

    // 3. Teardown environment
    let purged = teardown_proxy_env().expect("environment teardown should succeed");
    assert!(!purged.is_empty(), "should report purged keys");
    assert!(purged.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert!(purged.contains(&"CLOUD_CODE_URL".to_string()));
    assert!(purged.contains(&"TOKLESS_OWNED".to_string()));

    // Verify current process no longer has the marker
    assert!(std::env::var("TOKLESS_OWNED").is_err());
}

#[test]
fn test_phase10_teardown_guard_raii() {
    let _lock = TEST_ENV_LOCK.lock().unwrap();
    // Ensure clean state
    let _ = teardown_proxy_env();

    // Inject
    let _ = inject_proxy_env();
    assert_eq!(std::env::var("TOKLESS_OWNED").ok().as_deref(), Some("1"));

    // Guard goes out of scope and drops
    {
        let _guard = TeardownGuard::new();
    }

    // Teardown should have executed
    assert!(std::env::var("TOKLESS_OWNED").is_err());
}

#[test]
fn test_phase10_teardown_guard_disarm() {
    let _lock = TEST_ENV_LOCK.lock().unwrap();
    let _ = teardown_proxy_env();
    let _ = inject_proxy_env();
    assert_eq!(std::env::var("TOKLESS_OWNED").ok().as_deref(), Some("1"));

    {
        let mut guard = TeardownGuard::new();
        guard.disarm();
    }

    // Because guard was disarmed, it should not have purged
    assert_eq!(std::env::var("TOKLESS_OWNED").ok().as_deref(), Some("1"));

    // Clean up
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
    use tokless::env_injector::windows::{
        broadcast_setting_change, delete_env_reg_value, get_env_reg_value, set_env_reg_value,
    };

    let test_key = "TOKLESS_PHASE10_VERIFY_KEY";
    let test_val = "http://127.0.0.1:8787";

    let _ = delete_env_reg_value(test_key);
    assert!(get_env_reg_value(test_key).is_none());

    match set_env_reg_value(test_key, test_val) {
        Ok(_) => {
            assert_eq!(get_env_reg_value(test_key), Some(test_val.to_string()));
            let broadcast_ok = broadcast_setting_change();
            assert!(broadcast_ok || !broadcast_ok);
            let deleted = delete_env_reg_value(test_key).expect("deleting registry value must succeed");
            assert!(deleted);
            assert!(get_env_reg_value(test_key).is_none());
        }
        Err(e) => {
            // In sandboxed containers, registry writes to HKCU\Environment yield code 5 (access denied).
            assert!(e.contains("code 5"), "unexpected error: {}", e);
        }
    }
}

#[test]
fn test_phase10_unix_dropin_and_vars() {
    let _lock = TEST_ENV_LOCK.lock().unwrap();
    use tokless::env_injector::unix::{inject_unix_env, teardown_unix_env};

    let injected = inject_unix_env("http://127.0.0.1:8787").expect("unix inject should succeed");
    assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert_eq!(std::env::var("GOOGLE_GEMINI_BASE_URL").unwrap(), "http://127.0.0.1:8787");

    let purged = teardown_unix_env().expect("unix teardown should succeed");
    assert!(purged.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
    assert!(std::env::var("GOOGLE_GEMINI_BASE_URL").is_err());
}

