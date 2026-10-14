// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/process_identity.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

use super::proxy::ProxyOwnership;
use crate::util::paths::IS_WIN;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProcessIdentityInfo {
    pub executable: String,
    pub args: Vec<String>,
    pub start: String,
}

impl ProcessIdentityInfo {
    pub fn matches(&self, executable: &str, args: &[String]) -> bool {
        let Ok(want) = normalized_executable(executable) else {
            return false;
        };
        let self_exe = normalized_executable(&self.executable).unwrap_or_else(|_| self.executable.clone());
        if self_exe == want && self.args == args {
            return true;
        }
        if self.args.is_empty() || self.args[1..] != *args {
            return false;
        }
        if let Ok(launcher) = normalized_executable(&self.args[0]) {
            launcher == want
        } else {
            false
        }
    }

    pub fn matches_record(&self, r: &ProxyOwnership) -> bool {
        let self_exe = normalized_executable(&self.executable).unwrap_or_else(|_| self.executable.clone());
        let rec_exe = normalized_executable(&r.executable).unwrap_or_else(|_| r.executable.clone());
        self.start == r.start_fingerprint && self_exe == rec_exe && self.args == r.args
    }

    pub fn equal(&self, other: &ProcessIdentityInfo) -> bool {
        let self_exe = normalized_executable(&self.executable).unwrap_or_else(|_| self.executable.clone());
        let other_exe = normalized_executable(&other.executable).unwrap_or_else(|_| other.executable.clone());
        self.start == other.start && self_exe == other_exe && self.args == other.args
    }
}

pub fn normalized_executable(path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if path.trim().is_empty() {
        return Err("empty executable path".into());
    }
    let p = Path::new(path);
    let abs: PathBuf = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()?.join(p)
    };
    let resolved = std::fs::canonicalize(&abs).unwrap_or(abs);
    let s = resolved.to_string_lossy().to_string();
    if IS_WIN {
        let stripped = s.strip_prefix(r"\\?\").unwrap_or(&s);
        Ok(stripped.to_string())
    } else {
        Ok(s)
    }
}

pub fn process_identity_supported() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))
}

#[derive(Deserialize)]
struct WindowsProcess {
    #[serde(rename = "ExecutablePath", default)]
    executable: Option<String>,
    #[serde(rename = "CommandLine", default)]
    command: Option<String>,
    #[serde(rename = "CreationDate", default)]
    created: Option<String>,
}

pub fn identify_process(
    pid: u32,
) -> Result<ProcessIdentityInfo, Box<dyn std::error::Error + Send + Sync>> {
    if pid == 0 {
        return Err("invalid pid 0".into());
    }
    #[cfg(target_os = "windows")]
    {
        return identify_process_windows(pid);
    }
    #[cfg(target_os = "linux")]
    {
        return identify_process_linux(pid);
    }
    #[cfg(target_os = "macos")]
    {
        return identify_process_darwin(pid);
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("process identity unavailable".into())
    }
}

#[cfg(target_os = "windows")]
fn identify_process_windows(
    pid: u32,
) -> Result<ProcessIdentityInfo, Box<dyn std::error::Error + Send + Sync>> {
    let filter = format!("ProcessId = {}", pid);
    let script = format!(
        "$p = Get-CimInstance Win32_Process -Filter '{}'; if ($null -eq $p) {{ exit 1 }}; $p | Select-Object ExecutablePath,CommandLine,CreationDate | ConvertTo-Json -Compress",
        filter
    );
    let mut out_bytes: Option<Vec<u8>> = None;
    let mut last_err: Option<String> = None;

    for shell in ["powershell", "pwsh"] {
        if crate::util::paths::which(shell).is_empty() {
            continue;
        }
        let mut cmd = Command::new(shell);
        cmd.args(["-NoProfile", "-Command", &script]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        match cmd.output() {
            Ok(output) if output.status.success() => {
                out_bytes = Some(output.stdout);
                break;
            }
            Ok(output) => {
                last_err = Some(String::from_utf8_lossy(&output.stderr).trim().to_string());
            }
            Err(e) => {
                last_err = Some(e.to_string());
            }
        }
    }

    let Some(out) = out_bytes else {
        return Err(last_err
            .unwrap_or_else(|| "failed to query process identity".to_string())
            .into());
    };

    let p: WindowsProcess =
        serde_json::from_slice(&out).map_err(|_| "incomplete process identity")?;
    let exe_raw = p.executable.unwrap_or_default();
    let cmd_raw = p.command.unwrap_or_default();
    let created = p.created.unwrap_or_default();
    if exe_raw.is_empty() || cmd_raw.is_empty() || created.is_empty() {
        return Err("incomplete process identity".into());
    }
    let executable = normalized_executable(&exe_raw)?;
    let args = windows_command_line_args(&cmd_raw);
    if args.is_empty() {
        return Err("incomplete process identity".into());
    }
    Ok(ProcessIdentityInfo {
        executable,
        args: args[1..].to_vec(),
        start: created,
    })
}

#[cfg(target_os = "linux")]
fn identify_process_linux(
    pid: u32,
) -> Result<ProcessIdentityInfo, Box<dyn std::error::Error + Send + Sync>> {
    let base = PathBuf::from(format!("/proc/{}", pid));
    let exe_link = std::fs::read_link(base.join("exe"))?;
    let exe_str = exe_link.to_string_lossy();
    let exe_trimmed = exe_str.strip_suffix(" (deleted)").unwrap_or(&exe_str);
    let executable = normalized_executable(exe_trimmed)?;

    let cmdline_bytes = std::fs::read(base.join("cmdline"))?;
    let cmdline_str = String::from_utf8_lossy(&cmdline_bytes);
    let trimmed = cmdline_str.trim_end_matches('\0');
    let parts: Vec<&str> = trimmed.split('\0').collect();
    if parts.len() < 2 || parts[0].is_empty() {
        return Err("incomplete process identity".into());
    }

    let stat_bytes = std::fs::read(base.join("stat"))?;
    let stat_str = String::from_utf8_lossy(&stat_bytes);
    let Some(close) = stat_str.rfind(')') else {
        return Err("incomplete process start identity".into());
    };
    if close + 2 > stat_str.len() {
        return Err("incomplete process start identity".into());
    }
    let fields: Vec<&str> = stat_str[close + 2..].split_whitespace().collect();
    if fields.len() <= 19 {
        return Err("incomplete process start identity".into());
    }
    let _start_ticks: u64 = fields[19].parse()?;
    Ok(ProcessIdentityInfo {
        executable,
        args: parts[1..].iter().map(|s| s.to_string()).collect(),
        start: fields[19].to_string(),
    })
}

#[cfg(target_os = "macos")]
fn identify_process_darwin(
    pid: u32,
) -> Result<ProcessIdentityInfo, Box<dyn std::error::Error + Send + Sync>> {
    let start_out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "lstart="])
        .output()?;
    if !start_out.status.success() {
        return Err("ps failed for process identity".into());
    }
    let start = String::from_utf8_lossy(&start_out.stdout).trim().to_string();
    if start.is_empty() {
        return Err("incomplete process identity".into());
    }

    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as i32];
    let mut size: usize = 0;
    let ret = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            3,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 || size == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut buf = vec![0u8; size];
    let ret2 = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            3,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret2 != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    buf.truncate(size);
    let Some(argv) = darwin_proc_args(&buf) else {
        return Err("incomplete process identity".into());
    };
    if argv.len() < 2 {
        return Err("incomplete process identity".into());
    }
    let executable = std::fs::canonicalize(&argv[0])?
        .to_string_lossy()
        .to_string();
    Ok(ProcessIdentityInfo {
        executable,
        args: argv[1..].to_vec(),
        start,
    })
}

pub fn darwin_proc_args(data: &[u8]) -> Option<Vec<String>> {
    if data.len() < 4 {
        return None;
    }
    let argc = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if argc < 1 || argc > (1 << 16) || argc > data.len() - 4 {
        return None;
    }
    let mut rem = &data[4..];
    let read_next = |rem: &mut &[u8]| -> String {
        if rem.is_empty() {
            return String::new();
        }
        let i = rem.iter().position(|&b| b == 0).unwrap_or(rem.len());
        let v = String::from_utf8_lossy(&rem[..i]).to_string();
        if i < rem.len() {
            *rem = &rem[i + 1..];
        } else {
            *rem = &[];
        }
        v
    };
    let mut args = Vec::with_capacity(argc);
    let executable = read_next(&mut rem);
    if !executable.is_empty() {
        args.push(executable);
    }
    while args.len() < argc {
        if rem.is_empty() {
            return None;
        }
        if rem[0] == 0 {
            rem = &rem[1..];
            continue;
        }
        args.push(read_next(&mut rem));
    }
    Some(args)
}

#[cfg(windows)]
#[link(name = "shell32")]
unsafe extern "system" {
    fn CommandLineToArgvW(lpCmdLine: *const u16, pNumArgs: *mut i32) -> *mut *mut u16;
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn OpenProcess(
        dwDesiredAccess: u32,
        bInheritHandle: i32,
        dwProcessId: u32,
    ) -> *mut std::ffi::c_void;
    fn GetExitCodeProcess(hProcess: *mut std::ffi::c_void, lpExitCode: *mut u32) -> i32;
    fn TerminateProcess(hProcess: *mut std::ffi::c_void, uExitCode: u32) -> i32;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
}

pub fn windows_command_line_args(command: &str) -> Vec<String> {
    if command.trim().is_empty() {
        return Vec::new();
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        let wide: Vec<u16> = std::ffi::OsStr::new(command)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut argc: i32 = 0;
        let argv = unsafe { CommandLineToArgvW(wide.as_ptr(), &mut argc) };
        if argv.is_null() || argc <= 0 {
            return Vec::new();
        }
        let mut args = Vec::with_capacity(argc as usize);
        for i in 0..(argc as usize) {
            let p = unsafe { *argv.add(i) };
            if p.is_null() {
                unsafe {
                    LocalFree(argv as *mut std::ffi::c_void);
                }
                return Vec::new();
            }
            let mut len = 0usize;
            while len < 32768 && unsafe { *p.add(len) } != 0 {
                len += 1;
            }
            let slice = unsafe { std::slice::from_raw_parts(p, len) };
            args.push(std::ffi::OsString::from_wide(slice).to_string_lossy().to_string());
        }
        unsafe {
            LocalFree(argv as *mut std::ffi::c_void);
        }
        return args;
    }
    #[cfg(not(windows))]
    {
        parse_windows_cmd_fallback(command)
    }
}

#[cfg(not(windows))]
fn parse_windows_cmd_fallback(command: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = command.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' {
            chars.next();
        } else {
            break;
        }
    }

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let mut backslashes = 1;
                while let Some(&'\\') = chars.peek() {
                    chars.next();
                    backslashes += 1;
                }
                if let Some(&'"') = chars.peek() {
                    chars.next();
                    for _ in 0..(backslashes / 2) {
                        cur.push('\\');
                    }
                    if backslashes % 2 == 0 {
                        in_quotes = !in_quotes;
                    } else {
                        cur.push('"');
                    }
                } else {
                    for _ in 0..backslashes {
                        cur.push('\\');
                    }
                }
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                args.push(std::mem::take(&mut cur));
                while let Some(&next_c) = chars.peek() {
                    if next_c == ' ' || next_c == '\t' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() || in_quotes {
        args.push(cur);
    }
    args
}

pub fn kill_process(pid: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if pid == 0 {
        return Err("invalid pid 0".into());
    }
    #[cfg(windows)]
    {
        const PROCESS_TERMINATE: u32 = 0x0001;
        let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
        if !handle.is_null() {
            let ok = unsafe { TerminateProcess(handle, 1) };
            unsafe {
                CloseHandle(handle);
            }
            if ok != 0 {
                return Ok(());
            }
        }
        let status = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()?;
        if status.status.success() || process_gone(pid) {
            return Ok(());
        }
        return Err(format!(
            "failed to terminate pid {}: {}",
            pid,
            String::from_utf8_lossy(&status.stderr).trim()
        )
        .into());
    }
    #[cfg(unix)]
    {
        let ret = unsafe { libc::kill(pid as i32, libc::SIGKILL) };
        if ret == 0 {
            return Ok(());
        }
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            return Ok(());
        }
        return Err(err.into());
    }
    #[cfg(not(any(windows, unix)))]
    {
        Err(format!("process identity unavailable for pid {}", pid).into())
    }
}

pub fn process_gone(pid: u32) -> bool {
    if pid == 0 {
        return true;
    }
    #[cfg(windows)]
    {
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
        const STILL_ACTIVE: u32 = 259;
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            return std::io::Error::last_os_error().raw_os_error() == Some(87);
        }
        let mut code: u32 = 0;
        let ret = unsafe { GetExitCodeProcess(handle, &mut code) };
        unsafe {
            CloseHandle(handle);
        }
        return ret != 0 && code != STILL_ACTIVE;
    }
    #[cfg(target_os = "linux")]
    {
        return !Path::new(&format!("/proc/{}", pid)).exists();
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let ret = unsafe { libc::kill(pid as i32, 0) };
        return ret != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
    }
    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}

pub fn spawn_detached(cmd: &mut Command) -> std::io::Result<std::process::Child> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
        cmd.creation_flags(0x08000000 | 0x00000008 | 0x00000200);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.spawn()
}
