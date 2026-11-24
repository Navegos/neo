// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/service_dbs.rs
// created: 2026-11-24

//! Per-service telemetry SQLite stores (`~/.toksi/services/{gateway,headroom,quotas,siagent,cluster,otel}.db`),
//! isolated from configuration state in `~/.toksi/toksi.db`.

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::util::paths::{home, normalize_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceDbKind {
    Gateway,
    Headroom,
    Quotas,
    Siagent,
    Cluster,
    Otel,
}

impl ServiceDbKind {
    pub const ALL: [ServiceDbKind; 6] = [
        Self::Gateway,
        Self::Headroom,
        Self::Quotas,
        Self::Siagent,
        Self::Cluster,
        Self::Otel,
    ];

    pub const fn file_stem(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Headroom => "headroom",
            Self::Quotas => "quotas",
            Self::Siagent => "siagent",
            Self::Cluster => "cluster",
            Self::Otel => "otel",
        }
    }
}

/// Resolves per-service DB path under `TOKSI_DATA_DIR/services` or `~/.toksi/services/<service>.db`.
pub fn service_db_path(kind: ServiceDbKind) -> PathBuf {
    let fname = format!("{}.db", kind.file_stem());
    if let Ok(dir) = std::env::var("TOKSI_DATA_DIR")
        && !dir.trim().is_empty()
    {
        return PathBuf::from(normalize_path(dir.trim()))
            .join("services")
            .join(&fname);
    }
    let h = home();
    if !h.is_empty() {
        return PathBuf::from(normalize_path(&h))
            .join(".toksi")
            .join("services")
            .join(&fname);
    }
    PathBuf::from(normalize_path(&std::env::temp_dir().to_string_lossy()))
        .join("toksi-services")
        .join(&fname)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEventRecord {
    pub id: i64,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: i64,
}

pub struct ServiceTelemetryDb {
    pub kind: ServiceDbKind,
    conn: Connection,
    path: PathBuf,
}

impl ServiceTelemetryDb {
    pub fn open(kind: ServiceDbKind) -> Result<Self, rusqlite::Error> {
        let p = service_db_path(kind);
        Self::open_at(kind, &p)
    }

    pub fn open_at(kind: ServiceDbKind, path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS service_telemetry_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_srv_events_ts ON service_telemetry_events(created_at);
            CREATE TABLE IF NOT EXISTS service_kv_state (
                k TEXT PRIMARY KEY,
                v TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )?;
        Ok(Self {
            kind,
            conn,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record_event(
        &mut self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<i64, rusqlite::Error> {
        let ts = chrono::Utc::now().timestamp();
        let json_str = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
        self.conn.execute(
            "INSERT INTO service_telemetry_events (event_type, payload_json, created_at) VALUES (?1, ?2, ?3)",
            params![event_type, json_str, ts],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_recent_events(&self, limit: usize) -> Result<Vec<ServiceEventRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_type, payload_json, created_at FROM service_telemetry_events ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit.max(1) as i64], |r| {
            Ok(ServiceEventRecord {
                id: r.get(0)?,
                event_type: r.get(1)?,
                payload_json: r.get(2)?,
                created_at: r.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn event_count(&self) -> Result<usize, rusqlite::Error> {
        self.conn
            .query_row("SELECT count(*) FROM service_telemetry_events", [], |r| r.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_db_kind_paths_and_roundtrip() {
        assert_eq!(ServiceDbKind::ALL.len(), 6);
        let p = service_db_path(ServiceDbKind::Gateway);
        let s = p.to_string_lossy();
        assert!(s.ends_with("services/gateway.db") || s.ends_with("services\\gateway.db"));

        let tmp = tempfile::TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("services/otel.db");
        let mut db = ServiceTelemetryDb::open_at(ServiceDbKind::Otel, &db_path).expect("open");
        db.record_event("gen_ai.span", &serde_json::json!({"model": "gemini-2.5-pro"}))
            .expect("record");
        assert_eq!(db.event_count().expect("count"), 1);
        let events = db.list_recent_events(5).expect("list");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "gen_ai.span");
    }
}
