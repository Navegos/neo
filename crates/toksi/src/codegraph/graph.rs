// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/graph.rs
// created: 2026-10-28
// lastModified: 2026-11-16

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::path::Path;

use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};

use super::kernel::{langs, SymbolEdge, SymbolNode, SymbolRef};
use super::walker;

pub const MAX_BLAST_RADIUS_NODES: usize = 400;
pub const MAX_IMPLEMENTATIONS: usize = 400;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedNode {
    pub node: SymbolNode,
    pub depth: usize,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationEntry {
    pub node: SymbolNode,
    pub depth: usize,
    pub relation: String,
    pub is_test: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedSearchQuery {
    pub text: String,
    pub terms: Vec<String>,
    pub kinds: Vec<String>,
    pub languages: Vec<String>,
    pub path_filters: Vec<String>,
    pub name_filters: Vec<String>,
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
    #[serde(default)]
    pub truncated: bool,
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
        if let Some(existing) = self
            .graph
            .edges_connecting(src_idx, tgt_idx)
            .find(|er| er.weight().kind == edge.kind && er.weight().line == edge.line)
        {
            return Some(existing.id());
        }
        Some(self.graph.add_edge(src_idx, tgt_idx, edge))
    }

    pub fn get_node(&self, id: &str) -> Option<&SymbolNode> {
        let idx = self.id_to_index.get(id)?;
        Some(&self.graph[*idx])
    }

    pub fn find_symbols_by_name(&self, name: &str) -> Vec<&SymbolNode> {
        let trimmed = name.trim();
        let name_lower = trimmed.to_ascii_lowercase();
        if let Some(indices) = self.name_to_indices.get(&name_lower) {
            return indices.iter().map(|&idx| &self.graph[idx]).collect();
        }
        if trimmed.contains("::") || trimmed.contains('.') {
            let norm_q = trimmed.replace('.', "::").to_ascii_lowercase();
            let leaf = norm_q.rsplit("::").next().unwrap_or(&norm_q);
            if let Some(indices) = self.name_to_indices.get(leaf) {
                let suffix = format!("::{}", norm_q);
                let mut exact_scope = Vec::new();
                for &idx in indices {
                    let node = &self.graph[idx];
                    let qn = node.qualified_name.replace('.', "::").to_ascii_lowercase();
                    if qn == norm_q || qn.ends_with(&suffix) {
                        exact_scope.push(node);
                    }
                }
                if !exact_scope.is_empty() {
                    return exact_scope;
                }
            }
        }
        Vec::new()
    }

    pub fn get_file_symbols(&self, file_path: &str) -> Vec<&SymbolNode> {
        let normalized = crate::util::paths::normalize_path(file_path);
        if let Some(indices) = self.file_to_indices.get(&normalized) {
            return indices.iter().map(|&idx| &self.graph[idx]).collect();
        }
        let clean = normalized.trim_start_matches("./");
        if let Some(indices) = self.file_to_indices.get(clean) {
            return indices.iter().map(|&idx| &self.graph[idx]).collect();
        }
        for (f, indices) in &self.file_to_indices {
            if f == clean || f.ends_with(&format!("/{}", clean)) {
                return indices.iter().map(|&idx| &self.graph[idx]).collect();
            }
        }
        Vec::new()
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

    fn resolve_import_to_file_node(&self, from_file: &str, import_spec: &str) -> Option<NodeIndex> {
        let spec = import_spec
            .trim()
            .trim_matches(|c| c == '"' || c == '\'' || c == ';');
        if spec.is_empty() {
            return None;
        }

        let candidate_stem = if spec.starts_with("./") || spec.starts_with("../") {
            let from_dir = Path::new(from_file).parent().unwrap_or_else(|| Path::new(""));
            let mut segs: Vec<String> = crate::util::paths::normalize_path(&from_dir.to_string_lossy())
                .split('/')
                .filter(|s| !s.is_empty() && *s != ".")
                .map(str::to_string)
                .collect();
            for part in spec.split('/') {
                match part {
                    "" | "." => {}
                    ".." => {
                        segs.pop();
                    }
                    other => segs.push(other.to_string()),
                }
            }
            segs.join("/")
        } else if spec.contains("::") {
            let cleaned = spec
                .trim_start_matches("crate::")
                .trim_start_matches("self::")
                .trim_start_matches("super::");
            cleaned.replace("::", "/")
        } else if spec.contains('/') {
            spec.to_string()
        } else {
            spec.replace('.', "/")
        };

        let cand_lower = candidate_stem.to_ascii_lowercase();
        if cand_lower.is_empty() {
            return None;
        }

        let mut matched_file: Option<&Vec<NodeIndex>> = None;
        for (fp, indices) in &self.file_to_indices {
            if fp == from_file {
                continue;
            }
            let fp_lower = fp.to_ascii_lowercase();
            let fp_no_ext = fp_lower
                .rsplit_once('.')
                .map(|(stem, _)| stem)
                .unwrap_or(&fp_lower);
            let is_match = fp_lower == cand_lower
                || fp_no_ext == cand_lower
                || fp_no_ext == format!("{}/index", cand_lower)
                || fp_no_ext == format!("{}/mod", cand_lower)
                || fp_no_ext == format!("{}/__init__", cand_lower)
                || fp_no_ext.ends_with(&format!("/{}", cand_lower))
                || fp_no_ext.ends_with(&format!("/{}/mod", cand_lower))
                || fp_no_ext.ends_with(&format!("/{}/index", cand_lower));
            if is_match {
                matched_file = Some(indices);
                break;
            }
        }

        let indices = matched_file?;
        indices
            .iter()
            .copied()
            .find(|&idx| self.graph[idx].kind == "file")
            .or_else(|| indices.first().copied())
    }

    pub fn resolve_references(&self, refs: &[SymbolRef]) -> Vec<SymbolEdge> {
        let mut new_edges = Vec::new();

        for r in refs {
            let src_idx = match self.id_to_index.get(&r.from_node_id) {
                Some(&i) => i,
                None => continue,
            };

            if r.kind == "imports"
                && let Some(tgt_idx) = self.resolve_import_to_file_node(&r.file_path, &r.name)
                && tgt_idx != src_idx
            {
                new_edges.push(SymbolEdge {
                    source_id: self.graph[src_idx].id.clone(),
                    target_id: self.graph[tgt_idx].id.clone(),
                    kind: "imports".to_string(),
                    line: Some(r.line),
                    column: Some(r.column),
                });
                continue;
            }

            let base_name = r.name.split('<').next().unwrap_or(&r.name).trim();
            let after_scope = base_name.rsplit("::").next().unwrap_or(base_name);
            let name_clean = after_scope.rsplit('.').next().unwrap_or(after_scope);
            let name_lower = name_clean.to_lowercase();

            let candidates = match self.name_to_indices.get(&name_lower) {
                Some(c) => c,
                None => continue,
            };

            let mut matched_target: Option<NodeIndex> = None;
            let is_call_ref = r.kind == "function_ref" || r.kind == "calls";
            let is_type_rel = r.kind == "implements" || r.kind == "extends";

            // 1. Same file match (non-self preferred)
            for &c_idx in candidates {
                if c_idx != src_idx && self.graph[c_idx].file_path == r.file_path {
                    matched_target = Some(c_idx);
                    break;
                }
            }

            // 2. Global match
            if matched_target.is_none() {
                if candidates.len() == 1 && candidates[0] != src_idx {
                    matched_target = Some(candidates[0]);
                } else {
                    for &c_idx in candidates {
                        if c_idx == src_idx {
                            continue;
                        }
                        let k = self.graph[c_idx].kind.as_str();
                        let kind_ok = if is_type_rel {
                            matches!(
                                k,
                                "trait" | "interface" | "protocol" | "class" | "struct" | "type" | "type_alias" | "enum"
                            )
                        } else {
                            matches!(k, "function" | "method" | "struct" | "class")
                        };
                        if kind_ok {
                            matched_target = Some(c_idx);
                            break;
                        }
                    }
                }
            }

            // 3. Allow recursive self-call inside the same function/method
            if matched_target.is_none() && is_call_ref {
                for &c_idx in candidates {
                    if c_idx == src_idx && self.graph[c_idx].file_path == r.file_path {
                        matched_target = Some(c_idx);
                        break;
                    }
                }
            }

            if let Some(tgt_idx) = matched_target {
                let edge_kind = match r.kind.as_str() {
                    "function_ref" | "calls" => "calls",
                    "imports" => "imports",
                    "implements" => "implements",
                    "extends" => "extends",
                    _ => "references",
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
        let parsed = parse_search_query(query);
        let q_clean = parsed.text.trim().to_lowercase();
        let mut results = Vec::new();

        for node in self.graph.node_weights() {
            if node.kind == "file" {
                continue;
            }
            if let Some(kf) = kind_filter
                && !node.kind.eq_ignore_ascii_case(kf)
            {
                continue;
            }
            if !parsed.kinds.is_empty()
                && !parsed.kinds.iter().any(|k| {
                    node.kind.eq_ignore_ascii_case(k)
                        || (k == "type_alias" && node.kind.eq_ignore_ascii_case("type"))
                        || (k == "type" && node.kind.eq_ignore_ascii_case("type_alias"))
                })
            {
                continue;
            }
            if !parsed.languages.is_empty() {
                let node_lang = walker::detect_language(Path::new(&node.file_path))
                    .map(langs::canonical_language);
                let Some(nl) = node_lang else {
                    continue;
                };
                if !parsed.languages.iter().any(|l| l == nl) {
                    continue;
                }
            }

            let name_lower = node.name.to_lowercase();
            let qn_lower = node.qualified_name.to_lowercase();
            let file_lower = node.file_path.to_lowercase();

            if !parsed.path_filters.is_empty()
                && !parsed.path_filters.iter().any(|p| file_lower.contains(p))
            {
                continue;
            }
            if !parsed.name_filters.is_empty()
                && !parsed
                    .name_filters
                    .iter()
                    .any(|n| name_lower.contains(n) || qn_lower.contains(n))
            {
                continue;
            }

            let matches_all = parsed.terms.iter().all(|term| {
                let scope_term = term.replace('.', "::");
                name_lower.contains(term)
                    || qn_lower.contains(term)
                    || qn_lower.contains(&scope_term)
                    || file_lower.contains(term)
            });

            if matches_all {
                results.push(node);
            }
        }

        // Rank by exact name match first, then shorter names
        results.sort_by(|a, b| {
            let a_exact = !q_clean.is_empty() && a.name.eq_ignore_ascii_case(&q_clean);
            let b_exact = !q_clean.is_empty() && b.name.eq_ignore_ascii_case(&q_clean);
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

    /// Computes circular file dependencies via iterative Tarjan SCC over cross-file graph edges.
    pub fn find_file_cycles(&self) -> Vec<Vec<String>> {
        let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut nodes_set: BTreeSet<String> = BTreeSet::new();

        for edge_ref in self.graph.edge_references() {
            if edge_ref.weight().kind == "contains" {
                continue;
            }
            let src_file = &self.graph[edge_ref.source()].file_path;
            let tgt_file = &self.graph[edge_ref.target()].file_path;
            if !src_file.is_empty() && !tgt_file.is_empty() && src_file != tgt_file {
                nodes_set.insert(src_file.clone());
                nodes_set.insert(tgt_file.clone());
                adj.entry(src_file.clone())
                    .or_default()
                    .insert(tgt_file.clone());
            }
        }

        let nodes: Vec<String> = nodes_set.into_iter().collect();
        let adj_vec: HashMap<String, Vec<String>> = adj
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        let mut index_map: HashMap<String, usize> = HashMap::new();
        let mut low_map: HashMap<String, usize> = HashMap::new();
        let mut on_stack: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = Vec::new();
        let mut components: Vec<Vec<String>> = Vec::new();
        let mut counter = 0usize;

        struct Frame {
            id: String,
            at: usize,
        }

        for start in &nodes {
            if index_map.contains_key(start) {
                continue;
            }
            let mut work = vec![Frame {
                id: start.clone(),
                at: 0,
            }];
            index_map.insert(start.clone(), counter);
            low_map.insert(start.clone(), counter);
            counter += 1;
            stack.push(start.clone());
            on_stack.insert(start.clone());

            while let Some(top_idx) = work.len().checked_sub(1) {
                let cur_id = work[top_idx].id.clone();
                let next_opt = adj_vec
                    .get(&cur_id)
                    .and_then(|edges| edges.get(work[top_idx].at))
                    .cloned();

                if let Some(next) = next_opt {
                    work[top_idx].at += 1;
                    if !index_map.contains_key(&next) {
                        index_map.insert(next.clone(), counter);
                        low_map.insert(next.clone(), counter);
                        counter += 1;
                        stack.push(next.clone());
                        on_stack.insert(next.clone());
                        work.push(Frame { id: next, at: 0 });
                    } else if on_stack.contains(&next) {
                        let cur_low = low_map.get(&cur_id).copied().unwrap_or(0);
                        let next_idx = index_map.get(&next).copied().unwrap_or(0);
                        low_map.insert(cur_id, cur_low.min(next_idx));
                    }
                    continue;
                }

                work.pop();
                let cur_low = low_map.get(&cur_id).copied().unwrap_or(0);
                let cur_idx = index_map.get(&cur_id).copied().unwrap_or(0);
                if cur_low == cur_idx {
                    let mut comp = Vec::new();
                    while let Some(popped) = stack.pop() {
                        on_stack.remove(&popped);
                        let is_root = popped == cur_id;
                        comp.push(popped);
                        if is_root {
                            break;
                        }
                    }
                    if comp.len() > 1 {
                        comp.sort();
                        components.push(comp);
                    }
                }
                if let Some(parent) = work.last() {
                    let parent_low = low_map.get(&parent.id).copied().unwrap_or(0);
                    low_map.insert(parent.id.clone(), parent_low.min(cur_low));
                }
            }
        }

        components.sort_by(|a, b| {
            a.len()
                .cmp(&b.len())
                .then_with(|| a.first().cmp(&b.first()))
        });
        components
    }

    /// Finds types implementing or extending a trait, interface, protocol, or base class.
    pub fn find_implementations(&self, symbol: &str, max_depth: usize) -> Vec<ImplementationEntry> {
        let depth_limit = max_depth.clamp(1, 6);
        let focals: Vec<NodeIndex> = if let Some(&idx) = self.id_to_index.get(symbol) {
            vec![idx]
        } else {
            self.find_symbols_by_name(symbol)
                .into_iter()
                .filter_map(|n| self.id_to_index.get(&n.id).copied())
                .collect()
        };

        let mut visited: HashSet<NodeIndex> = focals.iter().copied().collect();
        let mut queue: VecDeque<(NodeIndex, usize)> = focals.into_iter().map(|i| (i, 0)).collect();
        let mut entries = Vec::new();

        while let Some((curr_idx, depth)) = queue.pop_front() {
            if depth >= depth_limit || entries.len() >= MAX_IMPLEMENTATIONS {
                continue;
            }
            for edge_ref in self.graph.edges_directed(curr_idx, Direction::Incoming) {
                let rel = edge_ref.weight().kind.as_str();
                if rel != "implements" && rel != "extends" {
                    continue;
                }
                let sub_idx = edge_ref.source();
                if visited.insert(sub_idx) {
                    let node = self.graph[sub_idx].clone();
                    let is_test = walker::is_test_file(&node.file_path)
                        || node.qualified_name.to_ascii_lowercase().contains("::tests::");
                    entries.push(ImplementationEntry {
                        node,
                        depth: depth + 1,
                        relation: rel.to_string(),
                        is_test,
                    });
                    if entries.len() >= MAX_IMPLEMENTATIONS {
                        break;
                    }
                    queue.push_back((sub_idx, depth + 1));
                }
            }
        }

        entries.sort_by(|a, b| {
            a.depth
                .cmp(&b.depth)
                .then_with(|| a.node.file_path.cmp(&b.node.file_path))
                .then_with(|| a.node.start_line.cmp(&b.node.start_line))
                .then_with(|| a.node.name.cmp(&b.node.name))
        });
        entries
    }

    pub fn compute_blast_radius(&self, focal_id: &str, max_depth: usize) -> Option<BlastRadiusReport> {
        let focal_idx = *self.id_to_index.get(focal_id)?;
        let focal_node = self.graph[focal_idx].clone();

        let mut upstream_dependents = Vec::new();
        let mut downstream_dependencies = Vec::new();
        let mut affected_files = HashSet::new();
        affected_files.insert(focal_node.file_path.clone());
        let mut truncated = false;

        // BFS Upstream (Incoming callers / users)
        let mut visited_upstream = HashSet::new();
        visited_upstream.insert(focal_idx);
        let mut queue_upstream = VecDeque::new();
        queue_upstream.push_back((focal_idx, 0usize));

        while let Some((curr_idx, depth)) = queue_upstream.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if upstream_dependents.len() >= MAX_BLAST_RADIUS_NODES {
                truncated = true;
                break;
            }

            for edge_ref in self.graph.edges_directed(curr_idx, Direction::Incoming) {
                let parent_idx = edge_ref.source();
                let e = edge_ref.weight();
                if visited_upstream.insert(parent_idx) {
                    if upstream_dependents.len() >= MAX_BLAST_RADIUS_NODES {
                        truncated = true;
                        break;
                    }
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
            if upstream_dependents.len() + downstream_dependencies.len() >= MAX_BLAST_RADIUS_NODES {
                truncated = true;
                break;
            }

            for edge_ref in self.graph.edges_directed(curr_idx, Direction::Outgoing) {
                let child_idx = edge_ref.target();
                let e = edge_ref.weight();
                if visited_downstream.insert(child_idx) {
                    if upstream_dependents.len() + downstream_dependencies.len() >= MAX_BLAST_RADIUS_NODES {
                        truncated = true;
                        break;
                    }
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
            truncated,
        })
    }
}

fn is_known_kind(s: &str) -> bool {
    matches!(
        s,
        "function"
            | "method"
            | "class"
            | "interface"
            | "type"
            | "type_alias"
            | "struct"
            | "union"
            | "trait"
            | "enum"
            | "component"
            | "route"
            | "module"
            | "property"
            | "field"
            | "variable"
            | "constant"
            | "import"
            | "export"
            | "parameter"
            | "namespace"
            | "file"
            | "protocol"
            | "enum_member"
            | "service"
            | "section"
            | "rule"
            | "target"
            | "block"
            | "stage"
            | "instruction"
            | "key"
            | "symbol"
    )
}

fn normalize_call_token(tok: &str) -> String {
    let trimmed = tok.trim().trim_end_matches([';', ',']);
    if let Some(paren_pos) = trimmed.find('(')
        && paren_pos > 0
    {
        let callee = trimmed[..paren_pos].trim();
        if !callee.is_empty() && !callee.contains(char::is_whitespace) {
            if callee.contains('.') && !callee.contains('/') {
                return callee.rsplit('.').next().unwrap_or(callee).to_string();
            }
            return callee.to_string();
        }
    }
    trimmed.to_string()
}

/// Parses field-qualified queries (`kind:`, `lang:`, `path:`, `name:`) and normalizes
/// pasted call expressions (`res.send(body)` -> `send`, `leaf(entry)` -> `leaf`).
pub fn parse_search_query(raw: &str) -> ParsedSearchQuery {
    let mut out = ParsedSearchQuery::default();
    let mut tokens = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        let mut paren_depth = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if b == b'"' {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            if b == b'(' && i > start && !raw[start..i].contains(':') {
                paren_depth += 1;
                i += 1;
                continue;
            }
            if b == b')' && paren_depth > 0 {
                paren_depth -= 1;
                i += 1;
                continue;
            }
            if b.is_ascii_whitespace() && paren_depth == 0 {
                break;
            }
            if let Some(ch) = raw[i..].chars().next() {
                i += ch.len_utf8();
            } else {
                break;
            }
        }
        tokens.push(&raw[start..i]);
    }

    let mut text_parts = Vec::new();
    for tok in tokens {
        if let Some(colon) = tok.find(':')
            && colon > 0
            && colon + 1 < tok.len()
            && !tok[..colon].contains('/')
        {
            let key = tok[..colon].to_ascii_lowercase();
            let raw_val = &tok[colon + 1..];
            let unquoted = if raw_val.len() >= 2 && raw_val.starts_with('"') && raw_val.ends_with('"') {
                &raw_val[1..raw_val.len() - 1]
            } else {
                raw_val
            };
            if !unquoted.is_empty() {
                match key.as_str() {
                    "kind" => {
                        let k_lower = unquoted.to_ascii_lowercase();
                        if is_known_kind(&k_lower) {
                            out.kinds.push(k_lower);
                            continue;
                        }
                    }
                    "lang" | "language" => {
                        let canon = langs::canonical_language(unquoted);
                        if langs::LANGUAGES.contains(&canon) {
                            out.languages.push(canon.to_string());
                            continue;
                        }
                    }
                    "path" => {
                        out.path_filters
                            .push(crate::util::paths::normalize_path(unquoted).to_ascii_lowercase());
                        continue;
                    }
                    "name" => {
                        out.name_filters.push(unquoted.to_ascii_lowercase());
                        continue;
                    }
                    _ => {}
                }
            }
        }

        let norm = normalize_call_token(tok);
        if !norm.is_empty() {
            text_parts.push(norm);
        }
    }

    out.text = text_parts.join(" ");
    out.terms = out
        .text
        .to_ascii_lowercase()
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    out
}

