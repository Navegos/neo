// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/config/mod.rs
// created: 2026-11-16
// lastModified: 2026-11-16

//! Canonical `~/.toksi/config.toml` loader, validator, and runtime override store.
//!
//! Enforces `#![deny(unsafe_code)]` by storing CLI/daemon runtime overrides in
//! process-safe synchronized memory rather than mutating global `environ` via
//! `std::env::set_var`.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::{normalize_path, write_file_atomic};

pub const DEFAULT_HEADROOM_PROXY_PORT: u16 = 8787;
pub const DEFAULT_GROK_OAUTH_PROXY_PORT: u16 = 8788;
pub const DEFAULT_COPILOT_VSCODE_PORT: u16 = 8789;
pub const DEFAULT_COPILOT_CLI_PORT: u16 = 8790;
pub const DEFAULT_PROTOCOL_BUS_PORT: u16 = 9098;
pub const DEFAULT_AI_GATEWAY_PORT: u16 = 9099;
pub const DEFAULT_DASHBOARD_PORT: u16 = 9100;
pub const DEFAULT_BYOK_GATEWAY_PORT: u16 = 18787;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config at '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Malformed config TOML at '{path}': {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to serialize config TOML: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("Invalid port 0 for [ports].{field}")]
    ZeroPort { field: &'static str },

    #[error("Port collision on {port}: [ports].{first} conflicts with [ports].{second}")]
    PortCollision {
        port: u16,
        first: &'static str,
        second: &'static str,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PortsConfig {
    #[serde(alias = "headroom")]
    pub headroom_proxy: u16,
    #[serde(alias = "grok_oauth")]
    pub grok_oauth_proxy: u16,
    #[serde(alias = "copilot_vscode")]
    pub copilot_vscode_proxy: u16,
    #[serde(alias = "copilot_cli")]
    pub copilot_cli_proxy: u16,
    #[serde(alias = "protocol")]
    pub protocol_bus: u16,
    #[serde(alias = "byok")]
    pub byok_gateway: u16,
    #[serde(alias = "gateway")]
    pub ai_gateway: u16,
    #[serde(alias = "cockpit")]
    pub dashboard: u16,
}

impl Default for PortsConfig {
    fn default() -> Self {
        Self {
            headroom_proxy: DEFAULT_HEADROOM_PROXY_PORT,
            grok_oauth_proxy: DEFAULT_GROK_OAUTH_PROXY_PORT,
            copilot_vscode_proxy: DEFAULT_COPILOT_VSCODE_PORT,
            copilot_cli_proxy: DEFAULT_COPILOT_CLI_PORT,
            protocol_bus: DEFAULT_PROTOCOL_BUS_PORT,
            byok_gateway: DEFAULT_BYOK_GATEWAY_PORT,
            ai_gateway: DEFAULT_AI_GATEWAY_PORT,
            dashboard: DEFAULT_DASHBOARD_PORT,
        }
    }
}

impl PortsConfig {
    pub fn all_ports(&self) -> [(&'static str, u16); 8] {
        [
            ("headroom_proxy", self.headroom_proxy),
            ("grok_oauth_proxy", self.grok_oauth_proxy),
            ("copilot_vscode_proxy", self.copilot_vscode_proxy),
            ("copilot_cli_proxy", self.copilot_cli_proxy),
            ("protocol_bus", self.protocol_bus),
            ("byok_gateway", self.byok_gateway),
            ("ai_gateway", self.ai_gateway),
            ("dashboard", self.dashboard),
        ]
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut seen: HashMap<u16, &'static str> = HashMap::with_capacity(8);
        for (name, port) in self.all_ports() {
            if port == 0 {
                return Err(ConfigError::ZeroPort { field: name });
            }
            if let Some(prev) = seen.insert(port, name) {
                return Err(ConfigError::PortCollision {
                    port,
                    first: prev,
                    second: name,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeneralConfig {
    pub default_agents: Vec<String>,
    pub direct_wire: bool,
    pub auto_install_user_hooks: bool,
    pub auto_install_repo_hooks: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_agents: Vec::new(),
            direct_wire: true,
            auto_install_user_hooks: true,
            auto_install_repo_hooks: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AuthGcpAdcConfig {
    pub agy_adc_auth: bool,
    pub gcp_project: String,
    pub gcp_location: String,
}

impl Default for AuthGcpAdcConfig {
    fn default() -> Self {
        Self {
            agy_adc_auth: false,
            gcp_project: String::new(),
            gcp_location: "us-central1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AuthConfig {
    pub gcp_adc: AuthGcpAdcConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ByokProfileConfig {
    pub id: String,
    pub provider_id: String,
    pub instance_label: String,
    pub model_id: String,
    pub thinking_effort: String,
    pub base_url: String,
    pub enabled: bool,
    pub auth_mode: String,
    pub gcp_project: String,
    pub gcp_location: String,
}

impl Default for ByokProfileConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            provider_id: String::new(),
            instance_label: "default".to_string(),
            model_id: String::new(),
            thinking_effort: "medium".to_string(),
            base_url: String::new(),
            enabled: true,
            auth_mode: "api_key".to_string(),
            gcp_project: String::new(),
            gcp_location: "us-central1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TestConfig {
    pub incremental: bool,
    pub baseline_path: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            incremental: true,
            baseline_path: ".toksi/test-baseline.json".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ByokTableEntry {
    pub base_url: String,
    #[serde(alias = "model_id")]
    pub model: String,
    #[serde(alias = "thinking_effort")]
    pub effort: String,
    pub enabled: bool,
    pub provider_id: String,
}

impl Default for ByokTableEntry {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            model: String::new(),
            effort: "medium".to_string(),
            enabled: true,
            provider_id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ToksiTomlConfig {
    pub general: GeneralConfig,
    pub ports: PortsConfig,
    pub base_urls: HashMap<String, String>,
    pub default_agents: Vec<String>,
    pub auth: AuthConfig,
    pub byok: HashMap<String, ByokTableEntry>,
    pub byok_profiles: Vec<ByokProfileConfig>,
    pub test: TestConfig,
}

impl ToksiTomlConfig {
    pub fn has_enabled_byok_profiles(&self) -> bool {
        self.byok_profiles.iter().any(|p| p.enabled)
            || self.byok.values().any(|p| p.enabled)
    }

    pub fn enabled_profiles_for_provider(&self, provider_id: &str) -> Vec<&ByokProfileConfig> {
        self.byok_profiles
            .iter()
            .filter(|p| p.enabled && p.provider_id.eq_ignore_ascii_case(provider_id))
            .collect()
    }
}

#[derive(Debug, Default)]
struct RuntimeOverrides {
    headroom_proxy_port: Option<u16>,
    base_urls: HashMap<String, String>,
    default_agents: Option<Vec<String>>,
}

static RUNTIME_OVERRIDES: LazyLock<RwLock<RuntimeOverrides>> =
    LazyLock::new(|| RwLock::new(RuntimeOverrides::default()));

pub fn set_runtime_headroom_port_override(port: Option<u16>) {
    let mut guard = RUNTIME_OVERRIDES
        .write()
        .unwrap_or_else(|e| e.into_inner());
    guard.headroom_proxy_port = port.filter(|p| *p > 0);
}

pub fn runtime_headroom_port_override() -> Option<u16> {
    RUNTIME_OVERRIDES
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .headroom_proxy_port
}

pub fn set_runtime_base_url_override(provider: &str, url: &str) {
    let key = provider.trim().to_ascii_lowercase();
    let val = url.trim().trim_end_matches('/').to_string();
    let mut guard = RUNTIME_OVERRIDES
        .write()
        .unwrap_or_else(|e| e.into_inner());
    if key.is_empty() || val.is_empty() {
        guard.base_urls.remove(&key);
    } else {
        guard.base_urls.insert(key, val);
    }
}

pub fn get_runtime_base_url_override(provider: &str) -> Option<String> {
    let key = provider.trim().to_ascii_lowercase();
    RUNTIME_OVERRIDES
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .base_urls
        .get(&key)
        .cloned()
}

pub fn set_runtime_default_agents_override(agents: Option<Vec<String>>) {
    let mut guard = RUNTIME_OVERRIDES
        .write()
        .unwrap_or_else(|e| e.into_inner());
    guard.default_agents = agents.map(|list| {
        list.into_iter()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });
}

pub fn clear_runtime_overrides() {
    let mut guard = RUNTIME_OVERRIDES
        .write()
        .unwrap_or_else(|e| e.into_inner());
    *guard = RuntimeOverrides::default();
}

pub fn toksi_config_toml_path() -> PathBuf {
    let raw = PathBuf::from(crate::util::paths::toksi_data_dir()).join("config.toml");
    PathBuf::from(normalize_path(&raw.to_string_lossy()))
}

pub fn load_toksi_toml_config_checked() -> Result<ToksiTomlConfig, ConfigError> {
    let path = toksi_config_toml_path();
    if !path.exists() {
        return Ok(ToksiTomlConfig::default());
    }
    let norm = normalize_path(&path.to_string_lossy());
    let raw = std::fs::read_to_string(&path).map_err(|source| ConfigError::Io {
        path: norm.clone(),
        source,
    })?;
    toml::from_str::<ToksiTomlConfig>(&raw).map_err(|source| ConfigError::Parse { path: norm, source })
}

pub fn load_toksi_toml_config() -> ToksiTomlConfig {
    load_toksi_toml_config_checked().unwrap_or_default()
}

pub fn save_toksi_toml_config(cfg: &ToksiTomlConfig) -> Result<(), ConfigError> {
    let path = toksi_config_toml_path();
    let norm = normalize_path(&path.to_string_lossy());
    let content = toml::to_string_pretty(cfg)?;
    write_file_atomic(&path, &content).map_err(|source| ConfigError::Io { path: norm, source })?;
    Ok(())
}

pub const DEFAULT_COMMENTED_CONFIG_TOML: &str = r#"# Toksi Global Configuration (~/.toksi/config.toml)
# All paths use forward slashes ('/').

[general]
default_agents = []
direct_wire = true
auto_install_user_hooks = true
auto_install_repo_hooks = false

[ports]
headroom = 8787
grok_oauth = 8788
copilot_vscode = 8789
copilot_cli = 8790
protocol_bus = 9098
gateway = 9099
dashboard = 9100
byok_gateway = 18787

[test]
incremental = true
baseline_path = ".toksi/test-baseline.json"

[auth.gcp_adc]
agy_adc_auth = false
gcp_project = ""
gcp_location = "us-central1"

# Example BYOK profile override:
# [byok.openai-fast]
# base_url = "https://api.openai.com/v1"
# model = "gpt-4o-mini"
# effort = "medium"
# enabled = true
"#;

pub fn ensure_default_config_toml() -> Result<PathBuf, ConfigError> {
    let path = toksi_config_toml_path();
    if !path.exists() {
        let norm = normalize_path(&path.to_string_lossy());
        write_file_atomic(&path, DEFAULT_COMMENTED_CONFIG_TOML)
            .map_err(|source| ConfigError::Io { path: norm, source })?;
    }
    Ok(path)
}

fn parse_env_port(keys: &[&str]) -> Option<u16> {
    for key in keys {
        if let Ok(raw) = std::env::var(key)
            && let Ok(p) = raw.trim().parse::<u16>()
            && p > 0
        {
            return Some(p);
        }
    }
    None
}

pub fn resolved_ports() -> PortsConfig {
    let mut ports = load_toksi_toml_config().ports;

    if let Some(p) = runtime_headroom_port_override()
        .or_else(|| parse_env_port(&["TOKSI_HEADROOM_PROXY_PORT", "TOKLESS_HEADROOM_PROXY_PORT", "TOKSI_HEADROOM_PORT"]))
    {
        ports.headroom_proxy = p;
    }
    if let Some(p) = parse_env_port(&["TOKLESS_GROK_PROXY_PORT", "TOKSI_GROK_PORT"]) {
        ports.grok_oauth_proxy = p;
    }
    if let Some(p) = parse_env_port(&["TOKSI_COPILOT_PROXY_PORT", "TOKSI_COPILOT_PORT"]) {
        ports.copilot_vscode_proxy = p;
    }
    if let Some(p) = parse_env_port(&["TOKSI_COPILOT_CLI_PROXY_PORT", "TOKSI_COPILOT_CLI_PORT"]) {
        ports.copilot_cli_proxy = p;
    }
    if let Some(p) = parse_env_port(&["TOKSI_PROTOCOL_BUS_PORT"]) {
        ports.protocol_bus = p;
    }
    if let Some(p) = parse_env_port(&["TOKLESS_BYOK_GATEWAY_PORT", "TOKSI_BYOK_GATEWAY_PORT"]) {
        ports.byok_gateway = p;
    }
    if let Some(p) = parse_env_port(&["TOKSI_GATEWAY_PORT"]) {
        ports.ai_gateway = p;
    }
    if let Some(p) = parse_env_port(&["TOKSI_DASHBOARD_PORT"]) {
        ports.dashboard = p;
    }

    if ports.headroom_proxy == 0 {
        ports.headroom_proxy = DEFAULT_HEADROOM_PROXY_PORT;
    }
    if ports.grok_oauth_proxy == 0 {
        ports.grok_oauth_proxy = DEFAULT_GROK_OAUTH_PROXY_PORT;
    }
    if ports.copilot_vscode_proxy == 0 {
        ports.copilot_vscode_proxy = DEFAULT_COPILOT_VSCODE_PORT;
    }
    if ports.copilot_cli_proxy == 0 {
        ports.copilot_cli_proxy = DEFAULT_COPILOT_CLI_PORT;
    }
    if ports.protocol_bus == 0 {
        ports.protocol_bus = DEFAULT_PROTOCOL_BUS_PORT;
    }
    if ports.byok_gateway == 0 {
        ports.byok_gateway = DEFAULT_BYOK_GATEWAY_PORT;
    }
    if ports.ai_gateway == 0 {
        ports.ai_gateway = DEFAULT_AI_GATEWAY_PORT;
    }
    if ports.dashboard == 0 {
        ports.dashboard = DEFAULT_DASHBOARD_PORT;
    }

    ports
}

pub fn resolve_custom_base_url(provider: &str) -> Option<String> {
    if let Some(rt_url) = get_runtime_base_url_override(provider) {
        return Some(rt_url);
    }
    let cfg = load_toksi_toml_config();
    let lower = provider.trim().to_ascii_lowercase();
    cfg.base_urls.into_iter().find_map(|(k, v)| {
        let trimmed = v.trim();
        if k.trim().eq_ignore_ascii_case(&lower) && !trimmed.is_empty() {
            Some(trimmed.trim_end_matches('/').to_string())
        } else {
            None
        }
    })
}

pub fn resolve_default_agents() -> Vec<String> {
    if let Some(rt_agents) = RUNTIME_OVERRIDES
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .default_agents
        .clone()
    {
        return rt_agents;
    }

    let cfg = load_toksi_toml_config();
    let raw = if !cfg.general.default_agents.is_empty() {
        cfg.general.default_agents
    } else {
        cfg.default_agents
    };
    raw.into_iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn enabled_byok_profile_count() -> usize {
    let cfg = load_toksi_toml_config();
    let table_count = cfg
        .byok
        .iter()
        .filter(|(id, p)| p.enabled && (!id.trim().is_empty() || !p.provider_id.trim().is_empty()))
        .count();
    let cfg_count = cfg
        .byok_profiles
        .into_iter()
        .filter(|p| p.enabled && !p.provider_id.trim().is_empty())
        .count();
    let db_count = crate::db::shared_db()
        .ok()
        .and_then(|arc| {
            arc.lock().ok().and_then(|db| {
                db.list_byok_profiles()
                    .ok()
                    .map(|list| list.into_iter().filter(|p| p.is_active).count())
            })
        })
        .unwrap_or(0);
    table_count + cfg_count + db_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ports_config_defaults_and_validation() {
        let ports = PortsConfig::default();
        assert_eq!(ports.headroom_proxy, 8787);
        assert_eq!(ports.grok_oauth_proxy, 8788);
        assert_eq!(ports.copilot_vscode_proxy, 8789);
        assert_eq!(ports.copilot_cli_proxy, 8790);
        assert_eq!(ports.protocol_bus, 9098);
        assert_eq!(ports.ai_gateway, 9099);
        assert_eq!(ports.dashboard, 9100);
        assert_eq!(ports.byok_gateway, 18787);
        assert!(ports.validate().is_ok());
    }

    #[test]
    fn test_ports_collision_detected() {
        let mut ports = PortsConfig::default();
        ports.copilot_cli_proxy = ports.copilot_vscode_proxy;
        let err = ports.validate().err();
        assert!(matches!(err, Some(ConfigError::PortCollision { port: 8789, .. })));
    }

    #[test]
    fn test_toml_parsing_with_overrides_and_default_agents() {
        let raw = r#"
[general]
default_agents = ["Claude", "Copilot"]
direct_wire = true

[ports]
headroom = 18700
protocol_bus = 19098
gateway = 19099
dashboard = 19100

[base_urls]
openai = "https://custom.openai.example/v1/"
copilot = "https://copilot.internal.example"
"#;
        let parsed: ToksiTomlConfig = toml::from_str(raw).unwrap_or_default();
        assert_eq!(parsed.general.default_agents, vec!["Claude", "Copilot"]);
        assert_eq!(parsed.ports.headroom_proxy, 18700);
        assert_eq!(parsed.ports.protocol_bus, 19098);
        assert_eq!(parsed.ports.ai_gateway, 19099);
        assert_eq!(parsed.ports.dashboard, 19100);
        assert_eq!(parsed.ports.grok_oauth_proxy, 8788);
        assert_eq!(
            parsed.base_urls.get("openai").map(String::as_str),
            Some("https://custom.openai.example/v1/")
        );
        assert!(parsed.ports.validate().is_ok());
    }

    #[test]
    fn test_runtime_overrides_without_env_mutation() {
        clear_runtime_overrides();
        set_runtime_headroom_port_override(Some(28787));
        set_runtime_base_url_override("anthropic", "https://mock.anthropic.local/");
        assert_eq!(runtime_headroom_port_override(), Some(28787));
        assert_eq!(
            resolve_custom_base_url("anthropic").as_deref(),
            Some("https://mock.anthropic.local")
        );
        clear_runtime_overrides();
        assert_eq!(runtime_headroom_port_override(), None);
    }
}
