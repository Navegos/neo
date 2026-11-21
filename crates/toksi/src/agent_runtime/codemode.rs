// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/codemode.rs
// created: 2026-11-21

//! `opencode` (`codemode`, `http-recorder`, Drizzle/SQLite session state, `.opencode/hooks.json`)
//! synthesis for `<toksi-siagent>`.

#![deny(unsafe_code)]

use std::path::Path;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRecordedExchange {
    pub id: String,
    pub session_id: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub request_bytes: usize,
    pub response_bytes: usize,
    pub recorded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeHooksConfig {
    pub pre_tool_use: Vec<String>,
    pub post_tool_use: Vec<String>,
    pub session_start: Vec<String>,
}

impl Default for OpenCodeHooksConfig {
    fn default() -> Self {
        Self {
            pre_tool_use: vec!["rtk-hook opencode".to_string()],
            post_tool_use: vec!["projectmem-hook opencode".to_string()],
            session_start: vec!["codegraph-hook opencode".to_string()],
        }
    }
}

pub struct OpenCodeSessionDb {
    conn: Connection,
}

impl OpenCodeSessionDb {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("opencode mkdir failed: {e}"))?;
        }
        let conn = Connection::open(db_path).map_err(|e| format!("opencode db open failed: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS opencode_sessions (
                session_id TEXT PRIMARY KEY,
                mode TEXT NOT NULL,
                model_id TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS opencode_http_recorder (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER NOT NULL,
                request_bytes INTEGER NOT NULL,
                response_bytes INTEGER NOT NULL,
                recorded_at INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("opencode schema failed: {e}"))?;
        Ok(Self { conn })
    }

    pub fn upsert_session(&self, session_id: &str, mode: &str, model_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO opencode_sessions (session_id, mode, model_id, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![session_id, mode, model_id, chrono::Utc::now().timestamp()],
            )
            .map_err(|e| format!("opencode session insert failed: {e}"))?;
        Ok(())
    }

    pub fn record_http(&self, ex: &HttpRecordedExchange) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO opencode_http_recorder
                 (id, session_id, method, url, status, request_bytes, response_bytes, recorded_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    ex.id,
                    ex.session_id,
                    ex.method,
                    ex.url,
                    ex.status as i64,
                    ex.request_bytes as i64,
                    ex.response_bytes as i64,
                    ex.recorded_at,
                ],
            )
            .map_err(|e| format!("opencode http insert failed: {e}"))?;
        Ok(())
    }

    pub fn count_exchanges(&self, session_id: &str) -> Result<usize, String> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM opencode_http_recorder WHERE session_id = ?1",
                params![session_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c.max(0) as usize)
            .map_err(|e| format!("opencode count failed: {e}"))
    }
}
