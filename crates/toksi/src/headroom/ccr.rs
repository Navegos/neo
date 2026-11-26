// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/ccr.rs
// created: 2026-11-13
// lastModified: 2026-11-18

//! Compress-Cache-Retrieve (CCR) reversible storage (`<<ccr:HASH>>`) with SQLite WAL +
//! in-memory backends and MCP retrieval (`headroom_retrieve` / `headroom_search`).

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
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
/// Default bounded capacity for in-memory CCR stores.
pub const DEFAULT_IN_MEMORY_CAPACITY: usize = 2048;
/// Maximum clamped TTL in seconds so `now + ttl_seconds` in SQLite signed 64-bit arithmetic never overflows.
pub const MAX_SQLITE_TTL_SECS: i64 = i64::MAX / 4;

/// Clamps an arbitrary `u64` TTL into `1..=MAX_SQLITE_TTL_SECS` (zero stays 0).
fn clamp_ttl_secs(secs: u64) -> i64 {
    i64::try_from(secs)
        .unwrap_or(MAX_SQLITE_TTL_SECS)
        .min(MAX_SQLITE_TTL_SECS)
}

/// Redacts credentials (`user:password@` or `:password@`, plus sensitive query parameters) from URLs before logging.
pub fn redact_url_credentials(raw_url: &str) -> String {
    let trimmed = raw_url.trim();
    let Some((scheme, rest)) = trimmed.split_once("://") else {
        return "[redacted-url]".to_string();
    };
    let no_query = rest.split(&['?', '#'][..]).next().unwrap_or(rest);
    let (authority, path) = match no_query.find('/') {
        Some(idx) => (&no_query[..idx], &no_query[idx..]),
        None => (no_query, ""),
    };
    let clean_host = match authority.rfind('@') {
        Some(idx) => &authority[idx + 1..],
        None => authority,
    };
    if path.is_empty() || path == "/" {
        format!("{}://{}", scheme, clean_host)
    } else {
        format!("{}://{}{}", scheme, clean_host, path)
    }
}

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
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(MAX_SQLITE_TTL_SECS))
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
struct MemEntry {
    original: String,
    seq: u64,
    created_at: i64,
    last_accessed: i64,
    ttl_seconds: i64,
}

/// Thread-safe bounded in-memory CCR store (default for unit tests and ephemeral runs).
#[derive(Debug)]
pub struct InMemoryCcrStore {
    entries: RwLock<HashMap<String, MemEntry>>,
    order: Mutex<VecDeque<(String, u64)>>,
    next_seq: AtomicU64,
    max_entries: usize,
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
        Self::with_capacity(default_ttl_seconds, DEFAULT_IN_MEMORY_CAPACITY)
    }

    pub fn with_capacity(default_ttl_seconds: u64, max_entries: usize) -> Self {
        let ttl = clamp_ttl_secs(default_ttl_seconds);
        let max_life = ttl
            .saturating_mul(i64::from(DEFAULT_MAX_LIFETIME_MULTIPLIER))
            .min(MAX_SQLITE_TTL_SECS);
        Self {
            entries: RwLock::new(HashMap::new()),
            order: Mutex::new(VecDeque::new()),
            next_seq: AtomicU64::new(1),
            max_entries: max_entries.max(1),
            default_ttl_seconds: ttl,
            max_lifetime_seconds: max_life,
        }
    }

    fn is_expired(&self, e: &MemEntry, now: i64) -> bool {
        e.last_accessed.saturating_add(e.ttl_seconds) < now
            || e.created_at.saturating_add(self.max_lifetime_seconds) < now
    }

    fn compact_order(&self, order: &mut VecDeque<(String, u64)>, map: &HashMap<String, MemEntry>) {
        let limit = self.max_entries.max(16).saturating_mul(2);
        if order.len() > limit {
            order.retain(|(k, seq)| map.get(k).is_some_and(|e| e.seq == *seq));
        }
    }
}

impl CcrStore for InMemoryCcrStore {
    fn put(&self, hash: &str, payload: &str) {
        let now = now_unix_secs();
        let mut order = self.order.lock().unwrap_or_else(|e| e.into_inner());
        let mut map = self.entries.write().unwrap_or_else(|e| e.into_inner());
        map.retain(|_, v| !self.is_expired(v, now));

        if let Some(existing) = map.get_mut(hash) {
            existing.original = payload.to_string();
            existing.created_at = now;
            existing.last_accessed = now;
            existing.ttl_seconds = self.default_ttl_seconds;
            return;
        }

        while map.len() >= self.max_entries {
            let Some((oldest_key, oldest_seq)) = order.pop_front() else {
                break;
            };
            if map.get(&oldest_key).is_some_and(|e| e.seq == oldest_seq) {
                map.remove(&oldest_key);
            }
        }

        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        let saved_tokens = (payload.len() as u64) / 4;
        if saved_tokens > 0 {
            crate::telemetry::global_collector().record_layer_savings("ccr", saved_tokens);
        }
        map.insert(
            hash.to_string(),
            MemEntry {
                original: payload.to_string(),
                seq,
                created_at: now,
                last_accessed: now,
                ttl_seconds: self.default_ttl_seconds,
            },
        );
        order.push_back((hash.to_string(), seq));
        self.compact_order(&mut order, &map);
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
    read_only: bool,
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

        let ttl = clamp_ttl_secs(default_ttl_seconds);
        let max_life = ttl
            .saturating_mul(i64::from(DEFAULT_MAX_LIFETIME_MULTIPLIER))
            .min(MAX_SQLITE_TTL_SECS);
        Ok(Self {
            conn: Mutex::new(conn),
            default_ttl_seconds: ttl,
            max_lifetime_seconds: max_life,
            path: path_buf,
            read_only: false,
        })
    }

    pub fn open_read_only(path: impl AsRef<Path>, default_ttl_seconds: u64) -> HeadroomResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let conn = Connection::open_with_flags(
            &path_buf,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let _ = conn.pragma_update(None, "busy_timeout", 5000);
        let ttl = clamp_ttl_secs(default_ttl_seconds);
        let max_life = ttl
            .saturating_mul(i64::from(DEFAULT_MAX_LIFETIME_MULTIPLIER))
            .min(MAX_SQLITE_TTL_SECS);
        Ok(Self {
            conn: Mutex::new(conn),
            default_ttl_seconds: ttl,
            max_lifetime_seconds: max_life,
            path: path_buf,
            read_only: true,
        })
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn purge_expired(&self, conn: &Connection, now: i64) {
        if self.read_only {
            return;
        }
        let _ = conn.execute(
            "DELETE FROM ccr_entries WHERE last_accessed + ttl_seconds < ?1 OR created_at + ?2 < ?1",
            params![now, self.max_lifetime_seconds],
        );
    }
}

impl CcrStore for SqliteCcrStore {
    fn put(&self, hash: &str, payload: &str) {
        if self.read_only {
            return;
        }
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
            if !self.read_only {
                let _ = conn.execute(
                    "UPDATE ccr_entries SET last_accessed = ?1 WHERE hash = ?2",
                    params![now, hash],
                );
            }
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

/// Handles MCP `headroom_{compress,retrieve,stats,read,search}` tool calls against the CCR store.
pub fn handle_mcp_ccr_tool(tool_name: &str, args: &Value, store: &dyn CcrStore) -> HeadroomResult<Value> {
    if tool_name == "headroom_stats" {
        let c = crate::telemetry::collector::global_collector();
        let raw = c.headroom_raw_bytes_total.load(std::sync::atomic::Ordering::Relaxed);
        let compressed = c.headroom_compressed_bytes_total.load(std::sync::atomic::Ordering::Relaxed);
        let pruned = c.headroom_mcp_pruned_tools_total.load(std::sync::atomic::Ordering::Relaxed);
        return Ok(json!({
            "raw_bytes_total": raw,
            "compressed_bytes_total": compressed,
            "bytes_saved": raw.saturating_sub(compressed),
            "mcp_tools_pruned": pruned,
        }));
    }

    if tool_name == "headroom_compress" {
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let hint = args.get("hint").and_then(|v| v.as_str());
        let compressed = crate::headroom::compression::compress_content(content, hint);
        return Ok(json!({
            "compressed": compressed,
            "original_bytes": content.len(),
            "compressed_bytes": compressed.len(),
        }));
    }

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
        "headroom_retrieve" | "headroom_read" => {
            if let Some(original) = store.get(hash) {
                let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let limit = args.get("limit").and_then(|v| v.as_u64()).map(|l| l as usize);
                let sliced = if offset > 0 || limit.is_some() {
                    let start = offset.min(original.len());
                    let end = limit
                        .map(|l| start.saturating_add(l).min(original.len()))
                        .unwrap_or(original.len());
                    String::from_utf8_lossy(&original.as_bytes()[start..end]).to_string()
                } else {
                    original
                };
                Ok(json!({ "hash": hash, "found": true, "content": sliced }))
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
        let sqlite_res = SqliteCcrStore::open(&db_file, 60);
        assert!(sqlite_res.is_ok());
        if let Ok(sqlite) = sqlite_res {
            sqlite.put(&key, payload);
            assert_eq!(sqlite.get(&key).as_deref(), Some(payload));

            let mcp_res = handle_mcp_ccr_tool(
                "headroom_retrieve",
                &json!({ "hash": marker_for(&key) }),
                &sqlite,
            );
            assert!(mcp_res.is_ok());
            if let Ok(res_val) = mcp_res {
                assert_eq!(res_val["found"], Value::Bool(true));
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sqlite_huge_ttl_and_url_redaction() {
        let dir = std::env::temp_dir().join(format!("toksi_ccr_huge_ttl_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let db_file = dir.join("ccr_huge.db");
        let _ = std::fs::remove_file(&db_file);
        let sqlite_res = SqliteCcrStore::open(&db_file, u64::MAX);
        assert!(sqlite_res.is_ok());
        if let Ok(sqlite) = sqlite_res {
            sqlite.put("k", "payload");
            assert_eq!(sqlite.get("k").as_deref(), Some("payload"));
        }
        let _ = std::fs::remove_dir_all(&dir);

        let redacted = redact_url_credentials("redis://user:secret@cache.internal:6379/0?token=abc");
        assert!(!redacted.contains("secret"));
        assert!(!redacted.contains("token=abc"));
        assert_eq!(redacted, "redis://cache.internal:6379/0");
    }

    #[test]
    fn test_in_memory_reput_and_capacity_eviction() {
        let store = InMemoryCcrStore::with_capacity(60, 2);
        store.put("a", "v1");
        store.put("b", "v2");
        store.put("a", "v1-updated");
        store.put("c", "v3");
        // Capacity 2 keeps at most 2 entries and preserves the updated "a" or newest "c"
        assert!(store.len() <= 2);
        assert_eq!(store.get("c").as_deref(), Some("v3"));
    }

    #[test]
    fn test_sqlite_ccr_read_only_mode() {
        let dir = std::env::temp_dir().join(format!("toksi_ccr_ro_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let db_file = dir.join("ccr_ro.db");
        let _ = std::fs::remove_file(&db_file);

        if let Ok(rw) = SqliteCcrStore::open(&db_file, 60) {
            assert!(!rw.is_read_only());
            rw.put("k1", "orig-value");
        }
        if let Ok(ro) = SqliteCcrStore::open_read_only(&db_file, 60) {
            assert!(ro.is_read_only());
            assert_eq!(ro.get("k1").as_deref(), Some("orig-value"));
            ro.put("k2", "ignored");
            assert_eq!(ro.get("k2"), None);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
