// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/env_ops.rs
// created: 2026-10-16

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};

use super::config::{NeoConfig, DEFAULT_UNIX_TOOLCHAINS_DIR, DEFAULT_WINDOWS_TOOLCHAINS_DIR};

pub struct EnvManager;

impl EnvManager {
    /// Check if TOOLCHAINS_DIR is present in environment or user profile.
    pub fn get_toolchains_dir_env() -> Option<PathBuf> {
        // 1. Current process env
        if let Ok(val) = env::var("TOOLCHAINS_DIR") {
            if !val.trim().is_empty() {
                return Some(PathBuf::from(val.trim()));
            }
        }

        // 2. Windows User Environment registry check via PowerShell
        #[cfg(windows)]
        {
            let output = Command::new("powershell")
                .args(["-NoProfile", "-Command", "[Environment]::GetEnvironmentVariable('TOOLCHAINS_DIR', 'User')"])
                .output();
            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(PathBuf::from(s));
                }
            }
        }

        None
    }

    /// Check if $TOOLCHAINS_DIR/usr/bin is in PATH.
    pub fn is_usr_bin_in_path(config: &NeoConfig) -> bool {
        let expected = config.host_usr_bin();
        let expected_str = expected.to_string_lossy().to_lowercase();

        if let Ok(path_var) = env::var("PATH") {
            for entry in env::split_paths(&path_var) {
                if entry.to_string_lossy().to_lowercase() == expected_str {
                    return true;
                }
            }
        }

        #[cfg(windows)]
        {
            let output = Command::new("powershell")
                .args(["-NoProfile", "-Command", "[Environment]::GetEnvironmentVariable('Path', 'User')"])
                .output();
            if let Ok(out) = output {
                let user_path = String::from_utf8_lossy(&out.stdout).to_lowercase();
                if user_path.contains(&expected_str) {
                    return true;
                }
            }
        }

        false
    }

    /// Persist TOOLCHAINS_DIR to user environment.
    pub fn persist_toolchains_dir<P: AsRef<Path>>(dir: P) -> Result<()> {
        let dir_str = dir.as_ref().to_string_lossy().to_string();

        // Update current process env
        unsafe {
            env::set_var("TOOLCHAINS_DIR", &dir_str);
        }

        #[cfg(windows)]
        {
            let ps_cmd = format!(
                "[Environment]::SetEnvironmentVariable('TOOLCHAINS_DIR', '{}', 'User')",
                dir_str.replace('\'', "''")
            );
            let status = Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_cmd])
                .status()
                .with_context(|| "Failed to execute powershell to set TOOLCHAINS_DIR")?;

            if !status.success() {
                anyhow::bail!("Failed to persist TOOLCHAINS_DIR in Windows user environment");
            }
        }

        #[cfg(unix)]
        {
            let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            let bashrc = PathBuf::from(home).join(".bashrc");
            let line = format!("\nexport TOOLCHAINS_DIR=\"{}\"\n", dir_str);
            let mut content = fs::read_to_string(&bashrc).unwrap_or_default();
            if !content.contains("TOOLCHAINS_DIR") {
                content.push_str(&line);
                let _ = fs::write(&bashrc, content);
            }
        }

        Ok(())
    }

    /// Ensure $TOOLCHAINS_DIR/usr/bin is in PATH. Heals user PATH if missing.
    pub fn heal_path_env(config: &NeoConfig) -> Result<bool> {
        let usr_bin = config.host_usr_bin();

        // 1. Ensure directory hierarchy exists
        fs::create_dir_all(&usr_bin)?;
        fs::create_dir_all(config.host_usr_dir().join("include"))?;
        fs::create_dir_all(config.host_usr_dir().join("lib"))?;
        fs::create_dir_all(config.host_usr_dir().join("share"))?;

        if Self::is_usr_bin_in_path(config) {
            return Ok(false); // Already present
        }

        let usr_bin_str = usr_bin.to_string_lossy().to_string();

        // Update current process env
        if let Ok(current_path) = env::var("PATH") {
            let mut paths = env::split_paths(&current_path).collect::<Vec<_>>();
            paths.insert(0, usr_bin.clone());
            if let Ok(new_path) = env::join_paths(paths) {
                unsafe {
                    env::set_var("PATH", new_path);
                }
            }
        }

        #[cfg(windows)]
        {
            let ps_cmd = format!(
                "$p = [Environment]::GetEnvironmentVariable('Path', 'User'); \
                 if ($p -notlike '*{}*') {{ \
                     [Environment]::SetEnvironmentVariable('Path', \"{};$p\", 'User') \
                 }}",
                usr_bin_str.replace('\'', "''"),
                usr_bin_str.replace('\'', "''")
            );
            let status = Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_cmd])
                .status()
                .with_context(|| "Failed to execute powershell to update user PATH")?;

            if !status.success() {
                anyhow::bail!("Failed to update user PATH in Windows environment");
            }
        }

        #[cfg(unix)]
        {
            let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            let bashrc = PathBuf::from(home).join(".bashrc");
            let line = format!("\nexport PATH=\"{}:$PATH\"\n", usr_bin_str);
            let mut content = fs::read_to_string(&bashrc).unwrap_or_default();
            if !content.contains(&usr_bin_str) {
                content.push_str(&line);
                let _ = fs::write(&bashrc, content);
            }
        }

        Ok(true) // Healed
    }

    /// Auto-discover or interactively configure TOOLCHAINS_DIR and PATH.
    pub fn ensure_environment(interactive: bool) -> Result<NeoConfig> {
        let existing = Self::get_toolchains_dir_env();

        let toolchains_dir = match existing {
            Some(d) => d,
            None => {
                let default_dir = if cfg!(windows) {
                    PathBuf::from(DEFAULT_WINDOWS_TOOLCHAINS_DIR)
                } else {
                    PathBuf::from(DEFAULT_UNIX_TOOLCHAINS_DIR)
                };

                let selected = if interactive {
                    println!("[nbuild] TOOLCHAINS_DIR environment variable not found.");
                    println!("[nbuild] Default suggested path: {}", default_dir.display());
                    print!("[nbuild] Press Enter to accept or type custom directory: ");
                    use std::io::{stdout, stdin, Write};
                    let _ = stdout().flush();
                    let mut input = String::new();
                    let _ = stdin().read_line(&mut input);
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        default_dir
                    } else {
                        PathBuf::from(trimmed)
                    }
                } else {
                    default_dir
                };

                println!("[nbuild] Setting TOOLCHAINS_DIR to: {}", selected.display());
                Self::persist_toolchains_dir(&selected)?;
                selected
            }
        };

        let config = NeoConfig::new().with_toolchains_dir(toolchains_dir);
        let healed = Self::heal_path_env(&config)?;
        if healed {
            println!("[nbuild] Added {} to user PATH and created directory hierarchy.", config.host_usr_bin().display());
        }

        Ok(config)
    }
}
