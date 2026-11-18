// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/operations.rs
// created: 2026-11-04
// lastModified: 2026-11-18

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStats {
    pub db_path: String,
    pub providers_count: usize,
    pub keys_count: usize,
    pub models_count: usize,
    pub routes_count: usize,
    pub agents_count: usize,
    pub gateway_configs_count: usize,
    #[serde(default)]
    pub byok_profiles_count: usize,
    #[serde(default)]
    pub quota_windows_count: usize,
    #[serde(default)]
    pub plugins_count: usize,
    #[serde(default)]
    pub blind_signatures_count: usize,
    #[serde(default)]
    pub advisory_embargoes_count: usize,
    #[serde(default)]
    pub adr_audit_events_count: usize,
    pub file_size_bytes: u64,
}

pub struct ToksiConfigDb {
    conn: Connection,
    path: PathBuf,
}

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl ToksiConfigDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let p = path.as_ref().to_path_buf();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&p)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let mut db = Self { conn, path: p };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut db = Self {
            conn,
            path: PathBuf::from(":memory:"),
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn init_schema(&mut self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(SCHEMA_DDL)?;
        self.migrate_byok_profiles_columns()?;
        Ok(())
    }

    fn migrate_byok_profiles_columns(&mut self) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(byok_profiles)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .collect();
        drop(stmt);

        if !cols.iter().any(|c| c == "auth_mode") {
            self.conn.execute(
                "ALTER TABLE byok_profiles ADD COLUMN auth_mode TEXT NOT NULL DEFAULT 'api_key'",
                [],
            )?;
        }
        if !cols.iter().any(|c| c == "gcp_project") {
            self.conn.execute(
                "ALTER TABLE byok_profiles ADD COLUMN gcp_project TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
        if !cols.iter().any(|c| c == "gcp_location") {
            self.conn.execute(
                "ALTER TABLE byok_profiles ADD COLUMN gcp_location TEXT NOT NULL DEFAULT 'us-central1'",
                [],
            )?;
        }
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // -------------------------------------------------------------
    // Providers
    // -------------------------------------------------------------
    pub fn upsert_provider(&mut self, p: &ProviderRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO providers (id, name, protocol, base_url, is_enabled, auth_type, metadata_json, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
               ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   protocol = excluded.protocol,
                   base_url = excluded.base_url,
                   is_enabled = excluded.is_enabled,
                   auth_type = excluded.auth_type,
                   metadata_json = excluded.metadata_json,
                   updated_at = excluded.updated_at"#,
            params![
                p.id,
                p.name,
                p.protocol,
                p.base_url,
                p.is_enabled as i32,
                p.auth_type,
                p.metadata_json,
                p.created_at,
                p.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_provider(&self, id: &str) -> Result<Option<ProviderRecord>, rusqlite::Error> {
        self.conn
            .query_row(
                "SELECT id, name, protocol, base_url, is_enabled, auth_type, metadata_json, created_at, updated_at FROM providers WHERE id = ?1",
                params![id],
                |row| {
                    Ok(ProviderRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: row.get(2)?,
                        base_url: row.get(3)?,
                        is_enabled: row.get::<_, i32>(4)? != 0,
                        auth_type: row.get(5)?,
                        metadata_json: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_providers(&self) -> Result<Vec<ProviderRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, protocol, base_url, is_enabled, auth_type, metadata_json, created_at, updated_at FROM providers ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProviderRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                protocol: row.get(2)?,
                base_url: row.get(3)?,
                is_enabled: row.get::<_, i32>(4)? != 0,
                auth_type: row.get(5)?,
                metadata_json: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_provider(&mut self, id: &str) -> Result<bool, rusqlite::Error> {
        let n = self.conn.execute("DELETE FROM providers WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    // -------------------------------------------------------------
    // Provider Keys
    // -------------------------------------------------------------
    pub fn upsert_key(&mut self, k: &ProviderKeyRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO provider_keys (id, provider_id, key_alias, secret_ref, is_active, priority, weight, last_status, expires_at, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
               ON CONFLICT(id) DO UPDATE SET
                   provider_id = excluded.provider_id,
                   key_alias = excluded.key_alias,
                   secret_ref = excluded.secret_ref,
                   is_active = excluded.is_active,
                   priority = excluded.priority,
                   weight = excluded.weight,
                   last_status = excluded.last_status,
                   expires_at = excluded.expires_at"#,
            params![
                k.id,
                k.provider_id,
                k.key_alias,
                k.secret_ref,
                k.is_active as i32,
                k.priority,
                k.weight,
                k.last_status,
                k.expires_at,
                k.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_keys_for_provider(&self, provider_id: &str) -> Result<Vec<ProviderKeyRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider_id, key_alias, secret_ref, is_active, priority, weight, last_status, expires_at, created_at
             FROM provider_keys WHERE provider_id = ?1 ORDER BY priority DESC, weight DESC",
        )?;
        let rows = stmt.query_map(params![provider_id], |row| {
            Ok(ProviderKeyRecord {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                key_alias: row.get(2)?,
                secret_ref: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                priority: row.get(5)?,
                weight: row.get(6)?,
                last_status: row.get(7)?,
                expires_at: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------
    // BYOK Routes
    // -------------------------------------------------------------
    pub fn upsert_byok_route(&mut self, r: &ByokRouteRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO byok_routes (id, token, protocol, upstream, provider_id, is_active, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
               ON CONFLICT(id) DO UPDATE SET
                   token = excluded.token,
                   protocol = excluded.protocol,
                   upstream = excluded.upstream,
                   provider_id = excluded.provider_id,
                   is_active = excluded.is_active,
                   updated_at = excluded.updated_at"#,
            params![
                r.id,
                r.token,
                r.protocol,
                r.upstream,
                r.provider_id,
                r.is_active as i32,
                r.created_at,
                r.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_byok_route(&self, id: &str) -> Result<Option<ByokRouteRecord>, rusqlite::Error> {
        self.conn
            .query_row(
                "SELECT id, token, protocol, upstream, provider_id, is_active, created_at, updated_at FROM byok_routes WHERE id = ?1",
                params![id],
                |row| {
                    Ok(ByokRouteRecord {
                        id: row.get(0)?,
                        token: row.get(1)?,
                        protocol: row.get(2)?,
                        upstream: row.get(3)?,
                        provider_id: row.get(4)?,
                        is_active: row.get::<_, i32>(5)? != 0,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_byok_routes(&self) -> Result<Vec<ByokRouteRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, token, protocol, upstream, provider_id, is_active, created_at, updated_at FROM byok_routes ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ByokRouteRecord {
                id: row.get(0)?,
                token: row.get(1)?,
                protocol: row.get(2)?,
                upstream: row.get(3)?,
                provider_id: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_byok_route(&mut self, id: &str) -> Result<bool, rusqlite::Error> {
        let n = self.conn.execute("DELETE FROM byok_routes WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    // -------------------------------------------------------------
    // Agent Configs
    // -------------------------------------------------------------
    pub fn upsert_agent_config(&mut self, a: &AgentConfigRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO agent_configs (agent_id, provider_id, keyring_service, keyring_account, upstream_route_id, config_json, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(agent_id) DO UPDATE SET
                   provider_id = excluded.provider_id,
                   keyring_service = excluded.keyring_service,
                   keyring_account = excluded.keyring_account,
                   upstream_route_id = excluded.upstream_route_id,
                   config_json = excluded.config_json,
                   updated_at = excluded.updated_at"#,
            params![
                a.agent_id,
                a.provider_id,
                a.keyring_service,
                a.keyring_account,
                a.upstream_route_id,
                a.config_json,
                a.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_agent_config(&self, agent_id: &str) -> Result<Option<AgentConfigRecord>, rusqlite::Error> {
        self.conn
            .query_row(
                "SELECT agent_id, provider_id, keyring_service, keyring_account, upstream_route_id, config_json, updated_at FROM agent_configs WHERE agent_id = ?1",
                params![agent_id],
                |row| {
                    Ok(AgentConfigRecord {
                        agent_id: row.get(0)?,
                        provider_id: row.get(1)?,
                        keyring_service: row.get(2)?,
                        keyring_account: row.get(3)?,
                        upstream_route_id: row.get(4)?,
                        config_json: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_agent_configs(&self) -> Result<Vec<AgentConfigRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, provider_id, keyring_service, keyring_account, upstream_route_id, config_json, updated_at FROM agent_configs ORDER BY agent_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AgentConfigRecord {
                agent_id: row.get(0)?,
                provider_id: row.get(1)?,
                keyring_service: row.get(2)?,
                keyring_account: row.get(3)?,
                upstream_route_id: row.get(4)?,
                config_json: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------
    // Gateway Configs
    // -------------------------------------------------------------
    pub fn set_gateway_config(&mut self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        let now = now_epoch_secs();
        self.conn.execute(
            r#"INSERT INTO gateway_configs (key, value, updated_at) VALUES (?1, ?2, ?3)
               ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"#,
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_gateway_config(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        self.conn
            .query_row("SELECT value FROM gateway_configs WHERE key = ?1", params![key], |r| r.get(0))
            .optional()
    }

    pub fn list_gateway_configs(&self) -> Result<Vec<GatewayConfigRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT key, value, updated_at FROM gateway_configs ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok(GatewayConfigRecord {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------
    // Provider Models
    // -------------------------------------------------------------
    pub fn upsert_provider_model(&mut self, m: &ProviderModelRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO provider_models (id, provider_id, display_name, context_window, max_output_tokens, supports_streaming, supports_tools, is_default, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
               ON CONFLICT(id) DO UPDATE SET
                   provider_id = excluded.provider_id,
                   display_name = excluded.display_name,
                   context_window = excluded.context_window,
                   max_output_tokens = excluded.max_output_tokens,
                   supports_streaming = excluded.supports_streaming,
                   supports_tools = excluded.supports_tools,
                   is_default = excluded.is_default"#,
            params![
                m.id,
                m.provider_id,
                m.display_name,
                m.context_window,
                m.max_output_tokens,
                m.supports_streaming as i32,
                m.supports_tools as i32,
                m.is_default as i32,
                m.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_provider_models(&self, provider_id: Option<&str>) -> Result<Vec<ProviderModelRecord>, rusqlite::Error> {
        let sql = match provider_id {
            Some(_) => "SELECT id, provider_id, display_name, context_window, max_output_tokens, supports_streaming, supports_tools, is_default, created_at FROM provider_models WHERE provider_id = ?1 ORDER BY id",
            None => "SELECT id, provider_id, display_name, context_window, max_output_tokens, supports_streaming, supports_tools, is_default, created_at FROM provider_models ORDER BY provider_id, id",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let map_row = |row: &rusqlite::Row<'_>| -> Result<ProviderModelRecord, rusqlite::Error> {
            Ok(ProviderModelRecord {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                display_name: row.get(2)?,
                context_window: row.get(3)?,
                max_output_tokens: row.get(4)?,
                supports_streaming: row.get::<_, i32>(5)? != 0,
                supports_tools: row.get::<_, i32>(6)? != 0,
                is_default: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
            })
        };
        let mut list = Vec::new();
        if let Some(pid) = provider_id {
            for r in stmt.query_map(params![pid], map_row)? {
                list.push(r?);
            }
        } else {
            for r in stmt.query_map([], map_row)? {
                list.push(r?);
            }
        }
        Ok(list)
    }

    // -------------------------------------------------------------
    // Multi-Instance BYOK Profiles & Read-Only Quota Windows
    // -------------------------------------------------------------
    pub fn upsert_byok_profile(&mut self, p: &ByokProfileRecord) -> Result<(), rusqlite::Error> {
        let auth_mode = if p.auth_mode.trim().is_empty() {
            "api_key"
        } else {
            p.auth_mode.trim()
        };
        let gcp_location = if p.gcp_location.trim().is_empty() {
            "us-central1"
        } else {
            p.gcp_location.trim()
        };
        self.conn.execute(
            r#"INSERT INTO byok_profiles (id, provider_id, instance_label, model_id, thinking_effort, base_url, secret_ref, auth_mode, gcp_project, gcp_location, priority, weight, is_active, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
               ON CONFLICT(provider_id, instance_label, model_id, thinking_effort) DO UPDATE SET
                   base_url = excluded.base_url,
                   secret_ref = excluded.secret_ref,
                   auth_mode = excluded.auth_mode,
                   gcp_project = excluded.gcp_project,
                   gcp_location = excluded.gcp_location,
                   priority = excluded.priority,
                   weight = excluded.weight,
                   is_active = excluded.is_active,
                   updated_at = excluded.updated_at"#,
            params![
                p.id,
                p.provider_id,
                p.instance_label,
                p.model_id,
                p.thinking_effort,
                p.base_url,
                p.secret_ref,
                auth_mode,
                p.gcp_project,
                gcp_location,
                p.priority,
                p.weight,
                p.is_active as i32,
                p.created_at,
                p.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_byok_profiles(&self) -> Result<Vec<ByokProfileRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider_id, instance_label, model_id, thinking_effort, base_url, secret_ref, auth_mode, gcp_project, gcp_location, priority, weight, is_active, created_at, updated_at FROM byok_profiles ORDER BY priority DESC, provider_id, instance_label, model_id",
        )?;
        let rows = stmt.query_map([], |row| {
            let active: i32 = row.get(12)?;
            Ok(ByokProfileRecord {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                instance_label: row.get(2)?,
                model_id: row.get(3)?,
                thinking_effort: row.get(4)?,
                base_url: row.get(5)?,
                secret_ref: row.get(6)?,
                auth_mode: row.get(7)?,
                gcp_project: row.get(8)?,
                gcp_location: row.get(9)?,
                priority: row.get(10)?,
                weight: row.get(11)?,
                is_active: active != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn upsert_agent_profile_binding(&mut self, b: &AgentProfileBindingRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO agent_profile_bindings (agent_id, byok_profile_id, is_primary, fallback_order, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5)
               ON CONFLICT(agent_id, byok_profile_id) DO UPDATE SET
                   is_primary = excluded.is_primary,
                   fallback_order = excluded.fallback_order,
                   updated_at = excluded.updated_at"#,
            params![b.agent_id, b.byok_profile_id, b.is_primary as i32, b.fallback_order, b.updated_at],
        )?;
        Ok(())
    }

    pub fn list_agent_profile_bindings(&self) -> Result<Vec<AgentProfileBindingRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, byok_profile_id, is_primary, fallback_order, updated_at FROM agent_profile_bindings ORDER BY agent_id, fallback_order",
        )?;
        let rows = stmt.query_map([], |row| {
            let primary: i32 = row.get(2)?;
            Ok(AgentProfileBindingRecord {
                agent_id: row.get(0)?,
                byok_profile_id: row.get(1)?,
                is_primary: primary != 0,
                fallback_order: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn upsert_quota_window(&mut self, q: &QuotaWindowRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO quota_windows (id, provider_id, instance_label, window_kind, limit_requests, remaining_requests, limit_tokens, remaining_tokens, reset_at_epoch, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
               ON CONFLICT(provider_id, instance_label, window_kind) DO UPDATE SET
                   limit_requests = COALESCE(excluded.limit_requests, quota_windows.limit_requests),
                   remaining_requests = COALESCE(excluded.remaining_requests, quota_windows.remaining_requests),
                   limit_tokens = COALESCE(excluded.limit_tokens, quota_windows.limit_tokens),
                   remaining_tokens = COALESCE(excluded.remaining_tokens, quota_windows.remaining_tokens),
                   reset_at_epoch = COALESCE(excluded.reset_at_epoch, quota_windows.reset_at_epoch),
                   updated_at = excluded.updated_at"#,
            params![
                q.id,
                q.provider_id,
                q.instance_label,
                q.window_kind,
                q.limit_requests,
                q.remaining_requests,
                q.limit_tokens,
                q.remaining_tokens,
                q.reset_at_epoch,
                q.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_quota_windows(&self, window_kind: Option<&str>) -> Result<Vec<QuotaWindowRecord>, rusqlite::Error> {
        let mut list = Vec::new();
        let sql = match window_kind {
            Some(_) => "SELECT id, provider_id, instance_label, window_kind, limit_requests, remaining_requests, limit_tokens, remaining_tokens, reset_at_epoch, updated_at FROM quota_windows WHERE window_kind = ?1 ORDER BY provider_id, instance_label",
            None => "SELECT id, provider_id, instance_label, window_kind, limit_requests, remaining_requests, limit_tokens, remaining_tokens, reset_at_epoch, updated_at FROM quota_windows ORDER BY provider_id, instance_label, window_kind",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let map_row = |row: &rusqlite::Row<'_>| -> Result<QuotaWindowRecord, rusqlite::Error> {
            Ok(QuotaWindowRecord {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                instance_label: row.get(2)?,
                window_kind: row.get(3)?,
                limit_requests: row.get(4)?,
                remaining_requests: row.get(5)?,
                limit_tokens: row.get(6)?,
                remaining_tokens: row.get(7)?,
                reset_at_epoch: row.get(8)?,
                updated_at: row.get(9)?,
            })
        };
        if let Some(kind) = window_kind {
            let rows = stmt.query_map(params![kind], map_row)?;
            for r in rows {
                list.push(r?);
            }
        } else {
            let rows = stmt.query_map([], map_row)?;
            for r in rows {
                list.push(r?);
            }
        }
        Ok(list)
    }

    // -------------------------------------------------------------
    // Wave 18: Plugins Registry, Blind Signatures, Advisory Embargoes, ADR Audit
    // -------------------------------------------------------------
    pub fn upsert_plugin(&mut self, p: &PluginRegistryRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO plugins_registry (id, name, version, manifest_json, blake3_hash, capabilities_json, protocols_json, security_scan_status, is_enabled, installed_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
               ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   version = excluded.version,
                   manifest_json = excluded.manifest_json,
                   blake3_hash = excluded.blake3_hash,
                   capabilities_json = excluded.capabilities_json,
                   protocols_json = excluded.protocols_json,
                   security_scan_status = excluded.security_scan_status,
                   is_enabled = excluded.is_enabled,
                   updated_at = excluded.updated_at"#,
            params![
                p.id,
                p.name,
                p.version,
                p.manifest_json,
                p.blake3_hash,
                p.capabilities_json,
                p.protocols_json,
                p.security_scan_status,
                p.is_enabled as i32,
                p.installed_at,
                p.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_plugins(&self) -> Result<Vec<PluginRegistryRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, manifest_json, blake3_hash, capabilities_json, protocols_json, security_scan_status, is_enabled, installed_at, updated_at FROM plugins_registry ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PluginRegistryRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                manifest_json: row.get(3)?,
                blake3_hash: row.get(4)?,
                capabilities_json: row.get(5)?,
                protocols_json: row.get(6)?,
                security_scan_status: row.get(7)?,
                is_enabled: row.get::<_, i32>(8)? != 0,
                installed_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Validates that only blind hashes/DFAs are stored (never raw exploit PoCs or regexes).
    fn validate_blind_signature(sig: &BlindSignatureRecord) -> Result<(), rusqlite::Error> {
        let valid_kind = matches!(
            sig.signature_kind.as_str(),
            "blake3_ngram" | "sparse_dfa_b64"
        );
        if !valid_kind {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "invalid blind signature kind '{}'",
                sig.signature_kind
            )));
        }
        let payload = sig.signature_payload.trim();
        let is_safe_payload = match sig.signature_kind.as_str() {
            "blake3_ngram" => {
                !payload.is_empty() && payload.bytes().all(|b| b.is_ascii_hexdigit())
            }
            "sparse_dfa_b64" => {
                !payload.is_empty()
                    && payload
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'-' | b'_' | b'='))
            }
            _ => false,
        };
        if !is_safe_payload {
            return Err(rusqlite::Error::InvalidParameterName(
                "blind_signatures rejects raw PoCs or regex payloads".to_string(),
            ));
        }
        Ok(())
    }

    pub fn upsert_blind_signature(&mut self, s: &BlindSignatureRecord) -> Result<(), rusqlite::Error> {
        Self::validate_blind_signature(s)?;
        self.conn.execute(
            r#"INSERT INTO blind_signatures (id, signature_kind, signature_payload, severity, language, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)
               ON CONFLICT(id) DO UPDATE SET
                   signature_kind = excluded.signature_kind,
                   signature_payload = excluded.signature_payload,
                   severity = excluded.severity,
                   language = excluded.language"#,
            params![
                s.id,
                s.signature_kind,
                s.signature_payload.trim(),
                s.severity,
                s.language,
                s.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_blind_signatures(&self) -> Result<Vec<BlindSignatureRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, signature_kind, signature_payload, severity, language, created_at FROM blind_signatures ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BlindSignatureRecord {
                id: row.get(0)?,
                signature_kind: row.get(1)?,
                signature_payload: row.get(2)?,
                severity: row.get(3)?,
                language: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn upsert_advisory_embargo(&mut self, e: &AdvisoryEmbargoRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO advisory_embargoes (advisory_id, disclosure_state, embargo_until_epoch, upstream_repo, pr_url, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)
               ON CONFLICT(advisory_id) DO UPDATE SET
                   disclosure_state = excluded.disclosure_state,
                   embargo_until_epoch = excluded.embargo_until_epoch,
                   upstream_repo = excluded.upstream_repo,
                   pr_url = excluded.pr_url,
                   updated_at = excluded.updated_at"#,
            params![
                e.advisory_id,
                e.disclosure_state,
                e.embargo_until_epoch,
                e.upstream_repo,
                e.pr_url,
                e.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_advisory_embargoes(&self) -> Result<Vec<AdvisoryEmbargoRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT advisory_id, disclosure_state, embargo_until_epoch, upstream_repo, pr_url, updated_at FROM advisory_embargoes ORDER BY advisory_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AdvisoryEmbargoRecord {
                advisory_id: row.get(0)?,
                disclosure_state: row.get(1)?,
                embargo_until_epoch: row.get(2)?,
                upstream_repo: row.get(3)?,
                pr_url: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn insert_adr_audit_event(&mut self, ev: &AdrAuditEventRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"INSERT INTO adr_audit_events (id, agent_id, action, decision, reason, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)
               ON CONFLICT(id) DO UPDATE SET
                   agent_id = excluded.agent_id,
                   action = excluded.action,
                   decision = excluded.decision,
                   reason = excluded.reason"#,
            params![ev.id, ev.agent_id, ev.action, ev.decision, ev.reason, ev.created_at],
        )?;
        Ok(())
    }

    pub fn list_adr_audit_events(&self) -> Result<Vec<AdrAuditEventRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, action, decision, reason, created_at FROM adr_audit_events ORDER BY created_at DESC, id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AdrAuditEventRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                action: row.get(2)?,
                decision: row.get(3)?,
                reason: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------
    // Export / Import / Backup
    // -------------------------------------------------------------
    pub fn export_dump(&self) -> Result<ToksiDbDump, rusqlite::Error> {
        let providers = self.list_providers()?;
        let mut provider_keys = Vec::new();
        for p in &providers {
            if let Ok(keys) = self.list_keys_for_provider(&p.id) {
                provider_keys.extend(keys);
            }
        }
        let provider_models = self.list_provider_models(None)?;
        let byok_routes = self.list_byok_routes()?;
        let agent_configs = self.list_agent_configs()?;
        let gateway_configs = self.list_gateway_configs()?;
        let byok_profiles = self.list_byok_profiles()?;
        let agent_profile_bindings = self.list_agent_profile_bindings()?;
        let quota_windows = self.list_quota_windows(None)?;
        let plugins_registry = self.list_plugins()?;
        let blind_signatures = self.list_blind_signatures()?;
        let advisory_embargoes = self.list_advisory_embargoes()?;
        let adr_audit_events = self.list_adr_audit_events()?;

        Ok(ToksiDbDump {
            version: 2,
            exported_at: now_epoch_secs(),
            providers,
            provider_keys,
            provider_models,
            byok_routes,
            agent_configs,
            gateway_configs,
            byok_profiles,
            agent_profile_bindings,
            quota_windows,
            plugins_registry,
            blind_signatures,
            advisory_embargoes,
            adr_audit_events,
        })
    }

    pub fn export_json(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let dump = self.export_dump()?;
        Ok(serde_json::to_string_pretty(&dump)?)
    }

    pub fn import_dump(&mut self, dump: &ToksiDbDump, overwrite: bool) -> Result<usize, rusqlite::Error> {
        for s in &dump.blind_signatures {
            Self::validate_blind_signature(s)?;
        }

        let tx = self.conn.transaction()?;
        let mut count = 0;

        for p in &dump.providers {
            let sql = if overwrite {
                r#"INSERT INTO providers (id, name, protocol, base_url, is_enabled, auth_type, metadata_json, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                   ON CONFLICT(id) DO UPDATE SET
                       name = excluded.name, protocol = excluded.protocol, base_url = excluded.base_url,
                       is_enabled = excluded.is_enabled, auth_type = excluded.auth_type,
                       metadata_json = excluded.metadata_json, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO providers (id, name, protocol, base_url, is_enabled, auth_type, metadata_json, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#
            };
            count += tx.execute(
                sql,
                params![p.id, p.name, p.protocol, p.base_url, p.is_enabled as i32, p.auth_type, p.metadata_json, p.created_at, p.updated_at],
            )?;
        }

        for k in &dump.provider_keys {
            let sql = if overwrite {
                r#"INSERT INTO provider_keys (id, provider_id, key_alias, secret_ref, is_active, priority, weight, last_status, expires_at, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                   ON CONFLICT(id) DO UPDATE SET
                       provider_id = excluded.provider_id, key_alias = excluded.key_alias, secret_ref = excluded.secret_ref,
                       is_active = excluded.is_active, priority = excluded.priority, weight = excluded.weight,
                       last_status = excluded.last_status, expires_at = excluded.expires_at"#
            } else {
                r#"INSERT OR IGNORE INTO provider_keys (id, provider_id, key_alias, secret_ref, is_active, priority, weight, last_status, expires_at, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#
            };
            count += tx.execute(
                sql,
                params![k.id, k.provider_id, k.key_alias, k.secret_ref, k.is_active as i32, k.priority, k.weight, k.last_status, k.expires_at, k.created_at],
            )?;
        }

        for m in &dump.provider_models {
            let sql = if overwrite {
                r#"INSERT INTO provider_models (id, provider_id, display_name, context_window, max_output_tokens, supports_streaming, supports_tools, is_default, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                   ON CONFLICT(id) DO UPDATE SET
                       provider_id = excluded.provider_id, display_name = excluded.display_name,
                       context_window = excluded.context_window, max_output_tokens = excluded.max_output_tokens,
                       supports_streaming = excluded.supports_streaming, supports_tools = excluded.supports_tools,
                       is_default = excluded.is_default"#
            } else {
                r#"INSERT OR IGNORE INTO provider_models (id, provider_id, display_name, context_window, max_output_tokens, supports_streaming, supports_tools, is_default, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#
            };
            count += tx.execute(
                sql,
                params![m.id, m.provider_id, m.display_name, m.context_window, m.max_output_tokens, m.supports_streaming as i32, m.supports_tools as i32, m.is_default as i32, m.created_at],
            )?;
        }

        for r in &dump.byok_routes {
            let sql = if overwrite {
                r#"INSERT INTO byok_routes (id, token, protocol, upstream, provider_id, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                   ON CONFLICT(id) DO UPDATE SET
                       token = excluded.token, protocol = excluded.protocol, upstream = excluded.upstream,
                       provider_id = excluded.provider_id, is_active = excluded.is_active, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO byok_routes (id, token, protocol, upstream, provider_id, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#
            };
            count += tx.execute(
                sql,
                params![r.id, r.token, r.protocol, r.upstream, r.provider_id, r.is_active as i32, r.created_at, r.updated_at],
            )?;
        }

        for a in &dump.agent_configs {
            let sql = if overwrite {
                r#"INSERT INTO agent_configs (agent_id, provider_id, keyring_service, keyring_account, upstream_route_id, config_json, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                   ON CONFLICT(agent_id) DO UPDATE SET
                       provider_id = excluded.provider_id, keyring_service = excluded.keyring_service,
                       keyring_account = excluded.keyring_account, upstream_route_id = excluded.upstream_route_id,
                       config_json = excluded.config_json, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO agent_configs (agent_id, provider_id, keyring_service, keyring_account, upstream_route_id, config_json, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#
            };
            count += tx.execute(
                sql,
                params![a.agent_id, a.provider_id, a.keyring_service, a.keyring_account, a.upstream_route_id, a.config_json, a.updated_at],
            )?;
        }

        for g in &dump.gateway_configs {
            let sql = if overwrite {
                r#"INSERT INTO gateway_configs (key, value, updated_at) VALUES (?1, ?2, ?3)
                   ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO gateway_configs (key, value, updated_at) VALUES (?1, ?2, ?3)"#
            };
            count += tx.execute(sql, params![g.key, g.value, g.updated_at])?;
        }

        for bp in &dump.byok_profiles {
            let auth_mode = if bp.auth_mode.trim().is_empty() { "api_key" } else { bp.auth_mode.trim() };
            let gcp_location = if bp.gcp_location.trim().is_empty() { "us-central1" } else { bp.gcp_location.trim() };
            let sql = if overwrite {
                r#"INSERT INTO byok_profiles (id, provider_id, instance_label, model_id, thinking_effort, base_url, secret_ref, auth_mode, gcp_project, gcp_location, priority, weight, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                   ON CONFLICT(provider_id, instance_label, model_id, thinking_effort) DO UPDATE SET
                       base_url = excluded.base_url, secret_ref = excluded.secret_ref,
                       auth_mode = excluded.auth_mode, gcp_project = excluded.gcp_project, gcp_location = excluded.gcp_location,
                       priority = excluded.priority, weight = excluded.weight,
                       is_active = excluded.is_active, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO byok_profiles (id, provider_id, instance_label, model_id, thinking_effort, base_url, secret_ref, auth_mode, gcp_project, gcp_location, priority, weight, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"#
            };
            count += tx.execute(
                sql,
                params![bp.id, bp.provider_id, bp.instance_label, bp.model_id, bp.thinking_effort, bp.base_url, bp.secret_ref, auth_mode, bp.gcp_project, gcp_location, bp.priority, bp.weight, bp.is_active as i32, bp.created_at, bp.updated_at],
            )?;
        }

        for b in &dump.agent_profile_bindings {
            let sql = if overwrite {
                r#"INSERT INTO agent_profile_bindings (agent_id, byok_profile_id, is_primary, fallback_order, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5)
                   ON CONFLICT(agent_id, byok_profile_id) DO UPDATE SET
                       is_primary = excluded.is_primary, fallback_order = excluded.fallback_order, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO agent_profile_bindings (agent_id, byok_profile_id, is_primary, fallback_order, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5)"#
            };
            count += tx.execute(
                sql,
                params![b.agent_id, b.byok_profile_id, b.is_primary as i32, b.fallback_order, b.updated_at],
            )?;
        }

        for q in &dump.quota_windows {
            let sql = if overwrite {
                r#"INSERT INTO quota_windows (id, provider_id, instance_label, window_kind, limit_requests, remaining_requests, limit_tokens, remaining_tokens, reset_at_epoch, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                   ON CONFLICT(provider_id, instance_label, window_kind) DO UPDATE SET
                       limit_requests = COALESCE(excluded.limit_requests, quota_windows.limit_requests),
                       remaining_requests = COALESCE(excluded.remaining_requests, quota_windows.remaining_requests),
                       limit_tokens = COALESCE(excluded.limit_tokens, quota_windows.limit_tokens),
                       remaining_tokens = COALESCE(excluded.remaining_tokens, quota_windows.remaining_tokens),
                       reset_at_epoch = COALESCE(excluded.reset_at_epoch, quota_windows.reset_at_epoch),
                       updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO quota_windows (id, provider_id, instance_label, window_kind, limit_requests, remaining_requests, limit_tokens, remaining_tokens, reset_at_epoch, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#
            };
            count += tx.execute(
                sql,
                params![q.id, q.provider_id, q.instance_label, q.window_kind, q.limit_requests, q.remaining_requests, q.limit_tokens, q.remaining_tokens, q.reset_at_epoch, q.updated_at],
            )?;
        }

        for pl in &dump.plugins_registry {
            let sql = if overwrite {
                r#"INSERT INTO plugins_registry (id, name, version, manifest_json, blake3_hash, capabilities_json, protocols_json, security_scan_status, is_enabled, installed_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                   ON CONFLICT(id) DO UPDATE SET
                       name = excluded.name, version = excluded.version, manifest_json = excluded.manifest_json,
                       blake3_hash = excluded.blake3_hash, capabilities_json = excluded.capabilities_json,
                       protocols_json = excluded.protocols_json, security_scan_status = excluded.security_scan_status,
                       is_enabled = excluded.is_enabled, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO plugins_registry (id, name, version, manifest_json, blake3_hash, capabilities_json, protocols_json, security_scan_status, is_enabled, installed_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#
            };
            count += tx.execute(
                sql,
                params![pl.id, pl.name, pl.version, pl.manifest_json, pl.blake3_hash, pl.capabilities_json, pl.protocols_json, pl.security_scan_status, pl.is_enabled as i32, pl.installed_at, pl.updated_at],
            )?;
        }

        for sig in &dump.blind_signatures {
            let sql = if overwrite {
                r#"INSERT INTO blind_signatures (id, signature_kind, signature_payload, severity, language, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                   ON CONFLICT(id) DO UPDATE SET
                       signature_kind = excluded.signature_kind, signature_payload = excluded.signature_payload,
                       severity = excluded.severity, language = excluded.language"#
            } else {
                r#"INSERT OR IGNORE INTO blind_signatures (id, signature_kind, signature_payload, severity, language, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#
            };
            count += tx.execute(
                sql,
                params![sig.id, sig.signature_kind, sig.signature_payload.trim(), sig.severity, sig.language, sig.created_at],
            )?;
        }

        for adv in &dump.advisory_embargoes {
            let sql = if overwrite {
                r#"INSERT INTO advisory_embargoes (advisory_id, disclosure_state, embargo_until_epoch, upstream_repo, pr_url, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                   ON CONFLICT(advisory_id) DO UPDATE SET
                       disclosure_state = excluded.disclosure_state, embargo_until_epoch = excluded.embargo_until_epoch,
                       upstream_repo = excluded.upstream_repo, pr_url = excluded.pr_url, updated_at = excluded.updated_at"#
            } else {
                r#"INSERT OR IGNORE INTO advisory_embargoes (advisory_id, disclosure_state, embargo_until_epoch, upstream_repo, pr_url, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#
            };
            count += tx.execute(
                sql,
                params![adv.advisory_id, adv.disclosure_state, adv.embargo_until_epoch, adv.upstream_repo, adv.pr_url, adv.updated_at],
            )?;
        }

        for ev in &dump.adr_audit_events {
            let sql = if overwrite {
                r#"INSERT INTO adr_audit_events (id, agent_id, action, decision, reason, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                   ON CONFLICT(id) DO UPDATE SET
                       agent_id = excluded.agent_id, action = excluded.action,
                       decision = excluded.decision, reason = excluded.reason"#
            } else {
                r#"INSERT OR IGNORE INTO adr_audit_events (id, agent_id, action, decision, reason, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#
            };
            count += tx.execute(
                sql,
                params![ev.id, ev.agent_id, ev.action, ev.decision, ev.reason, ev.created_at],
            )?;
        }

        tx.commit()?;
        Ok(count)
    }

    pub fn import_json(&mut self, json_str: &str, overwrite: bool) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let dump: ToksiDbDump = serde_json::from_str(json_str)?;
        let n = self.import_dump(&dump, overwrite)?;
        Ok(n)
    }

    pub fn merge_from_file(&mut self, source_path: &Path) -> Result<usize, rusqlite::Error> {
        if !source_path.exists() {
            return Ok(0);
        }
        let source_db = ToksiConfigDb::open(source_path)?;
        let dump = source_db.export_dump()?;
        self.import_dump(&dump, true)
    }

    pub fn backup_to_file(&self, dest_path: &Path) -> Result<(), rusqlite::Error> {
        if let Some(parent) = dest_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if dest_path.exists() {
            let _ = std::fs::remove_file(dest_path);
        }
        let dest_str = crate::util::paths::normalize_path(&dest_path.to_string_lossy());
        self.conn.execute("VACUUM INTO ?1", params![dest_str])?;
        Ok(())
    }

    pub fn stats(&self) -> Result<DbStats, rusqlite::Error> {
        let providers_count: usize = self.conn.query_row("SELECT count(*) FROM providers", [], |r| r.get(0))?;
        let keys_count: usize = self.conn.query_row("SELECT count(*) FROM provider_keys", [], |r| r.get(0))?;
        let models_count: usize = self.conn.query_row("SELECT count(*) FROM provider_models", [], |r| r.get(0))?;
        let routes_count: usize = self.conn.query_row("SELECT count(*) FROM byok_routes", [], |r| r.get(0))?;
        let agents_count: usize = self.conn.query_row("SELECT count(*) FROM agent_configs", [], |r| r.get(0))?;
        let gateway_configs_count: usize = self.conn.query_row("SELECT count(*) FROM gateway_configs", [], |r| r.get(0))?;
        let byok_profiles_count: usize = self.conn.query_row("SELECT count(*) FROM byok_profiles", [], |r| r.get(0)).unwrap_or(0);
        let quota_windows_count: usize = self.conn.query_row("SELECT count(*) FROM quota_windows", [], |r| r.get(0)).unwrap_or(0);
        let plugins_count: usize = self.conn.query_row("SELECT count(*) FROM plugins_registry", [], |r| r.get(0)).unwrap_or(0);
        let blind_signatures_count: usize = self.conn.query_row("SELECT count(*) FROM blind_signatures", [], |r| r.get(0)).unwrap_or(0);
        let advisory_embargoes_count: usize = self.conn.query_row("SELECT count(*) FROM advisory_embargoes", [], |r| r.get(0)).unwrap_or(0);
        let adr_audit_events_count: usize = self.conn.query_row("SELECT count(*) FROM adr_audit_events", [], |r| r.get(0)).unwrap_or(0);

        let file_size_bytes = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);

        Ok(DbStats {
            db_path: crate::util::paths::normalize_path(&self.path.to_string_lossy()),
            providers_count,
            keys_count,
            models_count,
            routes_count,
            agents_count,
            gateway_configs_count,
            byok_profiles_count,
            quota_windows_count,
            plugins_count,
            blind_signatures_count,
            advisory_embargoes_count,
            adr_audit_events_count,
            file_size_bytes,
        })
    }
}

/// Merges records from another toksi.db into target_path row-by-row without data loss.
pub fn merge_toksi_db(target_path: &Path, source_path: &Path) -> Result<usize, rusqlite::Error> {
    if !source_path.exists() {
        return Ok(0);
    }
    if !target_path.exists() {
        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::copy(source_path, target_path).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        return Ok(1);
    }
    let mut target_db = ToksiConfigDb::open(target_path)?;
    let source_db = ToksiConfigDb::open(source_path)?;
    let dump = source_db.export_dump()?;
    target_db.import_dump(&dump, true)
}

/// Merges records from another memory.db (ICM) into target_path row-by-row without data loss.
pub fn merge_memory_db(target_path: &Path, source_path: &Path) -> Result<usize, rusqlite::Error> {
    merge_memory_db_with_subproject(target_path, source_path, "")
}

/// Merges records from a nested subproject memory.db into target_path, tagging memories with `subproject_id`.
pub fn merge_memory_db_with_subproject(
    target_path: &Path,
    source_path: &Path,
    subproject_id: &str,
) -> Result<usize, rusqlite::Error> {
    if !source_path.exists() {
        return Ok(0);
    }
    let clean_sub = crate::util::paths::normalize_path(subproject_id)
        .trim_matches('/')
        .to_string();
    if !target_path.exists() {
        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::copy(source_path, target_path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        if !clean_sub.is_empty() && clean_sub != "." {
            if let Ok(conn) = Connection::open(target_path) {
                let _ = conn.execute(
                    "UPDATE memories SET subproject_id = ?1 WHERE subproject_id = '' OR subproject_id = '.'",
                    params![clean_sub],
                );
            }
        }
        return Ok(1);
    }
    let target_conn = Connection::open(target_path)?;
    target_conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    let src_str = crate::util::paths::normalize_path(&source_path.to_string_lossy());
    target_conn.execute("ATTACH DATABASE ?1 AS src_mem", params![src_str])?;

    let mut rows_merged = 0;
    if let Ok(n) = target_conn.execute(
        "INSERT OR REPLACE INTO memories SELECT * FROM src_mem.memories",
        [],
    ) {
        rows_merged += n;
        if !clean_sub.is_empty() && clean_sub != "." {
            let _ = target_conn.execute(
                "UPDATE memories SET subproject_id = ?1 WHERE (subproject_id = '' OR subproject_id = '.') AND id IN (SELECT id FROM src_mem.memories)",
                params![clean_sub],
            );
        }
    }
    if let Ok(n) = target_conn.execute(
        "INSERT OR REPLACE INTO facts SELECT * FROM src_mem.facts",
        [],
    ) {
        rows_merged += n;
    }
    if let Ok(n) = target_conn.execute(
        "INSERT OR REPLACE INTO sessions SELECT * FROM src_mem.sessions",
        [],
    ) {
        rows_merged += n;
    }
    if let Ok(n) = target_conn.execute(
        "INSERT OR REPLACE INTO messages SELECT * FROM src_mem.messages",
        [],
    ) {
        rows_merged += n;
    }
    let _ = target_conn.execute("DETACH DATABASE src_mem", []);
    Ok(rows_merged)
}

/// Merges records from another symbols.db (CodeGraph) into target_path row-by-row without data loss.
pub fn merge_symbols_db(target_path: &Path, source_path: &Path) -> Result<usize, rusqlite::Error> {
    merge_symbols_db_with_subproject(target_path, source_path, ".")
}

/// Merges records from a nested subproject symbols.db into target_path, tagging rows with `subproject_id`.
pub fn merge_symbols_db_with_subproject(
    target_path: &Path,
    source_path: &Path,
    subproject_id: &str,
) -> Result<usize, rusqlite::Error> {
    if !source_path.exists() {
        return Ok(0);
    }
    let mut cache = crate::codegraph::SymbolCache::open(target_path)?;
    cache.merge_from_db(source_path, subproject_id)
}
