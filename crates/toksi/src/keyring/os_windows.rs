// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/os_windows.rs
// created: 2026-11-04
// lastModified: 2026-11-04

#[cfg(windows)]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct FILETIME {
        dwLowDateTime: u32,
        dwHighDateTime: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    struct CREDENTIALW {
        Flags: u32,
        Type: u32,
        TargetName: *mut u16,
        Comment: *mut u16,
        LastWritten: FILETIME,
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
            Credential: *mut *mut CREDENTIALW,
        ) -> i32;

        fn CredWriteW(Credential: *const CREDENTIALW, Flags: u32) -> i32;

        fn CredDeleteW(TargetName: *const u16, Type: u32, Flags: u32) -> i32;

        fn CredFree(Buffer: *mut std::ffi::c_void);
    }

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn read_credential(target_name: &str) -> Option<String> {
        let wide_target = to_wide(target_name);
        let mut cred_ptr: *mut CREDENTIALW = std::ptr::null_mut();

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
                let slice = std::slice::from_raw_parts(blob_ptr, blob_len);
                // Windows credential blobs can be UTF-8 or UTF-16LE
                if let Ok(utf8) = std::str::from_utf8(slice) {
                    Some(utf8.trim_end_matches('\0').to_string())
                } else if blob_len % 2 == 0 {
                    let u16_slice: &[u16] = std::slice::from_raw_parts(blob_ptr as *const u16, blob_len / 2);
                    String::from_utf16(u16_slice).ok().map(|s| s.trim_end_matches('\0').to_string())
                } else {
                    None
                }
            } else {
                None
            };

            CredFree(cred_ptr as *mut std::ffi::c_void);
            result
        }
    }

    pub fn write_credential(target_name: &str, user_name: Option<&str>, secret: &str) -> bool {
        let mut wide_target = to_wide(target_name);
        let mut wide_user = user_name.map(to_wide);
        let mut blob_bytes = secret.as_bytes().to_vec();

        let mut cred = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: wide_target.as_mut_ptr(),
            Comment: std::ptr::null_mut(),
            LastWritten: FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 },
            CredentialBlobSize: blob_bytes.len() as u32,
            CredentialBlob: blob_bytes.as_mut_ptr(),
            Persist: CRED_PERSIST_ENTERPRISE,
            AttributeCount: 0,
            Attributes: std::ptr::null_mut(),
            TargetAlias: std::ptr::null_mut(),
            UserName: wide_user.as_mut().map(|w| w.as_mut_ptr()).unwrap_or(std::ptr::null_mut()),
        };

        unsafe {
            if CredWriteW(&cred, 0) != 0 {
                return true;
            }
            // Fallback with Persist = LOCAL_MACHINE and NULL UserName
            cred.Persist = CRED_PERSIST_LOCAL_MACHINE;
            cred.UserName = std::ptr::null_mut();
            CredWriteW(&cred, 0) != 0
        }
    }

    pub fn delete_credential(target_name: &str) -> bool {
        let wide_target = to_wide(target_name);
        unsafe {
            CredDeleteW(wide_target.as_ptr(), CRED_TYPE_GENERIC, 0) != 0
        }
    }
}

#[cfg(windows)]
pub use imp::*;

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
