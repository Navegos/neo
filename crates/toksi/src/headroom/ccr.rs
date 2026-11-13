// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/ccr.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Compress-Cache-Retrieve (CCR) reversible storage (`<<ccr:HASH>>`) with SQLite WAL +
//! in-memory backends and MCP retrieval (`headroom_retrieve` / `headroom_search`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::error::{HeadroomError, HeadroomResult};

/// Default sliding idle TTL (30 minutes).
pub const DEFAULT_TTL: Duration = Duration::from_secs(1800);
/// Absolute maximum lifetime multiplier over idle TTL.
pub const DEFAULT_MAX_LIFETIME_MULTIPLIER: u32 = 8;

/// Trait shared across CCR storage backends.
pub trait CcrStore: Send + Sync {
    fn put(&self, hash: &str, payload: &str);
    fn get(&self, hash: &str) -> Option<String>;
    fn search(&self, hash: &str, query: &str) -> Vec<String>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Computes canonical 24-hex-char SHA-256 key for `payload`.
pub fn compute_key(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(24);
    for byte in digest.iter().take(12) {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    hex
}

/// Standard CCR retrieval marker inserted into compressed output.
pub fn marker_for(hash: &str) -> String {
    format!("<<ccr:{}>>", hash)
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
struct MemEntry {
    original: String,
    created_at: i64,
    last_accessed: i64,
    ttl_seconds: i64,
}

/// Thread-safe in-memory CCR store (default for unit tests and ephemeral runs).
#[derive(Debug)]
pub struct InMemoryCcrStore {
    entries: RwLock<HashMap<String, MemEntry>>,
    default_ttl_seconds: i64,
    max_lifetime_seconds: i64,
}

impl Default for InMemoryCcrStore {
    fn default() -> Self {
        Self::new(DEFAULT_TTL.as_secs())
    }
}

impl InMemoryCcrStore {
    pub fn new(default_ttl_seconds: u64) -> Self {
        let ttl = i64::try_from(default_ttl_seconds).unwrap_or(i64::MAX);
        let max_life = ttl.saturating_mul(i64::from(DEFAULT_MAX_LIFETIME_MULTIPLIER));
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl_seconds: ttl,
            max_lifetime_seconds: max_life,
        }
    }

    fn is_expired(&self, e: &MemEntry, now: i64) -> bool {
        e.last_accessed.saturating_add(e.ttl_seconds) < now
            || e.created_at.saturating_add(self.max_lifetime_seconds) < now
    }
}

impl CcrStore for InMemoryCcrStore {
    fn put(&self, hash: &str, payload: &str) {
        let now = now_unix_secs();
        let mut map = self.entries.write().unwrap_or_else(|e| e.into_inner());
        map.retain(|_, v| !self.is_expired(v, now));
        map.insert(
            hash.to_string(),
            MemEntry {
                original: payload.to_string(),
                created_at: now,
                last_accessed: now,
                ttl_seconds: self.default_ttl_seconds,
            },
        );
    }

    fn get(&self, hash: &str) -> Option<String> {
        let now = now_unix_secs();
        let mut map = self.entries.write().unwrap_or_else(|e| e.into_inner());
        map.retain(|_, v| !self.is_expired(v, now));
        if let Some(entry) = map.get_mut(hash) {
            entry.last_accessed = now;
            return Some(entry.original.clone());
        }
        None
    }

    fn search(&self, hash: &str, query: &str) -> Vec<String> {
        let Some(content) = self.get(hash) else {
            return Vec::new();
        };
        filter_matching_lines(&content, query)
    }

    fn len(&self) -> usize {
        let now = now_unix_secs();
        let map = self.entries.read().unwrap_or_else(|e| e.into_inner());
        map.values().filter(|v| !self.is_expired(v, now)).count()
    }
}

/// Persistent SQLite WAL-backed CCR store with sliding idle TTL.
pub struct SqliteCcrStore {
    conn: Mutex<Connection>,
    default_ttl_seconds: i64,
    max_lifetime_seconds: i64,
    path: PathBuf,
}

impl SqliteCcrStore {
    pub fn open(path: impl AsRef<Path>, default_ttl_seconds: u64) -> HeadroomResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        if let Some(parent) = path_buf.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path_buf)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ccr_entries (
                 hash          TEXT PRIMARY KEY,
                 original      BLOB NOT NULL,
                 created_at    INTEGER NOT NULL,
                 ttl_seconds   INTEGER NOT NULL,
                 last_accessed INTEGER NOT NULL
             )",
            [],
        )?;

        let ttl = i64::try_from(default_ttl_seconds).unwrap_or(i64::MAX);
        let max_life = ttl.saturating_mul(i64::from(DEFAULT_MAX_LIFETIME_MULTIPLIER));
        Ok(Self {
            conn: Mutex::new(conn),
            default_ttl_seconds: ttl,
            max_lifetime_seconds: max_life,
            path: path_buf,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn purge_expired(&self, conn: &Connection, now: i64) {
        let _ = conn.execute(
            "DELETE FROM ccr_entries WHERE last_accessed + ttl_seconds < ?1 OR created_at + ?2 < ?1",
            params![now, self.max_lifetime_seconds],
        );
    }
}

impl CcrStore for SqliteCcrStore {
    fn put(&self, hash: &str, payload: &str) {
        let now = now_unix_secs();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let _ = conn.execute(
            "INSERT INTO ccr_entries (hash, original, created_at, ttl_seconds, last_accessed)
             VALUES (?1, ?2, ?3, ?4, ?3)
             ON CONFLICT(hash) DO UPDATE SET
                 original = excluded.original,
                 created_at = excluded.created_at,
                 ttl_seconds = excluded.ttl_seconds,
                 last_accessed = excluded.last_accessed",
            params![hash, payload.as_bytes(), now, self.default_ttl_seconds],
        );
    }

    fn get(&self, hash: &str) -> Option<String> {
        let now = now_unix_secs();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        self.purge_expired(&conn, now);
        let row: Option<Vec<u8>> = conn
            .query_row(
                "SELECT original FROM ccr_entries
                 WHERE hash = ?1 AND last_accessed + ttl_seconds >= ?2 AND created_at + ?3 >= ?2",
                params![hash, now, self.max_lifetime_seconds],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        if let Some(bytes) = row {
            let _ = conn.execute(
                "UPDATE ccr_entries SET last_accessed = ?1 WHERE hash = ?2",
                params![now, hash],
            );
            return String::from_utf8(bytes).ok();
        }
        None
    }

    fn search(&self, hash: &str, query: &str) -> Vec<String> {
        let Some(content) = self.get(hash) else {
            return Vec::new();
        };
        filter_matching_lines(&content, query)
    }

    fn len(&self) -> usize {
        let now = now_unix_secs();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        self.purge_expired(&conn, now);
        conn.query_row("SELECT count(*) FROM ccr_entries", [], |r| r.get::<_, i64>(0))
            .map(|c| usize::try_from(c).unwrap_or(0))
            .unwrap_or(0)
    }
}

fn filter_matching_lines(content: &str, query: &str) -> Vec<String> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return content.lines().take(50).map(|l| l.to_string()).collect();
    }
    let terms: Vec<&str> = q.split_whitespace().collect();
    content
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let low = line.to_ascii_lowercase();
            if terms.iter().any(|t| low.contains(t)) {
                Some(format!("{}: {}", idx + 1, line))
            } else {
                None
            }
        })
        .take(50)
        .collect()
}

/// Process-wide singleton CCR store rooted under Toksi headroom state directory.
pub fn global_ccr_store() -> Arc<dyn CcrStore> {
    static STORE: OnceLock<Arc<dyn CcrStore>> = OnceLock::new();
    STORE
        .get_or_init(|| {
            let db_path = Path::new(&crate::util::headroom::headroom_paths_resolved().root).join("ccr.db");
            match SqliteCcrStore::open(&db_path, DEFAULT_TTL.as_secs()) {
                Ok(sqlite) => Arc::new(sqlite),
                Err(_) => Arc::new(InMemoryCcrStore::default()),
            }
        })
        .clone()
}

/// Handles MCP `headroom_retrieve` / `headroom_search` tool calls against the CCR store.
pub fn handle_mcp_ccr_tool(tool_name: &str, args: &Value, store: &dyn CcrStore) -> HeadroomResult<Value> {
    let hash = args
        .get("hash")
        .or_else(|| args.get("key"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().trim_start_matches("<<ccr:").trim_end_matches(">>"))
        .unwrap_or("");
    if hash.is_empty() {
        return Err(HeadroomError::LiveZoneSplice("missing CCR hash argument".to_string()));
    }

    match tool_name {
        "headroom_retrieve" => {
            if let Some(original) = store.get(hash) {
                Ok(json!({ "hash": hash, "found": true, "content": original }))
            } else {
                Ok(json!({ "hash": hash, "found": false, "content": null }))
            }
        }
        "headroom_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let matches = store.search(hash, query);
            Ok(json!({ "hash": hash, "query": query, "matches": matches }))
        }
        other => Err(HeadroomError::LiveZoneSplice(format!("unknown headroom MCP tool: {}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_and_sqlite_ccr_roundtrip() {
        let mem = InMemoryCcrStore::default();
        let payload = "line one\nerror: database connection reset\nline three";
        let key = compute_key(payload.as_bytes());
        assert_eq!(key.len(), 24);
        assert_eq!(marker_for(&key), format!("<<ccr:{}>>", key));

        mem.put(&key, payload);
        assert_eq!(mem.get(&key).as_deref(), Some(payload));
        let hits = mem.search(&key, "database");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("database connection reset"));

        let dir = std::env::temp_dir().join(format!("toksi_ccr_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let db_file = dir.join("ccr_test.db");
        let _ = std::fs::remove_file(&db_file);
        let sqlite = SqliteCcrStore::open(&db_file, 60).expect("sqlite open");
        sqlite.put(&key, payload);
        assert_eq!(sqlite.get(&key).as_deref(), Some(payload));

        let mcp_res = handle_mcp_ccr_tool("headroom_retrieve", &json!({ "hash": marker_for(&key) }), &sqlite)
            .expect("mcp retrieve");
        assert_eq!(mcp_res["found"], Value::Bool(true));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
