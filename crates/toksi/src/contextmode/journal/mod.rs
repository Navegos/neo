// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/journal/mod.rs
// created: 2026-11-13

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalRow {
    pub seq: i64,
    pub session_id: String,
    pub worktree: String,
    pub agent: String,
    pub kind: String,
    pub tool: Option<String>,
    pub key: Option<String>,
    pub path: Option<String>,
    pub ok: Option<i32>,
    pub exit_code: Option<i32>,
    pub bytes: usize,
    pub body: String,
    pub turn: usize,
    pub sandbox_bytes: usize,
    pub returned_bytes: usize,
    pub created_at: i64,
    pub accessed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalRow {
    pub session_id: String,
    pub worktree: String,
    pub agent: String,
    pub kind: String,
    pub tool: Option<String>,
    pub key: Option<String>,
    pub path: Option<String>,
    pub ok: Option<i32>,
    pub exit_code: Option<i32>,
    pub bytes: usize,
    pub body: String,
    pub turn: usize,
    pub sandbox_bytes: usize,
    pub returned_bytes: usize,
}

/// Quote-aware command canonicalizer: folds whitespace runs outside quotes into a single space,
/// while keeping content inside quotes byte-identical.
pub fn canonical_key(tool: &str, code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut gap = false;

    for ch in code.chars() {
        if quote.is_none() && !escaped && ch.is_whitespace() {
            gap = !out.is_empty();
            continue;
        }
        if gap {
            out.push(' ');
            gap = false;
        }
        out.push(ch);
        if escaped {
            escaped = false;
        } else if ch == '\\' && quote != Some('\'') {
            escaped = true;
        } else if quote.is_none() && (ch == '\'' || ch == '"') {
            quote = Some(ch);
        } else if quote == Some(ch) {
            quote = None;
        }
    }

    format!("{}:{}", tool, out.trim())
}

/// Appends a new event row into the `events` table and returns its seq.
pub fn append_event(conn: &Connection, row: &NewJournalRow) -> Result<i64, rusqlite::Error> {
    let now = chrono::Utc::now().timestamp_millis();
    let is_cost = row.kind == "cost";

    let sql = if is_cost {
        "INSERT OR IGNORE INTO events (
            session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn,
            sandbox_bytes, returned_bytes, created_at, accessed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
    } else {
        "INSERT INTO events (
            session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn,
            sandbox_bytes, returned_bytes, created_at, accessed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
    };

    let mut stmt = conn.prepare(sql)?;
    let changes = stmt.execute(params![
        row.session_id,
        row.worktree,
        row.agent,
        row.kind,
        row.tool,
        row.key,
        row.path,
        row.ok,
        row.exit_code,
        row.bytes as i64,
        row.body,
        row.turn as i64,
        row.sandbox_bytes as i64,
        row.returned_bytes as i64,
        now,
        now,
    ])?;

    if changes == 1 {
        Ok(conn.last_insert_rowid())
    } else if is_cost {
        let kept_seq: i64 = conn.query_row(
            "SELECT seq FROM events WHERE kind = 'cost' AND session_id = ?1 AND turn = ?2",
            params![row.session_id, row.turn as i64],
            |r| r.get(0),
        )?;
        Ok(kept_seq)
    } else {
        Ok(conn.last_insert_rowid())
    }
}

fn map_journal_row(row: &rusqlite::Row) -> rusqlite::Result<JournalRow> {
    Ok(JournalRow {
        seq: row.get(0)?,
        session_id: row.get(1)?,
        worktree: row.get(2)?,
        agent: row.get(3)?,
        kind: row.get(4)?,
        tool: row.get(5)?,
        key: row.get(6)?,
        path: row.get(7)?,
        ok: row.get(8)?,
        exit_code: row.get(9)?,
        bytes: row.get::<_, i64>(10)? as usize,
        body: row.get(11)?,
        turn: row.get::<_, i64>(12)? as usize,
        sandbox_bytes: row.get::<_, i64>(13)? as usize,
        returned_bytes: row.get::<_, i64>(14)? as usize,
        created_at: row.get(15)?,
        accessed_at: row.get(16)?,
    })
}

/// Reads session events in ascending sequence order.
pub fn read_session_events(
    conn: &Connection,
    session_id: &str,
    kind: Option<&str>,
) -> Result<Vec<JournalRow>, rusqlite::Error> {
    if session_id.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    if let Some(k) = kind {
        let sql = "SELECT seq, session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn, sandbox_bytes, returned_bytes, created_at, accessed_at
                   FROM events WHERE session_id = ?1 AND kind = ?2 ORDER BY seq ASC";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![session_id, k], map_journal_row)?;
        for r in rows.flatten() {
            out.push(r);
        }
    } else {
        let sql = "SELECT seq, session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn, sandbox_bytes, returned_bytes, created_at, accessed_at
                   FROM events WHERE session_id = ?1 ORDER BY seq ASC";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![session_id], map_journal_row)?;
        for r in rows.flatten() {
            out.push(r);
        }
    }

    Ok(out)
}

/// Reads a specific event by primary key sequence number.
pub fn read_event_by_seq(conn: &Connection, seq: i64) -> Result<Option<JournalRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT seq, session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn, sandbox_bytes, returned_bytes, created_at, accessed_at
         FROM events WHERE seq = ?1",
    )?;

    stmt.query_row(params![seq], map_journal_row).optional()
}

/// Returns the newest user prompt body of a session.
pub fn read_last_prompt(conn: &Connection, session_id: &str) -> String {
    if session_id.is_empty() {
        return String::new();
    }
    conn.query_row(
        "SELECT body FROM events WHERE session_id = ?1 AND kind = 'prompt' ORDER BY seq DESC LIMIT 1",
        params![session_id],
        |r| r.get(0),
    ).unwrap_or_default()
}

/// Returns the first user prompt body of a session (session goal).
pub fn read_first_prompt(conn: &Connection, session_id: &str) -> String {
    if session_id.is_empty() {
        return String::new();
    }
    conn.query_row(
        "SELECT body FROM events WHERE session_id = ?1 AND kind = 'prompt' ORDER BY seq ASC LIMIT 1",
        params![session_id],
        |r| r.get(0),
    ).unwrap_or_default()
}

/// Reads candidates for recall: this session's events plus prior session events from the same worktree.
pub fn read_recall_rows(
    conn: &Connection,
    session_id: &str,
    worktree: &str,
    limit: usize,
) -> Result<Vec<JournalRow>, rusqlite::Error> {
    if session_id.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT seq, session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn, sandbox_bytes, returned_bytes, created_at, accessed_at
         FROM events WHERE kind <> 'cost' AND (session_id = ?1 OR worktree = ?2)
         ORDER BY seq DESC LIMIT ?3",
    )?;

    let rows = stmt.query_map(params![session_id, worktree, limit as i64], map_journal_row)?;

    let mut out = Vec::new();
    for r in rows.flatten() {
        out.push(r);
    }
    Ok(out)
}

pub fn has_agent_rows(conn: &Connection, worktree: &str, agent: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM events WHERE worktree = ?1 AND agent = ?2 LIMIT 1",
        params![worktree, agent],
        |_| Ok(()),
    ).is_ok()
}

pub fn has_session_rows(conn: &Connection, session_id: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM events WHERE session_id = ?1 LIMIT 1",
        params![session_id],
        |_| Ok(()),
    ).is_ok()
}
