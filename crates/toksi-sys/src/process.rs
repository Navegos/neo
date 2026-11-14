// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi-sys/src/process.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Process detachment, liveness, atomic replacement, and environment helpers isolated in `toksi-sys`.

use std::path::Path;
use std::process::Command;

/// Spawns `cmd` detached into its own process group/session without inheriting controlling TTY.
pub fn spawn_detached(mut cmd: Command) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        unsafe extern "C" {
            fn setsid() -> i32;
        }

        // SAFETY: `setsid()` takes no pointers or arguments, has no memory preconditions,
        // and is async-signal-safe per POSIX for detached child session initialization.
        unsafe {
            cmd.pre_exec(|| {
                let _ = setsid();
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }
    cmd.spawn().map(|_| ())
}

/// Checks whether a process with `pid` is currently alive.
pub fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    #[cfg(windows)]
    {
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
        const SYNCHRONIZE: u32 = 0x0010_0000;
        const WAIT_TIMEOUT: u32 = 258;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn OpenProcess(
                dwDesiredAccess: u32,
                bInheritHandle: i32,
                dwProcessId: u32,
            ) -> *mut std::ffi::c_void;
            fn WaitForSingleObject(hHandle: *mut std::ffi::c_void, dwMilliseconds: u32) -> u32;
            fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        }

        // SAFETY: `OpenProcess` returns either NULL or a valid OS handle. We null-check before
        // passing it to `WaitForSingleObject` and always release it with `CloseHandle`.
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid);
            if handle.is_null() {
                return false;
            }
            let wait_res = WaitForSingleObject(handle, 0);
            let _ = CloseHandle(handle);
            wait_res == WAIT_TIMEOUT
        }
    }

    #[cfg(not(windows))]
    {
        unsafe extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        let Ok(raw_pid) = i32::try_from(pid) else {
            return false;
        };
        // SAFETY: Signal `0` sends no signal and only performs POSIX existence/permission check on `raw_pid`.
        unsafe { kill(raw_pid, 0) == 0 }
    }
}

/// Atomically replaces `to` with `from`.
pub fn replace_file_atomic(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
        const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MoveFileExW(
                lpExistingFileName: *const u16,
                lpNewFileName: *const u16,
                dwFlags: u32,
            ) -> i32;
        }

        let from_w: Vec<u16> = from
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let to_w: Vec<u16> = to
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // SAFETY: `from_w` and `to_w` are NUL-terminated UTF-16 buffers that remain valid for
        // the synchronous duration of `MoveFileExW`.
        let r = unsafe {
            MoveFileExW(
                from_w.as_ptr(),
                to_w.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if r == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[cfg(not(windows))]
    {
        std::fs::rename(from, to)
    }
}

/// Sets a process-local environment variable.
///
/// Edition 2024 marks `std::env::set_var` `unsafe` because concurrent getenv/setenv in
/// multi-threaded FFI is UB; isolated here for controlled synchronous CLI updates.
pub fn set_process_env(key: &str, value: &str) {
    // SAFETY: Caller invokes during synchronous CLI setup without foreign C threads reading `environ`.
    unsafe {
        std::env::set_var(key, value);
    }
}

/// Removes a process-local environment variable.
///
/// Edition 2024 marks `std::env::remove_var` `unsafe` for the same reason as `set_process_env`.
pub fn remove_process_env(key: &str) {
    // SAFETY: Caller invokes during synchronous CLI setup/cleanup without foreign C threads reading `environ`.
    unsafe {
        std::env::remove_var(key);
    }
}
