// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase3_contextmode_verification.rs

use std::fs;
use std::path::Path;
use serde_json::json;
use tempfile::tempdir;
use tokless::contextmode::*;

#[test]
fn test_phase3_sandbox_execution_and_security() {
    let sandbox = PolyglotSandbox::new();

    // 1. Basic execution
    let cmd = if cfg!(windows) {
        "Write-Output 'ctx-sandbox-live-test'"
    } else {
        "echo 'ctx-sandbox-live-test'"
    };
    let req = ExecRequest {
        language: "shell".to_string(),
        code: cmd.to_string(),
        cwd: None,
        timeout_secs: Some(10),
        max_lines: Some(50),
        max_bytes: None,
    };
    let res = sandbox.execute(&req).expect("sandbox execute");
    assert_eq!(res.exit_code, 0);
    assert!(res.stdout.contains("ctx-sandbox-live-test"));
    assert!(!res.timed_out);

    // 2. Dangerous command denial
    let dangerous_req = ExecRequest {
        language: "shell".to_string(),
        code: "rm -rf / --no-preserve-root".to_string(),
        cwd: None,
        timeout_secs: Some(5),
        max_lines: None,
        max_bytes: None,
    };
    let res_denied = sandbox.execute(&dangerous_req).expect("sandbox execute denied");
    assert_eq!(res_denied.exit_code, 126);
    assert!(res_denied.stderr.contains("Command blocked by sandbox security policy"));

    // 3. Sensitive path denial in execute_file
    let tmp = tempdir().unwrap();
    let env_file = tmp.path().join(".env");
    fs::write(&env_file, "DATABASE_URL=postgres://user:pass@localhost/db").unwrap();

    let res_path_denied = sandbox
        .execute_file(&env_file, "print(FILE_CONTENT)", Some("python"), None, Some(5))
        .expect("execute_file denied");
    assert_eq!(res_path_denied.exit_code, 126);
    assert!(res_path_denied.stderr.contains("Access to sensitive file blocked by security policy"));
}

#[test]
fn test_phase3_filter_rtk_pipeline_and_slicing() {
    // 1. Slicing with line budget
    let lines: Vec<String> = (1..=60).map(|i| format!("log line {:02}", i)).collect();
    let text = lines.join("\n");
    let sliced = slice_lines(&text, 10);
    assert!(sliced.contains("log line 01"));
    assert!(sliced.contains("log line 05"));
    assert!(sliced.contains("... [50 lines omitted by context-mode] ..."));
    assert!(sliced.contains("log line 60"));

    // 2. Byte-safe UTF-8 slicing
    let multi = "Rust🦀Programming⚡Rocks";
    let prefix = filter::byte_safe_prefix(multi, 8); // 'Rust' (4) + '🦀' (4) = 8 bytes
    assert_eq!(prefix, "Rust🦀");

    // 3. RTK sandbox output processing: filter git status output through RTK
    let git_raw = "## main...origin/main\nM  crates/tokless/src/main.rs\n?? new_file.txt\n";
    let filtered = process_sandbox_output(Some("git status"), git_raw, 20, 1024);
    assert!(filtered.contains("staged (1):") || filtered.contains("main.rs"));

    // 4. ANSI stripping in output filter
    let ansi_raw = "\x1b[32mPASS\x1b[0m \x1b[31mFAIL\x1b[0m test_example";
    let filtered_ansi = process_sandbox_output(None, ansi_raw, 20, 1024);
    assert!(!filtered_ansi.contains("\x1b[32m"));
    assert!(filtered_ansi.contains("PASS FAIL test_example"));
}

#[test]
fn test_phase3_fts5_chunking_and_bm25_search() {
    let mut store = ContentStore::open_in_memory().expect("open in-memory FTS5 store");

    // 1. Markdown document chunking and indexing
    let md_doc = "# Tokless Engine Architecture\n\nTokless embeds RTK, ProjectMem, and ContextMode into a single Rust binary.\n\n## Subsystem Benchmarks\n\n```json\n{\"reduction\": \"98%\", \"overhead_ms\": 2}\n```\n\nDetailed benchmark notes and explanations.";
    let res = store
        .index(md_doc, "tokless-arch", Some("markdown"), Some("sess-1"))
        .expect("index markdown");
    assert_eq!(res.label, "tokless-arch");
    assert_eq!(res.total_chunks, 2);
    assert_eq!(res.code_chunks, 1);

    // 2. Plain text chunking and indexing
    let text_doc = "Config Section 1\nWorker threads: 8\n\nConfig Section 2\nMemory limit: 512MB";
    store
        .index(text_doc, "tokless-cfg", Some("prose"), Some("sess-1"))
        .expect("index prose");

    // 3. JSON chunking and indexing
    let json_doc = json!({
        "server": {
            "host": "127.0.0.1",
            "port": 9099
        },
        "limits": {
            "max_connections": 1000
        }
    });
    store
        .index(&json_doc.to_string(), "tokless-json", Some("json"), Some("sess-1"))
        .expect("index json");

    // 4. BM25 search
    let search_res = store
        .search(&SearchOptions {
            query: "subsystem benchmarks reduction".to_string(),
            limit: 5,
            source: None,
            content_type: None,
            session_id: None,
        })
        .expect("search");
    assert!(!search_res.is_empty());
    assert_eq!(search_res[0].label, "tokless-arch");
    assert!(search_res[0].content.contains("98%"));

    // 5. Purge all
    let purged = store.purge_all().expect("purge all");
    assert_eq!(purged, 3);
    let search_after = store
        .search(&SearchOptions {
            query: "benchmarks".to_string(),
            limit: 5,
            source: None,
            content_type: None,
            session_id: None,
        })
        .expect("search after purge");
    assert!(search_after.is_empty());
}

#[test]
fn test_phase3_directory_indexing_and_traversal() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let docs_dir = root.join("docs");
    let src_dir = root.join("src");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(root.join("README.md"), "# Project Root\n\nOverview of the project.").unwrap();
    fs::write(docs_dir.join("api.md"), "# API Reference\n\nEndpoints documentation.").unwrap();
    fs::write(src_dir.join("lib.rs"), "// Core Rust library\npub fn run() {}\n").unwrap();

    let mut store = ContentStore::open_in_memory().expect("open in-memory store");
    let dir_res = store.index_directory(root, 100, 5, Some("my-workspace")).expect("index directory");

    assert_eq!(dir_res.files_indexed, 3);
    let expected_root = root.canonicalize().unwrap().to_string_lossy().to_string();
    assert_eq!(dir_res.root, expected_root);
    assert!(!dir_res.capped);

    let search_res = store
        .search(&SearchOptions {
            query: "Endpoints documentation".to_string(),
            limit: 5,
            source: Some("my-workspace".to_string()),
            content_type: None,
            session_id: None,
        })
        .expect("search directory docs");
    assert!(!search_res.is_empty());
    assert!(search_res[0].content.contains("Endpoints documentation"));
}

#[test]
fn test_phase3_fetch_markdown_and_ssrf() {
    // 1. SSRF validations
    assert!(ssrf_guard("https://example.com/api/v1/specs").is_ok());
    assert!(ssrf_guard("http://doc.rust-lang.org/cargo").is_ok());

    assert!(ssrf_guard("http://127.0.0.1:8080").is_err());
    assert!(ssrf_guard("http://localhost:3000/admin").is_err());
    assert!(ssrf_guard("http://169.254.169.254/latest/meta-data/").is_err());
    assert!(ssrf_guard("http://10.20.30.40/secret").is_err());
    assert!(ssrf_guard("http://192.168.1.1/router").is_err());
    assert!(ssrf_guard("ftp://example.com/file").is_err());

    // 2. HTML to Markdown conversion
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Documentation</title>
            <style>body { background: #000; }</style>
            <script>console.log("tracker");</script>
        </head>
        <body>
            <h1>Tokless Manual</h1>
            <p>Welcome to the <strong>Tokless</strong> guide. See <a href="https://github.com/Navegos/neo">GitHub</a>.</p>
            <pre><code>cargo build --release</code></pre>
            <ul>
                <li>Zero-allocation ANSI stripping</li>
                <li>FTS5 BM25 search</li>
            </ul>
        </body>
        </html>
    "#;

    let md = html_to_markdown(html);
    assert!(!md.contains("<style>"));
    assert!(!md.contains("console.log"));
    assert!(md.contains("# Tokless Manual"));
    assert!(md.contains("Tokless guide"));
    assert!(md.contains("[GitHub](https://github.com/Navegos/neo)"));
    assert!(md.contains("cargo build --release"));
    assert!(md.contains("* Zero-allocation ANSI stripping"));
}

#[test]
fn test_phase3_inprocess_mcp_server_protocol() {
    let tmp = tempdir().unwrap();
    let server = ContextMcpServer::new(Some(tmp.path().to_path_buf()));

    // 1. MCP initialize handshake
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let init_res = server.handle_mcp_message(&init_req).expect("handle initialize");
    assert_eq!(init_res["result"]["serverInfo"]["name"], "context-mode");
    assert_eq!(init_res["result"]["protocolVersion"], "2024-11-05");

    // 2. MCP tools/list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let list_res = server.handle_mcp_message(&list_req).expect("handle tools/list");
    let tools = list_res["result"]["tools"].as_array().expect("tools array");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"ctx_execute"));
    assert!(tool_names.contains(&"ctx_execute_file"));
    assert!(tool_names.contains(&"ctx_batch_execute"));
    assert!(tool_names.contains(&"ctx_index"));
    assert!(tool_names.contains(&"ctx_search"));
    assert!(tool_names.contains(&"ctx_fetch_and_index"));
    assert!(tool_names.contains(&"ctx_stats"));
    assert!(tool_names.contains(&"ctx_doctor"));
    assert!(tool_names.contains(&"ctx_purge"));

    // 3. MCP tool call: ctx_index
    let index_res = server.execute_tool(
        "ctx_index",
        &json!({
            "content": "# Test Module\n\nTesting in-process context-mode MCP server execution.",
            "source": "mcp-test-src"
        }),
    );
    assert!(index_res.contains("Indexed 'mcp-test-src'"));

    // 4. MCP tool call: ctx_search
    let search_res = server.execute_tool(
        "ctx_search",
        &json!({
            "query": "server execution"
        }),
    );
    assert!(search_res.contains("Results for query 'server execution'"));
    assert!(search_res.contains("Test Module"));

    // 5. MCP tool call: ctx_stats and ctx_doctor
    let stats_res = server.execute_tool("ctx_stats", &json!({}));
    assert!(stats_res.contains("Context-Mode Session Statistics"));

    let doctor_res = server.execute_tool("ctx_doctor", &json!({}));
    assert!(doctor_res.contains("Context-Mode Doctor Diagnostics"));
    assert!(doctor_res.contains("READY"));
}

#[test]
fn test_phase3_cli_interface_verification() {
    // 1. Doctor command
    assert_eq!(cli::run_ctx_cli(&["doctor".to_string()]), 0);

    // 2. Stats command
    assert_eq!(cli::run_ctx_cli(&["stats".to_string()]), 0);

    // 3. Help command
    assert_eq!(cli::run_ctx_cli(&["--help".to_string()]), 0);

    // 4. Eval command
    let eval_cmd = if cfg!(windows) {
        "Write-Output 'cli-eval-pass'"
    } else {
        "echo 'cli-eval-pass'"
    };
    assert_eq!(cli::run_ctx_cli(&["eval".to_string(), eval_cmd.to_string()]), 0);

    // 5. Index and search via CLI
    let tmp = tempdir().unwrap();
    let test_file = tmp.path().join("cli_test.md");
    fs::write(&test_file, "# CLI Indexed File\n\nCLI verification payload text.").unwrap();

    let index_code = cli::run_ctx_cli(&[
        "index".to_string(),
        test_file.to_string_lossy().to_string(),
        "cli-file-test".to_string(),
    ]);
    assert_eq!(index_code, 0);

    let search_code = cli::run_ctx_cli(&["search".to_string(), "verification payload".to_string()]);
    assert_eq!(search_code, 0);

    // 6. Purge via CLI
    let purge_code = cli::run_ctx_cli(&["purge".to_string()]);
    assert_eq!(purge_code, 0);
}
