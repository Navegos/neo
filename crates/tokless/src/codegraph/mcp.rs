// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use super::CodeGraph;

pub fn run_mcp_server() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line) {
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        if let Ok(req) = serde_json::from_str::<Value>(trimmed) {
            let resp = handle_mcp_message(&req);
            if let Some(resp_val) = resp {
                let _ = writeln!(writer, "{}", resp_val);
                let _ = writer.flush();
            }
        }
        line.clear();
    }
    0
}

pub fn handle_mcp_message(req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

    match method {
        "initialize" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "tokless-codegraph",
                        "version": "1.0.5"
                    }
                }
            }))
        }
        "notifications/initialized" => None,
        "tools/list" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "codegraph_explore",
                            "description": "PRIMARY TOOL — call FIRST for almost any question OR before an edit: how does X work, architecture, a bug, where/what is X, surveying an area, or the symbols you are about to change. Returns verbatim source of relevant symbols grouped by file in ONE call plus call paths.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string", "description": "Symbol names, file names, or short code terms to explore." },
                                    "maxFiles": { "type": "number", "description": "Maximum number of files to return source from, 1–20. Defaults to 5." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "codegraph_impact",
                            "description": "Computes forward and backward blast radius of modifying a symbol: upstream callers, downstream dependencies, and affected files.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "symbol": { "type": "string", "description": "Symbol name or node id to analyze." },
                                    "depth": { "type": "number", "description": "Traversal depth, 1–10. Defaults to 2." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["symbol"]
                            }
                        },
                        {
                            "name": "codegraph_symbol",
                            "description": "Inspect AST symbol definition: file, lines, docstring, kind, signature, callers, callees, and dependencies.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "symbol": { "type": "string", "description": "Symbol name or node id to inspect." },
                                    "id": { "type": "string", "description": "Optional symbol id alias." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["symbol"]
                            }
                        },
                        {
                            "name": "codegraph_callers",
                            "description": "Finds all upstream functions and methods that call or reference a symbol.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "symbol": { "type": "string", "description": "Target symbol name." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["symbol"]
                            }
                        },
                        {
                            "name": "codegraph_callees",
                            "description": "Finds all downstream functions, methods, and symbols called by a given symbol.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "symbol": { "type": "string", "description": "Source symbol name." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["symbol"]
                            }
                        },
                        {
                            "name": "codegraph_search",
                            "description": "Fast symbol search across definitions by name, kind, and location.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string", "description": "Search query terms." },
                                    "kind": { "type": "string", "description": "Optional kind filter: function, struct, class, method, trait, interface, enum, variable." },
                                    "limit": { "type": "number", "description": "Max results. Defaults to 10." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "codegraph_node",
                            "description": "Detailed view of a symbol node: location, signature, docstring, callers, and callees.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "id": { "type": "string", "description": "Symbol node id or exact name." },
                                    "symbol": { "type": "string", "description": "Symbol name alias." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["id"]
                            }
                        },
                        {
                            "name": "codegraph_status",
                            "description": "Index health check: indexed files, nodes, edges, and cache status.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                }
                            }
                        },
                        {
                            "name": "codegraph_files",
                            "description": "Lists all indexed files and symbol counts.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                }
                            }
                        }
                    ]
                }
            }))
        }
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            let project_path = args
                .get("projectPath")
                .and_then(|p| p.as_str())
                .map(String::from)
                .or_else(|| env::var("CODEGRAPH_PROJECT_DIR").ok())
                .unwrap_or_else(|| ".".into());

            let mut cg = match CodeGraph::open(&PathBuf::from(&project_path)) {
                Ok(c) => c,
                Err(e) => {
                    return Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": format!("Failed to open codegraph: {}", e) }],
                            "isError": true
                        }
                    }));
                }
            };

            let _ = cg.ensure_indexed();

            let result = match tool_name {
                "codegraph_explore" => handle_explore(&cg, &args),
                "codegraph_impact" => handle_impact(&cg, &args),
                "codegraph_symbol" | "codegraph_node" => handle_node(&cg, &args),
                "codegraph_callers" => handle_callers(&cg, &args),
                "codegraph_callees" => handle_callees(&cg, &args),
                "codegraph_search" => handle_search(&cg, &args),
                "codegraph_status" => handle_status(&cg),
                "codegraph_files" => handle_files(&cg),
                _ => format!("Unknown tool: {}", tool_name),
            };

            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": result }]
                }
            }))
        }
        _ => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }))
        }
    }
}

pub(crate) fn handle_explore(cg: &CodeGraph, args: &Value) -> String {
    let query = match args.get("query").and_then(|q| q.as_str()) {
        Some(q) if !q.trim().is_empty() => q.trim(),
        _ => return "Missing required 'query' argument.".into(),
    };

    let max_files = args
        .get("maxFiles")
        .and_then(|m| m.as_u64())
        .map(|m| m.clamp(1, 20) as usize)
        .unwrap_or(5);

    let matched_nodes = cg.engine.search_symbols(query, None, 25);
    if matched_nodes.is_empty() {
        return format!("No symbols matching query '{}' found in the codebase.", query);
    }

    // Group symbols by file
    let mut file_groups: HashMap<String, Vec<&super::kernel::SymbolNode>> = HashMap::new();
    for node in matched_nodes {
        file_groups.entry(node.file_path.clone()).or_default().push(node);
    }

    let mut sorted_files: Vec<(String, Vec<&super::kernel::SymbolNode>)> = file_groups.into_iter().collect();
    sorted_files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    if sorted_files.len() > max_files {
        sorted_files.truncate(max_files);
    }

    let mut out = String::new();
    out.push_str(&format!("# CodeGraph Explore: {}\n\n", query));

    // Summary list of symbols
    out.push_str("### Matched Symbols\n");
    for (file, nodes) in &sorted_files {
        let sym_names: Vec<String> = nodes
            .iter()
            .map(|n| format!("`{}` ({})", n.name, n.kind))
            .collect();
        out.push_str(&format!("- **{}**: {}\n", file, sym_names.join(", ")));
    }
    out.push('\n');

    // Call paths between matched symbols
    let matched_ids: HashSet<&str> = sorted_files
        .iter()
        .flat_map(|(_, nodes)| nodes.iter().map(|n| n.id.as_str()))
        .collect();

    let mut calls = Vec::new();
    for (_, nodes) in &sorted_files {
        for node in nodes {
            for (callee, _) in cg.engine.get_callees(&node.id) {
                if matched_ids.contains(callee.id.as_str()) {
                    calls.push(format!("`{}` -> `{}`", node.name, callee.name));
                }
            }
        }
    }

    if !calls.is_empty() {
        out.push_str("### Call Path Flow\n");
        for c in calls {
            out.push_str(&format!("- {}\n", c));
        }
        out.push('\n');
    }

    // Verbatim source code slices
    out.push_str("### Source Snippets\n");
    for (file, nodes) in &sorted_files {
        let full_path = cg.root.join(file);
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();

        out.push_str(&format!("#### File: {}\n", file));

        for node in nodes {
            if node.start_line == 0 || node.start_line as usize > lines.len() {
                continue;
            }
            let start_idx = (node.start_line as usize).saturating_sub(1);
            let end_idx = (node.end_line as usize).min(lines.len());
            let snippet = lines[start_idx..end_idx].join("\n");

            let lang = full_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            out.push_str(&format!(
                "**{}** `{}` (lines {}-{}):\n```{}\n{}\n```\n\n",
                node.kind, node.qualified_name, node.start_line, node.end_line, lang, snippet
            ));
        }
    }

    out
}

pub(crate) fn handle_impact(cg: &CodeGraph, args: &Value) -> String {
    let symbol = match args.get("symbol").and_then(|s| s.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return "Missing required 'symbol' argument.".into(),
    };

    let depth = args
        .get("depth")
        .and_then(|d| d.as_u64())
        .map(|d| d.clamp(1, 10) as usize)
        .unwrap_or(2);

    let nodes = cg.engine.find_symbols_by_name(symbol);
    let focal = if !nodes.is_empty() {
        nodes[0]
    } else if let Some(n) = cg.engine.get_node(symbol) {
        n
    } else {
        return format!("Symbol '{}' not found in codebase.", symbol);
    };

    let report = match cg.engine.compute_blast_radius(&focal.id, depth) {
        Some(r) => r,
        None => return format!("Failed to compute blast radius for '{}'.", symbol),
    };

    let mut out = String::new();
    out.push_str(&format!(
        "# Blast Radius: `{}` ({})\n",
        report.focal_name, report.focal_file
    ));
    out.push_str(&format!(
        "Risk Score: **{:.1}/100** | Traversal Depth: **{}**\n\n",
        report.risk_score, report.depth
    ));

    out.push_str("### Affected Files\n");
    for f in &report.affected_files {
        out.push_str(&format!("- `{}`\n", f));
    }
    out.push('\n');

    out.push_str(&format!(
        "### Upstream Callers & Dependents ({})\n",
        report.upstream_dependents.len()
    ));
    if report.upstream_dependents.is_empty() {
        out.push_str("*(No upstream callers found)*\n");
    } else {
        for imp in &report.upstream_dependents {
            out.push_str(&format!(
                "- [depth {}] `{}` ({}) in `{}`: {}\n",
                imp.depth, imp.node.name, imp.node.kind, imp.node.file_path, imp.relation
            ));
        }
    }
    out.push('\n');

    out.push_str(&format!(
        "### Downstream Dependencies ({})\n",
        report.downstream_dependencies.len()
    ));
    if report.downstream_dependencies.is_empty() {
        out.push_str("*(No downstream dependencies found)*\n");
    } else {
        for imp in &report.downstream_dependencies {
            out.push_str(&format!(
                "- [depth {}] `{}` ({}) in `{}`: {}\n",
                imp.depth, imp.node.name, imp.node.kind, imp.node.file_path, imp.relation
            ));
        }
    }

    out
}

pub(crate) fn handle_callers(cg: &CodeGraph, args: &Value) -> String {
    let symbol = match args.get("symbol").and_then(|s| s.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return "Missing required 'symbol' argument.".into(),
    };

    let nodes = cg.engine.find_symbols_by_name(symbol);
    let focal = if !nodes.is_empty() {
        nodes[0]
    } else if let Some(n) = cg.engine.get_node(symbol) {
        n
    } else {
        return format!("Symbol '{}' not found in codebase.", symbol);
    };

    let callers = cg.engine.get_callers(&focal.id);
    let mut out = format!("# Callers of `{}` in `{}`\n\n", focal.name, focal.file_path);
    if callers.is_empty() {
        out.push_str("No callers found.\n");
    } else {
        for (caller, edge) in callers {
            out.push_str(&format!(
                "- `{}` ({}) in `{}:{}` ({})\n",
                caller.name,
                caller.kind,
                caller.file_path,
                edge.line.unwrap_or(caller.start_line),
                edge.kind
            ));
        }
    }
    out
}

pub(crate) fn handle_callees(cg: &CodeGraph, args: &Value) -> String {
    let symbol = match args.get("symbol").and_then(|s| s.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return "Missing required 'symbol' argument.".into(),
    };

    let nodes = cg.engine.find_symbols_by_name(symbol);
    let focal = if !nodes.is_empty() {
        nodes[0]
    } else if let Some(n) = cg.engine.get_node(symbol) {
        n
    } else {
        return format!("Symbol '{}' not found in codebase.", symbol);
    };

    let callees = cg.engine.get_callees(&focal.id);
    let mut out = format!("# Callees called by `{}` in `{}`\n\n", focal.name, focal.file_path);
    if callees.is_empty() {
        out.push_str("No callees found.\n");
    } else {
        for (callee, edge) in callees {
            out.push_str(&format!(
                "- `{}` ({}) in `{}:{}` ({})\n",
                callee.name,
                callee.kind,
                callee.file_path,
                edge.line.unwrap_or(callee.start_line),
                edge.kind
            ));
        }
    }
    out
}

pub(crate) fn handle_search(cg: &CodeGraph, args: &Value) -> String {
    let query = match args.get("query").and_then(|q| q.as_str()) {
        Some(q) if !q.trim().is_empty() => q.trim(),
        _ => return "Missing required 'query' argument.".into(),
    };

    let kind = args.get("kind").and_then(|k| k.as_str());
    let limit = args
        .get("limit")
        .and_then(|l| l.as_u64())
        .map(|l| l as usize)
        .unwrap_or(10);

    let results = cg.engine.search_symbols(query, kind, limit);
    if results.is_empty() {
        return format!("No symbols found for query '{}'.", query);
    }

    let mut out = format!("# Search Results for '{}'\n\n", query);
    for sym in results {
        let sig = sym.signature.as_deref().unwrap_or("");
        out.push_str(&format!(
            "- **{}** `{}`: `{}:{}` {}\n",
            sym.kind, sym.qualified_name, sym.file_path, sym.start_line, sig
        ));
    }
    out
}

pub(crate) fn handle_node(cg: &CodeGraph, args: &Value) -> String {
    let id_or_name = match args
        .get("symbol")
        .and_then(|s| s.as_str())
        .or_else(|| args.get("id").and_then(|i| i.as_str()))
    {
        Some(i) if !i.trim().is_empty() => i.trim(),
        _ => return "Missing required 'symbol' or 'id' argument.".into(),
    };

    let node = if let Some(n) = cg.engine.get_node(id_or_name) {
        n
    } else {
        let matches = cg.engine.find_symbols_by_name(id_or_name);
        if !matches.is_empty() {
            matches[0]
        } else {
            return format!("Node '{}' not found.", id_or_name);
        }
    };

    let callers = cg.engine.get_callers(&node.id);
    let callees = cg.engine.get_callees(&node.id);

    let mut out = format!(
        "# Node: `{}`\n\
         - **Kind:** {}\n\
         - **Qualified Name:** {}\n\
         - **File:** `{}:{}-{}`\n\
         - **Visibility:** {}\n\
         - **Signature:** `{}`\n\
         - **Return Type:** `{}`\n\
         - **Docstring:** {}\n\n",
        node.name,
        node.kind,
        node.qualified_name,
        node.file_path,
        node.start_line,
        node.end_line,
        node.visibility.as_deref().unwrap_or("default"),
        node.signature.as_deref().unwrap_or("none"),
        node.return_type.as_deref().unwrap_or("none"),
        node.docstring.as_deref().unwrap_or("none")
    );

    out.push_str(&format!("### Callers ({})\n", callers.len()));
    for (caller, _) in callers {
        out.push_str(&format!("- `{}` ({}) in `{}`\n", caller.name, caller.kind, caller.file_path));
    }
    out.push('\n');

    out.push_str(&format!("### Callees ({})\n", callees.len()));
    for (callee, _) in callees {
        out.push_str(&format!("- `{}` ({}) in `{}`\n", callee.name, callee.kind, callee.file_path));
    }

    out
}

pub(crate) fn handle_status(cg: &CodeGraph) -> String {
    let stats = cg.cache.get_stats();
    format!(
        "# CodeGraph Status\n\
         - **Root:** `{}`\n\
         - **DB:** `{}`\n\
         - **Files Indexed:** {}\n\
         - **Nodes (Symbols):** {}\n\
         - **Edges (Relations):** {}\n\
         - **References:** {}\n",
        cg.root.display(),
        cg.cache.db_path.display(),
        stats.file_count,
        stats.node_count,
        stats.edge_count,
        stats.ref_count
    )
}

pub(crate) fn handle_files(cg: &CodeGraph) -> String {
    let files = cg.cache.get_indexed_files();
    let mut out = format!("# Indexed Files ({})\n\n", files.len());
    for (file, lang, sym_count) in files {
        out.push_str(&format!("- `{}` [{}] — {} symbols\n", file, lang, sym_count));
    }
    out
}
