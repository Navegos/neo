// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/cache.rs
// created: 2026-10-28
// lastModified: 2026-11-13

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rusqlite::{params, Connection, OpenFlags};
use sha2::{Digest, Sha256};

use super::kernel::{FileExtraction, SymbolEdge, SymbolNode, SymbolRef};
use super::walker::WalkedFile;
use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub file_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub ref_count: usize,
}

pub struct SymbolCache {
    conn: Connection,
    pub db_path: PathBuf,
}

impl SymbolCache {
    pub fn open(db_path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS files (
                 file_path TEXT PRIMARY KEY,
                 mtime INTEGER NOT NULL,
                 size INTEGER NOT NULL,
                 hash TEXT NOT NULL,
                 language TEXT NOT NULL,
                 indexed_at INTEGER NOT NULL,
                 subproject_id TEXT NOT NULL DEFAULT '.'
             );
             CREATE TABLE IF NOT EXISTS nodes (
                 id TEXT PRIMARY KEY,
                 file_path TEXT NOT NULL,
                 name TEXT NOT NULL,
                 qualified_name TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 visibility TEXT,
                 start_line INTEGER NOT NULL,
                 end_line INTEGER NOT NULL,
                 start_column INTEGER NOT NULL,
                 end_column INTEGER NOT NULL,
                 docstring TEXT,
                 signature TEXT,
                 return_type TEXT,
                 subproject_id TEXT NOT NULL DEFAULT '.',
                 start_byte INTEGER NOT NULL DEFAULT 0,
                 end_byte INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS edges (
                 source_id TEXT NOT NULL,
                 target_id TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 line INTEGER,
                 column INTEGER
             );
             CREATE TABLE IF NOT EXISTS refs (
                 from_node_id TEXT NOT NULL,
                 name TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 line INTEGER NOT NULL,
                 column INTEGER NOT NULL,
                 file_path TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(file_path);
             CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);
             CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
             CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
             CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
             CREATE INDEX IF NOT EXISTS idx_refs_from ON refs(from_node_id);
             CREATE INDEX IF NOT EXISTS idx_refs_name ON refs(name);
             CREATE INDEX IF NOT EXISTS idx_refs_file ON refs(file_path);",
        )?;

        ensure_column(&conn, "files", "subproject_id", "TEXT NOT NULL DEFAULT '.'")?;
        ensure_column(&conn, "nodes", "subproject_id", "TEXT NOT NULL DEFAULT '.'")?;
        ensure_column(&conn, "nodes", "start_byte", "INTEGER NOT NULL DEFAULT 0")?;
        ensure_column(&conn, "nodes", "end_byte", "INTEGER NOT NULL DEFAULT 0")?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_files_subproject ON files(subproject_id);
             CREATE INDEX IF NOT EXISTS idx_nodes_subproject ON nodes(subproject_id);",
        )?;

        Ok(Self {
            conn,
            db_path: db_path.to_path_buf(),
        })
    }

    pub fn get_cached_files(&self) -> HashMap<String, (u64, u64, String)> {
        let mut map = HashMap::new();
        let mut stmt = match self.conn.prepare("SELECT file_path, mtime, size, hash FROM files") {
            Ok(s) => s,
            Err(_) => return map,
        };
        let rows = stmt.query_map([], |row| {
            let p: String = row.get(0)?;
            let m: i64 = row.get(1)?;
            let s: i64 = row.get(2)?;
            let h: String = row.get(3)?;
            Ok((p, (m as u64, s as u64, h)))
        });
        if let Ok(iter) = rows {
            for item in iter.flatten() {
                map.insert(item.0, item.1);
            }
        }
        map
    }

    pub fn remove_file(&mut self, file_path: &str) -> Result<(), rusqlite::Error> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM files WHERE file_path = ?1", params![file_path])?;
        tx.execute("DELETE FROM nodes WHERE file_path = ?1", params![file_path])?;
        tx.execute("DELETE FROM refs WHERE file_path = ?1", params![file_path])?;
        tx.execute(
            "DELETE FROM edges WHERE source_id NOT IN (SELECT id FROM nodes) OR target_id NOT IN (SELECT id FROM nodes)",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn save_extraction(
        &mut self,
        file: &WalkedFile,
        hash: &str,
        extraction: &FileExtraction,
    ) -> Result<(), rusqlite::Error> {
        self.save_extraction_with_source(file, hash, extraction, None)
    }

    pub fn save_extraction_with_source(
        &mut self,
        file: &WalkedFile,
        hash: &str,
        extraction: &FileExtraction,
        source: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let rel_norm = normalize_path(&file.rel_path);
        let subproject_id = derive_subproject_id(&rel_norm);
        let line_offsets = source.map(build_line_offsets);

        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM files WHERE file_path = ?1", params![rel_norm])?;
        tx.execute("DELETE FROM nodes WHERE file_path = ?1", params![rel_norm])?;
        tx.execute("DELETE FROM refs WHERE file_path = ?1", params![rel_norm])?;

        tx.execute(
            "INSERT INTO files (file_path, mtime, size, hash, language, indexed_at, subproject_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                rel_norm,
                file.mtime as i64,
                file.size as i64,
                hash,
                file.language,
                now,
                subproject_id
            ],
        )?;

        {
            let mut insert_node = tx.prepare_cached(
                "INSERT OR REPLACE INTO nodes (id, file_path, name, qualified_name, kind, visibility,
                                              start_line, end_line, start_column, end_column,
                                              docstring, signature, return_type, subproject_id,
                                              start_byte, end_byte)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )?;

            for n in &extraction.nodes {
                let node_path = normalize_path(&n.file_path);
                let node_sub = derive_subproject_id(&node_path);
                let (sb, eb) = match (&line_offsets, source) {
                    (Some(offsets), Some(src)) => {
                        line_span_to_bytes(src, offsets, n.start_line, n.start_column, n.end_line, n.end_column)
                    }
                    _ => (0, 0),
                };
                insert_node.execute(params![
                    n.id,
                    node_path,
                    n.name,
                    n.qualified_name,
                    n.kind,
                    n.visibility,
                    n.start_line,
                    n.end_line,
                    n.start_column,
                    n.end_column,
                    n.docstring,
                    n.signature,
                    n.return_type,
                    node_sub,
                    sb as i64,
                    eb as i64
                ])?;
            }
        }

        {
            let mut insert_edge = tx.prepare_cached(
                "INSERT INTO edges (source_id, target_id, kind, line, column)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for e in &extraction.edges {
                insert_edge.execute(params![e.source_id, e.target_id, e.kind, e.line, e.column])?;
            }
        }

        {
            let mut insert_ref = tx.prepare_cached(
                "INSERT INTO refs (from_node_id, name, kind, line, column, file_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;

            for r in &extraction.refs {
                insert_ref.execute(params![
                    r.from_node_id,
                    r.name,
                    r.kind,
                    r.line,
                    r.column,
                    r.file_path
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn save_resolved_edges(&mut self, edges: &[SymbolEdge]) -> Result<(), rusqlite::Error> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM edges WHERE kind = 'calls'", [])?;
        {
            let mut insert_edge = tx.prepare_cached(
                "INSERT INTO edges (source_id, target_id, kind, line, column)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for e in edges {
                insert_edge.execute(params![e.source_id, e.target_id, e.kind, e.line, e.column])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_stats(&self) -> CacheStats {
        let file_count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))
            .unwrap_or(0);
        let node_count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap_or(0);
        let edge_count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))
            .unwrap_or(0);
        let ref_count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM refs", [], |r| r.get(0))
            .unwrap_or(0);

        CacheStats {
            file_count,
            node_count,
            edge_count,
            ref_count,
        }
    }

    pub fn get_all_nodes(&self) -> Vec<SymbolNode> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, file_path, name, qualified_name, kind, visibility,
                    start_line, end_line, start_column, end_column, docstring, signature, return_type
             FROM nodes",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |r| {
            Ok(SymbolNode {
                id: r.get(0)?,
                file_path: r.get(1)?,
                name: r.get(2)?,
                qualified_name: r.get(3)?,
                kind: r.get(4)?,
                visibility: r.get(5)?,
                start_line: r.get(6)?,
                end_line: r.get(7)?,
                start_column: r.get(8)?,
                end_column: r.get(9)?,
                docstring: r.get(10)?,
                signature: r.get(11)?,
                return_type: r.get(12)?,
            })
        });

        iter.into_iter().flat_map(|res| res.flatten()).collect()
    }

    pub fn get_all_edges(&self) -> Vec<SymbolEdge> {
        let mut stmt = match self.conn.prepare("SELECT source_id, target_id, kind, line, column FROM edges") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |r| {
            Ok(SymbolEdge {
                source_id: r.get(0)?,
                target_id: r.get(1)?,
                kind: r.get(2)?,
                line: r.get(3)?,
                column: r.get(4)?,
            })
        });

        iter.into_iter().flat_map(|res| res.flatten()).collect()
    }

    pub fn get_all_refs(&self) -> Vec<SymbolRef> {
        let mut stmt = match self.conn.prepare("SELECT from_node_id, name, kind, line, column, file_path FROM refs") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |r| {
            Ok(SymbolRef {
                from_node_id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                line: r.get(3)?,
                column: r.get(4)?,
                file_path: r.get(5)?,
            })
        });

        iter.into_iter().flat_map(|res| res.flatten()).collect()
    }

    pub fn get_indexed_files(&self) -> Vec<(String, String, usize)> {
        let mut stmt = match self.conn.prepare(
            "SELECT f.file_path, f.language, COUNT(n.id)
             FROM files f
             LEFT JOIN nodes n ON f.file_path = n.file_path
             GROUP BY f.file_path, f.language
             ORDER BY f.file_path",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([], |r| {
            let p: String = r.get(0)?;
            let l: String = r.get(1)?;
            let count: i64 = r.get(2)?;
            Ok((p, l, count as usize))
        });

        iter.into_iter().flat_map(|res| res.flatten()).collect()
    }

    /// Dynamic purge: delete indexed files that match ignore rules
    pub fn purge_ignored_files(&mut self, root: &Path) -> Result<usize, rusqlite::Error> {
        let ignore = super::walker::build_project_ignore(root);
        let files = self.get_cached_files();
        let mut purged = 0;
        for (rel_path, _) in files {
            let normalized = crate::util::paths::normalize_path(&rel_path);
            let full_path = root.join(&normalized);
            let is_dir = full_path.is_dir();
            let is_ignored = ignore.matched_path_or_any_parents(&full_path, is_dir).is_ignore()
                || super::walker::is_build_or_vendor_artifact(&normalized)
                || normalized.split('/').any(super::walker::is_build_or_vendor_artifact);

            if is_ignored {
                self.remove_file(&rel_path)?;
                purged += 1;
            }
        }
        Ok(purged)
    }

    /// Fetch O(1) SQLite byte-offset span `(file_path, start_byte, end_byte)` for a node.
    pub fn get_symbol_byte_range(&self, id_or_name: &str) -> Option<(SymbolNode, u64, u64)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, file_path, name, qualified_name, kind, visibility,
                        start_line, end_line, start_column, end_column, docstring, signature, return_type,
                        start_byte, end_byte
                 FROM nodes
                 WHERE id = ?1 OR qualified_name = ?1 OR name = ?1
                 ORDER BY CASE WHEN id = ?1 THEN 0 WHEN qualified_name = ?1 THEN 1 ELSE 2 END
                 LIMIT 1",
            )
            .ok()?;

        stmt.query_row(params![id_or_name], |r| {
            let node = SymbolNode {
                id: r.get(0)?,
                file_path: r.get(1)?,
                name: r.get(2)?,
                qualified_name: r.get(3)?,
                kind: r.get(4)?,
                visibility: r.get(5)?,
                start_line: r.get(6)?,
                end_line: r.get(7)?,
                start_column: r.get(8)?,
                end_column: r.get(9)?,
                docstring: r.get(10)?,
                signature: r.get(11)?,
                return_type: r.get(12)?,
            };
            let sb: i64 = r.get(13)?;
            let eb: i64 = r.get(14)?;
            Ok((node, sb.max(0) as u64, eb.max(0) as u64))
        })
        .ok()
    }

    /// O(1) byte-offset slice read using SQLite `start_byte..end_byte` directly via file Seek.
    pub fn get_symbol_slice_o1(&self, root: &Path, id_or_name: &str) -> Option<(SymbolNode, String)> {
        let (node, sb, eb) = self.get_symbol_byte_range(id_or_name)?;
        if eb <= sb {
            return None;
        }
        let abs = root.join(normalize_path(&node.file_path));
        let mut f = fs::File::open(abs).ok()?;
        let len = (eb - sb).min(512 * 1024) as usize;
        f.seek(SeekFrom::Start(sb)).ok()?;
        let mut buf = vec![0u8; len];
        let read_n = f.read(&mut buf).ok()?;
        buf.truncate(read_n);
        let text = String::from_utf8_lossy(&buf).into_owned();
        Some((node, text))
    }

    /// Query nodes partitioned under a specific monorepo `subproject_id`.
    pub fn get_nodes_by_subproject(&self, subproject_id: &str) -> Vec<SymbolNode> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, file_path, name, qualified_name, kind, visibility,
                    start_line, end_line, start_column, end_column, docstring, signature, return_type
             FROM nodes WHERE subproject_id = ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let iter = stmt.query_map(params![subproject_id], |r| {
            Ok(SymbolNode {
                id: r.get(0)?,
                file_path: r.get(1)?,
                name: r.get(2)?,
                qualified_name: r.get(3)?,
                kind: r.get(4)?,
                visibility: r.get(5)?,
                start_line: r.get(6)?,
                end_line: r.get(7)?,
                start_column: r.get(8)?,
                end_column: r.get(9)?,
                docstring: r.get(10)?,
                signature: r.get(11)?,
                return_type: r.get(12)?,
            })
        });
        iter.into_iter().flat_map(|res| res.flatten()).collect()
    }

    /// Row-merge all records from a nested/legacy CodeGraph SQLite database into this root DB.
    pub fn merge_from_db(&mut self, other_db: &Path, subproject_prefix: &str) -> Result<usize, rusqlite::Error> {
        let src = Connection::open_with_flags(other_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let prefix = normalize_path(subproject_prefix).trim_matches('/').to_string();
        let qualify_rel = |p: &str| -> String {
            let norm = normalize_path(p).trim_start_matches("./").to_string();
            if prefix.is_empty() || prefix == "." || norm.starts_with(&format!("{}/", prefix)) {
                norm
            } else {
                format!("{}/{}", prefix, norm)
            }
        };

        let tx = self.conn.transaction()?;
        let mut merged_files = 0usize;

        if let Ok(mut stmt) = src.prepare("SELECT file_path, mtime, size, hash, language, indexed_at FROM files") {
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            });
            if let Ok(iter) = rows {
                let mut ins = tx.prepare_cached(
                    "INSERT OR IGNORE INTO files (file_path, mtime, size, hash, language, indexed_at, subproject_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                )?;
                for (fp, mtime, size, hash, lang, idx_at) in iter.flatten() {
                    let qfp = qualify_rel(&fp);
                    let sub = derive_subproject_id(&qfp);
                    merged_files += ins.execute(params![qfp, mtime, size, hash, lang, idx_at, sub])?;
                }
            }
        }

        if let Ok(mut stmt) = src.prepare(
            "SELECT id, file_path, name, qualified_name, kind, visibility,
                    start_line, end_line, start_column, end_column, docstring, signature, return_type
             FROM nodes",
        ) {
            let rows = stmt.query_map([], |r| {
                Ok(SymbolNode {
                    id: r.get(0)?,
                    file_path: r.get(1)?,
                    name: r.get(2)?,
                    qualified_name: r.get(3)?,
                    kind: r.get(4)?,
                    visibility: r.get(5)?,
                    start_line: r.get(6)?,
                    end_line: r.get(7)?,
                    start_column: r.get(8)?,
                    end_column: r.get(9)?,
                    docstring: r.get(10)?,
                    signature: r.get(11)?,
                    return_type: r.get(12)?,
                })
            });
            if let Ok(iter) = rows {
                let mut ins = tx.prepare_cached(
                    "INSERT OR IGNORE INTO nodes (id, file_path, name, qualified_name, kind, visibility,
                                                  start_line, end_line, start_column, end_column,
                                                  docstring, signature, return_type, subproject_id, start_byte, end_byte)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0, 0)",
                )?;
                for n in iter.flatten() {
                    let qfp = qualify_rel(&n.file_path);
                    let sub = derive_subproject_id(&qfp);
                    ins.execute(params![
                        n.id, qfp, n.name, n.qualified_name, n.kind, n.visibility,
                        n.start_line, n.end_line, n.start_column, n.end_column,
                        n.docstring, n.signature, n.return_type, sub
                    ])?;
                }
            }
        }

        if let Ok(mut stmt) = src.prepare("SELECT source_id, target_id, kind, line, column FROM edges") {
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<u32>>(3)?,
                    r.get::<_, Option<u32>>(4)?,
                ))
            });
            if let Ok(iter) = rows {
                let mut ins = tx.prepare_cached(
                    "INSERT OR IGNORE INTO edges (source_id, target_id, kind, line, column) VALUES (?1, ?2, ?3, ?4, ?5)",
                )?;
                for (s, t, k, l, c) in iter.flatten() {
                    ins.execute(params![s, t, k, l, c])?;
                }
            }
        }

        if let Ok(mut stmt) = src.prepare("SELECT from_node_id, name, kind, line, column, file_path FROM refs") {
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, u32>(3)?,
                    r.get::<_, u32>(4)?,
                    r.get::<_, String>(5)?,
                ))
            });
            if let Ok(iter) = rows {
                let mut ins = tx.prepare_cached(
                    "INSERT OR IGNORE INTO refs (from_node_id, name, kind, line, column, file_path)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;
                for (from, name, kind, line, col, fp) in iter.flatten() {
                    let qfp = qualify_rel(&fp);
                    ins.execute(params![from, name, kind, line, col, qfp])?;
                }
            }
        }

        tx.commit()?;
        Ok(merged_files)
    }
}

fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))?
        .filter_map(Result::ok)
        .collect();
    if !cols.iter().any(|c| c == column) {
        conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, ddl), [])?;
    }
    Ok(())
}

/// Partition monorepo subprojects (e.g. `crates/toksi/src/...` -> `crates/toksi`).
pub fn derive_subproject_id(rel_path: &str) -> String {
    let norm = normalize_path(rel_path);
    let trimmed = norm.trim_start_matches("./").trim_start_matches('/');
    let mut parts = trimmed.split('/');
    if let (Some(first), Some(second)) = (parts.next(), parts.next()) {
        if matches!(
            first,
            "crates" | "packages" | "apps" | "services" | "libs" | "modules" | "subprojects"
        ) && !second.is_empty()
        {
            return format!("{}/{}", first, second);
        }
    }
    ".".to_string()
}

fn build_line_offsets(src: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (idx, b) in src.bytes().enumerate() {
        if b == b'\n' {
            offsets.push(idx + 1);
        }
    }
    offsets
}

fn line_span_to_bytes(
    src: &str,
    offsets: &[usize],
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
) -> (usize, usize) {
    let s_idx = start_line.saturating_sub(1) as usize;
    let e_idx = end_line.saturating_sub(1) as usize;
    let start_base = offsets.get(s_idx).copied().unwrap_or(0).min(src.len());
    let start_byte = (start_base + start_col as usize).min(src.len());

    let end_base = offsets.get(e_idx).copied().unwrap_or(src.len()).min(src.len());
    let mut end_byte = if end_col == 0 && e_idx + 1 < offsets.len() {
        offsets[e_idx + 1].saturating_sub(1)
    } else if e_idx + 1 < offsets.len() {
        (end_base + end_col as usize).min(offsets[e_idx + 1])
    } else {
        (end_base + end_col as usize).min(src.len())
    };
    if end_byte <= start_byte && s_idx < offsets.len() {
        end_byte = if e_idx + 1 < offsets.len() {
            offsets[e_idx + 1].min(src.len())
        } else {
            src.len()
        };
    }
    (start_byte, end_byte.max(start_byte))
}

pub fn content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purge_ignored_files() {
        let temp_dir = std::env::temp_dir().join("toksi_cg_cache_purge_test");
        let _ = fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("symbols.db");
        let mut cache = SymbolCache::open(&db_path).unwrap();

        // Seed file in cache
        let file = WalkedFile {
            path: temp_dir.join("temp_build.tmp"),
            rel_path: "temp_build.tmp".to_string(),
            language: "rust",
            mtime: 100,
            size: 50,
        };
        let extraction = FileExtraction {
            nodes: Vec::new(),
            edges: Vec::new(),
            refs: Vec::new(),
        };
        cache.save_extraction(&file, "hash123", &extraction).unwrap();
        assert_eq!(cache.get_stats().file_count, 1);

        // Add gitignore rule
        let gitignore = temp_dir.join(".gitignore");
        fs::write(&gitignore, "*.tmp\n").unwrap();

        // Purge
        let purged = cache.purge_ignored_files(&temp_dir).unwrap();
        assert_eq!(purged, 1);
        assert_eq!(cache.get_stats().file_count, 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
