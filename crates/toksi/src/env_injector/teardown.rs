// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/env_injector/teardown.rs
// created: 2026-10-31
// lastModified: 2026-11-09

use std::sync::atomic::{AtomicBool, Ordering};

static HANDLERS_REGISTERED: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
unsafe extern "system" fn win_ctrl_handler(_ctrl_type: u32) -> i32 {
    let _ = teardown_proxy_env();
    0 // FALSE: allow standard termination
}

#[cfg(unix)]
unsafe extern "C" fn unix_signal_handler(sig: libc::c_int) {
    let _ = teardown_proxy_env();
    libc::signal(sig, libc::SIG_DFL);
    libc::raise(sig);
}

pub fn register_teardown_handlers() {
    if HANDLERS_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        SetConsoleCtrlHandler(Some(win_ctrl_handler), 1);
    }

    #[cfg(target_os = "linux")]
    unsafe {
        libc::signal(libc::SIGINT, unix_signal_handler as libc::sighandler_t);
        libc::signal(libc::SIGTERM, unix_signal_handler as libc::sighandler_t);
        libc::signal(libc::SIGHUP, unix_signal_handler as libc::sighandler_t);
    }

    #[cfg(target_os = "macos")]
    unsafe {
        libc::signal(libc::SIGINT, unix_signal_handler as libc::sig_t);
        libc::signal(libc::SIGTERM, unix_signal_handler as libc::sig_t);
        libc::signal(libc::SIGHUP, unix_signal_handler as libc::sig_t);
    }

    #[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
    unsafe {
        libc::signal(libc::SIGINT, unix_signal_handler as usize);
        libc::signal(libc::SIGTERM, unix_signal_handler as usize);
        libc::signal(libc::SIGHUP, unix_signal_handler as usize);
    }
}

pub fn inject_proxy_env() -> Result<Vec<String>, String> {
    let url = crate::util::headroom_proxy::headroom_proxy_url();
    register_teardown_handlers();

    #[cfg(windows)]
    {
        super::windows::inject_windows_env(&url)
    }

    #[cfg(not(windows))]
    {
        super::unix::inject_unix_env(&url)
    }
}

pub fn teardown_proxy_env() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    {
        super::windows::teardown_windows_env()
    }

    #[cfg(not(windows))]
    {
        super::unix::teardown_unix_env()
    }
}

pub struct TeardownGuard {
    active: bool,
}

impl TeardownGuard {
    pub fn new() -> Self {
        Self { active: true }
    }

    pub fn disarm(&mut self) {
        self.active = false;
    }
}

impl Default for TeardownGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TeardownGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = teardown_proxy_env();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teardown_guard_lifecycle() {
        let mut guard = TeardownGuard::new();
        assert!(guard.active);
        guard.disarm();
        assert!(!guard.active);
    }

    #[test]
    fn test_register_handlers_idempotent() {
        register_teardown_handlers();
        register_teardown_handlers();
    }

    #[test]
    fn test_inject_and_teardown_proxy_env() {
        let Ok(_lock) = crate::env_injector::TEST_ENV_LOCK.lock() else {
            return;
        };
        let inject_res = inject_proxy_env();
        if let Ok(injected) = inject_res {
            assert!(injected.contains(&"GOOGLE_GEMINI_BASE_URL".to_string()));
            let tear_res = teardown_proxy_env();
            assert!(tear_res.is_ok());
        }
    }
}
