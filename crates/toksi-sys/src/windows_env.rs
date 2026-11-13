// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi-sys/src/windows_env.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Windows `HKCU\Environment` Registry manipulation and `WM_SETTINGCHANGE` broadcast.

use crate::{RegEnvEntry, SysError};

pub const DEFAULT_BROADCAST_TIMEOUT_MS: u32 = 3000;

#[cfg(windows)]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, HWND, LPARAM, LRESULT, WPARAM,
    };
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
        RegSetValueExW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    };

    use crate::{RegEnvEntry, SysError};

    struct RegKeyGuard(HKEY);

    impl Drop for RegKeyGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: `self.0` is a valid open registry handle obtained from
                // `RegOpenKeyExW` or `RegCreateKeyExW` and is closed exactly once here.
                unsafe {
                    RegCloseKey(self.0);
                }
            }
        }
    }

    pub(super) fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    fn decode_utf16_le_bytes(bytes: &[u8]) -> String {
        let (chunks, _) = bytes.as_chunks::<2>();
        let units: Vec<u16> = chunks.iter().map(|&pair| u16::from_le_bytes(pair)).collect();
        let len = units.iter().position(|&c| c == 0).unwrap_or(units.len());
        String::from_utf16_lossy(&units[..len])
    }

    pub fn broadcast_env_setting_change(timeout_ms: u32) -> bool {
        let env_wide = to_wide("Environment");
        let mut result: usize = 0;
        // SAFETY: `HWND_BROADCAST` and `WM_SETTINGCHANGE` are valid broadcast targets.
        // `env_wide` is a NUL-terminated UTF-16 buffer alive for the duration of this
        // synchronous call, and `&mut result` is a valid aligned out-pointer.
        let ret: LRESULT = unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST as HWND,
                WM_SETTINGCHANGE,
                0 as WPARAM,
                env_wide.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                timeout_ms,
                &mut result,
            )
        };
        ret != 0
    }

    pub fn get_hkcu_env_entry(name: &str) -> Result<Option<RegEnvEntry>, SysError> {
        let subkey = to_wide("Environment");
        let mut raw_hkey: HKEY = null_mut();
        // SAFETY: `subkey` is a valid NUL-terminated UTF-16 string and `&mut raw_hkey`
        // is a valid aligned pointer to receive the opened `HKEY`.
        let open_res = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_QUERY_VALUE,
                &mut raw_hkey,
            )
        };
        if open_res != 0 {
            if open_res == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            return Err(SysError::Registry {
                op: "RegOpenKeyExW",
                code: open_res,
            });
        }
        let key_guard = RegKeyGuard(raw_hkey);

        let val_name = to_wide(name);
        let mut val_type: u32 = 0;
        let mut buf_bytes: u32 = 0;

        // SAFETY: `key_guard.0` is a valid open HKEY, `val_name` is NUL-terminated UTF-16,
        // data pointer is null for size-only query, and `&mut val_type` / `&mut buf_bytes`
        // are valid aligned out-pointers.
        let size_res = unsafe {
            RegQueryValueExW(
                key_guard.0,
                val_name.as_ptr(),
                null(),
                &mut val_type,
                null_mut(),
                &mut buf_bytes,
            )
        };
        if size_res == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if size_res != 0 {
            return Err(SysError::Registry {
                op: "RegQueryValueExW(size)",
                code: size_res,
            });
        }
        if buf_bytes == 0 {
            return Ok(Some(RegEnvEntry {
                value: String::new(),
                reg_type: val_type,
            }));
        }

        // Retry loop handles concurrent registry growth (ERROR_MORE_DATA).
        for _ in 0..4 {
            let mut buffer = vec![0u8; buf_bytes as usize];
            let mut actual_bytes = buf_bytes;
            // SAFETY: `key_guard.0` is valid, `val_name` is NUL-terminated UTF-16,
            // `buffer` has length `actual_bytes` bytes writable by Win32, and
            // `&mut val_type` / `&mut actual_bytes` are valid aligned pointers.
            let get_res = unsafe {
                RegQueryValueExW(
                    key_guard.0,
                    val_name.as_ptr(),
                    null(),
                    &mut val_type,
                    buffer.as_mut_ptr(),
                    &mut actual_bytes,
                )
            };
            if get_res == ERROR_MORE_DATA {
                buf_bytes = actual_bytes.max(buf_bytes.saturating_mul(2));
                continue;
            }
            if get_res == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if get_res != 0 {
                return Err(SysError::Registry {
                    op: "RegQueryValueExW(read)",
                    code: get_res,
                });
            }
            if val_type != REG_SZ && val_type != REG_EXPAND_SZ {
                return Err(SysError::UnexpectedValueType {
                    name: name.to_string(),
                    reg_type: val_type,
                });
            }
            let used = (actual_bytes as usize).min(buffer.len());
            return Ok(Some(RegEnvEntry {
                value: decode_utf16_le_bytes(&buffer[..used]),
                reg_type: val_type,
            }));
        }

        Err(SysError::Registry {
            op: "RegQueryValueExW(retry)",
            code: ERROR_MORE_DATA,
        })
    }

    pub fn set_hkcu_env_entry(name: &str, value: &str, reg_type: u32) -> Result<(), SysError> {
        let effective_type = if reg_type == REG_EXPAND_SZ {
            REG_EXPAND_SZ
        } else {
            REG_SZ
        };

        let subkey = to_wide("Environment");
        let mut raw_hkey: HKEY = null_mut();
        // SAFETY: `subkey` is NUL-terminated UTF-16 and `&mut raw_hkey` is a valid out-pointer.
        let mut open_res = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut raw_hkey,
            )
        };
        if open_res != 0 && open_res == ERROR_FILE_NOT_FOUND {
            let mut disposition: u32 = 0;
            // SAFETY: `subkey` is NUL-terminated UTF-16, optional class/security pointers
            // are null, and `&mut raw_hkey` / `&mut disposition` are valid out-pointers.
            open_res = unsafe {
                RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    subkey.as_ptr(),
                    0,
                    null(),
                    0,
                    KEY_SET_VALUE,
                    null(),
                    &mut raw_hkey,
                    &mut disposition,
                )
            };
        }
        if open_res != 0 {
            return Err(SysError::Registry {
                op: "RegOpenKeyExW/RegCreateKeyExW",
                code: open_res,
            });
        }
        let key_guard = RegKeyGuard(raw_hkey);

        let val_name = to_wide(name);
        let val_data = to_wide(value);
        let total_bytes = val_data.len().saturating_mul(2);
        let byte_len = u32::try_from(total_bytes).map_err(|_| SysError::ValueTooLarge {
            name: name.to_string(),
            len: total_bytes,
        })?;

        // SAFETY: `key_guard.0` is an open HKEY with `KEY_SET_VALUE`, `val_name` is
        // NUL-terminated UTF-16, and `val_data` is a NUL-terminated UTF-16 buffer whose
        // byte length is `byte_len` (`val_data.len() * 2`).
        let set_res = unsafe {
            RegSetValueExW(
                key_guard.0,
                val_name.as_ptr(),
                0,
                effective_type,
                val_data.as_ptr().cast::<u8>(),
                byte_len,
            )
        };
        if set_res != 0 {
            return Err(SysError::Registry {
                op: "RegSetValueExW",
                code: set_res,
            });
        }
        Ok(())
    }

    pub fn delete_hkcu_env_value(name: &str) -> Result<bool, SysError> {
        let subkey = to_wide("Environment");
        let mut raw_hkey: HKEY = null_mut();
        // SAFETY: `subkey` is NUL-terminated UTF-16 and `&mut raw_hkey` is a valid out-pointer.
        let open_res = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE | KEY_QUERY_VALUE,
                &mut raw_hkey,
            )
        };
        if open_res == ERROR_FILE_NOT_FOUND {
            return Ok(false);
        }
        if open_res != 0 {
            return Err(SysError::Registry {
                op: "RegOpenKeyExW(delete)",
                code: open_res,
            });
        }
        let key_guard = RegKeyGuard(raw_hkey);

        let val_name = to_wide(name);
        // SAFETY: `key_guard.0` is an open HKEY with `KEY_SET_VALUE` and `val_name` is
        // a valid NUL-terminated UTF-16 string.
        let del_res = unsafe { RegDeleteValueW(key_guard.0, val_name.as_ptr()) };
        if del_res == 0 {
            Ok(true)
        } else if del_res == ERROR_FILE_NOT_FOUND {
            Ok(false)
        } else {
            Err(SysError::Registry {
                op: "RegDeleteValueW",
                code: del_res,
            })
        }
    }

    #[cfg(test)]
    pub(super) fn reg_sz_type() -> u32 {
        REG_SZ
    }
}

#[cfg(windows)]
pub fn broadcast_env_setting_change(timeout_ms: u32) -> bool {
    imp::broadcast_env_setting_change(timeout_ms)
}

#[cfg(not(windows))]
pub fn broadcast_env_setting_change(_timeout_ms: u32) -> bool {
    false
}

#[cfg(windows)]
pub fn get_hkcu_env_entry(name: &str) -> Result<Option<RegEnvEntry>, SysError> {
    imp::get_hkcu_env_entry(name)
}

#[cfg(not(windows))]
pub fn get_hkcu_env_entry(_name: &str) -> Result<Option<RegEnvEntry>, SysError> {
    Err(SysError::UnsupportedPlatform("Windows HKCU\\Environment"))
}

pub fn get_hkcu_env_value(name: &str) -> Result<Option<String>, SysError> {
    get_hkcu_env_entry(name).map(|opt| opt.map(|entry| entry.value))
}

#[cfg(windows)]
pub fn set_hkcu_env_entry(name: &str, value: &str, reg_type: u32) -> Result<(), SysError> {
    imp::set_hkcu_env_entry(name, value, reg_type)
}

#[cfg(not(windows))]
pub fn set_hkcu_env_entry(_name: &str, _value: &str, _reg_type: u32) -> Result<(), SysError> {
    Err(SysError::UnsupportedPlatform("Windows HKCU\\Environment"))
}

pub fn set_hkcu_env_value(name: &str, value: &str) -> Result<(), SysError> {
    set_hkcu_env_entry(name, value, 1)
}

#[cfg(windows)]
pub fn delete_hkcu_env_value(name: &str) -> Result<bool, SysError> {
    imp::delete_hkcu_env_value(name)
}

#[cfg(not(windows))]
pub fn delete_hkcu_env_value(_name: &str) -> Result<bool, SysError> {
    Err(SysError::UnsupportedPlatform("Windows HKCU\\Environment"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_wide_string_is_nul_terminated() {
        let wide = imp::to_wide("Environment");
        assert_eq!(wide.last().copied(), Some(0));
        assert_eq!(imp::reg_sz_type(), 1);
    }

    #[test]
    #[cfg(windows)]
    fn test_hkcu_env_roundtrip() {
        let key = "TOKSI_SYS_UNIT_TEST_TMP";
        let val = "http://127.0.0.1:8787";
        let _ = delete_hkcu_env_value(key);

        match set_hkcu_env_value(key, val) {
            Ok(()) => {
                assert_eq!(get_hkcu_env_value(key).ok().flatten().as_deref(), Some(val));
                let deleted = delete_hkcu_env_value(key).ok();
                assert_eq!(deleted, Some(true));
                assert_eq!(get_hkcu_env_value(key).ok().flatten(), None);
            }
            Err(e) => {
                assert!(e.is_access_denied(), "unexpected error: {e}");
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_broadcast_setting_change_safe() {
        let _ = broadcast_env_setting_change(DEFAULT_BROADCAST_TIMEOUT_MS);
    }
}
