// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/mod.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#![deny(unsafe_code)]

pub mod ast_grep;
pub mod cache;
pub mod graph;
pub mod jcodemunch;
pub mod kernel;
pub mod mcp;
pub mod repomix;
pub mod scip;
pub mod slash;
pub mod walker;
pub mod wire;

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::DynError;
use crate::util::colors::C;

pub use ast_grep::{AstGrepEngine, AstMatch, AstRule, MatchStrictness};
pub use cache::{CacheStats, SymbolCache};
pub use graph::{BlastRadiusReport, CodeGraphEngine, ImpactedNode};
pub use jcodemunch::{extract_file_outline, generate_repo_outline, summarize_code_for_prompt, FileOutline, RepoOutline, SymbolOutline};
pub use kernel::{FileExtraction, SymbolEdge, SymbolNode, SymbolRef};
pub use repomix::{package_files, pack_workspace, RepomixBundle, RepomixFormat, RepomixOptions};
pub use scip::{format_scip_symbol, parse_scip_symbol, ScipDocument, ScipIndex, ScipOccurrence, ScipSymbolInformation, SymbolRole};
pub use slash::handle_slash_command;
pub use wire::DirectWireBus;

#[derive(thiserror::Error, Debug)]
pub enum CodeGraphError {
    #[error("CodeGraph requires a .git repository root: {0}")]
    RepoRequired(String),
    #[error("Security violation resolving CodeGraph path: {0}")]
    Security(String),
    #[error("I/O error in CodeGraph: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error in CodeGraph: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Extraction error in CodeGraph: {0}")]
    Extract(String),
}

/// Resolve `<repo>/.toksi/codegraph/symbols.db`, enforcing repo-only storage
/// in non-test builds and merging any nested/legacy CodeGraph DBs into root.
pub fn try_resolve_db_path(root: &Path) -> Result<PathBuf, CodeGraphError> {
    let clean_input = PathBuf::from(crate::util::paths::normalize_path(&root.to_string_lossy()));
    let repo_root = match crate::icm::store::find_repo_root_and_subproject(Some(root)) {
        Ok((r, _)) => {
            let clean = crate::util::paths::normalize_path(&r.to_string_lossy());
            PathBuf::from(clean)
        }
        Err(crate::icm::IcmError::Security(msg)) => {
            return Err(CodeGraphError::Security(msg));
        }
        Err(e) => {
            #[cfg(not(test))]
            {
                return Err(CodeGraphError::RepoRequired(e.to_string()));
            }
            #[cfg(test)]
            {
                let _ = e;
                clean_input.clone()
            }
        }
    };

    let toksi_dir = repo_root.join(".toksi").join("codegraph");
    let toksi_db = toksi_dir.join("symbols.db");
    migrate_nested_codegraph_dbs(&repo_root, &clean_input, &toksi_dir, &toksi_db);
    Ok(toksi_db)
}

/// Best-effort wrapper returning the repository-root CodeGraph DB path.
pub fn resolve_db_path(root: &Path) -> PathBuf {
    match try_resolve_db_path(root) {
        Ok(p) => p,
        Err(_) => {
            let clean = crate::util::paths::normalize_path(&root.to_string_lossy());
            PathBuf::from(clean).join(".toksi").join("codegraph").join("symbols.db")
        }
    }
}

fn migrate_nested_codegraph_dbs(
    repo_root: &Path,
    clean_input: &Path,
    toksi_dir: &Path,
    toksi_db: &Path,
) {
    let mut candidates: Vec<(PathBuf, String)> = Vec::new();
    let root_legacy = repo_root.join(".codegraph").join("symbols.db");
    if root_legacy.exists() {
        candidates.push((root_legacy, ".".to_string()));
    }
    if clean_input != repo_root {
        let rel_prefix = clean_input
            .strip_prefix(repo_root)
            .map(|p| crate::util::paths::normalize_path(&p.to_string_lossy()))
            .unwrap_or_else(|_| ".".to_string());
        for sub in [".toksi/codegraph/symbols.db", ".codegraph/symbols.db"] {
            let p = clean_input.join(sub);
            if p.exists() && p != *toksi_db {
                candidates.push((p, rel_prefix.clone()));
            }
        }
    }

    // Crawl workspace up to depth 5 for any nested subcrate `.toksi/codegraph` or `.codegraph` DBs
    let mut wb = ignore::WalkBuilder::new(repo_root);
    wb.hidden(false).max_depth(Some(5)).filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !matches!(
            name.as_ref(),
            ".git" | "target" | "node_modules" | "dist" | "build" | "inspect-vault"
        )
    });
    for entry in wb.build().flatten() {
        if entry.file_name() == "symbols.db" {
            let p = entry.path().to_path_buf();
            if p == *toksi_db {
                continue;
            }
            let norm_p = crate::util::paths::normalize_path(&p.to_string_lossy());
            if norm_p.ends_with("/.toksi/codegraph/symbols.db")
                || norm_p.ends_with("/.codegraph/symbols.db")
            {
                let sub_dir = if let Some(base) = norm_p
                    .strip_suffix("/.toksi/codegraph/symbols.db")
                    .or_else(|| norm_p.strip_suffix("/.codegraph/symbols.db"))
                {
                    let base_path = Path::new(base);
                    base_path
                        .strip_prefix(repo_root)
                        .map(|r| crate::util::paths::normalize_path(&r.to_string_lossy()))
                        .unwrap_or_else(|_| ".".to_string())
                } else {
                    ".".to_string()
                };
                if !candidates.iter().any(|(existing, _)| existing == &p) {
                    candidates.push((p, sub_dir));
                }
            }
        }
    }

    if candidates.is_empty() {
        return;
    }

    let _ = fs::create_dir_all(toksi_dir);
    if let Ok(mut root_cache) = SymbolCache::open(toksi_db) {
        for (nested_db, prefix) in candidates {
            if root_cache.merge_from_db(&nested_db, &prefix).is_ok() {
                let _ = fs::remove_file(&nested_db);
                if let Some(parent) = nested_db.parent() {
                    let _ = fs::remove_dir(parent);
                    if let Some(gp) = parent.parent() {
                        if gp.file_name().and_then(|s| s.to_str()) == Some(".toksi") {
                            let _ = fs::remove_dir(gp);
                        }
                    }
                }
            }
        }
    }
}

/// User home storage for codegraph is strictly prohibited by consolidation mandates.
pub fn user_home_db_path() -> Option<PathBuf> {
    None
}

pub struct CodeGraph {
    pub root: PathBuf,
    pub cache: SymbolCache,
    pub engine: CodeGraphEngine,
}

impl CodeGraph {
    pub fn open(root: &Path) -> Result<Self, CodeGraphError> {
        let abs_root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        let norm_root = PathBuf::from(crate::util::paths::normalize_path(&abs_root.to_string_lossy()));

        let db_path = try_resolve_db_path(&norm_root)?;
        let cache = SymbolCache::open(&db_path)?;
        let nodes = cache.get_all_nodes();
        let edges = cache.get_all_edges();
        let engine = CodeGraphEngine::build(nodes, edges);

        Ok(Self {
            root: norm_root,
            cache,
            engine,
        })
    }

    pub fn ensure_indexed(&mut self) -> Result<bool, DynError> {
        let files = walker::walk_project_files(&self.root);
        let cached = self.cache.get_cached_files();
        let mut on_disk_paths = HashSet::new();

        let mut changed_files = Vec::new();
        for f in &files {
            on_disk_paths.insert(f.rel_path.clone());
            match cached.get(&f.rel_path) {
                Some(&(mtime, size, _)) if mtime == f.mtime && size == f.size => {
                    // Cache hit on mtime & size
                }
                _ => {
                    changed_files.push(f.clone());
                }
            }
        }

        // Detect deleted or newly-ignored files (dynamic ignore purge)
        let gitignore = walker::build_project_ignore(&self.root);
        let mut deleted_files = Vec::new();
        for cached_path in cached.keys() {
            let full_path = self.root.join(cached_path);
            let is_ignored = gitignore.matched_path_or_any_parents(&full_path, false).is_ignore();
            if !on_disk_paths.contains(cached_path) || is_ignored {
                deleted_files.push(cached_path.clone());
            }
        }

        if changed_files.is_empty() && deleted_files.is_empty() {
            return Ok(false);
        }

        for deleted in deleted_files {
            let _ = self.cache.remove_file(&deleted);
        }

        for file in changed_files {
            let content_bytes = match fs::read(&file.path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let content_str = String::from_utf8_lossy(&content_bytes);

            if kernel::textutil::is_minified_content(&file.rel_path, &content_str) {
                continue;
            }

            let hash = cache::content_hash(&content_bytes);
            if let Ok(extraction) = kernel::extract_file(&file.rel_path, &content_str, file.language) {
                let _ = self.cache.save_extraction_with_source(&file, &hash, &extraction, Some(&content_str));
            }
        }

        // Rebuild graph engine and resolve cross-file references
        let all_nodes = self.cache.get_all_nodes();
        let all_edges = self.cache.get_all_edges();
        let mut engine = CodeGraphEngine::build(all_nodes, all_edges);

        let all_refs = self.cache.get_all_refs();
        let resolved = engine.resolve_references(&all_refs);
        let _ = self.cache.save_resolved_edges(&resolved);

        for e in resolved {
            engine.add_edge(e);
        }

        self.engine = engine;
        Ok(true)
    }

    pub fn index_force(&mut self) -> Result<CacheStats, DynError> {
        let db_path = resolve_db_path(&self.root);
        let _ = fs::remove_file(&db_path);
        self.cache = SymbolCache::open(&db_path)?;
        self.ensure_indexed()?;
        Ok(self.cache.get_stats())
    }
}

/// Cross-tool adaptation: maps file:line to enclosing symbol name (e.g. OrderService::process)
pub fn resolve_coordinate(root: &Path, file_path: &str, line: u32) -> Option<String> {
    let mut cg = CodeGraph::open(root).ok()?;
    let path = Path::new(file_path);
    let norm = crate::util::paths::normalize_path(file_path);
    let rel = if path.is_absolute() {
        path.strip_prefix(&cg.root)
            .ok()
            .map(|p| crate::util::paths::normalize_path(&p.to_string_lossy()))
    } else {
        Some(
            norm.trim_start_matches("./")
                .trim_start_matches(".\\")
                .to_string(),
        )
    };
    let rel_str = rel.as_deref().unwrap_or(&norm);

    if let Some(node) = cg.engine.resolve_coordinate(rel_str, line) {
        return Some(node.qualified_name.clone());
    }

    if cg.ensure_indexed().unwrap_or(false) {
        if let Some(node) = cg.engine.resolve_coordinate(rel_str, line) {
            return Some(node.qualified_name.clone());
        }
    }

    None
}

/// Cross-tool adaptation: provides symbol-level line slices for ctx_execute_file
pub fn get_symbol_slices(root: &Path, file_path: &str) -> Vec<(String, u32, u32)> {
    let cg = match CodeGraph::open(root) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let symbols = cg.engine.get_file_symbols(file_path);
    symbols
        .into_iter()
        .map(|s| (s.name.clone(), s.start_line, s.end_line))
        .collect()
}

pub fn run_graph_cli(argv: &[String]) -> i32 {
    let subcmd = argv.first().map(|s| s.as_str()).unwrap_or("status");
    let rest = if argv.is_empty() { &[][..] } else { &argv[1..] };

    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut force = false;
    let mut depth = 2usize;
    let mut kind_filter: Option<String> = None;
    let mut limit = 20usize;
    let mut max_files = 5usize;
    let mut query = String::new();

    let mut idx = 0;
    while idx < rest.len() {
        let arg = &rest[idx];
        if arg == "--root" && idx + 1 < rest.len() {
            root = PathBuf::from(&rest[idx + 1]);
            idx += 2;
        } else if arg == "--force" {
            force = true;
            idx += 1;
        } else if arg == "--depth" && idx + 1 < rest.len() {
            if let Ok(d) = rest[idx + 1].parse::<usize>() {
                depth = d;
            }
            idx += 2;
        } else if arg == "--kind" && idx + 1 < rest.len() {
            kind_filter = Some(rest[idx + 1].clone());
            idx += 2;
        } else if (arg == "--limit" || arg == "-n") && idx + 1 < rest.len() {
            if let Ok(l) = rest[idx + 1].parse::<usize>() {
                limit = l;
            }
            idx += 2;
        } else if arg == "--max-files" && idx + 1 < rest.len() {
            if let Ok(m) = rest[idx + 1].parse::<usize>() {
                max_files = m;
            }
            idx += 2;
        } else if query.is_empty() && !arg.starts_with('-') {
            query = arg.clone();
            idx += 1;
        } else {
            idx += 1;
        }
    }

    if subcmd == "mcp" || subcmd == "serve" {
        return mcp::run_mcp_server();
    }

    let mut cg = match CodeGraph::open(&root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to open codegraph: {}", C.red("Error:"), e);
            return 1;
        }
    };

    match subcmd {
        "index" => {
            println!("{} indexing project at {}", C.cyan("codegraph"), root.display());
            let stats = if force {
                match cg.index_force() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{} Index failed: {}", C.red("Error:"), e);
                        return 1;
                    }
                }
            } else {
                let _ = cg.ensure_indexed();
                cg.cache.get_stats()
            };
            println!(
                "{} indexed {} files, {} symbols, {} edges",
                C.green("Success:"),
                stats.file_count,
                stats.node_count,
                stats.edge_count
            );
            0
        }
        "status" => {
            let _ = cg.ensure_indexed();
            let stats = cg.cache.get_stats();
            println!("{}", C.bold("CodeGraph Status:"));
            println!("  Project Root: {}", cg.root.display());
            println!("  Database:     {}", cg.cache.db_path.display());
            println!("  Files:        {}", stats.file_count);
            println!("  Symbols:      {}", stats.node_count);
            println!("  Edges:        {}", stats.edge_count);
            println!("  References:   {}", stats.ref_count);
            0
        }
        "files" => {
            let _ = cg.ensure_indexed();
            let files = cg.cache.get_indexed_files();
            println!("{} indexed files:", files.len());
            for (file, lang, count) in files {
                println!("  {:<50} [{}] ({} symbols)", file, lang, count);
            }
            0
        }
        "symbols" | "search" => {
            let _ = cg.ensure_indexed();
            if query.is_empty() || query == "*" {
                let all_nodes = cg.cache.get_all_nodes();
                let filtered: Vec<&SymbolNode> = all_nodes
                    .iter()
                    .filter(|n| n.kind != "file")
                    .filter(|n| {
                        if let Some(ref kf) = kind_filter {
                            n.kind.eq_ignore_ascii_case(kf)
                        } else {
                            true
                        }
                    })
                    .take(limit)
                    .collect();

                println!("AST Symbols ({} shown):", filtered.len());
                for sym in filtered {
                    let sig = sym.signature.as_deref().unwrap_or("");
                    println!(
                        "  {:<12} {:<30} {}:{} {}",
                        sym.kind, sym.name, sym.file_path, sym.start_line, sig
                    );
                }
            } else {
                let results = cg.engine.search_symbols(&query, kind_filter.as_deref(), limit);
                println!("Found {} symbols matching '{}':", results.len(), query);
                for sym in results {
                    let sig = sym.signature.as_deref().unwrap_or("");
                    println!(
                        "  {:<12} {:<30} {}:{} {}",
                        sym.kind, sym.name, sym.file_path, sym.start_line, sig
                    );
                }
            }
            0
        }
        "symbol" | "node" => {
            if query.is_empty() {
                eprintln!("Usage: toksi graph symbol <name_or_id>");
                return 1;
            }
            let _ = cg.ensure_indexed();
            let res = mcp::handle_node(
                &cg,
                &serde_json::json!({
                    "symbol": query
                }),
            );
            println!("{}", res);
            0
        }
        "callers" => {
            if query.is_empty() {
                eprintln!("Usage: toksi graph callers <symbol>");
                return 1;
            }
            let _ = cg.ensure_indexed();
            let nodes = cg.engine.find_symbols_by_name(&query);
            if nodes.is_empty() {
                println!("Symbol '{}' not found.", query);
                return 0;
            }
            let focal = nodes[0];
            let callers = cg.engine.get_callers(&focal.id);
            println!("Callers of `{}` ({}):", focal.name, callers.len());
            for (caller, edge) in callers {
                println!(
                    "  {:<12} {:<30} {}:{} ({})",
                    caller.kind,
                    caller.name,
                    caller.file_path,
                    edge.line.unwrap_or(caller.start_line),
                    edge.kind
                );
            }
            0
        }
        "callees" => {
            if query.is_empty() {
                eprintln!("Usage: toksi graph callees <symbol>");
                return 1;
            }
            let _ = cg.ensure_indexed();
            let nodes = cg.engine.find_symbols_by_name(&query);
            if nodes.is_empty() {
                println!("Symbol '{}' not found.", query);
                return 0;
            }
            let focal = nodes[0];
            let callees = cg.engine.get_callees(&focal.id);
            println!("Callees called by `{}` ({}):", focal.name, callees.len());
            for (callee, edge) in callees {
                println!(
                    "  {:<12} {:<30} {}:{} ({})",
                    callee.kind,
                    callee.name,
                    callee.file_path,
                    edge.line.unwrap_or(callee.start_line),
                    edge.kind
                );
            }
            0
        }
        "impact" => {
            if query.is_empty() {
                eprintln!("Usage: toksi graph impact <symbol> [--depth <n>]");
                return 1;
            }
            let _ = cg.ensure_indexed();
            let nodes = cg.engine.find_symbols_by_name(&query);
            if nodes.is_empty() {
                println!("Symbol '{}' not found.", query);
                return 0;
            }
            let focal = nodes[0];
            if let Some(report) = cg.engine.compute_blast_radius(&focal.id, depth) {
                println!(
                    "Blast Radius for `{}` in `{}` (risk: {:.1}/100):",
                    report.focal_name, report.focal_file, report.risk_score
                );
                println!("Affected Files ({}):", report.affected_files.len());
                for f in &report.affected_files {
                    println!("  - {}", f);
                }
                println!("\nUpstream Callers & Dependents ({}):", report.upstream_dependents.len());
                for dep in &report.upstream_dependents {
                    println!(
                        "  [depth {}] {:<12} {:<25} {} ({})",
                        dep.depth, dep.node.kind, dep.node.name, dep.node.file_path, dep.relation
                    );
                }
                println!("\nDownstream Dependencies ({}):", report.downstream_dependencies.len());
                for dep in &report.downstream_dependencies {
                    println!(
                        "  [depth {}] {:<12} {:<25} {} ({})",
                        dep.depth, dep.node.kind, dep.node.name, dep.node.file_path, dep.relation
                    );
                }
            }
            0
        }
        "explore" => {
            if query.is_empty() {
                eprintln!("Usage: toksi graph explore <query> [--max-files <n>]");
                return 1;
            }
            let _ = cg.ensure_indexed();
            let res = mcp::handle_explore(
                &cg,
                &serde_json::json!({
                    "query": query,
                    "maxFiles": max_files
                }),
            );
            println!("{}", res);
            0
        }
        "help" | "--help" | "-h" => {
            println!("Usage: toksi graph <explore|impact|symbols|index> [options]");
            println!("Commands:");
            println!("  explore    Semantic and call-path code explorer");
            println!("  impact     Compute blast-radius of modifying a symbol");
            println!("  symbols    Search and list AST symbols (aliases: symbol, search)");
            println!("  index      Build/update AST call-graph index");
            println!("  status     Show index statistics");
            println!("  files      List indexed files");
            println!("  callers    Find callers of a symbol");
            println!("  callees    Find callees called by a symbol");
            println!("  mcp        Run in-process MCP server");
            0
        }
        _ => {
            eprintln!(
                "Unknown graph command '{}'. Available: explore, impact, symbols, index, status, files, callers, callees, mcp",
                subcmd
            );
            1
        }
    }
}
