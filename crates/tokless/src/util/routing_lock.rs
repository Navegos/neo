// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/routing_lock.rs
// created: 2026-10-05
// lastModified: 2026-10-20

use std::fs::File;
use std::path::Path;

pub struct FileLockGuard {
    _file: File,
    path: std::path::PathBuf,
    #[cfg(windows)]
    handle: isize,
}

#[cfg(windows)]
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        unsafe {
            use windows_sys::Win32::Storage::FileSystem::UnlockFile;
            UnlockFile(self.handle as _, 0, 0, 1, 0);
        }
        let pid_path = self.path.with_extension("pid");
        let _ = std::fs::remove_file(pid_path);
    }
}

#[cfg(unix)]
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        unsafe {
            libc::flock(self._file.as_raw_fd(), libc::LOCK_UN);
        }
        let pid_path = self.path.with_extension("pid");
        let _ = std::fs::remove_file(pid_path);
    }
}

pub fn acquire_routing_file_lock(path: impl AsRef<Path>) -> Result<FileLockGuard, Box<dyn std::error::Error + Send + Sync>> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pid_path = path.with_extension("pid");

    // Fast-path: check if existing lock owner PID is dead and clean up stale lock
    if let Ok(content) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            if pid > 0 && pid != std::process::id() && !crate::agents::proxy_provider::proxy_process_alive(pid) {
                let _ = std::fs::remove_file(&pid_path);
                let _ = std::fs::remove_file(path);
            }
        }
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::LockFile;

        let handle = file.as_raw_handle() as isize;
        unsafe {
            let ok = LockFile(handle as _, 0, 0, 1, 0);
            if ok == 0 {
                return Err("routing store lock failed".into());
            }
        }
        let _ = std::fs::write(&pid_path, std::process::id().to_string());
        Ok(FileLockGuard { _file: file, path: path.to_path_buf(), handle })
    }

    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let res = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if res != 0 {
            return Err("routing store lock failed".into());
        }
        let _ = std::fs::write(&pid_path, std::process::id().to_string());
        Ok(FileLockGuard { _file: file, path: path.to_path_buf() })
    }
}

