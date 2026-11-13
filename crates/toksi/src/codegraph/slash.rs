// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/slash.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use serde_json::json;
use crate::codegraph::CodeGraph;

pub fn handle_slash_command(input: &str, root: Option<&Path>) -> String {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() || parts[0] != "/codegraph" {
        return "Unknown command. Use `/codegraph help` for available options.".to_string();
    }

    let sub = parts.get(1).copied().unwrap_or("help");

    let current_dir = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut cg = match CodeGraph::open(&current_dir) {
        Ok(c) => c,
        Err(e) => return format!("Error opening codegraph: {e}"),
    };

    match sub {
        "help" => {
            "**CodeGraph Slash Commands:**\n\
             - `/codegraph status` - Show index statistics and database location\n\
             - `/codegraph explore <query>` - Semantic AST code and call-path explorer\n\
             - `/codegraph impact <symbol>` - Blast-radius impact analysis of modifying a symbol\n\
             - `/codegraph symbols [query]` - Search AST symbols (functions, classes, structs)\n\
             - `/codegraph callers <symbol>` - Find callers of a symbol\n\
             - `/codegraph callees <symbol>` - Find callees called by a symbol\n\
             - `/codegraph files` - List indexed project files\n\
             - `/codegraph index` - Trigger incremental re-index of project\n\
             - `/codegraph help` - Show this help message"
                .to_string()
        }
        "status" => {
            let _ = cg.ensure_indexed();
            let stats = cg.cache.get_stats();
            format!(
                "**CodeGraph Status:**\n\
                 - Project Root: `{}`\n\
                 - Database: `{}`\n\
                 - Files: {}\n\
                 - Symbols: {}\n\
                 - Edges: {}\n\
                 - References: {}",
                cg.root.display(),
                cg.cache.db_path.display(),
                stats.file_count,
                stats.node_count,
                stats.edge_count,
                stats.ref_count
            )
        }
        "index" => {
            let _ = cg.ensure_indexed();
            let stats = cg.cache.get_stats();
            format!(
                "**CodeGraph Indexed:** {} files, {} symbols, {} edges",
                stats.file_count, stats.node_count, stats.edge_count
            )
        }
        "explore" => {
            let query = parts.get(2..).map(|p| p.join(" ")).unwrap_or_default();
            if query.is_empty() {
                return "Usage: `/codegraph explore <query>`".to_string();
            }
            let _ = cg.ensure_indexed();
            super::mcp::handle_explore(&cg, &json!({ "query": query, "maxFiles": 5 }))
        }
        "impact" => {
            let symbol = parts.get(2).copied().unwrap_or("");
            if symbol.is_empty() {
                return "Usage: `/codegraph impact <symbol>`".to_string();
            }
            let _ = cg.ensure_indexed();
            super::mcp::handle_impact(&cg, &json!({ "symbol": symbol, "depth": 2 }))
        }
        "symbols" | "search" => {
            let query = parts.get(2).copied().unwrap_or("");
            let _ = cg.ensure_indexed();
            super::mcp::handle_search(&cg, &json!({ "query": query, "limit": 20 }))
        }
        "callers" => {
            let symbol = parts.get(2).copied().unwrap_or("");
            if symbol.is_empty() {
                return "Usage: `/codegraph callers <symbol>`".to_string();
            }
            let _ = cg.ensure_indexed();
            super::mcp::handle_callers(&cg, &json!({ "symbol": symbol }))
        }
        "callees" => {
            let symbol = parts.get(2).copied().unwrap_or("");
            if symbol.is_empty() {
                return "Usage: `/codegraph callees <symbol>`".to_string();
            }
            let _ = cg.ensure_indexed();
            super::mcp::handle_callees(&cg, &json!({ "symbol": symbol }))
        }
        "files" => {
            let _ = cg.ensure_indexed();
            let files = cg.cache.get_indexed_files();
            let mut out = format!("**Indexed Files ({}):**\n", files.len());
            for (file, lang, count) in files.iter().take(50) {
                out.push_str(&format!("- `{file}` [{lang}] ({count} symbols)\n"));
            }
            if files.len() > 50 {
                out.push_str(&format!("... and {} more files", files.len() - 50));
            }
            out
        }
        _ => format!("Unknown codegraph command `{sub}`. Use `/codegraph help`."),
    }
}
