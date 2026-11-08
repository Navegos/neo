// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/operations.rs
// created: 2026-11-04
// lastModified: 2026-11-04

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
        let byok_routes = self.list_byok_routes()?;
        let agent_configs = self.list_agent_configs()?;
        let gateway_configs = self.list_gateway_configs()?;

        Ok(ToksiDbDump {
            version: 1,
            exported_at: now_epoch_secs(),
            providers,
            provider_keys,
            provider_models: Vec::new(),
            byok_routes,
            agent_configs,
            gateway_configs,
        })
    }

    pub fn export_json(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let dump = self.export_dump()?;
        Ok(serde_json::to_string_pretty(&dump)?)
    }

    pub fn import_dump(&mut self, dump: &ToksiDbDump, _overwrite: bool) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        for p in &dump.providers {
            tx.execute(
                r#"INSERT INTO providers (id, name, protocol, base_url, is_enabled, auth_type, metadata_json, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                   ON CONFLICT(id) DO UPDATE SET
                       name = excluded.name, protocol = excluded.protocol, base_url = excluded.base_url,
                       is_enabled = excluded.is_enabled, auth_type = excluded.auth_type,
                       metadata_json = excluded.metadata_json, updated_at = excluded.updated_at"#,
                params![p.id, p.name, p.protocol, p.base_url, p.is_enabled as i32, p.auth_type, p.metadata_json, p.created_at, p.updated_at],
            )?;
            count += 1;
        }

        for k in &dump.provider_keys {
            tx.execute(
                r#"INSERT INTO provider_keys (id, provider_id, key_alias, secret_ref, is_active, priority, weight, last_status, expires_at, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                   ON CONFLICT(id) DO UPDATE SET
                       provider_id = excluded.provider_id, key_alias = excluded.key_alias, secret_ref = excluded.secret_ref,
                       is_active = excluded.is_active, priority = excluded.priority, weight = excluded.weight,
                       last_status = excluded.last_status, expires_at = excluded.expires_at"#,
                params![k.id, k.provider_id, k.key_alias, k.secret_ref, k.is_active as i32, k.priority, k.weight, k.last_status, k.expires_at, k.created_at],
            )?;
            count += 1;
        }

        for r in &dump.byok_routes {
            tx.execute(
                r#"INSERT INTO byok_routes (id, token, protocol, upstream, provider_id, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                   ON CONFLICT(id) DO UPDATE SET
                       token = excluded.token, protocol = excluded.protocol, upstream = excluded.upstream,
                       provider_id = excluded.provider_id, is_active = excluded.is_active, updated_at = excluded.updated_at"#,
                params![r.id, r.token, r.protocol, r.upstream, r.provider_id, r.is_active as i32, r.created_at, r.updated_at],
            )?;
            count += 1;
        }

        for a in &dump.agent_configs {
            tx.execute(
                r#"INSERT INTO agent_configs (agent_id, provider_id, keyring_service, keyring_account, upstream_route_id, config_json, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                   ON CONFLICT(agent_id) DO UPDATE SET
                       provider_id = excluded.provider_id, keyring_service = excluded.keyring_service,
                       keyring_account = excluded.keyring_account, upstream_route_id = excluded.upstream_route_id,
                       config_json = excluded.config_json, updated_at = excluded.updated_at"#,
                params![a.agent_id, a.provider_id, a.keyring_service, a.keyring_account, a.upstream_route_id, a.config_json, a.updated_at],
            )?;
            count += 1;
        }

        for g in &dump.gateway_configs {
            tx.execute(
                r#"INSERT INTO gateway_configs (key, value, updated_at) VALUES (?1, ?2, ?3)
                   ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"#,
                params![g.key, g.value, g.updated_at],
            )?;
            count += 1;
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
        let dest_str = dest_path.to_string_lossy().to_string();
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

        let file_size_bytes = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);

        Ok(DbStats {
            db_path: self.path.to_string_lossy().to_string(),
            providers_count,
            keys_count,
            models_count,
            routes_count,
            agents_count,
            gateway_configs_count,
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
    let target_conn = Connection::open(target_path)?;
    target_conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    let src_str = crate::util::paths::normalize_path(&source_path.to_string_lossy());
    target_conn.execute("ATTACH DATABASE ?1 AS src_sym", params![src_str])?;

    let mut rows_merged = 0;
    if let Ok(n) = target_conn.execute(
        "INSERT OR REPLACE INTO files SELECT * FROM src_sym.files",
        [],
    ) {
        rows_merged += n;
    }
    if let Ok(n) = target_conn.execute(
        "INSERT OR REPLACE INTO nodes SELECT * FROM src_sym.nodes",
        [],
    ) {
        rows_merged += n;
    }
    if let Ok(n) = target_conn.execute(
        "INSERT OR IGNORE INTO edges SELECT * FROM src_sym.edges",
        [],
    ) {
        rows_merged += n;
    }
    if let Ok(n) = target_conn.execute(
        "INSERT OR IGNORE INTO refs SELECT * FROM src_sym.refs",
        [],
    ) {
        rows_merged += n;
    }
    let _ = target_conn.execute("DETACH DATABASE src_sym", []);
    Ok(rows_merged)
}
