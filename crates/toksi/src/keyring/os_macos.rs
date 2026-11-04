// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/os_macos.rs
// created: 2026-11-04
// lastModified: 2026-11-04

#[cfg(target_os = "macos")]
use std::process::Command;

pub fn read_keychain(service: &str, account: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("/usr/bin/security")
            .args(["find-generic-password", "-s", service, "-a", account, "-w"])
            .output()
            .ok()?;
        if out.status.success() {
            let secret = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !secret.is_empty() {
                return Some(secret);
            }
        }
        None
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account);
        None
    }
}

pub fn write_keychain(service: &str, account: &str, secret: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("/usr/bin/security")
            .args(["add-generic-password", "-s", service, "-a", account, "-w", secret, "-U"])
            .output();
        out.map(|o| o.status.success()).unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account, secret);
        false
    }
}

pub fn delete_keychain(service: &str, account: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("/usr/bin/security")
            .args(["delete-generic-password", "-s", service, "-a", account])
            .output();
        out.map(|o| o.status.success()).unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account);
        false
    }
}
