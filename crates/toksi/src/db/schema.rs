// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/schema.rs
// created: 2026-11-04
// lastModified: 2026-11-04

use serde::{Deserialize, Serialize};

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
    pub last_status: String, // "ok", "rate_limited", "quota_exceeded", "invalid"
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
"#;

