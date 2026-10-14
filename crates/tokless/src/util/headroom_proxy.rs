// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/headroom_proxy.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::path::Path;
use serde::{Deserialize, Serialize};

use super::paths::{read_file_safe, write_file_atomic};
use super::headroom::headroom_paths_resolved;

pub const HEADROOM_PROXY_DEFAULT_PORT: u16 = 8787;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyRuntime {
    pub port: u16,
    #[serde(default)]
    pub anthropic_api_url: String,
    #[serde(default)]
    pub openai_api_url: String,
    #[serde(default)]
    pub gemini_api_url: String,
    #[serde(default)]
    pub cloudcode_api_url: String,
    #[serde(default)]
    pub provider: String,
}

pub fn headroom_proxy_runtime_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("proxy.runtime.json")
        .to_string_lossy()
        .to_string()
}

pub fn headroom_proxy_preference_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("proxy.preference")
        .to_string_lossy()
        .to_string()
}

pub fn read_proxy_runtime() -> Option<ProxyRuntime> {
    let raw = read_file_safe(headroom_proxy_runtime_file())?;
    let st: ProxyRuntime = serde_json::from_str(&raw).ok()?;
    if st.port > 0 {
        Some(st)
    } else {
        None
    }
}

pub fn save_headroom_proxy_runtime(st: &ProxyRuntime) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if st.port == 0 {
        return Err(format!("invalid headroom proxy port {}", st.port).into());
    }
    let s = serde_json::to_string(st)?;
    write_file_atomic(headroom_proxy_runtime_file(), &s)?;
    Ok(())
}

pub fn clear_headroom_proxy_runtime() -> std::io::Result<()> {
    let path = headroom_proxy_runtime_file();
    if Path::new(&path).exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn proxy_routing_enabled() -> bool {
    if let Some(raw) = read_file_safe(headroom_proxy_preference_file()) {
        match raw.trim() {
            "enabled" => true,
            "disabled" => false,
            _ => false,
        }
    } else {
        true
    }
}

pub fn proxy_routing_preference_set() -> bool {
    Path::new(&headroom_proxy_preference_file()).exists()
}

pub fn set_proxy_routing_enabled(enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = if enabled { "enabled\n" } else { "disabled\n" };
    write_file_atomic(headroom_proxy_preference_file(), state)?;
    Ok(())
}

pub fn clear_proxy_routing_preference() -> std::io::Result<()> {
    let path = headroom_proxy_preference_file();
    if Path::new(&path).exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn headroom_proxy_port() -> u16 {
    if let Ok(raw) = std::env::var("TOKLESS_HEADROOM_PROXY_PORT") {
        if let Ok(n) = raw.trim().parse::<u16>() {
            if n > 0 {
                return n;
            }
        }
    }
    if let Some(st) = read_proxy_runtime() {
        return st.port;
    }
    HEADROOM_PROXY_DEFAULT_PORT
}

pub fn headroom_proxy_url() -> String {
    format!("http://127.0.0.1:{}", headroom_proxy_port())
}

pub fn headroom_proxy_openai_url() -> String {
    format!("{}/v1", headroom_proxy_url())
}

