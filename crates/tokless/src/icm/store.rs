// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/icm/store.rs

use std::path::{Path, PathBuf};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::icm::schema::{Fact, Importance, Memory, SearchHit, Topic};

pub const DEFAULT_DB_FILENAME: &str = "memory.db";

pub fn compute_summary_hash(topic: &str, summary: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    topic.trim().to_lowercase().hash(&mut hasher);
    summary.trim().to_lowercase().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub struct IcmStore {
    conn: Connection,
    path: PathBuf,
}

impl IcmStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        let mut store = Self { conn, path };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_default() -> Result<Self, rusqlite::Error> {
        let path = crate::icm::mcp::resolve_icm_db_path();
        Self::open(path)
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self {
            conn,
            path: PathBuf::from(":memory:"),
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn init_schema(&mut self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

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
                summary_hash TEXT UNIQUE
            );

            CREATE INDEX IF NOT EXISTS idx_memories_topic ON memories(topic);
            CREATE INDEX IF NOT EXISTS idx_memories_weight ON memories(weight);
            CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at);
            CREATE INDEX IF NOT EXISTS idx_memories_last_accessed ON memories(last_accessed);
            CREATE INDEX IF NOT EXISTS idx_memories_topic_hash
                ON memories(summary_hash) WHERE summary_hash IS NOT NULL;

            -- Structured facts
            CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                entity TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                superseded_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_facts_entity ON facts(entity);
            CREATE INDEX IF NOT EXISTS idx_facts_entity_key ON facts(entity, key);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_facts_active_unique
                ON facts(entity, key) WHERE superseded_at IS NULL;

            -- Sessions and transcripts
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent TEXT NOT NULL DEFAULT '',
                project TEXT,
                started_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}'
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

            -- FTS5 virtual tables
            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                id UNINDEXED,
                topic,
                summary,
                keywords,
                content='memories',
                content_rowid='rowid'
            );

            -- Triggers to sync memories with FTS5
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
            ",
        )?;
        Ok(())
    }

    pub fn insert_or_update_memory(&mut self, mem: &Memory) -> Result<(), rusqlite::Error> {
        let keywords_json = serde_json::to_string(&mem.keywords).unwrap_or_else(|_| "[]".into());
        let related_json = serde_json::to_string(&mem.related_ids).unwrap_or_else(|_| "[]".into());
        let hash = mem.summary_hash.clone().unwrap_or_else(|| compute_summary_hash(&mem.topic, &mem.summary));

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO memories (
                    id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                ON CONFLICT(summary_hash) DO UPDATE SET
                    updated_at = excluded.updated_at,
                    last_accessed = excluded.last_accessed,
                    access_count = memories.access_count + 1,
                    weight = MAX(memories.weight, excluded.weight),
                    summary = excluded.summary,
                    raw_excerpt = coalesce(excluded.raw_excerpt, memories.raw_excerpt),
                    keywords = excluded.keywords,
                    importance = excluded.importance,
                    related_ids = excluded.related_ids;",
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
            ])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_memory(&self, id: &str) -> Result<Option<Memory>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash
             FROM memories WHERE id = ?1",
        )?;

        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_memory(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_memory(&mut self, id: &str) -> Result<bool, rusqlite::Error> {
        let count = self.conn.execute("DELETE FROM memories WHERE id = ?1", [id])?;
        Ok(count > 0)
    }

    pub fn touch_memory(&mut self, id: &str) -> Result<(), rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE memories SET last_accessed = ?1, access_count = access_count + 1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn list_topics(&self) -> Result<Vec<Topic>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT topic, COUNT(*), AVG(weight), MAX(last_accessed)
             FROM memories
             GROUP BY topic
             ORDER BY COUNT(*) DESC, topic ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let topic: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let avg_weight: f64 = row.get(2)?;
            let last_accessed: Option<String> = row.get(3)?;

            Ok(Topic {
                name: topic,
                count: count as usize,
                avg_weight: avg_weight as f32,
                last_accessed,
            })
        })?;

        let mut topics = Vec::new();
        for r in rows {
            topics.push(r?);
        }
        Ok(topics)
    }

    pub fn search_bm25(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>, rusqlite::Error> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.get_recent(limit);
        }

        // Clean query for FTS5 (escape quotes, tokens joined with OR)
        let sanitized = sanitize_fts5_query(trimmed);
        if sanitized.is_empty() {
            return self.get_recent(limit);
        }

        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.created_at, m.updated_at, m.last_accessed, m.access_count, m.weight,
                    m.topic, m.summary, m.raw_excerpt, m.keywords, m.importance, m.source_type,
                    m.source_data, m.related_ids, m.summary_hash,
                    bm25(memories_fts) as rank_score
             FROM memories_fts f
             JOIN memories m ON f.rowid = m.rowid
             WHERE memories_fts MATCH ?1
             ORDER BY rank_score ASC, m.weight DESC, m.last_accessed DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![sanitized, limit as i64], |row| {
            let memory = Self::row_to_memory(row)?;
            let bm25_score: f64 = row.get(15)?;
            // bm25 in sqlite returns negative score where lower is better match
            let score = (-bm25_score) as f32;
            Ok((memory, score))
        })?;

        let mut hits = Vec::new();
        for (idx, r) in rows.enumerate() {
            let (memory, score) = r?;
            hits.push(SearchHit {
                memory,
                score,
                rank: idx + 1,
            });
        }

        // If no FTS matches found, fall back to LIKE substring match
        if hits.is_empty() {
            return self.search_like(trimmed, limit);
        }

        Ok(hits)
    }

    pub fn search_like(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>, rusqlite::Error> {
        let like_pat = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash
             FROM memories
             WHERE topic LIKE ?1 OR summary LIKE ?1 OR keywords LIKE ?1
             ORDER BY weight DESC, last_accessed DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![like_pat, limit as i64], |row| {
            let memory = Self::row_to_memory(row)?;
            Ok(memory)
        })?;

        let mut hits = Vec::new();
        for (idx, r) in rows.enumerate() {
            let memory = r?;
            hits.push(SearchHit {
                memory,
                score: 0.5,
                rank: idx + 1,
            });
        }
        Ok(hits)
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<SearchHit>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, updated_at, last_accessed, access_count, weight,
                    topic, summary, raw_excerpt, keywords, importance, source_type,
                    source_data, related_ids, summary_hash
             FROM memories
             ORDER BY last_accessed DESC, weight DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit as i64], |row| {
            let memory = Self::row_to_memory(row)?;
            Ok(memory)
        })?;

        let mut hits = Vec::new();
        for (idx, r) in rows.enumerate() {
            let memory = r?;
            hits.push(SearchHit {
                memory,
                score: 1.0,
                rank: idx + 1,
            });
        }
        Ok(hits)
    }

    // Fact operations
    pub fn set_fact(&mut self, entity: &str, key: &str, value: &str, source: &str) -> Result<String, rusqlite::Error> {
        let now = Utc::now().to_rfc3339();
        let id = format!("{:x}", rand_u64());

        let tx = self.conn.transaction()?;
        {
            // Supersede existing active fact for this (entity, key)
            tx.execute(
                "UPDATE facts SET superseded_at = ?1 WHERE entity = ?2 AND key = ?3 AND superseded_at IS NULL",
                params![now, entity, key],
            )?;

            tx.execute(
                "INSERT INTO facts (id, entity, key, value, source, created_at, superseded_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)",
                params![id, entity, key, value, source, now],
            )?;
        }
        tx.commit()?;
        Ok(id)
    }

    pub fn get_fact(&self, entity: &str, key: &str) -> Result<Option<Fact>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity, key, value, source, created_at, superseded_at
             FROM facts
             WHERE entity = ?1 AND key = ?2 AND superseded_at IS NULL",
        )?;

        let mut rows = stmt.query(params![entity, key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_fact(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_facts(&self, entity: Option<&str>) -> Result<Vec<Fact>, rusqlite::Error> {
        let sql = match entity {
            Some(_) => "SELECT id, entity, key, value, source, created_at, superseded_at
                        FROM facts WHERE entity = ?1 AND superseded_at IS NULL ORDER BY key ASC",
            None => "SELECT id, entity, key, value, source, created_at, superseded_at
                     FROM facts WHERE superseded_at IS NULL ORDER BY entity ASC, key ASC",
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = if let Some(e) = entity {
            stmt.query_map([e], Self::row_to_fact)?
        } else {
            stmt.query_map([], Self::row_to_fact)?
        };

        let mut facts = Vec::new();
        for r in rows {
            facts.push(r?);
        }
        Ok(facts)
    }

    /// Syncs ProjectMem events into SQLite memory and fact tables with 0 LLM token spend.
    pub fn sync_from_projectmem(&mut self, events: &[crate::projectmem::schema::Event]) -> Result<usize, rusqlite::Error> {
        let mut count = 0;
        let now = Utc::now().to_rfc3339();

        for ev in events {
            let (topic, importance, weight) = match ev.event_type.as_str() {
                "decision" => ("decision", Importance::High, 1.8),
                "fix" => ("fix", Importance::High, 1.6),
                "note" => {
                    if ev.summary.to_lowercase().starts_with("gotcha") {
                        ("gotcha", Importance::Critical, 2.0)
                    } else {
                        ("note", Importance::Medium, 1.0)
                    }
                }
                "issue" => ("issue", Importance::Medium, 1.2),
                "attempt" => ("attempt", Importance::Low, 0.8),
                _ => ("general", Importance::Low, 0.5),
            };

            let hash = compute_summary_hash(topic, &ev.summary);
            let mem_id = format!("mem_pjm_{}", &hash[..8]);

            let keywords = vec![
                topic.to_string(),
                ev.event_type.clone(),
            ];

            let mem = Memory {
                id: mem_id,
                created_at: if ev.timestamp.is_empty() { now.clone() } else { ev.timestamp.clone() },
                updated_at: now.clone(),
                last_accessed: now.clone(),
                access_count: 0,
                weight,
                topic: topic.to_string(),
                summary: ev.summary.clone(),
                raw_excerpt: ev.location.clone(),
                keywords,
                importance,
                source_type: "projectmem".to_string(),
                source_data: None,
                related_ids: ev.issue_id.clone().into_iter().collect(),
                summary_hash: Some(hash),
            };

            if self.insert_or_update_memory(&mem).is_ok() {
                count += 1;
            }

            if ev.event_type == "decision" {
                let _ = self.set_fact("architecture", &format!("decision_{}", &ev.id), &ev.summary, "projectmem");
            }
        }

        Ok(count)
    }

    // Sessions and Transcripts
    pub fn create_session(&mut self, agent: &str, project: Option<&str>, metadata: &str) -> Result<String, rusqlite::Error> {
        let id = format!("sess_{:x}", rand_u64());
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sessions (id, agent, project, started_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, agent, project, now, now, metadata],
        )?;
        Ok(id)
    }

    pub fn log_message(&mut self, session_id: &str, role: &str, content: &str, tool_name: Option<&str>, tokens: Option<i64>) -> Result<String, rusqlite::Error> {
        let id = format!("msg_{:x}", rand_u64());
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, tool_name, tokens, ts, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '{}')",
            params![id, session_id, role, content, tool_name, tokens, now],
        )?;
        self.conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        Ok(id)
    }

    fn row_to_memory(row: &rusqlite::Row) -> Result<Memory, rusqlite::Error> {
        let id: String = row.get(0)?;
        let created_at: String = row.get(1)?;
        let updated_at: String = row.get(2)?;
        let last_accessed: String = row.get(3)?;
        let access_count: u32 = row.get(4)?;
        let weight: f32 = row.get(5)?;
        let topic: String = row.get(6)?;
        let summary: String = row.get(7)?;
        let raw_excerpt: Option<String> = row.get(8)?;
        let keywords_json: Option<String> = row.get(9)?;
        let importance_str: String = row.get(10)?;
        let source_type: String = row.get(11)?;
        let source_data: Option<String> = row.get(12)?;
        let related_json: Option<String> = row.get(13)?;
        let summary_hash: Option<String> = row.get(14)?;

        let keywords = keywords_json
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();
        let related_ids = related_json
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();
        let importance = importance_str.parse().unwrap_or(Importance::Medium);

        Ok(Memory {
            id,
            created_at,
            updated_at,
            last_accessed,
            access_count,
            weight,
            topic,
            summary,
            raw_excerpt,
            keywords,
            importance,
            source_type,
            source_data,
            related_ids,
            summary_hash,
        })
    }

    fn row_to_fact(row: &rusqlite::Row) -> Result<Fact, rusqlite::Error> {
        let id: String = row.get(0)?;
        let entity: String = row.get(1)?;
        let key: String = row.get(2)?;
        let value: String = row.get(3)?;
        let source: String = row.get(4)?;
        let created_at: String = row.get(5)?;
        let superseded_at: Option<String> = row.get(6)?;

        Ok(Fact {
            id,
            entity,
            key,
            value,
            source,
            created_at,
            superseded_at,
        })
    }
}

fn sanitize_fts5_query(query: &str) -> String {
    let mut tokens = Vec::new();
    for word in query.split_whitespace() {
        let clean: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if clean.len() >= 2 {
            tokens.push(format!("\"{}\"", clean));
        }
    }
    tokens.join(" OR ")
}

fn rand_u64() -> u64 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    d.as_nanos() as u64
}
