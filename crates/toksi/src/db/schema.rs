// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/schema.rs
// created: 2026-11-04
// lastModified: 2026-11-18

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

fn default_auth_mode() -> String {
    "api_key".to_string()
}

fn default_gcp_location() -> String {
    "us-central1".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    pub is_enabled: bool,
    pub auth_type: String, // "keyring", "api_key", "oauth", "adc", "none"
    pub metadata_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderKeyRecord {
    pub id: String,
    pub provider_id: String,
    pub key_alias: String,
    pub secret_ref: String, // "keyring://service/account" or raw key
    pub is_active: bool,
    pub priority: i32,
    pub weight: i32,
    pub last_status: String, // "ok", "rate_limited", "quota_exceeded", "invalid", "expired", "refresh_needed"
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderModelRecord {
    pub id: String,
    pub provider_id: String,
    pub display_name: String,
    pub context_window: i64,
    pub max_output_tokens: i64,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub is_default: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ByokRouteRecord {
    pub id: String,
    pub token: String,
    pub protocol: String,
    pub upstream: String,
    pub provider_id: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentConfigRecord {
    pub agent_id: String,
    pub provider_id: Option<String>,
    pub keyring_service: Option<String>,
    pub keyring_account: Option<String>,
    pub upstream_route_id: Option<String>,
    pub config_json: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayConfigRecord {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

fn default_models_json() -> String {
    "[]".to_string()
}

fn default_efforts_json() -> String {
    r#"["medium"]"#.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ByokProfileRecord {
    pub id: String,
    pub provider_id: String,
    pub instance_label: String,
    pub model_id: String,
    pub thinking_effort: String, // "none" | "low" | "medium" | "high" | "xhigh" | "max"
    pub base_url: String,
    pub secret_ref: String,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: String, // "api_key" | "oauth" | "adc" | "keyring"
    #[serde(default)]
    pub gcp_project: String,
    #[serde(default = "default_gcp_location")]
    pub gcp_location: String,
    pub priority: i32,
    pub weight: i32,
    #[serde(default = "default_true")]
    pub headroom_enabled: bool,
    pub is_active: bool,
    #[serde(default = "default_models_json")]
    pub models_json: String,
    #[serde(default = "default_efforts_json")]
    pub efforts_json: String,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub top_k: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for ByokProfileRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            provider_id: String::new(),
            instance_label: "default".to_string(),
            model_id: String::new(),
            thinking_effort: "medium".to_string(),
            base_url: String::new(),
            secret_ref: String::new(),
            auth_mode: default_auth_mode(),
            gcp_project: String::new(),
            gcp_location: default_gcp_location(),
            priority: 100,
            weight: 100,
            headroom_enabled: true,
            is_active: true,
            models_json: default_models_json(),
            efforts_json: default_efforts_json(),
            temperature: None,
            top_p: None,
            top_k: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentProfileBindingRecord {
    pub agent_id: String,
    pub byok_profile_id: String,
    pub is_primary: bool,
    pub fallback_order: i32,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuotaWindowRecord {
    pub id: String,
    pub provider_id: String,
    pub instance_label: String,
    pub window_kind: String, // "hour" | "day" | "week" | "month" | "year"
    pub limit_requests: Option<i64>,
    pub remaining_requests: Option<i64>,
    pub limit_tokens: Option<i64>,
    pub remaining_tokens: Option<i64>,
    pub reset_at_epoch: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginRegistryRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_json: String,
    pub blake3_hash: String,
    pub capabilities_json: String,
    pub protocols_json: String,
    pub security_scan_status: String, // "passed" | "pending" | "blocked"
    pub is_enabled: bool,
    pub installed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlindSignatureRecord {
    pub id: String,
    pub signature_kind: String, // "blake3_ngram" | "sparse_dfa_b64"
    pub signature_payload: String, // strictly blind blake3 hash or base64 sparse DFA; never raw PoC/regex
    pub severity: String,
    pub language: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdvisoryEmbargoRecord {
    pub advisory_id: String,
    pub disclosure_state: String, // "embargoed" | "ready" | "published" | "withdrawn"
    pub embargo_until_epoch: i64,
    pub upstream_repo: String,
    pub pr_url: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdrAuditEventRecord {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub decision: String, // "allowed" | "sandboxed" | "blocked" | "kill_switch_triggered"
    pub reason: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupervisorWorkerRecord {
    pub worker_role: String,
    pub pid: Option<i64>,
    pub ipc_endpoint: String,
    pub status: String, // "stopped" | "running" | "draining" | "crashed"
    pub restart_count: i64,
    pub last_heartbeat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrchestratorModeRouteRecord {
    pub mode_id: String,
    pub domain: String,
    pub primary_model: String,
    pub fallback_model: String,
    pub reasoning_effort: String,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterNodeRecord {
    pub node_id: String,
    pub backend: String,
    pub endpoint: String,
    pub gpu_model: String,
    pub gpu_count: i64,
    pub vram_gb: i64,
    pub health: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToksiDbDump {
    pub version: u32,
    pub exported_at: i64,
    pub providers: Vec<ProviderRecord>,
    pub provider_keys: Vec<ProviderKeyRecord>,
    pub provider_models: Vec<ProviderModelRecord>,
    pub byok_routes: Vec<ByokRouteRecord>,
    pub agent_configs: Vec<AgentConfigRecord>,
    pub gateway_configs: Vec<GatewayConfigRecord>,
    #[serde(default)]
    pub byok_profiles: Vec<ByokProfileRecord>,
    #[serde(default)]
    pub agent_profile_bindings: Vec<AgentProfileBindingRecord>,
    #[serde(default)]
    pub quota_windows: Vec<QuotaWindowRecord>,
    #[serde(default)]
    pub plugins_registry: Vec<PluginRegistryRecord>,
    #[serde(default)]
    pub blind_signatures: Vec<BlindSignatureRecord>,
    #[serde(default)]
    pub advisory_embargoes: Vec<AdvisoryEmbargoRecord>,
    #[serde(default)]
    pub adr_audit_events: Vec<AdrAuditEventRecord>,
    #[serde(default)]
    pub supervisor_workers: Vec<SupervisorWorkerRecord>,
    #[serde(default)]
    pub orchestrator_mode_routes: Vec<OrchestratorModeRouteRecord>,
    #[serde(default)]
    pub cluster_nodes: Vec<ClusterNodeRecord>,
}

pub const SCHEMA_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    protocol TEXT NOT NULL,
    base_url TEXT NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    auth_type TEXT NOT NULL DEFAULT 'keyring',
    metadata_json TEXT DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_keys (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    key_alias TEXT NOT NULL,
    secret_ref TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0,
    weight INTEGER NOT NULL DEFAULT 1,
    last_status TEXT DEFAULT 'ok',
    expires_at INTEGER,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_provider_keys_prov ON provider_keys(provider_id);

CREATE TABLE IF NOT EXISTS provider_models (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    context_window INTEGER NOT NULL DEFAULT 200000,
    max_output_tokens INTEGER NOT NULL DEFAULT 8192,
    supports_streaming INTEGER NOT NULL DEFAULT 1,
    supports_tools INTEGER NOT NULL DEFAULT 1,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_provider_models_prov ON provider_models(provider_id);

CREATE TABLE IF NOT EXISTS byok_routes (
    id TEXT PRIMARY KEY,
    token TEXT NOT NULL,
    protocol TEXT NOT NULL,
    upstream TEXT NOT NULL,
    provider_id TEXT REFERENCES providers(id) ON DELETE SET NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_configs (
    agent_id TEXT PRIMARY KEY,
    provider_id TEXT REFERENCES providers(id) ON DELETE SET NULL,
    keyring_service TEXT,
    keyring_account TEXT,
    upstream_route_id TEXT REFERENCES byok_routes(id) ON DELETE SET NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS gateway_configs (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS byok_profiles (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    instance_label TEXT NOT NULL DEFAULT 'default',
    model_id TEXT NOT NULL DEFAULT '',
    thinking_effort TEXT NOT NULL DEFAULT 'none',
    base_url TEXT NOT NULL,
    secret_ref TEXT NOT NULL DEFAULT '',
    auth_mode TEXT NOT NULL DEFAULT 'api_key',
    gcp_project TEXT NOT NULL DEFAULT '',
    gcp_location TEXT NOT NULL DEFAULT 'us-central1',
    priority INTEGER NOT NULL DEFAULT 0,
    weight INTEGER NOT NULL DEFAULT 1,
    headroom_enabled INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    models_json TEXT NOT NULL DEFAULT '[]',
    efforts_json TEXT NOT NULL DEFAULT '["medium"]',
    temperature REAL,
    top_p REAL,
    top_k INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(provider_id, instance_label, model_id, thinking_effort)
);
CREATE INDEX IF NOT EXISTS idx_byok_profiles_provider ON byok_profiles(provider_id);

CREATE TABLE IF NOT EXISTS agent_profile_bindings (
    agent_id TEXT NOT NULL,
    byok_profile_id TEXT NOT NULL REFERENCES byok_profiles(id) ON DELETE CASCADE,
    is_primary INTEGER NOT NULL DEFAULT 0,
    fallback_order INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY(agent_id, byok_profile_id)
);

CREATE TABLE IF NOT EXISTS quota_windows (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    instance_label TEXT NOT NULL DEFAULT 'default',
    window_kind TEXT NOT NULL CHECK(window_kind IN ('hour','day','week','month','year')),
    limit_requests INTEGER,
    remaining_requests INTEGER,
    limit_tokens INTEGER,
    remaining_tokens INTEGER,
    reset_at_epoch INTEGER,
    updated_at INTEGER NOT NULL,
    UNIQUE(provider_id, instance_label, window_kind)
);
CREATE INDEX IF NOT EXISTS idx_quota_windows_provider ON quota_windows(provider_id, window_kind);

CREATE TABLE IF NOT EXISTS plugins_registry (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    manifest_json TEXT NOT NULL DEFAULT '{}',
    blake3_hash TEXT NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    protocols_json TEXT NOT NULL DEFAULT '[]',
    security_scan_status TEXT NOT NULL DEFAULT 'pending',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    installed_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS blind_signatures (
    id TEXT PRIMARY KEY,
    signature_kind TEXT NOT NULL CHECK(signature_kind IN ('blake3_ngram','sparse_dfa_b64')),
    signature_payload TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    language TEXT NOT NULL DEFAULT 'polyglot',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS advisory_embargoes (
    advisory_id TEXT PRIMARY KEY,
    disclosure_state TEXT NOT NULL CHECK(disclosure_state IN ('embargoed','ready','published','withdrawn')),
    embargo_until_epoch INTEGER NOT NULL,
    upstream_repo TEXT NOT NULL,
    pr_url TEXT,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS adr_audit_events (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('allowed','sandboxed','blocked','kill_switch_triggered')),
    reason TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_adr_audit_agent ON adr_audit_events(agent_id, created_at);

CREATE TABLE IF NOT EXISTS supervisor_workers (
    worker_role TEXT PRIMARY KEY,
    pid INTEGER,
    ipc_endpoint TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'stopped',
    restart_count INTEGER NOT NULL DEFAULT 0,
    last_heartbeat INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS orchestrator_mode_routes (
    mode_id TEXT PRIMARY KEY,
    domain TEXT NOT NULL,
    primary_model TEXT NOT NULL,
    fallback_model TEXT NOT NULL DEFAULT '',
    reasoning_effort TEXT NOT NULL DEFAULT 'medium',
    temperature REAL,
    top_p REAL,
    top_k INTEGER,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS cluster_nodes (
    node_id TEXT PRIMARY KEY,
    backend TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    gpu_model TEXT NOT NULL DEFAULT '',
    gpu_count INTEGER NOT NULL DEFAULT 0,
    vram_gb INTEGER NOT NULL DEFAULT 0,
    health TEXT NOT NULL DEFAULT 'healthy',
    updated_at INTEGER NOT NULL
);
"#;
