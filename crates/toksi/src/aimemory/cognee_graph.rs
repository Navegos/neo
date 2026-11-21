// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/cognee_graph.rs
// created: 2026-11-21

//! Pure-Rust Cognee-style ECL (Extract-Cognify-Load) bi-temporal knowledge graph.

#![deny(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub subproject_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationEdge {
    pub source_id: String,
    pub relation: String,
    pub target_id: String,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub tx_from: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CogneeGraph {
    pub nodes: HashMap<String, EntityNode>,
    pub edges: Vec<RelationEdge>,
}

impl CogneeGraph {
    pub fn load_or_new(ai_memory_dir: &Path) -> Result<(Self, PathBuf), String> {
        let dir = ai_memory_dir.join("cognee");
        fs::create_dir_all(&dir).map_err(|e| format!("cognee mkdir failed: {e}"))?;
        let path = dir.join("graph.json");
        if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|e| format!("cognee read failed: {e}"))?;
            let graph: Self =
                serde_json::from_str(&raw).map_err(|e| format!("cognee parse failed: {e}"))?;
            Ok((graph, path))
        } else {
            Ok((Self::default(), path))
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("cognee serialize failed: {e}"))?;
        fs::write(path, json).map_err(|e| format!("cognee write failed: {e}"))
    }

    /// Extract-Cognify-Load: ingests a `(subject, predicate, object)` triplet with bi-temporal stamps.
    pub fn cognify_triplet(
        &mut self,
        subject: &str,
        predicate: &str,
        object: &str,
        subproject_id: &str,
        valid_from: i64,
    ) {
        let sid = subject.to_ascii_lowercase().replace(' ', "_");
        let oid = object.to_ascii_lowercase().replace(' ', "_");
        self.nodes.entry(sid.clone()).or_insert_with(|| EntityNode {
            id: sid.clone(),
            label: subject.to_string(),
            kind: "entity".to_string(),
            subproject_id: subproject_id.to_string(),
        });
        self.nodes.entry(oid.clone()).or_insert_with(|| EntityNode {
            id: oid.clone(),
            label: object.to_string(),
            kind: "entity".to_string(),
            subproject_id: subproject_id.to_string(),
        });
        self.edges.push(RelationEdge {
            source_id: sid,
            relation: predicate.to_string(),
            target_id: oid,
            valid_from,
            valid_to: None,
            tx_from: chrono::Utc::now().timestamp(),
        });
    }

    pub fn neighbors(&self, entity_id: &str, at_ts: Option<i64>) -> Vec<&RelationEdge> {
        let key = entity_id.to_ascii_lowercase().replace(' ', "_");
        self.edges
            .iter()
            .filter(|e| {
                (e.source_id == key || e.target_id == key)
                    && at_ts.is_none_or(|ts| {
                        e.valid_from <= ts && e.valid_to.is_none_or(|end| ts < end)
                    })
            })
            .collect()
    }

    pub fn search_entities(&self, query: &str) -> Vec<&EntityNode> {
        let q = query.to_ascii_lowercase();
        let mut seen = HashSet::new();
        self.nodes
            .values()
            .filter(|n| {
                (n.label.to_ascii_lowercase().contains(&q) || n.id.contains(&q))
                    && seen.insert(n.id.clone())
            })
            .collect()
    }
}
