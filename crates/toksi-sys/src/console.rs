// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi-sys/src/console.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Terminal console mode, codepage, and POSIX SIGTSTP helpers isolated in `toksi-sys`.

use std::sync::atomic::AtomicU32;

static SAVED_OUTPUT_CP: AtomicU32 = AtomicU32::new(0);

#[cfg(windows)]
mod platform {
    use std::os::windows::io::AsRawHandle;
    use std::sync::atomic::Ordering;

    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Console::{
        ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT,
        ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT,
        GetConsoleMode, GetConsoleOutputCP, SetConsoleMode, SetConsoleOutputCP,
    };

    use super::SAVED_OUTPUT_CP;

    fn get_console_mode(handle: HANDLE) -> Option<u32> {
        let mut mode: u32 = 0;
        // SAFETY: `handle` is obtained from `std::io::{stdin,stdout,stderr}().as_raw_handle()`
        // and `mode` is a valid, aligned stack `u32` pointer for the synchronous call.
        let ok = unsafe { GetConsoleMode(handle, &mut mode) };
        if ok != 0 { Some(mode) } else { None }
    }

    fn set_console_mode(handle: HANDLE, mode: u32) -> bool {
        // SAFETY: `handle` is a valid standard stream console handle or was just queried via
        // `GetConsoleMode`, and `mode` is a bitmask of Win32 console mode flags.
        unsafe { SetConsoleMode(handle, mode) != 0 }
    }

    pub fn restore_console_cp() {
        let saved = SAVED_OUTPUT_CP.load(Ordering::Relaxed);
        if saved != 0 && saved != 65001 {
            // SAFETY: `SetConsoleOutputCP` takes a plain `u32` code page ID with no pointers.
            unsafe {
                let _ = SetConsoleOutputCP(saved);
            }
        }
    }

    pub fn enable_vt() -> bool {
        // SAFETY: `GetConsoleOutputCP` and `SetConsoleOutputCP` take no pointers and have no
        // memory preconditions; UTF-8 code page `65001` is a standard Win32 console code page.
        unsafe {
            let cp = GetConsoleOutputCP();
            if cp != 0 {
                SAVED_OUTPUT_CP.store(cp, Ordering::Relaxed);
            }
            let _ = SetConsoleOutputCP(65001);
        }
        let mut ok = true;
        let stdout_h = std::io::stdout().as_raw_handle() as HANDLE;
        let stderr_h = std::io::stderr().as_raw_handle() as HANDLE;
        for h in [stdout_h, stderr_h] {
            if let Some(mode) = get_console_mode(h)
                && mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0
                && !set_console_mode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
            {
                ok = false;
            }
        }
        ok
    }

    pub fn raw_mode() -> Option<Box<dyn FnOnce()>> {
        let stdin_h = std::io::stdin().as_raw_handle() as HANDLE;
        let saved = get_console_mode(stdin_h)?;
        let mut raw = saved
            & !(ENABLE_ECHO_INPUT
                | ENABLE_LINE_INPUT
                | ENABLE_PROCESSED_INPUT
                | ENABLE_WINDOW_INPUT
                | ENABLE_MOUSE_INPUT);
        raw |= ENABLE_VIRTUAL_TERMINAL_INPUT;
        if !set_console_mode(stdin_h, raw) {
            return None;
        }
        let handle_addr = stdin_h as usize;
        Some(Box::new(move || {
            let _ = set_console_mode(handle_addr as HANDLE, saved);
        }))
    }

    pub fn send_sigtstp_self() {}
}

#[cfg(not(windows))]
mod platform {
    use std::process::{Command, Stdio};

    pub fn restore_console_cp() {}

    pub fn enable_vt() -> bool {
        true
    }

    pub fn raw_mode() -> Option<Box<dyn FnOnce()>> {
        let saved_out = Command::new("stty")
            .args(["-F", "/dev/tty", "-g"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .or_else(|| {
                Command::new("stty")
                    .arg("-g")
                    .stdin(Stdio::inherit())
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
            })?;
        let saved = String::from_utf8_lossy(&saved_out.stdout).trim().to_string();
        if saved.is_empty() {
            return None;
        }

        let set_ok = Command::new("stty")
            .args(["-F", "/dev/tty", "raw", "-echo"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("stty")
                .args(["raw", "-echo"])
                .stdin(Stdio::inherit())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if !set_ok {
            return None;
        }

        Some(Box::new(move || {
            let ok = Command::new("stty")
                .args(["-F", "/dev/tty", &saved])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok {
                let _ = Command::new("stty")
                    .arg(&saved)
                    .stdin(Stdio::inherit())
                    .status();
            }
        }))
    }

    pub fn send_sigtstp_self() {
        // SAFETY: `libc::getpid()` returns the current process ID and `libc::kill` with
        // `SIGTSTP` sends a standard POSIX job-control stop signal to self with no pointer args.
        unsafe {
            libc::kill(libc::getpid(), libc::SIGTSTP);
        }
    }
}

pub fn restore_console_cp() {
    platform::restore_console_cp();
}

pub fn enable_vt() -> bool {
    platform::enable_vt()
}

pub fn raw_mode() -> Option<Box<dyn FnOnce()>> {
    platform::raw_mode()
}

pub fn send_sigtstp_self() {
    platform::send_sigtstp_self();
}
