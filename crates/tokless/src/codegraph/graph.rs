// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};

use super::kernel::{SymbolEdge, SymbolNode, SymbolRef};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedNode {
    pub node: SymbolNode,
    pub depth: usize,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    pub focal_id: String,
    pub focal_name: String,
    pub focal_file: String,
    pub upstream_dependents: Vec<ImpactedNode>,
    pub downstream_dependencies: Vec<ImpactedNode>,
    pub affected_files: Vec<String>,
    pub risk_score: f64,
    pub depth: usize,
}

pub struct CodeGraphEngine {
    graph: DiGraph<SymbolNode, SymbolEdge>,
    id_to_index: HashMap<String, NodeIndex>,
    name_to_indices: HashMap<String, Vec<NodeIndex>>,
    file_to_indices: HashMap<String, Vec<NodeIndex>>,
}

impl Default for CodeGraphEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGraphEngine {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            id_to_index: HashMap::new(),
            name_to_indices: HashMap::new(),
            file_to_indices: HashMap::new(),
        }
    }

    pub fn build(nodes: Vec<SymbolNode>, edges: Vec<SymbolEdge>) -> Self {
        let mut engine = Self::new();
        for node in nodes {
            engine.add_node(node);
        }
        for edge in edges {
            engine.add_edge(edge);
        }
        engine
    }

    pub fn add_node(&mut self, node: SymbolNode) -> NodeIndex {
        if let Some(&idx) = self.id_to_index.get(&node.id) {
            self.graph[idx] = node;
            return idx;
        }

        let id = node.id.clone();
        let name_lower = node.name.to_lowercase();
        let file = node.file_path.clone();

        let idx = self.graph.add_node(node);
        self.id_to_index.insert(id, idx);
        self.name_to_indices.entry(name_lower).or_default().push(idx);
        self.file_to_indices.entry(file).or_default().push(idx);
        idx
    }

    pub fn add_edge(&mut self, edge: SymbolEdge) -> Option<EdgeIndex> {
        let src_idx = *self.id_to_index.get(&edge.source_id)?;
        let tgt_idx = *self.id_to_index.get(&edge.target_id)?;
        Some(self.graph.add_edge(src_idx, tgt_idx, edge))
    }

    pub fn get_node(&self, id: &str) -> Option<&SymbolNode> {
        let idx = self.id_to_index.get(id)?;
        Some(&self.graph[*idx])
    }

    pub fn find_symbols_by_name(&self, name: &str) -> Vec<&SymbolNode> {
        let name_lower = name.to_lowercase();
        match self.name_to_indices.get(&name_lower) {
            Some(indices) => indices.iter().map(|&idx| &self.graph[idx]).collect(),
            None => Vec::new(),
        }
    }

    pub fn get_file_symbols(&self, file_path: &str) -> Vec<&SymbolNode> {
        let normalized = file_path.replace('\\', "/");
        match self.file_to_indices.get(&normalized) {
            Some(indices) => indices.iter().map(|&idx| &self.graph[idx]).collect(),
            None => Vec::new(),
        }
    }

    pub fn resolve_coordinate(&self, file_path: &str, line: u32) -> Option<&SymbolNode> {
        let symbols = self.get_file_symbols(file_path);
        let mut best: Option<&SymbolNode> = None;
        let mut min_span = u32::MAX;

        for s in symbols {
            if s.kind == "file" {
                continue;
            }
            if line >= s.start_line && line <= s.end_line {
                let span = s.end_line.saturating_sub(s.start_line);
                if span < min_span {
                    min_span = span;
                    best = Some(s);
                }
            }
        }
        best
    }

    pub fn resolve_references(&self, refs: &[SymbolRef]) -> Vec<SymbolEdge> {
        let mut new_edges = Vec::new();

        for r in refs {
            let src_idx = match self.id_to_index.get(&r.from_node_id) {
                Some(&i) => i,
                None => continue,
            };

            let name_clean = r.name.split('.').last().unwrap_or(&r.name);
            let name_lower = name_clean.to_lowercase();

            let candidates = match self.name_to_indices.get(&name_lower) {
                Some(c) => c,
                None => continue,
            };

            let mut matched_target: Option<NodeIndex> = None;

            // 1. Same file match
            for &c_idx in candidates {
                if c_idx != src_idx && self.graph[c_idx].file_path == r.file_path {
                    matched_target = Some(c_idx);
                    break;
                }
            }

            // 2. Global match if exactly one or name is distinctive
            if matched_target.is_none() {
                if candidates.len() == 1 && candidates[0] != src_idx {
                    matched_target = Some(candidates[0]);
                } else {
                    for &c_idx in candidates {
                        if c_idx != src_idx && (self.graph[c_idx].kind == "function" || self.graph[c_idx].kind == "method" || self.graph[c_idx].kind == "struct" || self.graph[c_idx].kind == "class") {
                            matched_target = Some(c_idx);
                            break;
                        }
                    }
                }
            }

            if let Some(tgt_idx) = matched_target {
                let edge_kind = if r.kind == "function_ref" || r.kind == "calls" {
                    "calls"
                } else {
                    "references"
                };

                new_edges.push(SymbolEdge {
                    source_id: self.graph[src_idx].id.clone(),
                    target_id: self.graph[tgt_idx].id.clone(),
                    kind: edge_kind.to_string(),
                    line: Some(r.line),
                    column: Some(r.column),
                });
            }
        }

        new_edges
    }

    pub fn get_callers(&self, target_id: &str) -> Vec<(&SymbolNode, &SymbolEdge)> {
        let idx = match self.id_to_index.get(target_id) {
            Some(&i) => i,
            None => return Vec::new(),
        };

        let mut callers = Vec::new();
        for edge_ref in self.graph.edges_directed(idx, Direction::Incoming) {
            let e = edge_ref.weight();
            if e.kind == "calls" || e.kind == "references" {
                let caller_node = &self.graph[edge_ref.source()];
                callers.push((caller_node, e));
            }
        }
        callers
    }

    pub fn get_callees(&self, source_id: &str) -> Vec<(&SymbolNode, &SymbolEdge)> {
        let idx = match self.id_to_index.get(source_id) {
            Some(&i) => i,
            None => return Vec::new(),
        };

        let mut callees = Vec::new();
        for edge_ref in self.graph.edges_directed(idx, Direction::Outgoing) {
            let e = edge_ref.weight();
            if e.kind == "calls" || e.kind == "references" {
                let callee_node = &self.graph[edge_ref.target()];
                callees.push((callee_node, e));
            }
        }
        callees
    }

    pub fn search_symbols(
        &self,
        query: &str,
        kind_filter: Option<&str>,
        limit: usize,
    ) -> Vec<&SymbolNode> {
        let q_clean = query.trim().to_lowercase();
        let terms: Vec<&str> = q_clean.split_whitespace().collect();
        let mut results = Vec::new();

        for node in self.graph.node_weights() {
            if node.kind == "file" {
                continue;
            }
            if let Some(kf) = kind_filter {
                if !node.kind.eq_ignore_ascii_case(kf) {
                    continue;
                }
            }

            let name_lower = node.name.to_lowercase();
            let qn_lower = node.qualified_name.to_lowercase();
            let file_lower = node.file_path.to_lowercase();

            let matches_all = terms.iter().all(|term| {
                name_lower.contains(term) || qn_lower.contains(term) || file_lower.contains(term)
            });

            if matches_all {
                results.push(node);
            }
        }

        // Rank by exact name match first, then shorter names
        results.sort_by(|a, b| {
            let a_exact = a.name.eq_ignore_ascii_case(&q_clean);
            let b_exact = b.name.eq_ignore_ascii_case(&q_clean);
            if a_exact != b_exact {
                return b_exact.cmp(&a_exact);
            }
            a.name.len().cmp(&b.name.len())
        });

        if results.len() > limit {
            results.truncate(limit);
        }
        results
    }

    pub fn compute_blast_radius(&self, focal_id: &str, max_depth: usize) -> Option<BlastRadiusReport> {
        let focal_idx = *self.id_to_index.get(focal_id)?;
        let focal_node = self.graph[focal_idx].clone();

        let mut upstream_dependents = Vec::new();
        let mut downstream_dependencies = Vec::new();
        let mut affected_files = HashSet::new();
        affected_files.insert(focal_node.file_path.clone());

        // BFS Upstream (Incoming callers / users)
        let mut visited_upstream = HashSet::new();
        visited_upstream.insert(focal_idx);
        let mut queue_upstream = VecDeque::new();
        queue_upstream.push_back((focal_idx, 0usize));

        while let Some((curr_idx, depth)) = queue_upstream.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for edge_ref in self.graph.edges_directed(curr_idx, Direction::Incoming) {
                let parent_idx = edge_ref.source();
                let e = edge_ref.weight();
                if visited_upstream.insert(parent_idx) {
                    let parent_node = self.graph[parent_idx].clone();
                    affected_files.insert(parent_node.file_path.clone());
                    upstream_dependents.push(ImpactedNode {
                        node: parent_node,
                        depth: depth + 1,
                        relation: e.kind.clone(),
                    });
                    queue_upstream.push_back((parent_idx, depth + 1));
                }
            }
        }

        // BFS Downstream (Outgoing callees / requirements)
        let mut visited_downstream = HashSet::new();
        visited_downstream.insert(focal_idx);
        let mut queue_downstream = VecDeque::new();
        queue_downstream.push_back((focal_idx, 0usize));

        while let Some((curr_idx, depth)) = queue_downstream.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for edge_ref in self.graph.edges_directed(curr_idx, Direction::Outgoing) {
                let child_idx = edge_ref.target();
                let e = edge_ref.weight();
                if visited_downstream.insert(child_idx) {
                    let child_node = self.graph[child_idx].clone();
                    affected_files.insert(child_node.file_path.clone());
                    downstream_dependencies.push(ImpactedNode {
                        node: child_node,
                        depth: depth + 1,
                        relation: e.kind.clone(),
                    });
                    queue_downstream.push_back((child_idx, depth + 1));
                }
            }
        }

        let risk_score = ((upstream_dependents.len() * 3 + affected_files.len() * 2) as f64)
            .min(100.0);

        let mut sorted_files: Vec<String> = affected_files.into_iter().collect();
        sorted_files.sort();

        Some(BlastRadiusReport {
            focal_id: focal_node.id,
            focal_name: focal_node.name,
            focal_file: focal_node.file_path,
            upstream_dependents,
            downstream_dependencies,
            affected_files: sorted_files,
            risk_score,
            depth: max_depth,
        })
    }
}

