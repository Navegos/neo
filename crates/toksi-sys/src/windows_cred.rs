// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi-sys/src/windows_cred.rs
// created: 2026-11-18
// lastModified: 2026-11-18

//! Isolated Win32 Credential Manager (`advapi32.dll`) FFI wrappers.

#[cfg(windows)]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use zeroize::Zeroize;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct Filetime {
        dwLowDateTime: u32,
        dwHighDateTime: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    struct Credentialw {
        Flags: u32,
        Type: u32,
        TargetName: *mut u16,
        Comment: *mut u16,
        LastWritten: Filetime,
        CredentialBlobSize: u32,
        CredentialBlob: *mut u8,
        Persist: u32,
        AttributeCount: u32,
        Attributes: *mut std::ffi::c_void,
        TargetAlias: *mut u16,
        UserName: *mut u16,
    }

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    const CRED_PERSIST_ENTERPRISE: u32 = 3;

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn CredReadW(
            TargetName: *const u16,
            Type: u32,
            Flags: u32,
            Credential: *mut *mut Credentialw,
        ) -> i32;

        fn CredWriteW(Credential: *const Credentialw, Flags: u32) -> i32;

        fn CredDeleteW(TargetName: *const u16, Type: u32, Flags: u32) -> i32;

        fn CredFree(Buffer: *mut std::ffi::c_void);
    }

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn read_credential(target_name: &str) -> Option<String> {
        let wide_target = to_wide(target_name);
        let mut cred_ptr: *mut Credentialw = std::ptr::null_mut();

        // SAFETY: `wide_target` is a valid null-terminated UTF-16 buffer for the duration of
        // `CredReadW`. On success (`res != 0` and `!cred_ptr.is_null()`), `cred_ptr` points to a
        // Win32-allocated `CREDENTIALW` struct valid until `CredFree(cred_ptr)` is called.
        unsafe {
            let res = CredReadW(
                wide_target.as_ptr(),
                CRED_TYPE_GENERIC,
                0,
                &mut cred_ptr,
            );
            if res == 0 || cred_ptr.is_null() {
                return None;
            }

            let cred = &*cred_ptr;
            let blob_len = cred.CredentialBlobSize as usize;
            let blob_ptr = cred.CredentialBlob;

            let result = if !blob_ptr.is_null() && blob_len > 0 {
                // SAFETY: `blob_ptr` is non-null and `CredentialBlobSize` specifies its exact byte
                // length within the `CREDENTIALW` allocation owned by `CredReadW`.
                let slice = std::slice::from_raw_parts(blob_ptr, blob_len);
                if let Ok(utf8) = std::str::from_utf8(slice) {
                    Some(utf8.trim_end_matches('\0').to_string())
                } else if blob_len.is_multiple_of(2) {
                    // SAFETY: `blob_len` is even; `from_raw_parts_mut` is not used and we copy
                    // via unaligned reads if alignment is not guaranteed, or slice u16 when aligned.
                    let mut u16_buf = Vec::with_capacity(blob_len / 2);
                    for chunk in slice.chunks_exact(2) {
                        u16_buf.push(u16::from_le_bytes([chunk[0], chunk[1]]));
                    }
                    let decoded = String::from_utf16(&u16_buf)
                        .ok()
                        .map(|s| s.trim_end_matches('\0').to_string());
                    u16_buf.zeroize();
                    decoded
                } else {
                    None
                }
            } else {
                None
            };

            // SAFETY: `cred_ptr` was allocated by `CredReadW` and is freed exactly once here.
            CredFree(cred_ptr as *mut std::ffi::c_void);
            result
        }
    }

    pub fn write_credential(target_name: &str, user_name: Option<&str>, secret: &str) -> bool {
        let mut wide_target = to_wide(target_name);
        let mut wide_user = user_name.map(to_wide);
        let mut blob_bytes = secret.as_bytes().to_vec();
        let Ok(blob_size) = u32::try_from(blob_bytes.len()) else {
            blob_bytes.zeroize();
            return false;
        };

        let mut cred = Credentialw {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: wide_target.as_mut_ptr(),
            Comment: std::ptr::null_mut(),
            LastWritten: Filetime {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            CredentialBlobSize: blob_size,
            CredentialBlob: blob_bytes.as_mut_ptr(),
            Persist: CRED_PERSIST_ENTERPRISE,
            AttributeCount: 0,
            Attributes: std::ptr::null_mut(),
            TargetAlias: std::ptr::null_mut(),
            UserName: wide_user
                .as_mut()
                .map(|w| w.as_mut_ptr())
                .unwrap_or(std::ptr::null_mut()),
        };

        // SAFETY: `wide_target` and optional `wide_user` are valid null-terminated UTF-16 buffers,
        // and `blob_bytes` is a valid buffer of `blob_size` bytes; all outlive `CredWriteW`.
        let ok = unsafe {
            if CredWriteW(&cred, 0) != 0 {
                true
            } else {
                cred.Persist = CRED_PERSIST_LOCAL_MACHINE;
                cred.UserName = std::ptr::null_mut();
                CredWriteW(&cred, 0) != 0
            }
        };

        blob_bytes.zeroize();
        ok
    }

    pub fn delete_credential(target_name: &str) -> bool {
        let wide_target = to_wide(target_name);
        // SAFETY: `wide_target` is a valid null-terminated UTF-16 buffer for the duration of `CredDeleteW`.
        unsafe { CredDeleteW(wide_target.as_ptr(), CRED_TYPE_GENERIC, 0) != 0 }
    }
}

#[cfg(windows)]
pub use imp::{delete_credential, read_credential, write_credential};

#[cfg(not(windows))]
pub fn read_credential(_target_name: &str) -> Option<String> {
    None
}

#[cfg(not(windows))]
pub fn write_credential(_target_name: &str, _user_name: Option<&str>, _secret: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub fn delete_credential(_target_name: &str) -> bool {
    false
}
