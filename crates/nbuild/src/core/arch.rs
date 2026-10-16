// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/arch.rs
// created: 2026-10-14
// lastModified: 2026-10-16

use anyhow::{bail, Result};
use crate::archinfo::CPUFeatures;

/// Canonical platform group name for filesystem paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformGroup {
    Windows,
    Linux,
    Apple,
    FreeBSD,
    Android,
    Other,
}

impl PlatformGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Apple => "apple",
            Self::FreeBSD => "freebsd",
            Self::Android => "android",
            Self::Other => "other",
        }
    }

    pub fn from_triple(triple: &str) -> Self {
        let lower = triple.to_lowercase();
        if lower.contains("windows") || lower.contains("win32") {
            Self::Windows
        } else if lower.contains("apple") || lower.contains("darwin") || lower.contains("ios") || lower.contains("tvos") || lower.contains("xros") || lower.contains("watchos") {
            Self::Apple
        } else if lower.contains("android") {
            Self::Android
        } else if lower.contains("freebsd") {
            Self::FreeBSD
        } else if lower.contains("linux") {
            Self::Linux
        } else {
            Self::Other
        }
    }
}

/// Resolved Architecture & Target Triple representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetTriple {
    pub raw: String,
    pub platform: PlatformGroup,
    pub arch: String,
    pub vendor: String,
    pub os: String,
    pub env: Option<String>,
}

impl TargetTriple {
    pub fn parse(triple: &str) -> Result<Self> {
        let parts: Vec<&str> = triple.split('-').collect();
        if parts.is_empty() {
            bail!("Empty target triple string");
        }

        let arch = parts[0].to_string();
        let vendor = if parts.len() > 1 { parts[1].to_string() } else { "unknown".to_string() };
        let os = if parts.len() > 2 { parts[2].to_string() } else { "none".to_string() };
        let env = if parts.len() > 3 { Some(parts[3..].join("-")) } else { None };

        let platform = PlatformGroup::from_triple(triple);

        Ok(Self {
            raw: triple.to_string(),
            platform,
            arch,
            vendor,
            os,
            env,
        })
    }

    /// Detect the current host target triple
    pub fn detect_host() -> Self {
        let host_triple = if cfg!(target_os = "windows") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-pc-windows-msvc"
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-pc-windows-msvc"
            } else {
                "i686-pc-windows-msvc"
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "x86_64") {
                "x86_64-unknown-linux-gnu"
            } else if cfg!(target_arch = "aarch64") {
                "aarch64-unknown-linux-gnu"
            } else {
                "x86_64-unknown-linux-gnu"
            }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else {
                "x86_64-apple-darwin"
            }
        } else if cfg!(target_os = "freebsd") {
            "x86_64-unknown-freebsd"
        } else {
            "x86_64-unknown-linux-gnu"
        };

        Self::parse(host_triple).expect("Host triple parse must not fail")
    }

    /// Check if this target triple can be executed directly on the host machine
    pub fn is_host_compatible(&self) -> bool {
        let host = Self::detect_host();
        if self.platform != host.platform {
            return false;
        }

        // On Windows x86_64 host, msvc and gnu can both run
        if host.platform == PlatformGroup::Windows {
            return self.arch == host.arch || (host.arch == "x86_64" && self.arch == "i686");
        }

        // On Apple Silicon (aarch64), Rosetta 2 can run x86_64, but native is aarch64
        if host.platform == PlatformGroup::Apple {
            return self.arch == host.arch || (host.arch == "aarch64" && self.arch == "x86_64");
        }

        self.arch == host.arch
    }

    /// Detect host CPU features using archinfo
    pub fn detect_host_cpu_features() -> CPUFeatures {
        CPUFeatures::detect_host()
    }
}
