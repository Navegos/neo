// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/fts.rs
// created: 2026-10-27

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
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
                content_hash TEXT
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

            CREATE INDEX IF NOT EXISTS idx_sources_label ON sources(label);",
        )?;

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

        let mut stmt = match self.conn.prepare(&sql) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

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
}

/// Resolves standard content store DB path for a given project or global config.
pub fn resolve_content_store_path(project_dir: Option<&Path>) -> PathBuf {
    if let Some(root) = project_dir {
        root.join(".tokless").join("content.db")
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(".tokless").join("content.db")
    } else if let Ok(h) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        PathBuf::from(h).join(".tokless").join("content.db")
    } else {
        PathBuf::from(".tokless").join("content.db")
    }
}
