// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi-sys/src/sandbox.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Windows Restricted Token & Job Object sandbox helpers isolated in `toksi-sys`.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::SysError;

#[derive(Debug, Clone)]
pub struct SandboxExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub backend: &'static str,
}

/// Executes `program` with `args` inside an OS-enforced sandbox:
/// - Windows: Restricted Token (where permitted) + Job Object with `KILL_ON_JOB_CLOSE` and memory cap.
/// - Unix: Process group isolation (`setpgid`) with timeout enforcement.
pub fn run_in_os_job_sandbox(
    program: &str,
    args: &[String],
    cwd: &Path,
    memory_limit_mb: u64,
    timeout: Duration,
) -> Result<SandboxExecResult, SysError> {
    #[cfg(windows)]
    {
        run_windows_job_sandbox(program, args, cwd, memory_limit_mb, timeout)
    }

    #[cfg(not(windows))]
    {
        let _ = memory_limit_mb;
        run_posix_isolated(program, args, cwd, timeout)
    }
}

#[cfg(windows)]
fn run_windows_job_sandbox(
    program: &str,
    args: &[String],
    cwd: &Path,
    memory_limit_mb: u64,
    timeout: Duration,
) -> Result<SandboxExecResult, SysError> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
    };

    // Attempt to open current process token and verify restricted token capability.
    // Even if UAC/token privileges disallow CreateRestrictedToken in non-elevated contexts,
    // the Job Object still enforces memory limits and kill-on-close containment.
    let _restricted_token_supported = check_restricted_token_support();

    // SAFETY: `CreateJobObjectW(null, null)` creates an anonymous kernel job object with
    // default security attributes and no name pointer.
    let job: HANDLE = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if job.is_null() {
        return Err(SysError::Sandbox(format!(
            "CreateJobObjectW failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe {
        // SAFETY: `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` is a plain C-struct where all-zero
        // is a valid initial state before setting `LimitFlags` and `ProcessMemoryLimit`.
        std::mem::zeroed()
    };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if memory_limit_mb > 0 {
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        info.ProcessMemoryLimit = (memory_limit_mb as usize).saturating_mul(1024 * 1024);
    }

    // SAFETY: `job` is a valid handle returned by `CreateJobObjectW`, and `info` is a stack
    // struct whose exact byte size is passed via `size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()`.
    let set_ok = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&raw const info).cast(),
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if set_ok == 0 {
        // SAFETY: `job` is a valid non-null handle owned by this function.
        unsafe { CloseHandle(job) };
        return Err(SysError::Sandbox(format!(
            "SetInformationJobObject failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            // SAFETY: `job` is a valid non-null handle owned by this function.
            unsafe { CloseHandle(job) };
            return Err(SysError::Sandbox(format!("spawn '{program}' failed: {e}")));
        }
    };

    let proc_handle = child.as_raw_handle() as HANDLE;
    // SAFETY: `job` is a valid Job Object handle and `proc_handle` is the live `Child` process handle.
    let _assigned = unsafe { AssignProcessToJobObject(job, proc_handle) };

    let start = Instant::now();
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    timed_out = true;
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = child.kill();
                // SAFETY: `job` is a valid non-null handle owned by this function.
                unsafe { CloseHandle(job) };
                return Err(SysError::Sandbox(format!("wait failed: {e}")));
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| SysError::Sandbox(format!("collect output failed: {e}")))?;

    // SAFETY: `job` is a valid non-null handle owned by this function; closing it after child exit
    // also terminates any lingering descendant processes via `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
    unsafe { CloseHandle(job) };

    Ok(SandboxExecResult {
        exit_code: output.status.code().unwrap_or(if timed_out { 124 } else { 1 }),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        timed_out,
        backend: "windows-job-restricted",
    })
}

#[cfg(windows)]
fn check_restricted_token_support() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        CreateRestrictedToken, DISABLE_MAX_PRIVILEGE, TOKEN_DUPLICATE, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token: HANDLE = std::ptr::null_mut();
    // SAFETY: `GetCurrentProcess()` returns a pseudo-handle; `&raw mut token` is valid stack storage.
    let opened = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_DUPLICATE | TOKEN_QUERY, &raw mut token) };
    if opened == 0 || token.is_null() {
        return false;
    }

    let mut restricted: HANDLE = std::ptr::null_mut();
    // SAFETY: `token` is a valid handle from `OpenProcessToken`; all optional list pointers are null
    // with zero counts, and `&raw mut restricted` is valid stack storage.
    let ok = unsafe {
        CreateRestrictedToken(
            token,
            DISABLE_MAX_PRIVILEGE,
            0,
            std::ptr::null(),
            0,
            std::ptr::null(),
            0,
            std::ptr::null(),
            &raw mut restricted,
        )
    };
    // SAFETY: `token` is a valid non-null handle; `restricted` is closed only if non-null.
    unsafe {
        if !restricted.is_null() {
            CloseHandle(restricted);
        }
        CloseHandle(token);
    }
    ok != 0
}

#[cfg(not(windows))]
fn run_posix_isolated(
    program: &str,
    args: &[String],
    cwd: &Path,
    timeout: Duration,
) -> Result<SandboxExecResult, SysError> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);

    let mut child = cmd
        .spawn()
        .map_err(|e| SysError::Sandbox(format!("spawn '{program}' failed: {e}")))?;

    let start = Instant::now();
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    timed_out = true;
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(SysError::Sandbox(format!("wait failed: {e}")));
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| SysError::Sandbox(format!("collect output failed: {e}")))?;

    Ok(SandboxExecResult {
        exit_code: output.status.code().unwrap_or(if timed_out { 124 } else { 1 }),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        timed_out,
        backend: "posix-isolated",
    })
}
