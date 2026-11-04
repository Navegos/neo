// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/os_linux.rs
// created: 2026-11-04
// lastModified: 2026-11-04

#[cfg(target_os = "linux")]
use std::io::Write;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

pub fn read_secret_service(service: &str, account: &str) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let out = Command::new("secret-tool")
            .args(["lookup", "service", service, "account", account])
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
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (service, account);
        None
    }
}

pub fn write_secret_service(service: &str, account: &str, secret: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        let label = format!("{}:{}", service, account);
        let mut child = match Command::new("secret-tool")
            .args(["store", "--label", &label, "service", service, "account", account])
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(secret.as_bytes());
        }
        child.wait().map(|s| s.success()).unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (service, account, secret);
        false
    }
}

pub fn delete_secret_service(service: &str, account: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        let out = Command::new("secret-tool")
            .args(["clear", "service", service, "account", account])
            .output();
        out.map(|o| o.status.success()).unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (service, account);
        false
    }
}
