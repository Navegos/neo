// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/mcp.rs
// created: 2026-10-28
// lastModified: 2026-11-16

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::walker;
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
                        "name": "codegraph",
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
                            "description": "Inspect AST symbol definition: file, lines, byte slice, docstring, kind, signature, callers, callees, and dependencies.",
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
                            "name": "codegraph_implementations",
                            "description": "Lists types that implement or extend a trait, interface, protocol, or base class.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "symbol": { "type": "string", "description": "Trait, interface, protocol, or class name." },
                                    "depth": { "type": "number", "description": "Subclass traversal depth, 1–6. Defaults to 2." },
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                },
                                "required": ["symbol"]
                            }
                        },
                        {
                            "name": "codegraph_cycles",
                            "description": "Finds circular import/dependency strongly connected components across project files.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "projectPath": { "type": "string", "description": "Optional project root path." }
                                }
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
                            "description": "Lists all indexed files and symbol counts, with optional package grouping and test filters.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "format": { "type": "string", "description": "Listing format: 'list' (default) or 'package'." },
                                    "tests": { "type": "boolean", "description": "Include only test files when true, or exclude when false." },
                                    "noTests": { "type": "boolean", "description": "Exclude test files when true." },
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
            if let Some(raw_args) = params.get("arguments")
                && !raw_args.is_object()
                && !raw_args.is_null()
            {
                return Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: arguments must be an object"
                    }
                }));
            }
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            let project_path = args
                .get("projectPath")
                .and_then(|p| p.as_str())
                .map(String::from)
                .or_else(|| env::var("CODEGRAPH_PROJECT_DIR").ok())
                .unwrap_or_else(|| ".".into());

            let mut project_path_buf = PathBuf::from(&project_path);
            if project_path_buf.is_file()
                && let Some(parent) = project_path_buf.parent()
            {
                project_path_buf = parent.to_path_buf();
            }

            let mut cg = match CodeGraph::open(&project_path_buf) {
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

            let (result, _is_err) = handle_tool_call(&cg, tool_name, &args);

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

pub fn handle_tool_call(cg: &CodeGraph, tool_name: &str, args: &Value) -> (String, bool) {
    match tool_name {
        "explore" | "codegraph_explore" => (handle_explore(cg, args), false),
        "impact" | "codegraph_impact" => (handle_impact(cg, args), false),
        "symbol" | "codegraph_symbol" | "node" | "codegraph_node" => (handle_node(cg, args), false),
        "callers" | "codegraph_callers" => (handle_callers(cg, args), false),
        "callees" | "codegraph_callees" => (handle_callees(cg, args), false),
        "search" | "codegraph_search" => (handle_search(cg, args), false),
        "implementations" | "impls" | "codegraph_implementations" => {
            (handle_implementations(cg, args), false)
        }
        "cycles" | "codegraph_cycles" => (handle_cycles(cg), false),
        "status" | "codegraph_status" => (handle_status(cg), false),
        "files" | "codegraph_files" => (handle_files(cg, args), false),
        _ => (format!("Unknown tool: {}", tool_name), true),
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
    for node in &matched_nodes {
        file_groups.entry(node.file_path.clone()).or_default().push(*node);
    }

    let mut sorted_files: Vec<(String, Vec<&super::kernel::SymbolNode>)> = file_groups.into_iter().collect();
    sorted_files.sort_by_key(|a| std::cmp::Reverse(a.1.len()));
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

    // Blast radius impact summary & type implementors for focal symbol
    if let Some(primary) = matched_nodes.first() {
        if let Some(report) = cg.engine.compute_blast_radius(&primary.id, 2) {
            let trunc_note = if report.truncated { " *(truncated)*" } else { "" };
            out.push_str(&format!(
                "### Blast Radius: `{}` (risk: {:.1}/100){}\n\
                 - Upstream Callers: {}\n\
                 - Downstream Dependencies: {}\n\
                 - Affected Files ({}): {}\n\n",
                primary.name,
                report.risk_score,
                trunc_note,
                report.upstream_dependents.len(),
                report.downstream_dependencies.len(),
                report.affected_files.len(),
                report.affected_files.join(", ")
            ));
        }

        let impls = cg.engine.find_implementations(&primary.id, 2);
        if !impls.is_empty() {
            out.push_str(&format!("### Implementations & Subtypes ({})\n", impls.len()));
            for imp in impls.iter().take(10) {
                let test_tag = if imp.is_test { " [test]" } else { "" };
                out.push_str(&format!(
                    "- `{}` ({}, {}) in `{}:{}`{}\n",
                    imp.node.qualified_name,
                    imp.node.kind,
                    imp.relation,
                    imp.node.file_path,
                    imp.node.start_line,
                    test_tag
                ));
            }
            if impls.len() > 10 {
                out.push_str(&format!("- ... and {} more\n", impls.len() - 10));
            }
            out.push('\n');
        }
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
    let trunc_note = if report.truncated {
        " | *(truncated at 400 impacted symbols)*"
    } else {
        ""
    };
    out.push_str(&format!(
        "Risk Score: **{:.1}/100** | Traversal Depth: **{}**{}\n\n",
        report.risk_score, report.depth, trunc_note
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
            let rec_tag = if caller.id == focal.id { " (recursive)" } else { "" };
            out.push_str(&format!(
                "- `{}` ({}) in `{}:{}` ({}){}\n",
                caller.name,
                caller.kind,
                caller.file_path,
                edge.line.unwrap_or(caller.start_line),
                edge.kind,
                rec_tag
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

    let impls = cg.engine.find_implementations(&node.id, 2);
    if !impls.is_empty() {
        out.push_str(&format!("\n### Implementations / Subtypes ({})\n", impls.len()));
        for imp in impls {
            let test_tag = if imp.is_test { " [test]" } else { "" };
            out.push_str(&format!(
                "- [depth {}] `{}` ({}, {}) in `{}:{}`{}\n",
                imp.depth,
                imp.node.qualified_name,
                imp.node.kind,
                imp.relation,
                imp.node.file_path,
                imp.node.start_line,
                test_tag
            ));
        }
    }

    out
}

pub(crate) fn handle_implementations(cg: &CodeGraph, args: &Value) -> String {
    let symbol = match args.get("symbol").and_then(|s| s.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return "Missing required 'symbol' argument.".into(),
    };
    let depth = args
        .get("depth")
        .and_then(|d| d.as_u64())
        .map(|d| d.clamp(1, 6) as usize)
        .unwrap_or(2);

    let impls = cg.engine.find_implementations(symbol, depth);
    if impls.is_empty() {
        return format!("No implementations or subtypes found for '{}'.", symbol);
    }

    let mut out = format!("# Implementations of `{}` ({})\n\n", symbol, impls.len());
    for imp in impls {
        let test_tag = if imp.is_test { " [test]" } else { "" };
        out.push_str(&format!(
            "- [depth {}] `{}` ({}, {}) in `{}:{}`{}\n",
            imp.depth,
            imp.node.qualified_name,
            imp.node.kind,
            imp.relation,
            imp.node.file_path,
            imp.node.start_line,
            test_tag
        ));
    }
    out
}

pub(crate) fn handle_cycles(cg: &CodeGraph) -> String {
    let cycles = cg.engine.find_file_cycles();
    if cycles.is_empty() {
        return "No circular file dependencies found.".to_string();
    }

    let mut out = format!("# Circular File Dependencies ({} cycles)\n\n", cycles.len());
    for (idx, group) in cycles.iter().enumerate() {
        out.push_str(&format!(
            "{}. Cycle ({} files): {}\n",
            idx + 1,
            group.len(),
            group.join(" <-> ")
        ));
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

pub(crate) fn handle_files(cg: &CodeGraph, args: &Value) -> String {
    let format_mode = args
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("list");
    let tests_opt = args.get("tests").and_then(|v| v.as_bool());
    let no_tests = args.get("noTests").and_then(|v| v.as_bool()).unwrap_or(false);

    let files: Vec<(String, String, usize)> = cg
        .cache
        .get_indexed_files()
        .into_iter()
        .filter(|(file, _, _)| {
            let is_t = walker::is_test_file(file);
            if no_tests && is_t {
                return false;
            }
            match tests_opt {
                Some(true) => is_t,
                Some(false) => !is_t,
                None => true,
            }
        })
        .collect();

    if format_mode.eq_ignore_ascii_case("package") {
        let mut dir_cache: HashMap<String, String> = HashMap::new();
        let mut groups: BTreeMap<String, Vec<(String, String, usize)>> = BTreeMap::new();
        for item in &files {
            let pkg = resolve_owning_package(&cg.root, &item.0, &mut dir_cache);
            groups.entry(pkg).or_default().push(item.clone());
        }
        let pkg_count = groups.keys().filter(|k| k.as_str() != "no owning package").count();
        let mut out = format!(
            "# Files by Package ({} files, {} packages)\n\n",
            files.len(),
            pkg_count
        );
        for (pkg, pkg_files) in groups {
            out.push_str(&format!("## {} ({} files)\n", pkg, pkg_files.len()));
            for (file, lang, sym_count) in pkg_files {
                let test_tag = if walker::is_test_file(&file) { " [test]" } else { "" };
                out.push_str(&format!("- `{}` [{}]{} — {} symbols\n", file, lang, test_tag, sym_count));
            }
            out.push('\n');
        }
        return out;
    }

    let mut out = format!("# Indexed Files ({})\n\n", files.len());
    for (file, lang, sym_count) in files {
        let test_tag = if walker::is_test_file(&file) { " [test]" } else { "" };
        out.push_str(&format!("- `{}` [{}]{} — {} symbols\n", file, lang, test_tag, sym_count));
    }
    out
}

fn resolve_owning_package(root: &Path, rel_file: &str, cache: &mut HashMap<String, String>) -> String {
    let dir_rel = Path::new(rel_file)
        .parent()
        .map(|p| crate::util::paths::normalize_path(&p.to_string_lossy()))
        .unwrap_or_default();
    if let Some(cached) = cache.get(&dir_rel) {
        return cached.clone();
    }

    let mut cur = if dir_rel.is_empty() {
        root.to_path_buf()
    } else {
        root.join(&dir_rel)
    };

    let mut found: Option<String> = None;
    loop {
        if let Some(name) = read_package_manifest_name(&cur) {
            found = Some(name);
            break;
        }
        if cur == root {
            break;
        }
        if !cur.pop() || !cur.starts_with(root) {
            break;
        }
    }

    let resolved = found.unwrap_or_else(|| {
        let sub = super::cache::derive_subproject_id(rel_file);
        if sub == "." {
            "no owning package".to_string()
        } else {
            sub
        }
    });
    cache.insert(dir_rel, resolved.clone());
    resolved
}

fn read_package_manifest_name(dir: &Path) -> Option<String> {
    for manifest in ["package.json", "composer.json"] {
        let p = dir.join(manifest);
        if p.is_file()
            && let Ok(raw) = fs::read_to_string(&p)
            && let Ok(val) = serde_json::from_str::<Value>(raw.trim_start_matches('\u{feff}'))
            && let Some(name) = val.get("name").and_then(|v| v.as_str())
        {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.is_file()
        && let Ok(raw) = fs::read_to_string(&cargo_toml)
        && let Ok(val) = toml::from_str::<toml::Value>(&raw)
        && let Some(name) = val
            .get("package")
            .and_then(|pkg| pkg.get("name"))
            .and_then(|v| v.as_str())
    {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}
