// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use super::kernel::{FileExtraction, SymbolEdge, SymbolNode, SymbolRef};
use super::walker::WalkedFile;

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
                 indexed_at INTEGER NOT NULL
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
                 return_type TEXT
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
        tx.commit()?;
        Ok(())
    }

    pub fn save_extraction(
        &mut self,
        file: &WalkedFile,
        hash: &str,
        extraction: &FileExtraction,
    ) -> Result<(), rusqlite::Error> {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM files WHERE file_path = ?1", params![file.rel_path])?;
        tx.execute("DELETE FROM nodes WHERE file_path = ?1", params![file.rel_path])?;
        tx.execute("DELETE FROM refs WHERE file_path = ?1", params![file.rel_path])?;

        tx.execute(
            "INSERT INTO files (file_path, mtime, size, hash, language, indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                file.rel_path,
                file.mtime as i64,
                file.size as i64,
                hash,
                file.language,
                now
            ],
        )?;

        {
            let mut insert_node = tx.prepare_cached(
                "INSERT OR REPLACE INTO nodes (id, file_path, name, qualified_name, kind, visibility,
                                              start_line, end_line, start_column, end_column,
                                              docstring, signature, return_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )?;

            for n in &extraction.nodes {
                insert_node.execute(params![
                    n.id,
                    n.file_path,
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
                    n.return_type
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
