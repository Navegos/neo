// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/store.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::icm::fusion::{rrf_fuse, RankedList, RRF_K};
use crate::icm::schema::{
    Concept, ConceptLink, Fact, Feedback, IcmError, IcmResult, IcmStats, Importance,
    Memoir, Memory, Message, RankedHit, SearchHit, Session, Topic, TopicHealth,
};
use crate::icm::temporal::parse_query_window;
use crate::util::paths::normalize_path;

pub const DEFAULT_DB_FILENAME: &str = "memory.db";

pub fn compute_summary_hash(topic: &str, summary: &str) -> String {
    let mut hasher = DefaultHasher::new();
    topic.trim().to_lowercase().hash(&mut hasher);
    summary.trim().to_lowercase().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Locates the strict repository root containing `.git`.
/// Prohibits access inside `.git` or falling back to user home.
pub fn find_repo_root_and_subproject(start: Option<&Path>) -> Result<(PathBuf, String), IcmError> {
    let initial = if let Some(p) = start {
        p.to_path_buf()
    } else if let Ok(root) = std::env::var("PROJECTMEM_ROOT") {
        if !root.trim().is_empty() {
            PathBuf::from(root)
        } else {
            std::env::current_dir().map_err(IcmError::Io)?
        }
    } else {
        std::env::current_dir().map_err(IcmError::Io)?
    };

    let canonical = initial.canonicalize().unwrap_or(initial);

    // Prohibition: Never allow operations directly inside .git
    for comp in canonical.components() {
        if comp.as_os_str() == ".git" {
            return Err(IcmError::Security(
                "Access or storage inside '.git' directory is strictly prohibited.".to_string(),
            ));
        }
    }

    let mut current = canonical.as_path();
    let mut found_repo: Option<PathBuf> = None;

    loop {
        let git_entry = current.join(".git");
        if git_entry.exists() {
            found_repo = Some(current.to_path_buf());
            break;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    let repo_root = match found_repo {
        Some(r) => {
            // Prohibit bare home directory masquerading as repository
            if let Some(home) = dirs::home_dir() {
                if let Ok(canon_home) = home.canonicalize() {
                    if canon_home == r {
                        return Err(IcmError::RepoNotFound(
                            "Storage in user home root '~' is strictly prohibited; memory must live in a project repository.".to_string(),
                        ));
                    }
                }
            }
            r
        }
        None => {
            if cfg!(test) {
                canonical.clone()
            } else {
                return Err(IcmError::RepoNotFound(
                    "Strict repo-only consolidation: no '.git' repository root found in directory hierarchy.".to_string(),
                ));
            }
        }
    };

    let subproject_id = if canonical != repo_root {
        if let Ok(rel) = canonical.strip_prefix(&repo_root) {
            normalize_path(&rel.to_string_lossy())
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok((repo_root, subproject_id))
}

pub fn resolve_icm_db_path() -> PathBuf {
    resolve_icm_db_path_from(None).unwrap_or_else(|_| PathBuf::from(".toksi").join("icm").join(DEFAULT_DB_FILENAME))
}

pub fn resolve_icm_db_path_checked() -> Result<PathBuf, IcmError> {
    resolve_icm_db_path_from(None)
}

pub fn resolve_icm_db_path_from(start: Option<&Path>) -> Result<PathBuf, IcmError> {
    let (repo_root, _subproject_id) = find_repo_root_and_subproject(start)?;
    let icm_dir = repo_root.join(".toksi").join("icm");
    if let Err(e) = std::fs::create_dir_all(&icm_dir) {
        return Err(IcmError::Io(e));
    }
    let target = icm_dir.join(DEFAULT_DB_FILENAME);

    // Auto-migrate legacy root .toksi/memory.db if present and target does not exist
    let legacy_root = repo_root.join(".toksi").join(DEFAULT_DB_FILENAME);
    if legacy_root.exists() && !target.exists() {
        let _ = std::fs::rename(&legacy_root, &target);
    }
    // Auto-migrate legacy .icm/memory.db
    let legacy_icm = repo_root.join(".icm").join(DEFAULT_DB_FILENAME);
    if legacy_icm.exists() && !target.exists() {
        let _ = std::fs::rename(&legacy_icm, &target);
    }

    Ok(target)
}

pub struct IcmStore {
    conn: Connection,
    path: PathBuf,
    subproject_id: String,
}

impl IcmStore {
    pub fn open<P: AsRef<Path>>(path: P) -> IcmResult<Self> {
        let norm = normalize_path(&path.as_ref().to_string_lossy());
        let path = PathBuf::from(norm);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        let _ = conn.busy_timeout(std::time::Duration::from_millis(5000));
        let mut store = Self {
            conn,
            path,
            subproject_id: String::new(),
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_default() -> IcmResult<Self> {
        let (repo_root, subproject_id) = find_repo_root_and_subproject(None)?;
        let path = resolve_icm_db_path_from(Some(&repo_root))?;
        let mut store = Self::open(path)?;
        store.subproject_id = subproject_id;
        Ok(store)
    }

    pub fn open_in_memory() -> IcmResult<Self> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self {
            conn,
            path: PathBuf::from(":memory:"),
            subproject_id: String::new(),
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn subproject_id(&self) -> &str {
        &self.subproject_id
    }

    pub fn set_subproject_id(&mut self, id: impl Into<String>) {
        self.subproject_id = id.into();
    }

    fn init_schema(&mut self) -> IcmResult<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

            -- 1. Core Memories
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT '',
                last_accessed TEXT NOT NULL,
                access_count INTEGER DEFAULT 0,
                weight REAL DEFAULT 1.0,

                topic TEXT NOT NULL,
                summary TEXT NOT NULL,
                raw_excerpt TEXT,
                keywords TEXT, -- JSON array

                importance TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_data TEXT, -- JSON

                related_ids TEXT, -- JSON array
                summary_hash TEXT UNIQUE,
                subproject_id TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_memories_topic ON memories(topic);
            CREATE INDEX IF NOT EXISTS idx_memories_weight ON memories(weight);
            CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at);
            CREATE INDEX IF NOT EXISTS idx_memories_last_accessed ON memories(last_accessed);
            CREATE INDEX IF NOT EXISTS idx_memories_topic_hash ON memories(summary_hash) WHERE summary_hash IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_memories_subproject ON memories(subproject_id);

            -- FTS5 table for memories
            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                id UNINDEXED,
                topic,
                summary,
                keywords,
                content='memories',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, id, topic, summary, keywords)
                VALUES (new.rowid, new.id, new.topic, new.summary, new.keywords);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, id, topic, summary, keywords)
                VALUES('delete', old.rowid, old.id, old.topic, old.summary, old.keywords);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE OF topic, summary, keywords ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, id, topic, summary, keywords)
                VALUES('delete', old.rowid, old.id, old.topic, old.summary, old.keywords);
                INSERT INTO memories_fts(rowid, id, topic, summary, keywords)
                VALUES (new.rowid, new.id, new.topic, new.summary, new.keywords);
            END;

            -- 2. Structured Facts
            CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                entity TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                superseded_at TEXT,
                subproject_id TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_facts_entity ON facts(entity);
            CREATE INDEX IF NOT EXISTS idx_facts_entity_key ON facts(entity, key);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_facts_active_unique
                ON facts(entity, key) WHERE superseded_at IS NULL;

            -- 3. Memoirs and Concepts
            CREATE TABLE IF NOT EXISTS memoirs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                consolidation_threshold INTEGER NOT NULL DEFAULT 50
            );

            CREATE TABLE IF NOT EXISTS concepts (
                id TEXT PRIMARY KEY,
                memoir_id TEXT NOT NULL REFERENCES memoirs(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                definition TEXT NOT NULL,
                labels TEXT NOT NULL DEFAULT '[]',
                confidence REAL NOT NULL DEFAULT 0.5,
                revision INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                source_memory_ids TEXT NOT NULL DEFAULT '[]',
                UNIQUE(memoir_id, name)
            );
            CREATE INDEX IF NOT EXISTS idx_concepts_memoir ON concepts(memoir_id);
            CREATE INDEX IF NOT EXISTS idx_concepts_name ON concepts(name);

            CREATE VIRTUAL TABLE IF NOT EXISTS concepts_fts USING fts5(
                id UNINDEXED,
                name,
                definition,
                labels,
                content='concepts',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS concepts_ai AFTER INSERT ON concepts BEGIN
                INSERT INTO concepts_fts(rowid, id, name, definition, labels)
                VALUES (new.rowid, new.id, new.name, new.definition, new.labels);
            END;

            CREATE TRIGGER IF NOT EXISTS concepts_ad AFTER DELETE ON concepts BEGIN
                INSERT INTO concepts_fts(concepts_fts, rowid, id, name, definition, labels)
                VALUES('delete', old.rowid, old.id, old.name, old.definition, old.labels);
            END;

            CREATE TRIGGER IF NOT EXISTS concepts_au AFTER UPDATE OF name, definition, labels ON concepts BEGIN
                INSERT INTO concepts_fts(concepts_fts, rowid, id, name, definition, labels)
                VALUES('delete', old.rowid, old.id, old.name, old.definition, old.labels);
                INSERT INTO concepts_fts(rowid, id, name, definition, labels)
                VALUES (new.rowid, new.id, new.name, new.definition, new.labels);
            END;

            CREATE TABLE IF NOT EXISTS concept_links (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
                target_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
                relation TEXT NOT NULL,
                weight REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL,
                UNIQUE(source_id, target_id, relation),
                CHECK(source_id != target_id)
            );
            CREATE INDEX IF NOT EXISTS idx_concept_links_source ON concept_links(source_id);
            CREATE INDEX IF NOT EXISTS idx_concept_links_target ON concept_links(target_id);

            -- 4. Feedback
            CREATE TABLE IF NOT EXISTS feedback (
                id TEXT PRIMARY KEY,
                topic TEXT NOT NULL,
                context TEXT NOT NULL,
                predicted TEXT NOT NULL,
                corrected TEXT NOT NULL,
                reason TEXT,
                source TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                applied_count INTEGER DEFAULT 0,
                subproject_id TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_feedback_topic ON feedback(topic);

            CREATE VIRTUAL TABLE IF NOT EXISTS feedback_fts USING fts5(
                id UNINDEXED,
                topic,
                context,
                predicted,
                corrected,
                reason,
                content='feedback',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS feedback_ai AFTER INSERT ON feedback BEGIN
                INSERT INTO feedback_fts(rowid, id, topic, context, predicted, corrected, reason)
                VALUES (new.rowid, new.id, new.topic, new.context, new.predicted, new.corrected, new.reason);
            END;

            CREATE TRIGGER IF NOT EXISTS feedback_ad AFTER DELETE ON feedback BEGIN
                INSERT INTO feedback_fts(feedback_fts, rowid, id, topic, context, predicted, corrected, reason)
                VALUES('delete', old.rowid, old.id, old.topic, old.context, old.predicted, old.corrected, old.reason);
            END;

            -- 5. Transcripts (Sessions and Messages)
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent TEXT NOT NULL DEFAULT '',
                project TEXT,
                started_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                subproject_id TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project);
            CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                tool_name TEXT,
                tokens INTEGER,
                ts TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}'
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            CREATE INDEX IF NOT EXISTS idx_messages_ts ON messages(ts);

            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                id UNINDEXED,
                role,
                tool_name,
                content,
                content='messages',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, id, role, tool_name, content)
                VALUES (new.rowid, new.id, new.role, new.tool_name, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, id, role, tool_name, content)
                VALUES('delete', old.rowid, old.id, old.role, old.tool_name, old.content);
            END;

            -- 6. Metadata
            CREATE TABLE IF NOT EXISTS icm_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn insert_or_update_memory(&mut self, mem: &Memory) -> IcmResult<()> {
        let keywords_json = serde_json::to_string(&mem.keywords).unwrap_or_else(|_| "[]".into());
        let related_json = serde_json::to_string(&mem.related_ids).unwrap_or_else(|_| "[]".into());
        let hash = mem.summary_hash.clone().unwrap_or_else(|| compute_summary_hash(&mem.topic, &mem.summary));
        let sub = mem.subproject_id.as_deref().unwrap_or(&self.subproject_id);

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO memories (
                    id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash, subproject_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                ON CONFLICT(summary_hash) DO UPDATE SET
                    updated_at = excluded.updated_at,
                    last_accessed = excluded.last_accessed,
                    access_count = memories.access_count + 1,
                    weight = MAX(memories.weight, excluded.weight),
                    summary = excluded.summary,
                    raw_excerpt = coalesce(excluded.raw_excerpt, memories.raw_excerpt),
                    keywords = excluded.keywords,
                    importance = excluded.importance,
                    related_ids = excluded.related_ids,
                    subproject_id = excluded.subproject_id;",
            )?;

            stmt.execute(params![
                mem.id,
                mem.created_at,
                mem.updated_at,
                mem.last_accessed,
                mem.access_count,
                mem.weight,
                mem.topic,
                mem.summary,
                mem.raw_excerpt,
                keywords_json,
                mem.importance.as_str(),
                mem.source_type,
                mem.source_data,
                related_json,
                hash,
                sub,
            ])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_memory(&self, id: &str) -> IcmResult<Option<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash, subproject_id
             FROM memories WHERE id = ?1",
        )?;

        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_memory(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_memory(&mut self, id: &str) -> IcmResult<bool> {
        let count = self.conn.execute("DELETE FROM memories WHERE id = ?1", [id])?;
        Ok(count > 0)
    }

    pub fn pin_memory(&mut self, id: &str) -> IcmResult<Memory> {
        let now = Utc::now().to_rfc3339();
        let changed = self.conn.execute(
            "UPDATE memories SET importance = 'critical', weight = 1.0, updated_at = ?2, last_accessed = ?2 WHERE id = ?1",
            params![id, now],
        )?;
        if changed == 0 {
            return Err(IcmError::NotFound(format!("Memory id '{id}' not found")));
        }
        self.get_memory(id)?
            .ok_or_else(|| IcmError::NotFound(format!("Memory id '{id}' not found")))
    }

    pub fn forget_topic(&mut self, topic: &str) -> IcmResult<usize> {
        let count = self.conn.execute(
            "DELETE FROM memories WHERE topic = ?1 AND importance != 'critical'",
            [topic],
        )?;
        Ok(count)
    }

    pub fn touch_memory(&mut self, id: &str) -> IcmResult<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE memories SET last_accessed = ?1, access_count = access_count + 1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn apply_decay(&mut self, decay_factor: f32) -> IcmResult<usize> {
        let factor = decay_factor.clamp(0.01, 0.99);
        let changed = self.conn.execute(
            "UPDATE memories SET weight = weight * MAX(0.0,
                1.0 - (1.0 - ?1) *
                CASE importance
                    WHEN 'high' THEN 0.5
                    WHEN 'low' THEN 2.0
                    ELSE 1.0
                END
                / (1.0 + MIN(access_count, 5) * 0.1)
            )
            WHERE importance != 'critical'",
            params![factor],
        )?;
        Ok(changed)
    }

    pub fn consolidate_topic(&mut self, topic: &str, threshold: usize) -> IcmResult<usize> {
        let count: usize = self.conn.query_row(
            "SELECT count(*) FROM memories WHERE topic = ?1 AND importance != 'critical'",
            [topic],
            |row| row.get(0),
        )?;

        if count <= threshold {
            return Ok(0);
        }

        let to_remove = count - threshold;
        let deleted = self.conn.execute(
            "DELETE FROM memories WHERE id IN (
                SELECT id FROM memories
                WHERE topic = ?1 AND importance != 'critical'
                ORDER BY weight ASC, access_count ASC, last_accessed ASC
                LIMIT ?2
            )",
            params![topic, to_remove as i64],
        )?;

        Ok(deleted)
    }

    pub fn search_bm25(&self, query: &str, limit: usize) -> IcmResult<Vec<SearchHit>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.get_recent(limit);
        }

        let sanitized = sanitize_fts5_query_any(trimmed);
        if sanitized.is_empty() {
            return self.get_recent(limit);
        }

        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.created_at, m.updated_at, m.last_accessed, m.access_count, m.weight,
                    m.topic, m.summary, m.raw_excerpt, m.keywords, m.importance, m.source_type,
                    m.source_data, m.related_ids, m.summary_hash, m.subproject_id,
                    bm25(memories_fts) as rank_score
             FROM memories_fts f
             JOIN memories m ON f.rowid = m.rowid
             WHERE memories_fts MATCH ?1
             ORDER BY rank_score ASC, m.weight DESC, m.last_accessed DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![sanitized, limit as i64], |row| {
            let memory = Self::row_to_memory(row)?;
            let bm25_score: f64 = row.get(16)?;
            let score = (-bm25_score) as f32;
            Ok((memory, score))
        })?;

        let mut hits = Vec::new();
        for (i, item) in rows.flatten().enumerate() {
            hits.push(SearchHit {
                memory: item.0,
                score: item.1,
                rank: i + 1,
            });
        }
        Ok(hits)
    }

    /// Hybrid search: Lexical BM25 + Temporal Window RRF Fusion
    pub fn search_fused(&self, query: &str, limit: usize, now: DateTime<Utc>) -> IcmResult<Vec<RankedHit>> {
        let window = parse_query_window(query, now);

        // 1. Lexical list
        let fts_hits = self.search_bm25(query, limit * 2)?;
        let fts_ids: Vec<String> = fts_hits.iter().map(|h| h.memory.id.clone()).collect();

        // 2. Temporal recency list
        let mut time_ids = Vec::new();
        if let Some(win) = window {
            let start_str = win.start.to_rfc3339();
            let end_str = win.end.to_rfc3339();
            let mut stmt = self.conn.prepare(
                "SELECT id FROM memories
                 WHERE created_at >= ?1 AND created_at < ?2
                 ORDER BY weight DESC, last_accessed DESC
                 LIMIT ?3",
            )?;
            let rows = stmt.query_map(params![start_str, end_str, (limit * 2) as i64], |r| r.get::<_, String>(0))?;
            time_ids = rows.flatten().collect();
        }

        let lists = [
            RankedList {
                name: "fts",
                ids: &fts_ids,
                weight: 1.0,
            },
            RankedList {
                name: "temporal",
                ids: &time_ids,
                weight: if window.is_some() { 1.2 } else { 0.0 },
            },
        ];

        let fused = rrf_fuse(&lists, RRF_K);
        let mut ranked = Vec::new();

        for hit in fused.into_iter().take(limit) {
            if let Some(mem) = self.get_memory(&hit.id)? {
                let fts_pos = fts_ids.iter().position(|id| id == &hit.id).map(|p| (p + 1) as u32);
                let time_pos = time_ids.iter().position(|id| id == &hit.id).map(|p| (p + 1) as u32);
                ranked.push(RankedHit {
                    memory: mem,
                    score: hit.score,
                    fts_rank: fts_pos,
                    time_rank: time_pos,
                });
            }
        }

        Ok(ranked)
    }

    pub fn get_recent(&self, limit: usize) -> IcmResult<Vec<SearchHit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash, subproject_id
             FROM memories
             ORDER BY weight DESC, last_accessed DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit as i64], |row| {
            let memory = Self::row_to_memory(row)?;
            Ok(memory)
        })?;

        let mut hits = Vec::new();
        for (i, m) in rows.flatten().enumerate() {
            hits.push(SearchHit {
                memory: m,
                score: 1.0 / ((i + 1) as f32),
                rank: i + 1,
            });
        }
        Ok(hits)
    }

    pub fn list_topics(&self) -> IcmResult<Vec<Topic>> {
        let mut stmt = self.conn.prepare(
            "SELECT topic, count(*) as cnt, avg(weight) as avg_w, max(last_accessed) as last_acc
             FROM memories
             GROUP BY topic
             ORDER BY cnt DESC, avg_w DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Topic {
                name: row.get(0)?,
                count: row.get(1)?,
                avg_weight: row.get::<_, Option<f64>>(2)?.unwrap_or(1.0) as f32,
                last_accessed: row.get(3)?,
            })
        })?;

        let mut topics = Vec::new();
        for t in rows.flatten() {
            topics.push(t);
        }
        Ok(topics)
    }

    // --- Facts Operations ---

    pub fn set_fact(&mut self, entity: &str, key: &str, value: &str, source: &str) -> IcmResult<String> {
        let now = Utc::now().to_rfc3339();
        let fact_id = format!("fact_{:x}", rand_u64());

        let tx = self.conn.transaction()?;
        {
            tx.execute(
                "UPDATE facts SET superseded_at = ?1 WHERE entity = ?2 AND key = ?3 AND superseded_at IS NULL",
                params![now, entity, key],
            )?;

            tx.execute(
                "INSERT INTO facts (id, entity, key, value, source, created_at, superseded_at, subproject_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)",
                params![fact_id, entity, key, value, source, now, self.subproject_id],
            )?;
        }
        tx.commit()?;
        Ok(fact_id)
    }

    pub fn get_fact(&self, entity: &str, key: &str) -> IcmResult<Option<Fact>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity, key, value, source, created_at, superseded_at, subproject_id
             FROM facts
             WHERE entity = ?1 AND key = ?2 AND superseded_at IS NULL",
        )?;

        let mut rows = stmt.query([entity, key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Fact {
                id: row.get(0)?,
                entity: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
                superseded_at: row.get(6)?,
                subproject_id: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_facts(&self, entity: Option<&str>) -> IcmResult<Vec<Fact>> {
        let sql = match entity {
            Some(_) => "SELECT id, entity, key, value, source, created_at, superseded_at, subproject_id
                        FROM facts WHERE entity = ?1 AND superseded_at IS NULL ORDER BY key ASC",
            None => "SELECT id, entity, key, value, source, created_at, superseded_at, subproject_id
                     FROM facts WHERE superseded_at IS NULL ORDER BY entity ASC, key ASC",
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = if let Some(e) = entity {
            stmt.query_map([e], Self::row_to_fact)?
        } else {
            stmt.query_map([], Self::row_to_fact)?
        };

        let mut facts = Vec::new();
        for f in rows.flatten() {
            facts.push(f);
        }
        Ok(facts)
    }

    // --- Memoirs & Concepts Operations ---

    pub fn create_memoir(&mut self, name: &str, description: &str, threshold: usize) -> IcmResult<Memoir> {
        let now = Utc::now().to_rfc3339();
        let id = format!("memoir_{:x}", rand_u64());
        let thresh = if threshold == 0 { 50 } else { threshold };

        self.conn.execute(
            "INSERT INTO memoirs (id, name, description, created_at, updated_at, consolidation_threshold)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5)",
            params![id, name, description, now, thresh as i64],
        )?;

        Ok(Memoir {
            id,
            name: name.to_string(),
            description: description.to_string(),
            created_at: now.clone(),
            updated_at: now,
            consolidation_threshold: thresh,
        })
    }

    pub fn list_memoirs(&self) -> IcmResult<Vec<Memoir>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, updated_at, consolidation_threshold FROM memoirs ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Memoir {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                consolidation_threshold: row.get::<_, i64>(5)? as usize,
            })
        })?;
        let mut list = Vec::new();
        for m in rows.flatten() {
            list.push(m);
        }
        Ok(list)
    }

    pub fn get_memoir(&self, name_or_id: &str) -> IcmResult<Option<Memoir>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, updated_at, consolidation_threshold
             FROM memoirs WHERE id = ?1 OR name = ?1",
        )?;
        let mut rows = stmt.query([name_or_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Memoir {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                consolidation_threshold: row.get::<_, i64>(5)? as usize,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn add_concept(
        &mut self,
        memoir_id: &str,
        name: &str,
        definition: &str,
        labels: &[String],
        confidence: f32,
        source_memories: &[String],
    ) -> IcmResult<Concept> {
        let now = Utc::now().to_rfc3339();
        let id = format!("concept_{:x}", rand_u64());
        let labels_json = serde_json::to_string(labels).unwrap_or_else(|_| "[]".into());
        let sources_json = serde_json::to_string(source_memories).unwrap_or_else(|_| "[]".into());

        self.conn.execute(
            "INSERT INTO concepts (id, memoir_id, name, definition, labels, confidence, revision, created_at, updated_at, source_memory_ids)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7, ?8)",
            params![id, memoir_id, name, definition, labels_json, confidence, now, sources_json],
        )?;

        Ok(Concept {
            id,
            memoir_id: memoir_id.to_string(),
            name: name.to_string(),
            definition: definition.to_string(),
            labels: labels.to_vec(),
            confidence,
            revision: 1,
            created_at: now.clone(),
            updated_at: now,
            source_memory_ids: source_memories.to_vec(),
        })
    }

    pub fn refine_concept(&mut self, concept_id: &str, new_definition: &str, confidence: f32) -> IcmResult<Concept> {
        let now = Utc::now().to_rfc3339();
        let changed = self.conn.execute(
            "UPDATE concepts SET definition = ?1, confidence = ?2, revision = revision + 1, updated_at = ?3 WHERE id = ?4",
            params![new_definition, confidence, now, concept_id],
        )?;
        if changed == 0 {
            return Err(IcmError::NotFound(format!("Concept '{concept_id}' not found")));
        }

        let mut stmt = self.conn.prepare(
            "SELECT id, memoir_id, name, definition, labels, confidence, revision, created_at, updated_at, source_memory_ids
             FROM concepts WHERE id = ?1",
        )?;
        let row = stmt.query_row([concept_id], Self::row_to_concept)?;
        Ok(row)
    }

    pub fn search_concepts(&self, memoir_id: &str, query: &str, limit: usize) -> IcmResult<Vec<Concept>> {
        let sanitized = sanitize_fts5_query_any(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.memoir_id, c.name, c.definition, c.labels, c.confidence, c.revision, c.created_at, c.updated_at, c.source_memory_ids
             FROM concepts_fts f
             JOIN concepts c ON f.rowid = c.rowid
             WHERE concepts_fts MATCH ?1 AND c.memoir_id = ?2
             ORDER BY bm25(concepts_fts) ASC, c.confidence DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![sanitized, memoir_id, limit as i64], Self::row_to_concept)?;
        Ok(rows.flatten().collect())
    }

    pub fn search_all_concepts(&self, query: &str, limit: usize) -> IcmResult<Vec<Concept>> {
        let sanitized = sanitize_fts5_query_any(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.memoir_id, c.name, c.definition, c.labels, c.confidence, c.revision, c.created_at, c.updated_at, c.source_memory_ids
             FROM concepts_fts f
             JOIN concepts c ON f.rowid = c.rowid
             WHERE concepts_fts MATCH ?1
             ORDER BY bm25(concepts_fts) ASC, c.confidence DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![sanitized, limit as i64], Self::row_to_concept)?;
        Ok(rows.flatten().collect())
    }

    pub fn link_concepts(&mut self, source_id: &str, target_id: &str, relation: &str, weight: f32) -> IcmResult<ConceptLink> {
        let now = Utc::now().to_rfc3339();
        let id = format!("link_{:x}", rand_u64());

        self.conn.execute(
            "INSERT INTO concept_links (id, source_id, target_id, relation, weight, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(source_id, target_id, relation) DO UPDATE SET weight = excluded.weight",
            params![id, source_id, target_id, relation, weight, now],
        )?;

        Ok(ConceptLink {
            id,
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relation: relation.to_string(),
            weight,
            created_at: now,
        })
    }

    pub fn inspect_memoir(&self, memoir_id: &str) -> IcmResult<Value> {
        let memoir = self.get_memoir(memoir_id)?
            .ok_or_else(|| IcmError::NotFound(format!("Memoir '{memoir_id}' not found")))?;

        let mut stmt = self.conn.prepare(
            "SELECT id, memoir_id, name, definition, labels, confidence, revision, created_at, updated_at, source_memory_ids
             FROM concepts WHERE memoir_id = ?1 ORDER BY confidence DESC",
        )?;
        let concepts: Vec<Concept> = stmt.query_map([&memoir.id], Self::row_to_concept)?.flatten().collect();

        let mut link_stmt = self.conn.prepare(
            "SELECT l.id, l.source_id, l.target_id, l.relation, l.weight, l.created_at
             FROM concept_links l
             JOIN concepts c ON l.source_id = c.id
             WHERE c.memoir_id = ?1",
        )?;
        let links: Vec<ConceptLink> = link_stmt.query_map([&memoir.id], |r| {
            Ok(ConceptLink {
                id: r.get(0)?,
                source_id: r.get(1)?,
                target_id: r.get(2)?,
                relation: r.get(3)?,
                weight: r.get::<_, f64>(4)? as f32,
                created_at: r.get(5)?,
            })
        })?.flatten().collect();

        Ok(json!({
            "memoir": memoir,
            "concepts_count": concepts.len(),
            "concepts": concepts,
            "links_count": links.len(),
            "links": links
        }))
    }

    pub fn export_memoir(&self, memoir_id: &str) -> IcmResult<String> {
        let data = self.inspect_memoir(memoir_id)?;
        serde_json::to_string_pretty(&data).map_err(IcmError::Json)
    }

    // --- Feedback Operations ---

    pub fn record_feedback(
        &mut self,
        topic: &str,
        context: &str,
        predicted: &str,
        corrected: &str,
        reason: Option<&str>,
        source: &str,
    ) -> IcmResult<Feedback> {
        let now = Utc::now().to_rfc3339();
        let id = format!("fb_{:x}", rand_u64());

        self.conn.execute(
            "INSERT INTO feedback (id, topic, context, predicted, corrected, reason, source, created_at, applied_count, subproject_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)",
            params![id, topic, context, predicted, corrected, reason, source, now, self.subproject_id],
        )?;

        Ok(Feedback {
            id,
            topic: topic.to_string(),
            context: context.to_string(),
            predicted: predicted.to_string(),
            corrected: corrected.to_string(),
            reason: reason.map(|r| r.to_string()),
            source: source.to_string(),
            created_at: now,
            applied_count: 0,
            subproject_id: Some(self.subproject_id.clone()),
        })
    }

    pub fn search_feedback(&self, query: &str, topic: Option<&str>, limit: usize) -> IcmResult<Vec<Feedback>> {
        let sanitized = sanitize_fts5_query_any(query);
        let mut stmt = if sanitized.is_empty() {
            if let Some(t) = topic {
                let mut s = self.conn.prepare(
                    "SELECT id, topic, context, predicted, corrected, reason, source, created_at, applied_count, subproject_id
                     FROM feedback WHERE topic = ?1 ORDER BY created_at DESC LIMIT ?2",
                )?;
                let rows = s.query_map(params![t, limit as i64], Self::row_to_feedback)?;
                return Ok(rows.flatten().collect());
            } else {
                let mut s = self.conn.prepare(
                    "SELECT id, topic, context, predicted, corrected, reason, source, created_at, applied_count, subproject_id
                     FROM feedback ORDER BY created_at DESC LIMIT ?1",
                )?;
                let rows = s.query_map([limit as i64], Self::row_to_feedback)?;
                return Ok(rows.flatten().collect());
            }
        } else if topic.is_some() {
            self.conn.prepare(
                "SELECT f.id, f.topic, f.context, f.predicted, f.corrected, f.reason, f.source, f.created_at, f.applied_count, f.subproject_id
                 FROM feedback_fts s
                 JOIN feedback f ON s.rowid = f.rowid
                 WHERE feedback_fts MATCH ?1 AND f.topic = ?2
                 ORDER BY bm25(feedback_fts) ASC, f.created_at DESC LIMIT ?3",
            )?
        } else {
            self.conn.prepare(
                "SELECT f.id, f.topic, f.context, f.predicted, f.corrected, f.reason, f.source, f.created_at, f.applied_count, f.subproject_id
                 FROM feedback_fts s
                 JOIN feedback f ON s.rowid = f.rowid
                 WHERE feedback_fts MATCH ?1
                 ORDER BY bm25(feedback_fts) ASC, f.created_at DESC LIMIT ?2",
            )?
        };

        let rows = if topic.is_some() {
            stmt.query_map(params![sanitized, topic, limit as i64], Self::row_to_feedback)?
        } else {
            stmt.query_map(params![sanitized, limit as i64], Self::row_to_feedback)?
        };

        Ok(rows.flatten().collect())
    }

    pub fn feedback_stats(&self) -> IcmResult<Value> {
        let total: usize = self.conn.query_row("SELECT count(*) FROM feedback", [], |r| r.get(0))?;
        let applied: usize = self.conn.query_row("SELECT coalesce(sum(applied_count), 0) FROM feedback", [], |r| r.get(0))?;
        Ok(json!({
            "total_feedback": total,
            "total_applied": applied,
        }))
    }

    // --- Transcripts Operations ---

    pub fn create_session(&mut self, agent: &str, project: Option<&str>, metadata: &str) -> IcmResult<Session> {
        let now = Utc::now().to_rfc3339();
        let id = format!("sess_{:x}", rand_u64());

        self.conn.execute(
            "INSERT INTO sessions (id, agent, project, started_at, updated_at, metadata, subproject_id)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6)",
            params![id, agent, project, now, metadata, self.subproject_id],
        )?;

        Ok(Session {
            id,
            agent: agent.to_string(),
            project: project.map(String::from),
            started_at: now.clone(),
            updated_at: now,
            metadata: metadata.to_string(),
            subproject_id: Some(self.subproject_id.clone()),
        })
    }

    pub fn record_message(
        &mut self,
        session_id: &str,
        role: &str,
        content: &str,
        tool_name: Option<&str>,
        tokens: Option<i64>,
        metadata: &str,
    ) -> IcmResult<Message> {
        let now = Utc::now().to_rfc3339();
        let id = format!("msg_{:x}", rand_u64());

        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, tool_name, tokens, ts, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, session_id, role, content, tool_name, tokens, now, metadata],
        )?;

        let _ = self.conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now, session_id],
        );

        Ok(Message {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            tool_name: tool_name.map(String::from),
            tokens,
            ts: now,
            metadata: metadata.to_string(),
        })
    }

    pub fn search_transcripts(&self, query: &str, limit: usize) -> IcmResult<Vec<Message>> {
        let sanitized = sanitize_fts5_query_any(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.session_id, m.role, m.content, m.tool_name, m.tokens, m.ts, m.metadata
             FROM messages_fts s
             JOIN messages m ON s.rowid = m.rowid
             WHERE messages_fts MATCH ?1
             ORDER BY bm25(messages_fts) ASC, m.ts DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![sanitized, limit as i64], |r| {
            Ok(Message {
                id: r.get(0)?,
                session_id: r.get(1)?,
                role: r.get(2)?,
                content: r.get(3)?,
                tool_name: r.get(4)?,
                tokens: r.get(5)?,
                ts: r.get(6)?,
                metadata: r.get(7)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    pub fn get_session_messages(&self, session_id: &str) -> IcmResult<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, tool_name, tokens, ts, metadata
             FROM messages WHERE session_id = ?1 ORDER BY ts ASC",
        )?;
        let rows = stmt.query_map([session_id], |r| {
            Ok(Message {
                id: r.get(0)?,
                session_id: r.get(1)?,
                role: r.get(2)?,
                content: r.get(3)?,
                tool_name: r.get(4)?,
                tokens: r.get(5)?,
                ts: r.get(6)?,
                metadata: r.get(7)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    pub fn transcript_stats(&self) -> IcmResult<Value> {
        let sessions: usize = self.conn.query_row("SELECT count(*) FROM sessions", [], |r| r.get(0))?;
        let messages: usize = self.conn.query_row("SELECT count(*) FROM messages", [], |r| r.get(0))?;
        Ok(json!({
            "total_sessions": sessions,
            "total_messages": messages,
        }))
    }

    // --- Wake-up & Learn Operations ---

    pub fn build_wake_up_brief(&self, _project: Option<&str>) -> IcmResult<String> {
        let mut out = String::new();
        out.push_str("### ICM Standing Context & Wake-Up Memory\n\n");

        // 1. Critical and High importance memories
        let mut mem_stmt = self.conn.prepare(
            "SELECT id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash, subproject_id
             FROM memories
             WHERE importance IN ('critical', 'high')
             ORDER BY importance ASC, weight DESC, last_accessed DESC
             LIMIT 10",
        )?;
        let memories: Vec<Memory> = mem_stmt.query_map([], Self::row_to_memory)?.flatten().collect();
        if !memories.is_empty() {
            out.push_str("**Core Architectural Memories:**\n");
            for m in &memories {
                out.push_str(&format!("- [{}] {}: {}\n", m.importance.as_str().to_uppercase(), m.topic, m.summary));
            }
            out.push('\n');
        }

        // 2. Standing active facts
        let facts = self.list_facts(None)?;
        if !facts.is_empty() {
            out.push_str("**Active System Facts:**\n");
            for f in facts.iter().take(10) {
                out.push_str(&format!("- `{}::{}` = {}\n", f.entity, f.key, f.value));
            }
            out.push('\n');
        }

        // 3. Recent feedback
        let feedback = self.search_feedback("", None, 3)?;
        if !feedback.is_empty() {
            out.push_str("**Recent User Preferences & Feedback:**\n");
            for fb in &feedback {
                out.push_str(&format!("- [{}] Expected: {} (Corrected: {})\n", fb.topic, fb.predicted, fb.corrected));
            }
            out.push('\n');
        }

        Ok(out)
    }

    pub fn learn_from_text(&mut self, topic: &str, content: &str, source: &str) -> IcmResult<Memory> {
        let now = Utc::now().to_rfc3339();
        let id = format!("mem_learn_{:x}", rand_u64());

        // Simple zero-LLM keyword extraction from uppercase words or identifiers
        let keywords: Vec<String> = content
            .split_whitespace()
            .filter(|w| w.len() >= 4 && (w.contains('_') || w.contains('-') || w.chars().next().map_or(false, |c| c.is_uppercase())))
            .take(6)
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-').to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mem = Memory {
            id: id.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now,
            access_count: 0,
            weight: 1.0,
            topic: topic.to_string(),
            summary: content.to_string(),
            raw_excerpt: Some(content.to_string()),
            keywords,
            importance: Importance::High,
            source_type: source.to_string(),
            source_data: None,
            related_ids: Vec::new(),
            summary_hash: Some(compute_summary_hash(topic, content)),
            subproject_id: Some(self.subproject_id.clone()),
        };

        self.insert_or_update_memory(&mem)?;
        Ok(mem)
    }

    pub fn purge_ignored_memories(&mut self, root: &Path) -> IcmResult<usize> {
        let ignore = crate::codegraph::walker::build_project_ignore(root);
        let items: Vec<(String, String, Option<String>)> = {
            let mut stmt = self.conn.prepare("SELECT id, topic, source_data FROM memories")?;
            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let topic: String = row.get(1)?;
                let source_data: Option<String> = row.get(2)?;
                Ok((id, topic, source_data))
            })?;
            rows.flatten().collect()
        };

        let mut purged = 0;
        for (id, topic, source_data) in items {
            let mut matches_ignore = false;
            if ignore.matched(topic.as_str(), false).is_ignore() {
                matches_ignore = true;
            } else if let Some(ref data) = source_data {
                if let Ok(val) = serde_json::from_str::<Value>(data) {
                    if let Some(file_path) = val.get("file").or_else(|| val.get("path")).and_then(|v| v.as_str()) {
                        if ignore.matched(file_path, false).is_ignore() {
                            matches_ignore = true;
                        }
                    }
                }
            }

            if matches_ignore {
                if let Ok(true) = self.delete_memory(&id) {
                    purged += 1;
                }
            }
        }

        Ok(purged)
    }

    pub fn stats(&self) -> IcmResult<IcmStats> {
        let total_memories: usize = self.conn.query_row("SELECT count(*) FROM memories", [], |r| r.get(0))?;
        let total_topics: usize = self.conn.query_row("SELECT count(DISTINCT topic) FROM memories", [], |r| r.get(0))?;
        let total_facts: usize = self.conn.query_row("SELECT count(*) FROM facts WHERE superseded_at IS NULL", [], |r| r.get(0))?;
        let total_memoirs: usize = self.conn.query_row("SELECT count(*) FROM memoirs", [], |r| r.get(0))?;
        let total_concepts: usize = self.conn.query_row("SELECT count(*) FROM concepts", [], |r| r.get(0))?;
        let total_feedback: usize = self.conn.query_row("SELECT count(*) FROM feedback", [], |r| r.get(0))?;
        let total_transcripts: usize = self.conn.query_row("SELECT count(*) FROM messages", [], |r| r.get(0))?;
        let avg_weight: f64 = self.conn.query_row("SELECT coalesce(avg(weight), 1.0) FROM memories", [], |r| r.get(0))?;

        Ok(IcmStats {
            total_memories,
            total_topics,
            total_facts,
            total_memoirs,
            total_concepts,
            total_feedback,
            total_transcripts,
            avg_weight: avg_weight as f32,
        })
    }

    pub fn health(&self, topic_filter: Option<&str>) -> IcmResult<Vec<TopicHealth>> {
        let sql = match topic_filter {
            Some(_) => "SELECT topic, count(*), coalesce(avg(weight), 1.0),
                              sum(CASE WHEN importance = 'critical' THEN 1 ELSE 0 END),
                              sum(CASE WHEN weight < 0.2 THEN 1 ELSE 0 END)
                       FROM memories WHERE topic = ?1 GROUP BY topic",
            None => "SELECT topic, count(*), coalesce(avg(weight), 1.0),
                            sum(CASE WHEN importance = 'critical' THEN 1 ELSE 0 END),
                            sum(CASE WHEN weight < 0.2 THEN 1 ELSE 0 END)
                     FROM memories GROUP BY topic ORDER BY count(*) DESC",
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = if let Some(t) = topic_filter {
            stmt.query_map([t], Self::row_to_topic_health)?
        } else {
            stmt.query_map([], Self::row_to_topic_health)?
        };

        Ok(rows.flatten().collect())
    }

    pub fn sync_from_projectmem(&mut self, events: &[crate::projectmem::schema::Event]) -> IcmResult<usize> {
        let now = Utc::now().to_rfc3339();
        let mut count = 0;

        for ev in events {
            let topic = match ev.event_type.as_str() {
                "decision" => "decision".to_string(),
                "fix" => "errors-resolved".to_string(),
                "issue" => "open-issues".to_string(),
                "note" => {
                    if ev.summary.starts_with("gotcha:") || ev.summary.starts_with("Gotcha:") {
                        "gotcha".to_string()
                    } else {
                        "project-notes".to_string()
                    }
                }
                other => format!("projectmem-{}", other),
            };

            let importance = match ev.event_type.as_str() {
                "decision" => Importance::Critical,
                "fix" => Importance::High,
                "note" if ev.summary.starts_with("gotcha:") => Importance::Critical,
                _ => Importance::Medium,
            };

            let mem = Memory {
                id: format!("pjm_{}", ev.id),
                created_at: if ev.timestamp.is_empty() { now.clone() } else { ev.timestamp.clone() },
                updated_at: now.clone(),
                last_accessed: now.clone(),
                access_count: 0,
                weight: if importance == Importance::Critical { 1.0 } else { 0.8 },
                topic: topic.clone(),
                summary: ev.summary.clone(),
                raw_excerpt: None,
                keywords: vec![ev.event_type.clone(), topic.clone()],
                importance,
                source_type: "projectmem".into(),
                source_data: ev.location.as_ref().map(|loc| json!({ "location": loc }).to_string()),
                related_ids: Vec::new(),
                summary_hash: Some(compute_summary_hash(&topic, &ev.summary)),
                subproject_id: Some(self.subproject_id.clone()),
            };

            self.insert_or_update_memory(&mem)?;
            count += 1;

            if ev.event_type == "decision" {
                let _ = self.set_fact("architecture", &format!("decision_{}", ev.id), &ev.summary, "projectmem");
            } else if ev.summary.starts_with("gotcha:") {
                let _ = self.set_fact("gotcha", &format!("gotcha_{}", ev.id), &ev.summary, "projectmem");
            }
        }

        Ok(count)
    }

    fn row_to_memory(row: &rusqlite::Row<'_>) -> Result<Memory, rusqlite::Error> {
        let keywords_json: Option<String> = row.get(9)?;
        let keywords: Vec<String> = keywords_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let importance_str: String = row.get(10)?;
        let importance = importance_str.parse().unwrap_or(Importance::Medium);

        let related_json: Option<String> = row.get(13)?;
        let related_ids: Vec<String> = related_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Ok(Memory {
            id: row.get(0)?,
            created_at: row.get(1)?,
            updated_at: row.get(2)?,
            last_accessed: row.get(3)?,
            access_count: row.get(4)?,
            weight: row.get::<_, f64>(5)? as f32,
            topic: row.get(6)?,
            summary: row.get(7)?,
            raw_excerpt: row.get(8)?,
            keywords,
            importance,
            source_type: row.get(11)?,
            source_data: row.get(12)?,
            related_ids,
            summary_hash: row.get(14)?,
            subproject_id: row.get(15)?,
        })
    }

    fn row_to_fact(row: &rusqlite::Row<'_>) -> Result<Fact, rusqlite::Error> {
        Ok(Fact {
            id: row.get(0)?,
            entity: row.get(1)?,
            key: row.get(2)?,
            value: row.get(3)?,
            source: row.get(4)?,
            created_at: row.get(5)?,
            superseded_at: row.get(6)?,
            subproject_id: row.get(7)?,
        })
    }

    fn row_to_concept(row: &rusqlite::Row<'_>) -> Result<Concept, rusqlite::Error> {
        let labels_json: String = row.get(4)?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let sources_json: String = row.get(9)?;
        let sources: Vec<String> = serde_json::from_str(&sources_json).unwrap_or_default();
        Ok(Concept {
            id: row.get(0)?,
            memoir_id: row.get(1)?,
            name: row.get(2)?,
            definition: row.get(3)?,
            labels,
            confidence: row.get::<_, f64>(5)? as f32,
            revision: row.get::<_, i64>(6)? as u32,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            source_memory_ids: sources,
        })
    }

    fn row_to_feedback(row: &rusqlite::Row<'_>) -> Result<Feedback, rusqlite::Error> {
        Ok(Feedback {
            id: row.get(0)?,
            topic: row.get(1)?,
            context: row.get(2)?,
            predicted: row.get(3)?,
            corrected: row.get(4)?,
            reason: row.get(5)?,
            source: row.get(6)?,
            created_at: row.get(7)?,
            applied_count: row.get(8)?,
            subproject_id: row.get(9)?,
        })
    }

    fn row_to_topic_health(row: &rusqlite::Row<'_>) -> Result<TopicHealth, rusqlite::Error> {
        Ok(TopicHealth {
            topic: row.get(0)?,
            count: row.get(1)?,
            avg_weight: row.get::<_, f64>(2)? as f32,
            critical_count: row.get(3)?,
            stale_count: row.get(4)?,
        })
    }
}

pub fn sanitize_fts5_query(query: &str) -> String {
    quoted_fts_tokens(query).join(" ")
}

pub fn sanitize_fts5_query_any(query: &str) -> String {
    quoted_fts_tokens(query).join(" OR ")
}

fn quoted_fts_tokens(query: &str) -> Vec<String> {
    let query = if query.len() > 10_000 {
        let mut end = 10_000;
        while end > 0 && !query.is_char_boundary(end) {
            end -= 1;
        }
        &query[..end]
    } else {
        query
    };

    let cleaned: String = query
        .chars()
        .map(|c| {
            if matches!(
                c,
                '-' | '*' | '"' | '(' | ')' | '{' | '}' | ':' | '^' | '+' | '~' | '\\'
            ) {
                ' '
            } else {
                c
            }
        })
        .collect();

    cleaned
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .take(100)
        .map(|w| {
            let stripped = w.replace('"', "");
            format!("\"{stripped}\"")
        })
        .collect()
}

fn rand_u64() -> u64 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    d.as_nanos() as u64
}
