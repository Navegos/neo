// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/cli.rs
// created: 2026-10-27

use std::path::PathBuf;
use super::fts::{resolve_content_store_path, ContentStore, SearchOptions};
use super::mcp::run_mcp_server;
use super::sandbox::{ExecRequest, PolyglotSandbox};

pub fn run_ctx_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        println!("{}", ctx_help());
        return 0;
    }

    match args[0].as_str() {
        "mcp" => run_mcp_server(),
        "stats" => {
            let stats = super::stats::get_stats();
            println!("Context-Mode Statistics:");
            println!("  Tool calls:         {}", stats.calls.values().sum::<usize>());
            println!("  Bytes sandboxed:    {}", stats.bytes_sandboxed);
            println!("  Bytes indexed:      {}", stats.bytes_indexed);
            println!("  Bytes returned:     {}", stats.total_bytes_returned());
            println!("  Cache hits/misses:  {}/{}", stats.cache_hits, stats.cache_misses);
            println!("  Tokens saved:       ~{}", stats.estimated_tokens_saved());
            0
        }
        "doctor" => {
            let runtimes = super::sandbox::detect_runtimes();
            let db_path = resolve_content_store_path(None);
            println!("Context-Mode Doctor Diagnostics:");
            println!("  Default shell:      {}", runtimes.shell);
            println!("  Python runtime:     {}", runtimes.python.as_deref().unwrap_or("none"));
            println!("  Node.js runtime:    {}", runtimes.node.as_deref().unwrap_or("none"));
            println!("  Bun runtime:        {}", runtimes.bun.as_deref().unwrap_or("none"));
            println!("  Deno runtime:       {}", runtimes.deno.as_deref().unwrap_or("none"));
            println!("  Rust compiler:      {}", runtimes.rust.as_deref().unwrap_or("none"));
            println!("  Content DB path:    {}", db_path.display());
            println!("  Content DB exists:  {}", db_path.exists());
            println!("  Status:             READY (pure Rust in-process runner)");
            0
        }
        "search" => {
            if args.len() < 2 {
                eprintln!("Usage: tokless ctx search <query>");
                return 1;
            }
            let query = args[1..].join(" ");
            let db_path = resolve_content_store_path(None);
            let store = match ContentStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open content store: {}", e);
                    return 1;
                }
            };
            let opts = SearchOptions {
                query,
                limit: 5,
                source: None,
                content_type: None,
                session_id: None,
            };
            match store.search(&opts) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No matching chunks found.");
                    } else {
                        println!("Found {} results:", results.len());
                        for (i, r) in results.iter().enumerate() {
                            println!("\n[{}] {} (source: '{}', score: {:.2})", i + 1, r.title, r.label, r.score);
                            println!("{}", r.content);
                        }
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Search error: {}", e);
                    1
                }
            }
        }
        "index" => {
            if args.len() < 2 {
                eprintln!("Usage: tokless ctx index <file-path> [label]");
                return 1;
            }
            let path = PathBuf::from(&args[1]);
            let label = args.get(2).cloned().unwrap_or_else(|| args[1].clone());

            let content = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read file '{}': {}", path.display(), e);
                    return 1;
                }
            };

            let db_path = resolve_content_store_path(None);
            let mut store = match ContentStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open content store: {}", e);
                    return 1;
                }
            };

            match store.index(&content, &label, None, None) {
                Ok(ir) => {
                    println!("Successfully indexed '{}': {} chunks ({} bytes)", ir.label, ir.total_chunks, ir.total_bytes);
                    0
                }
                Err(e) => {
                    eprintln!("Indexing error: {}", e);
                    1
                }
            }
        }
        "exec" => {
            if args.len() < 2 {
                eprintln!("Usage: tokless ctx exec <command>");
                return 1;
            }
            let cmd = args[1..].join(" ");
            let sandbox = PolyglotSandbox::new();
            let req = ExecRequest {
                language: "shell".to_string(),
                code: cmd,
                cwd: None,
                timeout_secs: Some(30),
                max_lines: Some(200),
                max_bytes: None,
            };
            match sandbox.execute(&req) {
                Ok(res) => {
                    if !res.stdout.is_empty() {
                        print!("{}", res.stdout);
                    }
                    if !res.stderr.is_empty() {
                        eprint!("{}", res.stderr);
                    }
                    res.exit_code
                }
                Err(e) => {
                    eprintln!("Sandbox error: {}", e);
                    1
                }
            }
        }
        "purge" => {
            let db_path = resolve_content_store_path(None);
            let mut store = match ContentStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open content store: {}", e);
                    return 1;
                }
            };
            match store.purge_all() {
                Ok(count) => {
                    println!("Purged {} sources from content store.", count);
                    0
                }
                Err(e) => {
                    eprintln!("Purge error: {}", e);
                    1
                }
            }
        }
        "help" | "--help" | "-h" => {
            println!("{}", ctx_help());
            0
        }
        unknown => {
            eprintln!("Unknown subcommand: {}", unknown);
            println!("{}", ctx_help());
            1
        }
    }
}

fn ctx_help() -> String {
    format!(
        "tokless ctx — isolated sandbox execution and SQLite FTS5 knowledge base\n\n\
        Usage:\n\
          tokless ctx mcp             Run in-process JSON-RPC 2.0 MCP server (ctx_* tools)\n\
          tokless ctx stats           Display session token savings and cache statistics\n\
          tokless ctx doctor          Check available runtimes and database readiness\n\
          tokless ctx search <query>  Search indexed chunks using BM25 ranking\n\
          tokless ctx index <file>    Chunk and store local file into FTS5 index\n\
          tokless ctx exec <cmd>      Run shell command in sandbox with RTK filter\n\
          tokless ctx purge           Purge cached knowledge base chunks\n"
    )
}

