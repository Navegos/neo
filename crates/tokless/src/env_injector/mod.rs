// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/env_injector/mod.rs
// created: 2026-10-31

pub mod heal;
pub mod teardown;
pub mod unix;
pub mod windows;

pub use heal::{
    heal_legacy_hijacks, heal_legacy_hijacks_in, scan_legacy_hijacks, scan_legacy_hijacks_in,
    HealingReport,
};
pub use teardown::{
    inject_proxy_env, register_teardown_handlers, teardown_proxy_env, TeardownGuard,
};

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
        .map(|(s, _)| s.to_string_lossy().to_string())
        .collect();
    let shims_detected: Vec<String> = scan
        .removed_shims
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let state_files_detected: Vec<String> = scan
        .removed_state_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let mut active_proxy_vars = Vec::new();
    let mut details = Vec::new();
    let mut env_vars_healthy = true;

    #[cfg(windows)]
    {
        use windows::{get_env_reg_value, MANAGED_PROXY_KEYS, TOKLESS_OWNED_MARKER};
        let is_owned = get_env_reg_value(TOKLESS_OWNED_MARKER).as_deref() == Some("1");
        if is_owned {
            active_proxy_vars.push(TOKLESS_OWNED_MARKER.to_string());
            for &key in MANAGED_PROXY_KEYS {
                if let Some(val) = get_env_reg_value(key) {
                    active_proxy_vars.push(format!("{}={}", key, val));
                }
            }
            details.push("Windows HKCU\\Environment has active Tokless proxy configuration".to_string());
        } else {
            // Check if orphaned proxy vars exist in registry without TOKLESS_OWNED
            for &key in MANAGED_PROXY_KEYS {
                if let Some(val) = get_env_reg_value(key) {
                    if val.contains("127.0.0.1") || val.contains("localhost") {
                        env_vars_healthy = false;
                        details.push(format!("Stale registry value without TOKLESS_OWNED: {}={}", key, val));
                    }
                }
            }
            if env_vars_healthy {
                details.push("Windows HKCU\\Environment clean (no proxy override active)".to_string());
            }
        }
    }

    #[cfg(not(windows))]
    {
        for key in ["GOOGLE_GEMINI_BASE_URL", "CLOUD_CODE_URL", "ANTHROPIC_BASE_URL", "TOKLESS_OWNED"] {
            if let Ok(val) = std::env::var(key) {
                active_proxy_vars.push(format!("{}={}", key, val));
            }
        }
        details.push("Unix session environment clean".to_string());
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
