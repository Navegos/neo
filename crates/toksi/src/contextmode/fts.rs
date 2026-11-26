// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/fts.rs
// created: 2026-10-27
// lastModified: 2026-11-18

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use ignore::WalkBuilder;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_CHUNK_BYTES: usize = 4096;
pub const CHUNK_TITLE_MAX_CHARS: usize = 80;

static STOPWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
        "her", "was", "one", "our", "out", "has", "his", "how", "its", "may",
        "new", "now", "old", "see", "way", "who", "did", "get", "got", "let",
        "say", "she", "too", "use", "will", "with", "this", "that", "from",
        "they", "been", "have", "many", "some", "them", "than", "each", "make",
        "like", "just", "over", "such", "take", "into", "year", "your", "good",
        "could", "would", "about", "which", "their", "there", "other", "after",
        "should", "through", "also", "more", "most", "only", "very", "when",
        "what", "then", "these", "those", "being", "does", "done", "both",
        "same", "still", "while", "where", "here", "were", "much",
        "update", "updates", "updated", "deps", "dev", "tests", "test",
        "add", "added", "fix", "fixed", "run", "running", "using",
    ]
    .into_iter()
    .collect()
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chunk {
    pub title: String,
    pub content: String,
    pub has_code: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    pub label: String,
    pub total_chunks: usize,
    pub code_chunks: usize,
    pub total_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryIndexResult {
    pub root: String,
    pub files_indexed: usize,
    pub total_chunks: usize,
    pub total_bytes: usize,
    pub capped: bool,
    pub errors: Vec<String>,
}

pub const DEFAULT_INDEX_EXTENSIONS: &[&str] = &[
    "md", "markdown", "txt", "json", "yaml", "yml", "toml",
    "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rs", "go",
    "sh", "bash", "ps1", "c", "cpp", "h", "hpp", "html", "css",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub timestamp: Option<String>,
    pub label: String,
    pub score: f64,
    pub highlighted: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
    pub source: Option<String>,
    pub content_type: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub id: i64,
    pub label: String,
    pub chunk_count: usize,
    pub code_chunk_count: usize,
    pub indexed_at: String,
    pub file_path: Option<String>,
    pub content_hash: Option<String>,
}

// ─────────────────────────────────────────────────────────
// Query Sanitizers
// ─────────────────────────────────────────────────────────

pub fn sanitize_query(query: &str, mode: &str) -> String {
    let cleaned = query.replace(['\'', '"', '(', ')', '{', '}', '[', ']', '*', ':', '^', '~'], " ");
    let mut words: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for w in cleaned.split_whitespace() {
        let upper = w.to_uppercase();
        if !w.is_empty() && upper != "AND" && upper != "OR" && upper != "NOT" && upper != "NEAR" {
            let lower = w.to_lowercase();
            if seen.insert(lower) {
                words.push(w.to_string());
            }
        }
    }

    if words.is_empty() {
        return "\"\"".to_string();
    }

    let meaningful: Vec<String> = words
        .iter()
        .filter(|w| !STOPWORDS.contains(w.to_lowercase().as_str()))
        .cloned()
        .collect();

    let final_words = if !meaningful.is_empty() { meaningful } else { words };
    let quoted: Vec<String> = final_words.into_iter().map(|w| format!("\"{}\"", w)).collect();

    let joiner = if mode.eq_ignore_ascii_case("OR") { " OR " } else { " " };
    quoted.join(joiner)
}

pub fn sanitize_trigram_query(query: &str, mode: &str) -> String {
    let cleaned = query.replace(['\'', '"', '(', ')', '{', '}', '[', ']', '*', ':', '^', '~'], " ");
    let mut words: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for w in cleaned.split_whitespace() {
        if w.len() >= 3 {
            let lower = w.to_lowercase();
            if seen.insert(lower) {
                words.push(w.to_string());
            }
        }
    }

    if words.is_empty() {
        return String::new();
    }

    let meaningful: Vec<String> = words
        .iter()
        .filter(|w| !STOPWORDS.contains(w.to_lowercase().as_str()))
        .cloned()
        .collect();

    let final_words = if !meaningful.is_empty() { meaningful } else { words };
    let quoted: Vec<String> = final_words.into_iter().map(|w| format!("\"{}\"", w)).collect();

    let joiner = if mode.eq_ignore_ascii_case("OR") { " OR " } else { " " };
    quoted.join(joiner)
}

// ─────────────────────────────────────────────────────────
// Chunking Algorithms
// ─────────────────────────────────────────────────────────

pub fn chunk_markdown(content: &str, max_chunk_bytes: usize) -> Vec<Chunk> {
    if content.trim().is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_title = "(root)".to_string();
    let mut current_content = String::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
        }

        let is_heading = !in_code_block && (trimmed.starts_with('#') || trimmed.starts_with("==") || trimmed.starts_with("--"));

        if is_heading && trimmed.starts_with('#') {
            if !current_content.trim().is_empty() {
                sub_split_and_push(&current_title, &current_content, max_chunk_bytes, &mut chunks);
                current_content.clear();
            }
            let h_text = trimmed.trim_start_matches('#').trim();
            current_title = if h_text.is_empty() {
                "(heading)".to_string()
            } else {
                h_text.chars().take(CHUNK_TITLE_MAX_CHARS).collect()
            };
        }

        current_content.push_str(line);
        current_content.push('\n');
    }

    if !current_content.trim().is_empty() {
        sub_split_and_push(&current_title, &current_content, max_chunk_bytes, &mut chunks);
    }

    chunks
}

fn sub_split_and_push(title: &str, content: &str, max_bytes: usize, out: &mut Vec<Chunk>) {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return;
    }

    if trimmed.len() <= max_bytes {
        let has_code = trimmed.contains("```");
        out.push(Chunk {
            title: title.to_string(),
            content: trimmed.to_string(),
            has_code,
        });
        return;
    }

    // Split on double newlines (paragraphs)
    let paragraphs: Vec<&str> = trimmed.split("\n\n").collect();
    let mut buf = String::new();
    let mut part_idx = 1;

    for p in paragraphs {
        let p_trimmed = p.trim();
        if p_trimmed.is_empty() {
            continue;
        }

        if buf.len() + p_trimmed.len() + 2 > max_bytes && !buf.is_empty() {
            let has_code = buf.contains("```");
            out.push(Chunk {
                title: format!("{} (part {})", title, part_idx),
                content: buf.trim().to_string(),
                has_code,
            });
            part_idx += 1;
            buf.clear();
        }

        if p_trimmed.len() > max_bytes {
            // Oversized paragraph: line-by-line split
            for line in p_trimmed.lines() {
                if buf.len() + line.len() + 1 > max_bytes && !buf.is_empty() {
                    let has_code = buf.contains("```");
                    out.push(Chunk {
                        title: format!("{} (part {})", title, part_idx),
                        content: buf.trim().to_string(),
                        has_code,
                    });
                    part_idx += 1;
                    buf.clear();
                }
                buf.push_str(line);
                buf.push('\n');
            }
        } else {
            buf.push_str(p_trimmed);
            buf.push_str("\n\n");
        }
    }

    if !buf.trim().is_empty() {
        let has_code = buf.contains("```");
        let t = if part_idx > 1 {
            format!("{} (part {})", title, part_idx)
        } else {
            title.to_string()
        };
        out.push(Chunk {
            title: t,
            content: buf.trim().to_string(),
            has_code,
        });
    }
}

pub fn chunk_plain_text(content: &str, max_chunk_bytes: usize) -> Vec<Chunk> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let raw_sections: Vec<&str> = trimmed.split("\n\n").filter(|s| !s.trim().is_empty()).collect();
    let is_natural_sections = raw_sections.len() >= 3
        && raw_sections.len() <= 200
        && raw_sections.iter().all(|s| s.len() <= 5000);

    let mut chunks = Vec::new();

    if is_natural_sections {
        for (i, sec) in raw_sections.iter().enumerate() {
            let first_line = sec.lines().next().unwrap_or("").trim();
            let title = if first_line.is_empty() {
                format!("Section {}", i + 1)
            } else {
                first_line.chars().take(CHUNK_TITLE_MAX_CHARS).collect()
            };
            sub_split_and_push(&title, sec, max_chunk_bytes, &mut chunks);
        }
    } else {
        // Line grouping up to max_chunk_bytes
        let mut buf = String::new();
        let mut first_line = String::new();
        let mut part = 1;

        for line in trimmed.lines() {
            if first_line.is_empty() && !line.trim().is_empty() {
                first_line = line.trim().chars().take(CHUNK_TITLE_MAX_CHARS).collect();
            }

            if buf.len() + line.len() + 1 > max_chunk_bytes && !buf.is_empty() {
                let title = if first_line.is_empty() {
                    format!("Chunk {}", part)
                } else {
                    first_line.clone()
                };
                chunks.push(Chunk {
                    title,
                    content: buf.trim().to_string(),
                    has_code: false,
                });
                part += 1;
                buf.clear();
                first_line.clear();
            }
            buf.push_str(line);
            buf.push('\n');
        }

        if !buf.trim().is_empty() {
            let title = if first_line.is_empty() {
                format!("Chunk {}", part)
            } else {
                first_line
            };
            chunks.push(Chunk {
                title,
                content: buf.trim().to_string(),
                has_code: false,
            });
        }
    }

    chunks
}

pub fn chunk_json(val: &Value, max_chunk_bytes: usize) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    walk_json(val, &[], &mut chunks, max_chunk_bytes);
    chunks
}

fn walk_json(val: &Value, path: &[String], out: &mut Vec<Chunk>, max_bytes: usize) {
    let title = if path.is_empty() {
        "(root)".to_string()
    } else {
        path.join(" > ")
    };

    match val {
        Value::Object(map) => {
            let serialized = serde_json::to_string_pretty(val).unwrap_or_default();
            let is_flat = map.values().all(|v| !v.is_object() && !v.is_array());

            if serialized.len() <= max_bytes && is_flat {
                out.push(Chunk {
                    title,
                    content: serialized,
                    has_code: true,
                });
                return;
            }

            for (k, v) in map {
                let mut next_path = path.to_vec();
                next_path.push(k.clone());
                walk_json(v, &next_path, out, max_bytes);
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                out.push(Chunk {
                    title,
                    content: "[]".to_string(),
                    has_code: true,
                });
                return;
            }

            let mut batch: Vec<&Value> = Vec::new();
            let mut batch_start = 0;

            for (i, item) in arr.iter().enumerate() {
                batch.push(item);
                let candidate = serde_json::to_string_pretty(&batch).unwrap_or_default();
                if candidate.len() > max_bytes && batch.len() > 1 {
                    batch.pop();
                    let chunk_content = serde_json::to_string_pretty(&batch).unwrap_or_default();
                    let chunk_title = format!("{}[{}..{}]", title, batch_start, i - 1);
                    out.push(Chunk {
                        title: chunk_title,
                        content: chunk_content,
                        has_code: true,
                    });
                    batch = vec![item];
                    batch_start = i;
                }
            }

            if !batch.is_empty() {
                let chunk_content = serde_json::to_string_pretty(&batch).unwrap_or_default();
                let chunk_title = if batch_start == 0 && batch.len() == arr.len() {
                    title
                } else {
                    format!("{}[{}..{}]", title, batch_start, arr.len() - 1)
                };
                out.push(Chunk {
                    title: chunk_title,
                    content: chunk_content,
                    has_code: true,
                });
            }
        }
        _ => {
            let serialized = serde_json::to_string(val).unwrap_or_default();
            out.push(Chunk {
                title,
                content: serialized,
                has_code: false,
            });
        }
    }
}

// ─────────────────────────────────────────────────────────
// ContentStore Engine
// ─────────────────────────────────────────────────────────

impl Default for ContentStore {
    fn default() -> Self {
        match Self::open_in_memory() {
            Ok(s) => s,
            Err(_) => match Connection::open_in_memory() {
                Ok(conn) => Self { conn, has_trigram: false },
                Err(e) => {
                    eprintln!("[ERROR] In-memory SQLite initialization failed: {e}");
                    match Connection::open("file::memory:?cache=shared") {
                        Ok(conn) => Self { conn, has_trigram: false },
                        Err(_) => std::process::exit(1),
                    }
                }
            },
        }
    }
}

pub struct ContentStore {
    conn: Connection,
    has_trigram: bool,
}

impl ContentStore {
    pub fn open(db_path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(db_path)?;
        Self::from_connection(conn)
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> Result<Self, rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT NOT NULL,
                chunk_count INTEGER NOT NULL DEFAULT 0,
                code_chunk_count INTEGER NOT NULL DEFAULT 0,
                indexed_at TEXT NOT NULL DEFAULT (datetime('now')),
                file_path TEXT,
                content_hash TEXT,
                subproject_id TEXT NOT NULL DEFAULT '.'
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS chunks USING fts5(
                title,
                content,
                source_id UNINDEXED,
                content_type UNINDEXED,
                source_category UNINDEXED,
                session_id UNINDEXED,
                event_id UNINDEXED,
                timestamp UNINDEXED,
                tokenize='porter unicode61'
            );

            CREATE TABLE IF NOT EXISTS vocabulary (
                word TEXT PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                seq INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                worktree TEXT NOT NULL,
                agent TEXT NOT NULL,
                kind TEXT NOT NULL,
                tool TEXT,
                key TEXT,
                path TEXT,
                ok INTEGER,
                exit_code INTEGER,
                bytes INTEGER,
                body TEXT,
                turn INTEGER NOT NULL,
                sandbox_bytes INTEGER NOT NULL DEFAULT 0,
                returned_bytes INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_events_session_kind_seq ON events(session_id, kind, seq);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_events_cost_session_turn ON events(session_id, turn) WHERE kind = 'cost';

            CREATE INDEX IF NOT EXISTS idx_sources_label ON sources(label);",
        )?;

        let has_subproj: bool = conn
            .prepare("PRAGMA table_info(sources)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|c| c == "subproject_id");
        if !has_subproj {
            let _ = conn.execute(
                "ALTER TABLE sources ADD COLUMN subproject_id TEXT NOT NULL DEFAULT '.'",
                [],
            );
        }
        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sources_subproject ON sources(subproject_id)",
            [],
        );

        // Attempt trigram virtual table creation
        let has_trigram = conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_trigram USING fts5(
                    title,
                    content,
                    source_id UNINDEXED,
                    content_type UNINDEXED,
                    source_category UNINDEXED,
                    session_id UNINDEXED,
                    event_id UNINDEXED,
                    timestamp UNINDEXED,
                    tokenize='trigram'
                );",
            )
            .is_ok();

        Ok(Self { conn, has_trigram })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn meta_get(&self, key: &str) -> Option<String> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?1").ok()?;
        stmt.query_row(params![key], |r| r.get(0)).optional().ok().flatten()
    }

    pub fn meta_set(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn meta_take(&self, key: &str) -> Option<String> {
        let val = self.meta_get(key)?;
        let _ = self.conn.execute("DELETE FROM meta WHERE key = ?1", params![key]);
        Some(val)
    }

    pub fn append_journal_event(&self, row: &super::journal::NewJournalRow) -> Result<i64, rusqlite::Error> {
        super::journal::append_event(&self.conn, row)
    }

    pub fn get_session_events(&self, session_id: &str, kind: Option<&str>) -> Result<Vec<super::journal::JournalRow>, rusqlite::Error> {
        super::journal::read_session_events(&self.conn, session_id, kind)
    }

    pub fn get_event_by_seq(&self, seq: i64) -> Result<Option<super::journal::JournalRow>, rusqlite::Error> {
        super::journal::read_event_by_seq(&self.conn, seq)
    }

    pub fn get_last_prompt(&self, session_id: &str) -> String {
        super::journal::read_last_prompt(&self.conn, session_id)
    }

    pub fn get_first_prompt(&self, session_id: &str) -> String {
        super::journal::read_first_prompt(&self.conn, session_id)
    }

    pub fn get_recall_rows(&self, session_id: &str, worktree: &str, limit: usize) -> Result<Vec<super::journal::JournalRow>, rusqlite::Error> {
        super::journal::read_recall_rows(&self.conn, session_id, worktree, limit)
    }

    pub fn get_recall_rows_bounded(
        &self,
        session_id: &str,
        worktree: &str,
        limit: usize,
        max_seq: i64,
    ) -> Result<Vec<super::journal::JournalRow>, rusqlite::Error> {
        super::journal::read_recall_rows_bounded(&self.conn, session_id, worktree, limit, max_seq)
    }

    pub fn last_session_seq(&self, session_id: &str) -> i64 {
        super::journal::last_session_seq(&self.conn, session_id)
    }

    /// #1390: Resolve a `source` argument in scope-first order (exact label, forward-slash normalized label,
    /// `#id` handle, or label without `#<call_id>` suffix), checking `session_id` scope first when provided,
    /// then falling back to all stored sources.
    pub fn pick_source(&self, source: &str, session_id: Option<&str>, suffix: bool) -> (String, Option<i64>) {
        if let Some(sid) = session_id
            && !sid.is_empty()
            && let Some(found) = self.pick_source_in_scope(source, Some(sid), suffix)
        {
            return found;
        }
        if let Some(found) = self.pick_source_in_scope(source, None, suffix) {
            return found;
        }
        (source.to_string(), None)
    }

    fn pick_source_in_scope(&self, source: &str, session_id: Option<&str>, suffix: bool) -> Option<(String, Option<i64>)> {
        let canon = crate::util::paths::normalize_path(source);
        for candidate in [source, canon.as_str()] {
            if let Some(id) = self.newest_source_id_for_label(candidate, session_id) {
                return Some((candidate.to_string(), Some(id)));
            }
        }
        if let Some(id) = source.strip_prefix('#').and_then(|s| s.parse::<i64>().ok())
            && let Some(label) = self.label_for_source_id(id, session_id)
        {
            return Some((label, Some(id)));
        }
        if suffix && let Some((base, _)) = source.rsplit_once('#') {
            for candidate in [base, crate::util::paths::normalize_path(base).as_str()] {
                if let Some(id) = self.newest_source_id_for_label(candidate, session_id) {
                    return Some((candidate.to_string(), Some(id)));
                }
            }
        }
        None
    }

    fn newest_source_id_for_label(&self, label: &str, session_id: Option<&str>) -> Option<i64> {
        if let Some(sid) = session_id {
            self.conn
                .query_row(
                    "SELECT id FROM sources WHERE label = ?1 AND session_id = ?2 ORDER BY id DESC LIMIT 1",
                    params![label, sid],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()
        } else {
            self.conn
                .query_row(
                    "SELECT id FROM sources WHERE label = ?1 ORDER BY id DESC LIMIT 1",
                    params![label],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()
        }
    }

    fn label_for_source_id(&self, id: i64, session_id: Option<&str>) -> Option<String> {
        if let Some(sid) = session_id {
            self.conn
                .query_row(
                    "SELECT label FROM sources WHERE id = ?1 AND session_id = ?2",
                    params![id, sid],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()
        } else {
            self.conn
                .query_row(
                    "SELECT label FROM sources WHERE id = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()
        }
    }

    pub fn get_source_meta(&self, label: &str) -> Option<SourceMeta> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, label, chunk_count, code_chunk_count, indexed_at, file_path, content_hash
                 FROM sources WHERE label = ?1 ORDER BY id DESC LIMIT 1",
            )
            .ok()?;

        stmt.query_row(params![label], |row| {
            Ok(SourceMeta {
                id: row.get(0)?,
                label: row.get(1)?,
                chunk_count: row.get(2)?,
                code_chunk_count: row.get(3)?,
                indexed_at: row.get(4)?,
                file_path: row.get(5)?,
                content_hash: row.get(6)?,
            })
        })
        .optional()
        .ok()
        .flatten()
    }

    pub fn list_sources(&self) -> Result<Vec<SourceMeta>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, label, chunk_count, code_chunk_count, indexed_at, file_path, content_hash
             FROM sources ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SourceMeta {
                id: row.get(0)?,
                label: row.get(1)?,
                chunk_count: row.get(2)?,
                code_chunk_count: row.get(3)?,
                indexed_at: row.get(4)?,
                file_path: row.get(5)?,
                content_hash: row.get(6)?,
            })
        })?;
        let out = rows.flatten().collect();
        Ok(out)
    }

    pub fn delete_source(&mut self, label: &str) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.transaction()?;
        let source_ids: Vec<i64> = {
            let mut s = tx.prepare("SELECT id FROM sources WHERE label = ?1")?;
            let rows = s.query_map(params![label], |r| r.get(0))?;
            rows.flatten().collect()
        };

        for sid in &source_ids {
            tx.execute("DELETE FROM chunks WHERE source_id = ?1", params![sid.to_string()])?;
            if self.has_trigram {
                let _ = tx.execute("DELETE FROM chunks_trigram WHERE source_id = ?1", params![sid.to_string()]);
            }
        }
        let deleted = tx.execute("DELETE FROM sources WHERE label = ?1", params![label])?;
        tx.commit()?;
        Ok(deleted)
    }

    pub fn index(
        &mut self,
        content: &str,
        source: &str,
        content_type: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<IndexResult, rusqlite::Error> {
        let chunks = if content_type == Some("json") {
            if let Ok(v) = serde_json::from_str::<Value>(content) {
                chunk_json(&v, MAX_CHUNK_BYTES)
            } else {
                chunk_plain_text(content, MAX_CHUNK_BYTES)
            }
        } else if content_type == Some("prose") {
            chunk_plain_text(content, MAX_CHUNK_BYTES)
        } else {
            chunk_markdown(content, MAX_CHUNK_BYTES)
        };

        self.delete_source(source)?;

        let tx = self.conn.transaction()?;
        let code_chunk_count = chunks.iter().filter(|c| c.has_code).count();
        let total_chunks = chunks.len();

        tx.execute(
            "INSERT INTO sources (label, chunk_count, code_chunk_count, indexed_at)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![source, total_chunks, code_chunk_count],
        )?;

        let source_id = tx.last_insert_rowid().to_string();
        let sid = session_id.unwrap_or("");
        let ctype = content_type.unwrap_or("auto");
        let now_iso = chrono::Utc::now().to_rfc3339();

        for chunk in &chunks {
            tx.execute(
                "INSERT INTO chunks (title, content, source_id, content_type, source_category, session_id, event_id, timestamp)
                 VALUES (?1, ?2, ?3, ?4, 'content', ?5, '', ?6)",
                params![chunk.title, chunk.content, source_id, ctype, sid, now_iso],
            )?;

            if self.has_trigram {
                let _ = tx.execute(
                    "INSERT INTO chunks_trigram (title, content, source_id, content_type, source_category, session_id, event_id, timestamp)
                     VALUES (?1, ?2, ?3, ?4, 'content', ?5, '', ?6)",
                    params![chunk.title, chunk.content, source_id, ctype, sid, now_iso],
                );
            }
        }

        tx.commit()?;

        Ok(IndexResult {
            label: source.to_string(),
            total_chunks,
            code_chunks: code_chunk_count,
            total_bytes: content.len(),
        })
    }

    pub fn index_json(&mut self, json_str: &str, source: &str, session_id: Option<&str>) -> Result<IndexResult, rusqlite::Error> {
        self.index(json_str, source, Some("json"), session_id)
    }

    pub fn index_plain_text(&mut self, text: &str, source: &str, session_id: Option<&str>) -> Result<IndexResult, rusqlite::Error> {
        self.index(text, source, Some("prose"), session_id)
    }

    pub fn search(&self, opts: &SearchOptions) -> Result<Vec<SearchResult>, rusqlite::Error> {
        let limit = if opts.limit == 0 { 10 } else { opts.limit };
        let porter_query = sanitize_query(&opts.query, "AND");

        let mut results = self.execute_fts_search("chunks", &porter_query, opts, limit)?;

        // Fallback to OR query if 0 results
        if results.is_empty() {
            let or_query = sanitize_query(&opts.query, "OR");
            if or_query != porter_query {
                results = self.execute_fts_search("chunks", &or_query, opts, limit)?;
            }
        }

        // Fallback to trigram if available and still 0 results
        if results.is_empty() && self.has_trigram {
            let tri_query = sanitize_trigram_query(&opts.query, "AND");
            if !tri_query.is_empty() {
                results = self.execute_fts_search("chunks_trigram", &tri_query, opts, limit)?;
            }
        }

        Ok(results)
    }

    fn execute_fts_search(
        &self,
        table: &str,
        query: &str,
        opts: &SearchOptions,
        limit: usize,
    ) -> Result<Vec<SearchResult>, rusqlite::Error> {
        if query.is_empty() || query == "\"\"" {
            return Ok(Vec::new());
        }

        let mut sql = format!(
            "SELECT c.title, c.content, c.content_type, c.timestamp, s.label,
                    bm25({table}, 2.0, 1.0) AS rank,
                    snippet({table}, 1, '<b>', '</b>', '...', 32) AS highlighted,
                    c.session_id
             FROM {table} c
             JOIN sources s ON s.id = CAST(c.source_id AS INTEGER)
             WHERE {table} MATCH ?1"
        );

        let mut param_values: Vec<String> = vec![query.to_string()];

        if let Some(ref src) = opts.source {
            sql.push_str(" AND s.label LIKE ?2");
            param_values.push(format!("%{}%", src));
        }

        if let Some(ref ctype) = opts.content_type {
            let idx = param_values.len() + 1;
            sql.push_str(&format!(" AND c.content_type = ?{}", idx));
            param_values.push(ctype.clone());
        }

        sql.push_str(" ORDER BY rank ASC LIMIT ");
        sql.push_str(&limit.to_string());

        let mut stmt = self.conn.prepare(&sql)?;

        let params_slice: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(params_slice.as_slice(), |row| {
            let rank: f64 = row.get(5)?;
            Ok(SearchResult {
                title: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                timestamp: row.get(3)?,
                label: row.get(4)?,
                score: -rank, // Invert so higher is better
                highlighted: row.get(6)?,
                session_id: row.get(7)?,
            })
        })?;

        let mut out = Vec::new();
        for r in rows.flatten() {
            out.push(r);
        }
        Ok(out)
    }

    pub fn index_directory(
        &mut self,
        root: &Path,
        max_files: usize,
        max_depth: usize,
        label_prefix: Option<&str>,
    ) -> Result<DirectoryIndexResult, rusqlite::Error> {
        let abs_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if crate::codegraph::walker::is_inside_git_dir(&abs_root) {
            return Ok(DirectoryIndexResult {
                root: crate::util::paths::normalize_path(&abs_root.to_string_lossy()),
                files_indexed: 0,
                total_chunks: 0,
                total_bytes: 0,
                capped: false,
                errors: vec!["Refusing to index inside .git directory".to_string()],
            });
        }
        if let Err(e) = super::store::find_repo_root_and_subproject(Some(&abs_root)) {
            return Ok(DirectoryIndexResult {
                root: crate::util::paths::normalize_path(&abs_root.to_string_lossy()),
                files_indexed: 0,
                total_chunks: 0,
                total_bytes: 0,
                capped: false,
                errors: vec![e.to_string()],
            });
        }
        let root_str = crate::util::paths::normalize_path(&abs_root.to_string_lossy());

        let walker = WalkBuilder::new(&abs_root)
            .standard_filters(true)
            .hidden(true)
            .parents(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .add_custom_ignore_filename(".ignore")
            .max_depth(Some(max_depth))
            .filter_entry(|entry| {
                let name = entry.file_name().to_string_lossy();
                if entry.file_type().is_some_and(|ft| ft.is_dir())
                    && (name == ".git"
                        || name == "node_modules"
                        || name == "target"
                        || name == ".codegraph"
                        || name == ".projectmem"
                        || name == ".toksi"
                        || name == "dist"
                        || name == "build"
                        || name == "vendor"
                        || name == "__pycache__"
                        || name == ".venv"
                        || name == "venv")
                {
                    return false;
                }
                if entry.file_type().is_some_and(|ft| ft.is_file()) && name.ends_with(".lock") {
                    return false;
                }
                true
            })
            .build();

        let mut files_indexed = 0;
        let mut total_chunks = 0;
        let mut total_bytes = 0;
        let mut capped = false;
        let mut errors = Vec::new();

        for entry_res in walker {
            if files_indexed >= max_files {
                capped = true;
                break;
            }

            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(err.to_string());
                    continue;
                }
            };

            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.path();
            if super::security::check_path_deny(path).is_some() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !DEFAULT_INDEX_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }

            if std::fs::metadata(path).is_ok_and(|meta| meta.len() > 2 * 1024 * 1024) {
                continue;
            }

            let content = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let rel_path = path
                .strip_prefix(&abs_root)
                .map(|p| crate::util::paths::normalize_path(&p.to_string_lossy()))
                .unwrap_or_else(|_| crate::util::paths::normalize_path(&path.to_string_lossy()));

            let sub_id = crate::codegraph::cache::derive_subproject_id(&rel_path);
            let label = if let Some(prefix) = label_prefix {
                format!("{}/{}", prefix.trim_end_matches('/'), rel_path)
            } else {
                rel_path
            };

            let ctype = if ext == "json" {
                "json"
            } else if ext == "md" || ext == "markdown" {
                "markdown"
            } else {
                "prose"
            };

            match self.index(&content, &label, Some(ctype), None) {
                Ok(ir) => {
                    let _ = self.conn.execute(
                        "UPDATE sources SET subproject_id = ?1 WHERE label = ?2",
                        params![sub_id, label],
                    );
                    files_indexed += 1;
                    total_chunks += ir.total_chunks;
                    total_bytes += ir.total_bytes;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", label, e));
                }
            }
        }

        Ok(DirectoryIndexResult {
            root: root_str,
            files_indexed,
            total_chunks,
            total_bytes,
            capped,
            errors,
        })
    }

    pub fn purge_all(&mut self) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.transaction()?;
        let count: usize = tx.query_row("SELECT COUNT(*) FROM sources", [], |r| r.get(0)).unwrap_or(0);
        tx.execute("DELETE FROM chunks", [])?;
        if self.has_trigram {
            let _ = tx.execute("DELETE FROM chunks_trigram", []);
        }
        tx.execute("DELETE FROM sources", [])?;
        tx.commit()?;
        Ok(count)
    }

    /// Dynamic purge: delete indexed sources matching .gitignore or .ignore patterns
    pub fn purge_ignored_sources(&mut self, root: &Path) -> Result<usize, rusqlite::Error> {
        let ignore = crate::codegraph::walker::build_project_ignore(root);
        let sources = self.list_sources()?;
        let mut purged = 0;

        for src in sources {
            let label_path = src.file_path.as_deref().unwrap_or(&src.label);
            let normalized = crate::util::paths::normalize_path(label_path);
            let full_path = if Path::new(&normalized).is_absolute() {
                PathBuf::from(&normalized)
            } else {
                root.join(&normalized)
            };
            let is_dir = full_path.is_dir();
            let is_ignored = ignore.matched_path_or_any_parents(&full_path, is_dir).is_ignore()
                || crate::codegraph::walker::is_build_or_vendor_artifact(&normalized)
                || normalized.split('/').any(crate::codegraph::walker::is_build_or_vendor_artifact);

            if is_ignored {
                self.delete_source(&src.label)?;
                purged += 1;
            }
        }
        Ok(purged)
    }

    /// Transactionally merges all sources, FTS5 chunks, trigram chunks, vocabulary, meta,
    /// and journal events from a legacy SQLite database (`context.db`) into this `ctxmode.db` store.
    pub fn merge_from(&mut self, legacy_db: &Path) -> Result<usize, rusqlite::Error> {
        if !legacy_db.is_file() {
            return Ok(0);
        }
        let legacy_conn = Connection::open_with_flags(
            legacy_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let tx = self.conn.transaction()?;
        let mut migrated = 0usize;

        let has_sources_tbl: bool = legacy_conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='sources')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if has_sources_tbl {
            let has_subproj_col: bool = legacy_conn
                .prepare("PRAGMA table_info(sources)")
                .and_then(|mut s| {
                    let cols: Vec<String> = s
                        .query_map([], |r| r.get::<_, String>(1))?
                        .filter_map(Result::ok)
                        .collect();
                    Ok(cols.iter().any(|c| c == "subproject_id"))
                })
                .unwrap_or(false);

            let sql = if has_subproj_col {
                "SELECT id, label, chunk_count, code_chunk_count, indexed_at, file_path, content_hash, subproject_id FROM sources"
            } else {
                "SELECT id, label, chunk_count, code_chunk_count, indexed_at, file_path, content_hash, '.' FROM sources"
            };
            type LegacySourceRow = (i64, String, i64, i64, String, Option<String>, Option<String>, String);
            let mut src_stmt = legacy_conn.prepare(sql)?;
            let src_rows: Vec<LegacySourceRow> = src_stmt
                .query_map([], |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get::<_, Option<String>>(7)?.unwrap_or_else(|| ".".to_string()),
                    ))
                })?
                .filter_map(Result::ok)
                .collect();

            let has_chunks_tbl: bool = legacy_conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='chunks')",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(false);

            for (old_id, label, chunk_cnt, code_cnt, indexed_at, file_path, content_hash, subproj) in src_rows {
                let exists: Option<i64> = tx
                    .query_row(
                        "SELECT id FROM sources WHERE label = ?1 AND subproject_id = ?2 LIMIT 1",
                        params![label, subproj],
                        |r| r.get(0),
                    )
                    .optional()?;
                if exists.is_some() {
                    continue;
                }
                tx.execute(
                    "INSERT INTO sources (label, chunk_count, code_chunk_count, indexed_at, file_path, content_hash, subproject_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![label, chunk_cnt, code_cnt, indexed_at, file_path, content_hash, subproj],
                )?;
                let new_id = tx.last_insert_rowid().to_string();
                migrated += 1;

                if has_chunks_tbl {
                    let old_id_str = old_id.to_string();
                    let mut c_stmt = legacy_conn.prepare(
                        "SELECT title, content, content_type, source_category, session_id, event_id, timestamp
                         FROM chunks WHERE source_id = ?1",
                    )?;
                    let chunks_rows: Vec<(String, String, String, String, String, String, String)> = c_stmt
                        .query_map(params![old_id_str], |r| {
                            Ok((
                                r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                                r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                                r.get::<_, Option<String>>(2)?.unwrap_or_else(|| "prose".to_string()),
                                r.get::<_, Option<String>>(3)?.unwrap_or_else(|| "content".to_string()),
                                r.get::<_, Option<String>>(4)?.unwrap_or_default(),
                                r.get::<_, Option<String>>(5)?.unwrap_or_default(),
                                r.get::<_, Option<String>>(6)?.unwrap_or_default(),
                            ))
                        })?
                        .filter_map(Result::ok)
                        .collect();

                    for (title, content, ctype, scategory, sid, eid, ts) in chunks_rows {
                        tx.execute(
                            "INSERT INTO chunks (title, content, source_id, content_type, source_category, session_id, event_id, timestamp)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            params![title, content, new_id, ctype, scategory, sid, eid, ts],
                        )?;
                        if self.has_trigram {
                            let _ = tx.execute(
                                "INSERT INTO chunks_trigram (title, content, source_id, content_type, source_category, session_id, event_id, timestamp)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                params![title, content, new_id, ctype, scategory, sid, eid, ts],
                            );
                        }
                    }
                }
            }
        }

        if legacy_conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='vocabulary')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(false)
        {
            let mut v_stmt = legacy_conn.prepare("SELECT word FROM vocabulary")?;
            for word in v_stmt.query_map([], |r| r.get::<_, String>(0))?.flatten() {
                tx.execute("INSERT OR IGNORE INTO vocabulary (word) VALUES (?1)", params![word])?;
            }
        }

        if legacy_conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='meta')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(false)
        {
            let mut m_stmt = legacy_conn.prepare("SELECT key, value FROM meta")?;
            for pair in m_stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?.flatten() {
                tx.execute("INSERT OR IGNORE INTO meta (key, value) VALUES (?1, ?2)", params![pair.0, pair.1])?;
            }
        }

        if legacy_conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='events')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(false)
        {
            let mut e_stmt = legacy_conn.prepare(
                "SELECT session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn, sandbox_bytes, returned_bytes, created_at, accessed_at FROM events",
            )?;
            let ev_rows = e_stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<i64>>(7)?,
                    r.get::<_, Option<i64>>(8)?,
                    r.get::<_, Option<i64>>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, i64>(11)?,
                    r.get::<_, i64>(12)?,
                    r.get::<_, i64>(13)?,
                    r.get::<_, i64>(14)?,
                    r.get::<_, i64>(15)?,
                ))
            })?;
            for ev in ev_rows.flatten() {
                tx.execute(
                    "INSERT OR IGNORE INTO events (session_id, worktree, agent, kind, tool, key, path, ok, exit_code, bytes, body, turn, sandbox_bytes, returned_bytes, created_at, accessed_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    params![ev.0, ev.1, ev.2, ev.3, ev.4, ev.5, ev.6, ev.7, ev.8, ev.9, ev.10, ev.11, ev.12, ev.13, ev.14, ev.15],
                )?;
            }
        }

        tx.commit()?;
        Ok(migrated)
    }
}

/// Non-destructive transactional migration from a legacy `context.db` into canonical `ctxmode.db`.
/// Removes `legacy_db` and its `-wal`/`-shm` sidecars only after the SQLite transaction commits.
pub fn migrate_legacy_ctxmode_db(target_db: &Path, legacy_db: &Path) -> Result<usize, rusqlite::Error> {
    if !legacy_db.is_file() || target_db == legacy_db {
        return Ok(0);
    }
    if let Ok(bytes) = std::fs::read(legacy_db)
        && !bytes.starts_with(b"SQLite format 3\0") {
            if let Some(parent) = target_db.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if !target_db.exists() {
                let _ = std::fs::copy(legacy_db, target_db);
            }
            let _ = std::fs::remove_file(legacy_db);
            return Ok(0);
        }
    let mut store = ContentStore::open(target_db)?;
    let migrated = store.merge_from(legacy_db)?;
    drop(store);
    let _ = std::fs::remove_file(legacy_db);
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{}", legacy_db.to_string_lossy(), suffix));
        if sidecar.exists() {
            let _ = std::fs::remove_file(sidecar);
        }
    }
    Ok(migrated)
}

/// D-1330-RECALL-PROMPT-1 (#1330): Build post-compact recall query from the current prompt only.
/// Extracts at most 64 distinct non-stopword words from the prompt.
pub fn build_recall_query(prompt: &str) -> String {
    let cleaned = prompt.replace(['\'', '"', '(', ')', '{', '}', '[', ']', '*', ':', '^', '~', '`', '#', '<', '>'], " ");
    let mut words: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for w in cleaned.split_whitespace() {
        if words.len() >= 64 {
            break;
        }
        let lower = w.to_lowercase();
        if lower.len() >= 2 && !STOPWORDS.contains(lower.as_str()) && seen.insert(lower) {
            words.push(format!("\"{}\"", w));
        }
    }

    if words.is_empty() {
        sanitize_query(prompt, "OR")
    } else {
        words.join(" OR ")
    }
}

/// Resolves standard content store DB path for a given project or global config.
/// Strictly enforces `<repo>/.toksi/ctxmode/ctxmode.db` per master consolidation plan.
/// NEVER stores in `~/.toksi/ctxmode/`, NEVER outside `.git` roots, NEVER inside `.git/`.
pub fn resolve_content_store_path(project_dir: Option<&Path>) -> PathBuf {
    if let Ok(db_path) = super::store::resolve_context_db_path(project_dir) {
        return db_path;
    }
    if let Some(p) = project_dir {
        p.join(".toksi").join("ctxmode").join("ctxmode.db")
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(".toksi").join("ctxmode").join("ctxmode.db")
    } else {
        PathBuf::from(".toksi/ctxmode/ctxmode.db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contextmode_purge_ignored_sources() {
        let temp_dir = std::env::temp_dir().join("toksi_ctx_purge_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("context.db");
        let mut store = ContentStore::open(&db_path).unwrap();

        // Index two sources
        let _ = store.index("console.log('hello')", "src/app.ts", Some("prose"), None).unwrap();
        let _ = store.index("build artifact data", "target/output.tmp", Some("prose"), None).unwrap();
        assert_eq!(store.list_sources().unwrap().len(), 2);

        // Add gitignore rule
        let gitignore = temp_dir.join(".gitignore");
        std::fs::write(&gitignore, "target/\n*.tmp\n").unwrap();

        // Purge
        let purged = store.purge_ignored_sources(&temp_dir).unwrap();
        assert_eq!(purged, 1);
        let remaining = store.list_sources().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].label, "src/app.ts");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
