// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/routing_lock.rs
// created: 2026-10-05
// lastModified: 2026-11-18

#![deny(unsafe_code)]

use std::fs::File;
use std::path::Path;
use fs2::FileExt;

pub struct FileLockGuard {
    file: File,
    path: std::path::PathBuf,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
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
        .truncate(false)
        .open(path)?;

    if file.try_lock_exclusive().is_err() {
        return Err("routing store lock failed".into());
    }
    let _ = std::fs::write(&pid_path, std::process::id().to_string());
    Ok(FileLockGuard { file, path: path.to_path_buf() })
}

