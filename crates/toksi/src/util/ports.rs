// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/ports.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Unified 7-Port Topology loader backed by `~/.toksi/config.toml` `[ports]`.
//! Zero panics; falls back cleanly to canonical defaults when config is absent or malformed.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HEADROOM_PROXY_PORT: u16 = 8787;
pub const DEFAULT_GROK_OAUTH_PROXY_PORT: u16 = 8788;
pub const DEFAULT_COPILOT_VSCODE_PORT: u16 = 8789;
pub const DEFAULT_COPILOT_CLI_PORT: u16 = 8790;
pub const DEFAULT_BYOK_GATEWAY_PORT: u16 = 18787;
pub const DEFAULT_AI_GATEWAY_PORT: u16 = 9099;
pub const DEFAULT_DASHBOARD_PORT: u16 = 9100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PortsConfig {
    pub headroom_proxy: u16,
    pub grok_oauth_proxy: u16,
    pub copilot_vscode_proxy: u16,
    pub copilot_cli_proxy: u16,
    pub byok_gateway: u16,
    pub ai_gateway: u16,
    pub dashboard: u16,
}

impl Default for PortsConfig {
    fn default() -> Self {
        Self {
            headroom_proxy: DEFAULT_HEADROOM_PROXY_PORT,
            grok_oauth_proxy: DEFAULT_GROK_OAUTH_PROXY_PORT,
            copilot_vscode_proxy: DEFAULT_COPILOT_VSCODE_PORT,
            copilot_cli_proxy: DEFAULT_COPILOT_CLI_PORT,
            byok_gateway: DEFAULT_BYOK_GATEWAY_PORT,
            ai_gateway: DEFAULT_AI_GATEWAY_PORT,
            dashboard: DEFAULT_DASHBOARD_PORT,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ToksiTomlConfig {
    pub ports: PortsConfig,
    pub base_urls: HashMap<String, String>,
}

pub fn toksi_config_toml_path() -> PathBuf {
    PathBuf::from(crate::util::paths::toksi_data_dir()).join("config.toml")
}

pub fn load_toksi_toml_config() -> ToksiTomlConfig {
    let path = toksi_config_toml_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return ToksiTomlConfig::default();
    };
    toml::from_str::<ToksiTomlConfig>(&raw).unwrap_or_default()
}

pub fn resolved_ports() -> PortsConfig {
    let mut ports = load_toksi_toml_config().ports;
    if let Ok(v) = std::env::var("TOKLESS_HEADROOM_PROXY_PORT").or_else(|_| std::env::var("TOKSI_HEADROOM_PORT")) {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.headroom_proxy = p;
            }
        }
    }
    if let Ok(v) = std::env::var("TOKLESS_GROK_PROXY_PORT").or_else(|_| std::env::var("TOKSI_GROK_PORT")) {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.grok_oauth_proxy = p;
            }
        }
    }
    if let Ok(v) = std::env::var("TOKSI_COPILOT_PORT") {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.copilot_vscode_proxy = p;
            }
        }
    }
    if let Ok(v) = std::env::var("TOKSI_COPILOT_CLI_PORT") {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.copilot_cli_proxy = p;
            }
        }
    }
    if let Ok(v) = std::env::var("TOKLESS_BYOK_GATEWAY_PORT").or_else(|_| std::env::var("TOKSI_BYOK_GATEWAY_PORT")) {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.byok_gateway = p;
            }
        }
    }
    if let Ok(v) = std::env::var("TOKSI_GATEWAY_PORT") {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.ai_gateway = p;
            }
        }
    }
    if let Ok(v) = std::env::var("TOKSI_DASHBOARD_PORT") {
        if let Ok(p) = v.trim().parse::<u16>() {
            if p > 0 {
                ports.dashboard = p;
            }
        }
    }
    if ports.headroom_proxy == 0 { ports.headroom_proxy = DEFAULT_HEADROOM_PROXY_PORT; }
    if ports.grok_oauth_proxy == 0 { ports.grok_oauth_proxy = DEFAULT_GROK_OAUTH_PROXY_PORT; }
    if ports.copilot_vscode_proxy == 0 { ports.copilot_vscode_proxy = DEFAULT_COPILOT_VSCODE_PORT; }
    if ports.copilot_cli_proxy == 0 { ports.copilot_cli_proxy = DEFAULT_COPILOT_CLI_PORT; }
    if ports.byok_gateway == 0 { ports.byok_gateway = DEFAULT_BYOK_GATEWAY_PORT; }
    if ports.ai_gateway == 0 { ports.ai_gateway = DEFAULT_AI_GATEWAY_PORT; }
    if ports.dashboard == 0 { ports.dashboard = DEFAULT_DASHBOARD_PORT; }
    ports
}

pub fn resolve_custom_base_url(provider: &str) -> Option<String> {
    let cfg = load_toksi_toml_config();
    let lower = provider.trim().to_ascii_lowercase();
    cfg.base_urls.into_iter().find_map(|(k, v)| {
        if k.trim().eq_ignore_ascii_case(&lower) && !v.trim().is_empty() {
            Some(v.trim().to_string())
        } else {
            None
        }
    })
}
