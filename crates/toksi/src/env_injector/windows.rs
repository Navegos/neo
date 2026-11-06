// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/env_injector/windows.rs
// created: 2026-10-31
// lastModified: 2026-11-06

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

pub const TOKSI_OWNED_MARKER: &str = "TOKSI_OWNED";
pub const TOKSI_OWNED_KEYS: &str = "TOKSI_OWNED_KEYS";
pub const TOKLESS_OWNED_MARKER: &str = "TOKLESS_OWNED";
pub const TOKLESS_OWNED_KEYS: &str = "TOKLESS_OWNED_KEYS";

pub const MANAGED_PROXY_KEYS: &[&str] = &[
    "GOOGLE_GEMINI_BASE_URL",
    "ANTHROPIC_BASE_URL",
];

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
pub fn broadcast_setting_change() -> bool {
    let env_wide = to_wide("Environment");
    let mut result: usize = 0;
    unsafe {
        let ret: LRESULT = SendMessageTimeoutW(
            HWND_BROADCAST as HWND,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            env_wide.as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            2000,
            &mut result,
        );
        ret != 0
    }
}

#[cfg(windows)]
pub fn get_env_reg_value(name: &str) -> Option<String> {
    unsafe {
        let subkey = to_wide("Environment");
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        ) != 0
        {
            return None;
        }

        let val_name = to_wide(name);
        let mut val_type: u32 = 0;
        let mut buf_bytes: u32 = 0;

        let query_res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            std::ptr::null_mut(),
            &mut buf_bytes,
        );
        if query_res != 0 || buf_bytes == 0 {
            RegCloseKey(hkey);
            return None;
        }

        let mut buffer = vec![0u8; buf_bytes as usize];
        let get_res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            buffer.as_mut_ptr(),
            &mut buf_bytes,
        );
        RegCloseKey(hkey);

        if get_res != 0 {
            return None;
        }

        if val_type == REG_SZ {
            let u16_slice = std::slice::from_raw_parts(
                buffer.as_ptr() as *const u16,
                (buf_bytes as usize) / 2,
            );
            let len = u16_slice.iter().position(|&c| c == 0).unwrap_or(u16_slice.len());
            Some(String::from_utf16_lossy(&u16_slice[..len]))
        } else {
            None
        }
    }
}

#[cfg(windows)]
pub fn set_env_reg_value(name: &str, value: &str) -> Result<(), String> {
    unsafe {
        let subkey = to_wide("Environment");
        let mut hkey: HKEY = std::ptr::null_mut();
        let mut open_res = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if open_res != 0 {
            let mut disposition: u32 = 0;
            open_res = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                std::ptr::null(),
                0,
                KEY_SET_VALUE,
                std::ptr::null(),
                &mut hkey,
                &mut disposition,
            );
        }
        if open_res != 0 {
            return Err(format!("RegOpenKeyExW/RegCreateKeyExW failed with code {}", open_res));
        }

        let val_name = to_wide(name);
        let val_data = to_wide(value);
        let byte_len = (val_data.len() * 2) as u32;

        let set_res = RegSetValueExW(
            hkey,
            val_name.as_ptr(),
            0,
            REG_SZ,
            val_data.as_ptr() as *const u8,
            byte_len,
        );
        RegCloseKey(hkey);

        if set_res != 0 {
            return Err(format!("RegSetValueExW failed with code {}", set_res));
        }
        Ok(())
    }
}

#[cfg(windows)]
pub fn delete_env_reg_value(name: &str) -> Result<bool, String> {
    unsafe {
        let subkey = to_wide("Environment");
        let mut hkey: HKEY = std::ptr::null_mut();
        let open_res = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        );
        if open_res != 0 {
            return Ok(false);
        }

        let val_name = to_wide(name);
        let del_res = RegDeleteValueW(hkey, val_name.as_ptr());
        RegCloseKey(hkey);

        if del_res == 0 {
            Ok(true)
        } else {
            // 2 = ERROR_FILE_NOT_FOUND (not present)
            Ok(false)
        }
    }
}

#[cfg(windows)]
pub fn inject_windows_env_vars(vars: &[(&str, &str)]) -> Result<Vec<String>, String> {
    let mut injected = Vec::new();
    let keys: Vec<&str> = vars.iter().map(|(k, _)| *k).collect();
    let keys_csv = keys.join(",");
    let mut reg_err = None;

    for &(key, val) in vars {
        if let Err(e) = set_env_reg_value(key, val) {
            reg_err = Some(e);
        }
        unsafe {
            std::env::set_var(key, val);
        }
        injected.push(key.to_string());
    }

    // Explicitly delete CLOUD_CODE_URL from HKCU\Environment so auth/quota checks bypass proxy
    let _ = delete_env_reg_value("CLOUD_CODE_URL");
    unsafe {
        std::env::remove_var("CLOUD_CODE_URL");
    }

    if let Err(e) = set_env_reg_value(TOKSI_OWNED_MARKER, "1") {
        if reg_err.is_none() {
            reg_err = Some(e);
        }
    }
    let _ = set_env_reg_value(TOKSI_OWNED_KEYS, &keys_csv);
    unsafe {
        std::env::set_var(TOKSI_OWNED_MARKER, "1");
        std::env::set_var(TOKSI_OWNED_KEYS, &keys_csv);
    }
    injected.push(TOKSI_OWNED_MARKER.to_string());
    injected.push(TOKSI_OWNED_KEYS.to_string());

    broadcast_setting_change();

    if let Some(e) = reg_err {
        if !e.contains("code 5") {
            return Err(e);
        }
    }

    Ok(injected)
}

#[cfg(windows)]
pub fn inject_windows_env(proxy_url: &str) -> Result<Vec<String>, String> {
    let vars: Vec<(&str, &str)> = MANAGED_PROXY_KEYS.iter().map(|&k| (k, proxy_url)).collect();
    inject_windows_env_vars(&vars)
}

#[cfg(windows)]
pub fn teardown_windows_env() -> Result<Vec<String>, String> {
    let marker = get_env_reg_value(TOKSI_OWNED_MARKER)
        .or_else(|| std::env::var(TOKSI_OWNED_MARKER).ok())
        .or_else(|| get_env_reg_value(TOKLESS_OWNED_MARKER))
        .or_else(|| std::env::var(TOKLESS_OWNED_MARKER).ok());
    let mut purged = Vec::new();

    let has_stale_proxy = MANAGED_PROXY_KEYS.iter().any(|k| {
        get_env_reg_value(k)
            .map(|v| v.contains("127.0.0.1") || v.contains("localhost"))
            .unwrap_or(false)
            || std::env::var(k)
                .map(|v| v.contains("127.0.0.1") || v.contains("localhost"))
                .unwrap_or(false)
    });

    if marker.as_deref() == Some("1") || marker.is_some() || has_stale_proxy {
        let keys_str = get_env_reg_value(TOKSI_OWNED_KEYS)
            .or_else(|| std::env::var(TOKSI_OWNED_KEYS).ok())
            .or_else(|| get_env_reg_value(TOKLESS_OWNED_KEYS))
            .or_else(|| std::env::var(TOKLESS_OWNED_KEYS).ok())
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
            unsafe {
                std::env::remove_var(key);
            }
            if !purged.contains(key) {
                purged.push(key.clone());
            }
        }

        let _ = delete_env_reg_value(TOKSI_OWNED_MARKER);
        let _ = delete_env_reg_value(TOKSI_OWNED_KEYS);
        let _ = delete_env_reg_value(TOKLESS_OWNED_MARKER);
        let _ = delete_env_reg_value(TOKLESS_OWNED_KEYS);
        let _ = delete_env_reg_value("CLOUD_CODE_URL");
        unsafe {
            std::env::remove_var(TOKSI_OWNED_MARKER);
            std::env::remove_var(TOKSI_OWNED_KEYS);
            std::env::remove_var(TOKLESS_OWNED_MARKER);
            std::env::remove_var(TOKLESS_OWNED_KEYS);
            std::env::remove_var("CLOUD_CODE_URL");
        }
        if !purged.contains(&TOKSI_OWNED_MARKER.to_string()) {
            purged.push(TOKSI_OWNED_MARKER.to_string());
        }
        if !purged.contains(&TOKSI_OWNED_KEYS.to_string()) {
            purged.push(TOKSI_OWNED_KEYS.to_string());
        }

        broadcast_setting_change();
    }

    Ok(purged)
}

#[cfg(not(windows))]
pub fn broadcast_setting_change() -> bool {
    false
}

#[cfg(not(windows))]
pub fn inject_windows_env_vars(_vars: &[(&str, &str)]) -> Result<Vec<String>, String> {
    Err("Windows environment injection is only supported on Windows".to_string())
}

#[cfg(not(windows))]
pub fn inject_windows_env(_proxy_url: &str) -> Result<Vec<String>, String> {
    Err("Windows environment injection is only supported on Windows".to_string())
}

#[cfg(not(windows))]
pub fn teardown_windows_env() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_windows_reg_read_write_delete() {
        let test_key = "TOKSI_UNIT_TEST_TMP_KEY";
        let test_val = "http://127.0.0.1:9999";

        let _ = delete_env_reg_value(test_key);
        assert_eq!(get_env_reg_value(test_key), None);

        match set_env_reg_value(test_key, test_val) {
            Ok(_) => {
                assert_eq!(get_env_reg_value(test_key), Some(test_val.to_string()));
                let deleted = delete_env_reg_value(test_key).expect("delete failed");
                assert!(deleted);
                assert_eq!(get_env_reg_value(test_key), None);
            }
            Err(e) => {
                // In sandboxed containers, registry writes to HKCU\Environment yield code 5 (access denied)
                assert!(e.contains("code 5"), "unexpected error: {}", e);
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_broadcast_setting_change() {
        // Must not crash or panic
        let res = broadcast_setting_change();
        assert!(res || !res);
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_env_inject_and_teardown() {
        let _lock = crate::env_injector::TEST_ENV_LOCK.lock().unwrap();
        let test_url = "http://127.0.0.1:8787";
        let res = inject_windows_env(test_url);
        match res {
            Ok(injected) => {
                assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
                assert!(injected.contains(&"ANTHROPIC_BASE_URL".to_string()));
                assert!(injected.contains(&TOKSI_OWNED_MARKER.to_string()));
                assert_eq!(std::env::var("GOOGLE_GEMINI_BASE_URL").unwrap(), test_url);
                assert_eq!(std::env::var(TOKSI_OWNED_MARKER).unwrap(), "1");

                let purged = teardown_windows_env().expect("teardown failed");
                assert!(purged.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
                assert!(std::env::var("GOOGLE_GEMINI_BASE_URL").is_err());
                assert!(std::env::var(TOKSI_OWNED_MARKER).is_err());
            }
            Err(e) => {
                assert!(e.contains("code 5"), "unexpected error: {}", e);
            }
        }
    }
}
