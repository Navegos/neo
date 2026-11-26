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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ByokProfileConfig {
    pub id: String,
    pub provider_id: String,
    pub instance_label: String,
    pub model_id: String,
    pub thinking_effort: String,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
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
            temperature: None,
            top_p: None,
            top_k: None,
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
    pub live_safe_isolation: bool,
    pub sandbox_backend: String,
    pub test_image: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            incremental: true,
            baseline_path: ".toksi/test-baseline.json".to_string(),
            live_safe_isolation: true,
            sandbox_backend: "docker".to_string(),
            test_image: "toksi-test:local".to_string(),
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
pub struct RepositoriesConfig {
    pub neo_dir: String,
    pub advisory_dir: String,
    pub skills_dir: String,
    pub user_skills_dir: String,
}

fn default_true_sec() -> bool {
    true
}

fn default_skill_scanner_policy() -> String {
    "balanced".to_string()
}

fn default_policy_engine() -> String {
    "opa_cedar".to_string()
}

fn default_iam_attestation() -> String {
    "spiffe_spire".to_string()
}

fn default_sandbox_backend() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SecurityConfig {
    #[serde(default = "default_true_sec")]
    pub enforce_in_repo_skills: bool,
    #[serde(default = "default_true_sec")]
    pub auto_heal_skill_frontmatter: bool,
    #[serde(default = "default_true_sec")]
    pub static_analyzer_enabled: bool,
    #[serde(default = "default_true_sec")]
    pub defenseclaw_fail_closed: bool,
    #[serde(default = "default_skill_scanner_policy")]
    pub skill_scanner_policy: String,
    #[serde(default = "default_policy_engine")]
    pub policy_engine: String,
    #[serde(default = "default_iam_attestation")]
    pub iam_attestation: String,
    #[serde(default = "default_true_sec")]
    pub kill_all_services_on_bypass: bool,
    #[serde(default = "default_true_sec")]
    pub adr_enabled: bool,
    #[serde(default = "default_true_sec")]
    pub zero_plaintext_blind_signatures: bool,
    #[serde(default = "default_sandbox_backend")]
    pub sandbox_backend: String,
    pub external_analyzers: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enforce_in_repo_skills: true,
            auto_heal_skill_frontmatter: true,
            static_analyzer_enabled: true,
            defenseclaw_fail_closed: true,
            skill_scanner_policy: default_skill_scanner_policy(),
            policy_engine: default_policy_engine(),
            iam_attestation: default_iam_attestation(),
            kill_all_services_on_bypass: true,
            adr_enabled: true,
            zero_plaintext_blind_signatures: true,
            sandbox_backend: default_sandbox_backend(),
            external_analyzers: vec![
                "rust-analyzer".to_string(),
                "gopls".to_string(),
                "clang-tidy".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SupervisorConfig {
    pub restart_backoff_ms: u64,
    pub max_restarts_per_min: u32,
    pub heartbeat_interval_ms: u64,
    pub drain_timeout_ms: u64,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            restart_backoff_ms: 500,
            max_restarts_per_min: 10,
            heartbeat_interval_ms: 3000,
            drain_timeout_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SiagentModeOverrideConfig {
    pub model_tier: Option<String>,
    pub primary_model: Option<String>,
    pub fallback_model: Option<String>,
    pub embedding_model: Option<String>,
    pub reranker_model: Option<String>,
    pub query_rewriter_model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    #[serde(default)]
    pub architecture: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ToksiSiagentRouterConfig {
    pub gateway: Option<String>,
    pub router: Option<String>,
    pub default_fallback: Option<String>,
    #[serde(flatten)]
    pub domains: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, SiagentModeOverrideConfig>,
    >,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HeadroomConfig {
    pub enabled: bool,
    pub default_effort: String,
    pub align_1024_cache: bool,
    pub adaptive_memory: bool,
    pub mcp_pruning: bool,
    pub ccr_ttl_seconds: u64,
    pub ccr_max_entries: usize,
    pub loop_guard_threshold: usize,
    pub live_zone_surgery: bool,
    pub smart_crusher: bool,
    pub token_economizer: bool,
}

impl Default for HeadroomConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_effort: "medium".to_string(),
            align_1024_cache: true,
            adaptive_memory: true,
            mcp_pruning: true,
            ccr_ttl_seconds: 1800,
            ccr_max_entries: 2048,
            loop_guard_threshold: 3,
            live_zone_surgery: true,
            smart_crusher: true,
            token_economizer: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GatewayConfig {
    pub bind_host: String,
    pub auto_complete_v1: bool,
    pub semantic_cache_enabled: bool,
    pub circuit_breaker_enabled: bool,
    pub reconnect_watchdog_enabled: bool,
    pub load_balance_strategy: String,
    pub local_engines_enabled: bool,
    pub max_cascade_retries: u32,
    pub request_timeout_ms: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind_host: "127.0.0.1".to_string(),
            auto_complete_v1: true,
            semantic_cache_enabled: true,
            circuit_breaker_enabled: true,
            reconnect_watchdog_enabled: true,
            load_balance_strategy: "least_latency".to_string(),
            local_engines_enabled: true,
            max_cascade_retries: 3,
            request_timeout_ms: 60_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PluginsConfig {
    pub plugin_dirs: Vec<String>,
    pub enforce_security_scan: bool,
    pub fail_closed_hooks: bool,
    pub protocols: Vec<String>,
    pub allow_agent_to_agent: bool,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            plugin_dirs: vec![
                "~/.toksi/plugins".to_string(),
                ".toksi/plugins".to_string(),
                ".agents/plugins".to_string(),
            ],
            enforce_security_scan: true,
            fail_closed_hooks: true,
            protocols: vec![
                "mcp".to_string(),
                "a2a".to_string(),
                "acp".to_string(),
                "anp".to_string(),
                "stdio".to_string(),
                "sse".to_string(),
                "http".to_string(),
                "quic".to_string(),
            ],
            allow_agent_to_agent: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CheckpointConfig {
    pub enabled: bool,
    pub write_ahead_journal: bool,
    pub disjoint_worker_leases: bool,
    pub five_store_atomic_flush: bool,
    pub blake3_resume_verify: bool,
    pub checkpoints_dir: String,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            write_ahead_journal: true,
            disjoint_worker_leases: true,
            five_store_atomic_flush: true,
            blake3_resume_verify: true,
            checkpoints_dir: ".toksi/projectmem/plan_checkpoints".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SandboxConfig {
    pub backend: String,
    pub allow_network: bool,
    pub skills_mount_mode: String,
    pub sbxenv_path: String,
    pub compose_test_path: String,
    pub timeout_secs: u64,
    pub memory_limit_mb: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            backend: "auto".to_string(),
            allow_network: false,
            skills_mount_mode: "read_only".to_string(),
            sbxenv_path: "sbxenv.yaml".to_string(),
            compose_test_path: "docker/compose.test.yaml".to_string(),
            timeout_secs: 120,
            memory_limit_mb: 2048,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub default_backend: String,
    pub lan_pair_enabled: bool,
    pub nv_sentinel_auto_cordon: bool,
    pub aicr_profile: String,
    pub otel_collector_port: u16,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_backend: "kubernetes".to_string(),
            lan_pair_enabled: true,
            nv_sentinel_auto_cordon: true,
            aicr_profile: "h100-sxm".to_string(),
            otel_collector_port: 4318,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TrainingConfig {
    pub default_backend: String,
    pub default_method: String,
    pub default_quant_format: String,
    pub default_serving_engine: String,
    pub verify_safetensors_headers: bool,
    pub default_eval_harness: String,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            default_backend: "unsloth".to_string(),
            default_method: "lora".to_string(),
            default_quant_format: "gguf-q4km".to_string(),
            default_serving_engine: "oxillama".to_string(),
            verify_safetensors_headers: true,
            default_eval_harness: "deepeval".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TelemetryOtelConfig {
    pub enabled: bool,
    pub otlp_endpoint: String,
    pub service_name: String,
    pub semconv_version: String,
    pub fail_open: bool,
    pub per_service_dbs: bool,
    pub service_db_dir: String,
    pub prometheus_metrics: bool,
}

impl Default for TelemetryOtelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            otlp_endpoint: "http://127.0.0.1:4318/v1/traces".to_string(),
            service_name: "toksi".to_string(),
            semconv_version: "gen_ai.1.29".to_string(),
            fail_open: true,
            per_service_dbs: true,
            service_db_dir: "~/.toksi/services".to_string(),
            prometheus_metrics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DashboardConfig {
    pub bind_host: String,
    pub detached_from_daemon: bool,
    pub auto_open_browser: bool,
    pub sse_interval_ms: u64,
    pub tabs_count: u8,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            bind_host: "127.0.0.1".to_string(),
            detached_from_daemon: true,
            auto_open_browser: false,
            sse_interval_ms: 1000,
            tabs_count: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TuiConfig {
    pub mouse_capture: bool,
    pub tick_rate_ms: u64,
    pub tabs_count: u8,
    pub chat_ui_enabled: bool,
    pub chat_plain_fallback: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            mouse_capture: true,
            tick_rate_ms: 100,
            tabs_count: 10,
            chat_ui_enabled: true,
            chat_plain_fallback: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct KeyringConfig {
    pub backend: String,
    pub zeroize_on_drop: bool,
    pub service_prefix: String,
    pub sync_on_startup: bool,
}

impl Default for KeyringConfig {
    fn default() -> Self {
        Self {
            backend: "os-native".to_string(),
            zeroize_on_drop: true,
            service_prefix: "toksi".to_string(),
            sync_on_startup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ToolsConfig {
    pub repo_scoped_only: bool,
    pub respect_gitignore: bool,
    pub dynamic_ignore_purge: bool,
    pub rtk_enabled: bool,
    pub codegraph_enabled: bool,
    pub ctxmode_enabled: bool,
    pub projectmem_enabled: bool,
    pub icm_enabled: bool,
    pub aimemory_enabled: bool,
    pub ctxmode_replace_trigger_bytes: usize,
    pub codegraph_languages: usize,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            repo_scoped_only: true,
            respect_gitignore: true,
            dynamic_ignore_purge: true,
            rtk_enabled: true,
            codegraph_enabled: true,
            ctxmode_enabled: true,
            projectmem_enabled: true,
            icm_enabled: true,
            aimemory_enabled: true,
            ctxmode_replace_trigger_bytes: 4096,
            codegraph_languages: 45,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct HooksConfig {
    pub standard_syntax: String,
    pub auto_migrate_legacy: bool,
    pub map_mcp_to_skills: bool,
    pub inject_enforce_rules: bool,
    pub cleanup_dead_ide_hooks: bool,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            standard_syntax: "<tool>-hook <agent>".to_string(),
            auto_migrate_legacy: true,
            map_mcp_to_skills: true,
            inject_enforce_rules: true,
            cleanup_dead_ide_hooks: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct IdeBridgeConfig {
    pub enabled: bool,
    pub external_package_only: bool,
    pub default_transport: String,
    pub supported_ides: Vec<String>,
}

impl Default for IdeBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            external_package_only: true,
            default_transport: "http".to_string(),
            supported_ides: vec![
                "vscode".to_string(),
                "visual-studio".to_string(),
                "jetbrains".to_string(),
                "android-studio".to_string(),
                "zed".to_string(),
                "xcode".to_string(),
                "eclipse".to_string(),
                "cursor".to_string(),
                "windsurf".to_string(),
                "apache-netbeans".to_string(),
                "kiro".to_string(),
                "qoder".to_string(),
                "lapce".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ToksiTomlConfig {
    pub general: GeneralConfig,
    pub ports: PortsConfig,
    pub base_urls: HashMap<String, String>,
    pub default_agents: Vec<String>,
    pub auth: AuthConfig,
    pub byok: HashMap<String, ByokTableEntry>,
    pub byok_profiles: Vec<ByokProfileConfig>,
    pub repositories: RepositoriesConfig,
    pub security: SecurityConfig,
    #[serde(alias = "testing")]
    pub test: TestConfig,
    pub supervisor: SupervisorConfig,
    #[serde(alias = "optimization")]
    pub headroom: HeadroomConfig,
    pub gateway: GatewayConfig,
    pub plugins: PluginsConfig,
    pub checkpoint: CheckpointConfig,
    pub sandbox: SandboxConfig,
    pub cluster: ClusterConfig,
    pub training: TrainingConfig,
    #[serde(alias = "otel")]
    pub telemetry: TelemetryOtelConfig,
    pub dashboard: DashboardConfig,
    pub tui: TuiConfig,
    pub keyring: KeyringConfig,
    pub tools: ToolsConfig,
    pub hooks: HooksConfig,
    pub ide_bridge: IdeBridgeConfig,
    pub toksi_siagent_router: ToksiSiagentRouterConfig,
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
