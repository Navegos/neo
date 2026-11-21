// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/tokview.rs
// created: 2026-11-21

//! Local-first SQLite span & effort attribution (`Low` / `Medium` / `High`)
//! and live prompt/response compression diffs (`tokview` parity) surfaced on `:9098` & `:9100`.

#![deny(unsafe_code)]

use std::path::Path;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::headroom::effort::EffortLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokviewTurnRecord {
    pub turn_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub model_id: String,
    pub effort: EffortLevel,
    pub prompt_tokens: u64,
    pub tool_tokens: u64,
    pub context_tokens: u64,
    pub completion_tokens: u64,
    pub raw_bytes: usize,
    pub compressed_bytes: usize,
    pub prompt_diff_summary: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokviewEffortSummary {
    pub effort: String,
    pub turns: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_bytes_saved: u64,
}

pub struct TokviewStore {
    conn: Connection,
}

impl TokviewStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("tokview mkdir failed: {e}"))?;
        }
        let conn = Connection::open(db_path).map_err(|e| format!("tokview db open failed: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tokview_turns (
                turn_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                effort TEXT NOT NULL,
                prompt_tokens INTEGER NOT NULL,
                tool_tokens INTEGER NOT NULL,
                context_tokens INTEGER NOT NULL,
                completion_tokens INTEGER NOT NULL,
                raw_bytes INTEGER NOT NULL,
                compressed_bytes INTEGER NOT NULL,
                prompt_diff_summary TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("tokview schema failed: {e}"))?;
        Ok(Self { conn })
    }

    pub fn record_turn(&self, rec: &TokviewTurnRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO tokview_turns
                 (turn_id, session_id, agent_id, model_id, effort, prompt_tokens, tool_tokens,
                  context_tokens, completion_tokens, raw_bytes, compressed_bytes, prompt_diff_summary, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    rec.turn_id,
                    rec.session_id,
                    rec.agent_id,
                    rec.model_id,
                    rec.effort.as_str(),
                    rec.prompt_tokens as i64,
                    rec.tool_tokens as i64,
                    rec.context_tokens as i64,
                    rec.completion_tokens as i64,
                    rec.raw_bytes as i64,
                    rec.compressed_bytes as i64,
                    rec.prompt_diff_summary,
                    rec.created_at,
                ],
            )
            .map_err(|e| format!("tokview insert failed: {e}"))?;
        Ok(())
    }

    pub fn summarize_by_effort(&self) -> Result<Vec<TokviewEffortSummary>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT effort, COUNT(*), COALESCE(SUM(prompt_tokens), 0),
                        COALESCE(SUM(completion_tokens), 0),
                        COALESCE(SUM(MAX(0, raw_bytes - compressed_bytes)), 0)
                 FROM tokview_turns
                 GROUP BY effort
                 ORDER BY effort ASC",
            )
            .map_err(|e| format!("tokview query failed: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(TokviewEffortSummary {
                    effort: row.get(0)?,
                    turns: row.get::<_, i64>(1)? as u64,
                    total_prompt_tokens: row.get::<_, i64>(2)? as u64,
                    total_completion_tokens: row.get::<_, i64>(3)? as u64,
                    total_bytes_saved: row.get::<_, i64>(4)? as u64,
                })
            })
            .map_err(|e| format!("tokview map failed: {e}"))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| format!("tokview row failed: {e}"))?);
        }
        Ok(out)
    }
}
