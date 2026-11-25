// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/ipc/transport.rs
// created: 2026-11-24

//! Cross-platform IPC endpoint resolution (Windows Named Pipes & POSIX Unix Domain Sockets)
//! and canonical `WorkerRole` taxonomy.

#![deny(unsafe_code)]

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::util::paths::{normalize_path, toksi_data_dir};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerRole {
    NetworkManager,
    Port8787,
    Port8788,
    Port8789,
    Port8790,
    Port9098,
    Port9099,
    Port9100,
    Port18787,
    Siagent,
    Codegraph,
    Ctxmode,
    Projectmem,
    Icm,
    Aimemory,
    Rtk,
    Security,
}

impl WorkerRole {
    pub const ALL: &'static [WorkerRole] = &[
        Self::NetworkManager,
        Self::Port8787,
        Self::Port8788,
        Self::Port8789,
        Self::Port8790,
        Self::Port9098,
        Self::Port9099,
        Self::Port9100,
        Self::Port18787,
        Self::Siagent,
        Self::Codegraph,
        Self::Ctxmode,
        Self::Projectmem,
        Self::Icm,
        Self::Aimemory,
        Self::Rtk,
        Self::Security,
    ];

    pub const PORT_WORKERS: &'static [WorkerRole] = &[
        Self::Port8787,
        Self::Port8788,
        Self::Port8789,
        Self::Port8790,
        Self::Port9098,
        Self::Port9099,
        Self::Port9100,
        Self::Port18787,
    ];

    pub const TOOL_WORKERS: &'static [WorkerRole] = &[
        Self::Codegraph,
        Self::Ctxmode,
        Self::Projectmem,
        Self::Icm,
        Self::Aimemory,
        Self::Rtk,
        Self::Security,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NetworkManager => "network-manager",
            Self::Port8787 => "port-8787",
            Self::Port8788 => "port-8788",
            Self::Port8789 => "port-8789",
            Self::Port8790 => "port-8790",
            Self::Port9098 => "port-9098",
            Self::Port9099 => "port-9099",
            Self::Port9100 => "port-9100",
            Self::Port18787 => "port-18787",
            Self::Siagent => "siagent",
            Self::Codegraph => "codegraph",
            Self::Ctxmode => "ctxmode",
            Self::Projectmem => "projectmem",
            Self::Icm => "icm",
            Self::Aimemory => "aimemory",
            Self::Rtk => "rtk",
            Self::Security => "security",
        }
    }

    pub fn default_port(&self) -> Option<u16> {
        match self {
            Self::NetworkManager => None,
            Self::Port8787 => Some(8787),
            Self::Port8788 => Some(8788),
            Self::Port8789 => Some(8789),
            Self::Port8790 => Some(8790),
            Self::Port9098 => Some(9098),
            Self::Port9099 => Some(9099),
            Self::Port9100 => Some(9100),
            Self::Port18787 => Some(18787),
            _ => None,
        }
    }

    pub fn ipc_endpoint(&self) -> String {
        ipc_endpoint_for_role(self.as_str())
    }
}

impl fmt::Display for WorkerRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WorkerRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "network-manager" | "network-service" | "netmgr" => Ok(Self::NetworkManager),
            "port-8787" | "headroom" | "headroom-proxy" => Ok(Self::Port8787),
            "port-8788" | "grok" | "grok-oauth" => Ok(Self::Port8788),
            "port-8789" | "copilot-vscode" => Ok(Self::Port8789),
            "port-8790" | "copilot-cli" => Ok(Self::Port8790),
            "port-9098" | "protocol-bus" | "bus" => Ok(Self::Port9098),
            "port-9099" | "ai-gateway" | "gateway" => Ok(Self::Port9099),
            "port-9100" | "dashboard" | "cockpit" => Ok(Self::Port9100),
            "port-18787" | "byok" | "byok-gateway" => Ok(Self::Port18787),
            "siagent" | "toksi-siagent" | "agent" => Ok(Self::Siagent),
            "codegraph" => Ok(Self::Codegraph),
            "ctxmode" | "context" | "context-mode" => Ok(Self::Ctxmode),
            "projectmem" => Ok(Self::Projectmem),
            "icm" => Ok(Self::Icm),
            "aimemory" | "ai-memory" => Ok(Self::Aimemory),
            "rtk" => Ok(Self::Rtk),
            "security" => Ok(Self::Security),
            other => Err(format!("Unknown worker role: '{other}'")),
        }
    }
}

pub fn sanitize_ipc_segment(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "default".to_string()
    } else {
        cleaned
    }
}

pub fn current_ipc_namespace() -> String {
    if let Ok(ns) = std::env::var("TOKSI_IPC_NS") {
        let clean = sanitize_ipc_segment(&ns);
        if !clean.is_empty() {
            return clean;
        }
    }
    let raw_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "local".to_string());
    sanitize_ipc_segment(&raw_user)
}

pub fn supervisor_pids_dir() -> PathBuf {
    let base = PathBuf::from(toksi_data_dir()).join("supervisor").join("pids");
    PathBuf::from(normalize_path(&base.to_string_lossy()))
}

pub fn ipc_endpoint_for_role(role: &str) -> String {
    ipc_endpoint_for_ns_and_role(&current_ipc_namespace(), role)
}

pub fn ipc_endpoint_for_ns_and_role(namespace: &str, role: &str) -> String {
    let ns = sanitize_ipc_segment(namespace);
    let clean_role = sanitize_ipc_segment(role);
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\toksi-{ns}-{clean_role}")
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(toksi_data_dir()).join("supervisor").join("sockets"));
        let full = base.join(format!("toksi-{ns}")).join(format!("{clean_role}.sock"));
        normalize_path(&full.to_string_lossy())
    }
}
