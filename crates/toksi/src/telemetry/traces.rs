// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/traces.rs
// created: 2026-11-21

//! Langfuse-compatible local trace, span, and prompt/response diff recorder.

#![deny(unsafe_code)]

use std::path::Path;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub input_preview: String,
    pub output_preview: String,
    pub latency_ms: u64,
    pub started_at: i64,
}

pub struct TraceStore {
    conn: Connection,
}

impl TraceStore {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("trace mkdir failed: {e}"))?;
        }
        let conn = Connection::open(db_path).map_err(|e| format!("trace db open failed: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS langfuse_spans (
                span_id TEXT PRIMARY KEY,
                trace_id TEXT NOT NULL,
                parent_span_id TEXT,
                name TEXT NOT NULL,
                model TEXT,
                effort TEXT,
                input_preview TEXT NOT NULL,
                output_preview TEXT NOT NULL,
                latency_ms INTEGER NOT NULL,
                started_at INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("trace schema failed: {e}"))?;
        Ok(Self { conn })
    }

    pub fn record_span(&self, span: &TraceSpan) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO langfuse_spans
                 (span_id, trace_id, parent_span_id, name, model, effort, input_preview, output_preview, latency_ms, started_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    span.span_id,
                    span.trace_id,
                    span.parent_span_id,
                    span.name,
                    span.model,
                    span.effort,
                    span.input_preview,
                    span.output_preview,
                    span.latency_ms as i64,
                    span.started_at,
                ],
            )
            .map_err(|e| format!("trace insert failed: {e}"))?;
        Ok(())
    }

    pub fn list_trace(&self, trace_id: &str) -> Result<Vec<TraceSpan>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT span_id, trace_id, parent_span_id, name, model, effort, input_preview, output_preview, latency_ms, started_at
                 FROM langfuse_spans WHERE trace_id = ?1 ORDER BY started_at ASC",
            )
            .map_err(|e| format!("trace query failed: {e}"))?;
        let rows = stmt
            .query_map(params![trace_id], |row| {
                Ok(TraceSpan {
                    span_id: row.get(0)?,
                    trace_id: row.get(1)?,
                    parent_span_id: row.get(2)?,
                    name: row.get(3)?,
                    model: row.get(4)?,
                    effort: row.get(5)?,
                    input_preview: row.get(6)?,
                    output_preview: row.get(7)?,
                    latency_ms: row.get::<_, i64>(8)? as u64,
                    started_at: row.get(9)?,
                })
            })
            .map_err(|e| format!("trace map failed: {e}"))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| format!("trace row failed: {e}"))?);
        }
        Ok(out)
    }
}
