// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/graph_checkpoint.rs
// created: 2026-11-21

//! LangGraph-style cyclic super-step state graph & checkpoint persistence.

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::projectmem::store::atomic_write_file;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuperStepSnapshot {
    pub super_step: u64,
    pub node_id: String,
    pub next_nodes: Vec<String>,
    pub state_digest: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphCheckpoint {
    pub graph_id: String,
    pub current_super_step: u64,
    pub active_node: String,
    pub cycle_counts: BTreeMap<String, u32>,
    pub max_cycles_per_node: u32,
    pub snapshots: Vec<SuperStepSnapshot>,
}

impl GraphCheckpoint {
    pub fn new(graph_id: impl Into<String>, max_cycles_per_node: u32) -> Self {
        Self {
            graph_id: graph_id.into(),
            current_super_step: 0,
            active_node: "start".to_string(),
            cycle_counts: BTreeMap::new(),
            max_cycles_per_node: max_cycles_per_node.max(1),
            snapshots: Vec::new(),
        }
    }

    /// Advances to `node_id` in the cyclic super-step graph, guarding against infinite loops.
    pub fn advance_super_step(
        &mut self,
        node_id: impl Into<String>,
        next_nodes: Vec<String>,
        state_digest: impl Into<String>,
    ) -> Result<u64, String> {
        let node = node_id.into();
        let count = self.cycle_counts.entry(node.clone()).or_insert(0);
        *count += 1;
        if *count > self.max_cycles_per_node {
            return Err(format!(
                "GraphCheckpoint '{}' exceeded max cycle limit ({}) at node '{}'",
                self.graph_id, self.max_cycles_per_node, node
            ));
        }
        self.current_super_step += 1;
        self.active_node = node.clone();
        self.snapshots.push(SuperStepSnapshot {
            super_step: self.current_super_step,
            node_id: node,
            next_nodes,
            state_digest: state_digest.into(),
        });
        Ok(self.current_super_step)
    }

    pub fn save_to_repo(&self, repo_root: &Path) -> Result<(), String> {
        let dir = repo_root
            .join(".toksi")
            .join("projectmem")
            .join("plan_checkpoints");
        fs::create_dir_all(&dir).map_err(|e| format!("graph_checkpoint mkdir failed: {e}"))?;
        let path = dir.join(format!("graph_{}.json", self.graph_id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("graph_checkpoint serialize failed: {e}"))?;
        atomic_write_file(&path, &json).map_err(|e| format!("graph_checkpoint write failed: {e}"))
    }
}
