// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/mofa.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! MoFA multi-agent primitives: `MoFAAgent`, `AgentContext`, `InterruptSignal`,
//! `SubtaskDAG` (`petgraph::DiGraph`), wave `ParallelScheduler`, and `SwarmHITLGate`.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::direct_wire::{global_direct_wire_bus, DirectWireBus};

#[derive(Debug, Clone, Default)]
pub struct InterruptSignal {
    triggered: Arc<AtomicBool>,
    reason: Arc<RwLock<Option<String>>>,
}

impl InterruptSignal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger(&self, reason: impl Into<String>) {
        if let Ok(mut guard) = self.reason.write() {
            *guard = Some(reason.into());
        }
        self.triggered.store(true, Ordering::SeqCst);
    }

    pub fn is_triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    pub fn reason(&self) -> Option<String> {
        self.reason.read().ok().and_then(|g| g.clone())
    }
}

pub struct AgentContext {
    pub session_id: String,
    pub workspace_root: PathBuf,
    pub metadata: HashMap<String, Value>,
    pub interrupt: InterruptSignal,
    pub wire_bus: Arc<DirectWireBus>,
}

impl AgentContext {
    pub fn new(session_id: impl Into<String>, workspace_root: PathBuf) -> Self {
        Self {
            session_id: session_id.into(),
            workspace_root,
            metadata: HashMap::new(),
            interrupt: InterruptSignal::new(),
            wire_bus: global_direct_wire_bus(),
        }
    }
}

pub trait MoFAAgent: Send + Sync {
    fn id(&self) -> &str;
    fn role(&self) -> &str;
    fn capabilities(&self) -> &[String];
    fn execute(&self, ctx: &mut AgentContext, input: &str) -> Result<String, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskNode {
    pub id: String,
    pub title: String,
    pub agent_role: String,
    pub status: SubtaskStatus,
    pub output: Option<String>,
}

/// Directed Acyclic Graph (`petgraph::DiGraph`) of subtasks supporting wave extraction.
#[derive(Debug, Default)]
pub struct SubtaskDAG {
    graph: DiGraph<SubtaskNode, ()>,
    id_to_idx: HashMap<String, NodeIndex>,
}

impl SubtaskDAG {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        agent_role: impl Into<String>,
    ) -> NodeIndex {
        let id = id.into();
        if let Some(&existing) = self.id_to_idx.get(&id) {
            return existing;
        }
        let node = SubtaskNode {
            id: id.clone(),
            title: title.into(),
            agent_role: agent_role.into(),
            status: SubtaskStatus::Pending,
            output: None,
        };
        let idx = self.graph.add_node(node);
        self.id_to_idx.insert(id, idx);
        idx
    }

    pub fn add_dependency(&mut self, prerequisite_id: &str, dependent_id: &str) -> Result<(), String> {
        let from = *self
            .id_to_idx
            .get(prerequisite_id)
            .ok_or_else(|| format!("Unknown prerequisite task '{}'", prerequisite_id))?;
        let to = *self
            .id_to_idx
            .get(dependent_id)
            .ok_or_else(|| format!("Unknown dependent task '{}'", dependent_id))?;
        self.graph.add_edge(from, to, ());
        if is_cyclic_directed(&self.graph) {
            if let Some(edge) = self.graph.find_edge(from, to) {
                self.graph.remove_edge(edge);
            }
            return Err(format!(
                "Dependency '{} -> {}' would introduce a cycle",
                prerequisite_id, dependent_id
            ));
        }
        Ok(())
    }

    /// Computes topological execution waves (Kahn's level-order).
    pub fn waves(&self) -> Result<Vec<Vec<SubtaskNode>>, String> {
        if is_cyclic_directed(&self.graph) {
            return Err("SubtaskDAG contains a cycle".to_string());
        }
        let mut in_degree: HashMap<NodeIndex, usize> = HashMap::new();
        for idx in self.graph.node_indices() {
            in_degree.insert(
                idx,
                self.graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .count(),
            );
        }

        let mut remaining = self.graph.node_count();
        let mut waves = Vec::new();
        while remaining > 0 {
            let mut ready: Vec<NodeIndex> = in_degree
                .iter()
                .filter(|(_, deg)| **deg == 0)
                .map(|(idx, _)| *idx)
                .collect();
            if ready.is_empty() {
                return Err("Unable to schedule remaining DAG nodes".to_string());
            }
            ready.sort_by(|a, b| self.graph[*a].id.cmp(&self.graph[*b].id));

            let mut wave_nodes = Vec::with_capacity(ready.len());
            for idx in ready {
                in_degree.remove(&idx);
                remaining -= 1;
                wave_nodes.push(self.graph[idx].clone());
                for succ in self.graph.neighbors_directed(idx, petgraph::Direction::Outgoing) {
                    if let Some(d) = in_degree.get_mut(&succ) {
                        *d = d.saturating_sub(1);
                    }
                }
            }
            waves.push(wave_nodes);
        }
        Ok(waves)
    }

    pub fn mark_completed(&mut self, id: &str, output: String) {
        if let Some(&idx) = self.id_to_idx.get(id) {
            self.graph[idx].status = SubtaskStatus::Completed;
            self.graph[idx].output = Some(output);
        }
    }

    pub fn mark_failed(&mut self, id: &str, err: String) {
        if let Some(&idx) = self.id_to_idx.get(id) {
            self.graph[idx].status = SubtaskStatus::Failed;
            self.graph[idx].output = Some(err);
        }
    }

    pub fn nodes(&self) -> Vec<SubtaskNode> {
        self.graph.node_weights().cloned().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitlMode {
    AutoApprove,
    RequireApprovalForMutations,
    DenyAll,
}

/// Human-in-the-loop gate for swarm actions.
#[derive(Debug, Clone)]
pub struct SwarmHITLGate {
    pub mode: HitlMode,
}

impl Default for SwarmHITLGate {
    fn default() -> Self {
        Self {
            mode: HitlMode::RequireApprovalForMutations,
        }
    }
}

impl SwarmHITLGate {
    pub fn approve(&self, action: &str, is_mutating: bool) -> Result<(), String> {
        match self.mode {
            HitlMode::AutoApprove => Ok(()),
            HitlMode::RequireApprovalForMutations if !is_mutating => Ok(()),
            HitlMode::RequireApprovalForMutations => {
                // Non-interactive default: allow safe workspace mutations only if not marked destructive
                if action.to_ascii_lowercase().contains("destroy")
                    || action.to_ascii_lowercase().contains("rm -rf")
                {
                    Err(format!("SwarmHITLGate blocked destructive action: {action}"))
                } else {
                    Ok(())
                }
            }
            HitlMode::DenyAll => Err(format!("SwarmHITLGate denied action: {action}")),
        }
    }
}

/// Wave-by-wave parallel scheduler for `SubtaskDAG`.
pub struct ParallelScheduler {
    pub hitl_gate: SwarmHITLGate,
}

impl Default for ParallelScheduler {
    fn default() -> Self {
        Self {
            hitl_gate: SwarmHITLGate::default(),
        }
    }
}

impl ParallelScheduler {
    pub async fn run_dag<F>(
        &self,
        dag: &mut SubtaskDAG,
        interrupt: &InterruptSignal,
        executor: F,
    ) -> Result<Vec<SubtaskNode>, String>
    where
        F: Fn(&SubtaskNode) -> Result<String, String> + Send + Sync + Clone + 'static,
    {
        let waves = dag.waves()?;
        for wave in waves {
            if interrupt.is_triggered() {
                return Err(format!(
                    "ParallelScheduler interrupted: {}",
                    interrupt.reason().unwrap_or_else(|| "signal".to_string())
                ));
            }
            let mut join_set = tokio::task::JoinSet::new();
            for task in wave {
                self.hitl_gate.approve(&task.title, false)?;
                let exec = executor.clone();
                join_set.spawn_blocking(move || {
                    let res = exec(&task);
                    (task.id, res)
                });
            }
            while let Some(joined) = join_set.join_next().await {
                match joined {
                    Ok((id, Ok(output))) => dag.mark_completed(&id, output),
                    Ok((id, Err(err))) => {
                        dag.mark_failed(&id, err.clone());
                        return Err(format!("Subtask '{}' failed: {}", id, err));
                    }
                    Err(e) => return Err(format!("Worker join error: {}", e)),
                }
            }
        }
        Ok(dag.nodes())
    }
}
