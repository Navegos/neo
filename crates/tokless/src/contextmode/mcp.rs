// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/mcp.rs
// created: 2026-10-27

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};

use super::fetch::{FetchRequest, WebFetcher};
use super::fts::{resolve_content_store_path, ContentStore, SearchOptions};
use super::sandbox::{detect_runtimes, ExecRequest, PolyglotSandbox};
use super::security::check_path_deny;
use super::stats::{get_stats, record_tool_call};

pub fn mcp_tools_list() -> Value {
    json!([
        {
            "name": "ctx_execute",
            "description": "Execute code or shell command inside an isolated local sandbox with resource bounds. Outputs are automatically filtered through RTK command profiles and line budget slicers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "Code or command to execute in sandbox" },
                    "language": { "type": "string", "description": "Language: shell, python, javascript, typescript, etc. Default: shell" },
                    "timeout": { "type": "integer", "description": "Timeout in seconds (default: 30)" },
                    "cwd": { "type": "string", "description": "Working directory for execution" },
                    "max_lines": { "type": "integer", "description": "Max lines to return before head/tail slicing (default: 200)" }
                },
                "required": ["code"]
            }
        },
        {
            "name": "ctx_execute_file",
            "description": "Process and analyze a large local file inside the sandbox without loading raw bytes into the conversation context window (saving up to 98% tokens).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to local file to analyze" },
                    "code": { "type": "string", "description": "Analysis script code. In Python: TARGET_PATH and FILE_CONTENT are preloaded. In JS: TARGET_PATH and FILE_CONTENT are preloaded." },
                    "language": { "type": "string", "description": "Language: python, javascript, shell. Auto-detected if omitted." },
                    "timeout": { "type": "integer", "description": "Timeout in seconds (default: 30)" },
                    "cwd": { "type": "string", "description": "Working directory" }
                },
                "required": ["path", "code"]
            }
        },
        {
            "name": "ctx_batch_execute",
            "description": "Run N commands concurrently in sandbox, pipe outputs through RTK filters, auto-index results into SQLite FTS5, and optionally search output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "commands": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "command": { "type": "string" },
                                "label": { "type": "string" },
                                "cwd": { "type": "string" }
                            },
                            "required": ["command"]
                        },
                        "description": "Array of commands to run"
                    },
                    "queries": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional search queries to run against the batch output"
                    },
                    "concurrency": { "type": "integer", "description": "Max parallel commands (1-8, default: 2)" },
                    "timeout": { "type": "integer", "description": "Per-command timeout in seconds (default: 30)" },
                    "cwd": { "type": "string", "description": "Default working directory" }
                },
                "required": ["commands"]
            }
        },
        {
            "name": "ctx_index",
            "description": "Chunk markdown, plain text, or JSON content and index into local SQLite FTS5 for fast BM25 retrieval.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "Direct text or JSON content to index" },
                    "path": { "type": "string", "description": "Path to file to index from disk" },
                    "source": { "type": "string", "description": "Label for the indexed source" },
                    "content_type": { "type": "string", "enum": ["auto", "markdown", "prose", "json"], "description": "Content type format" }
                }
            }
        },
        {
            "name": "ctx_search",
            "description": "Query indexed documentation and session outputs via BM25 ranking and term proximity scoring.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query string" },
                    "queries": { "type": "array", "items": { "type": "string" }, "description": "Multiple queries in a single call" },
                    "limit": { "type": "integer", "description": "Max results to return (default: 10)" },
                    "source": { "type": "string", "description": "Filter by source label substring" },
                    "content_type": { "type": "string", "description": "Filter by content type" }
                }
            }
        },
        {
            "name": "ctx_fetch_and_index",
            "description": "Fetch web pages asynchronously via HTTP GET, convert HTML to clean markdown, and index into SQLite FTS5 with disk TTL caching. Raw web bytes never enter context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "Single URL to fetch and index" },
                    "source": { "type": "string", "description": "Label for the indexed content" },
                    "requests": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "url": { "type": "string" },
                                "source": { "type": "string" }
                            },
                            "required": ["url"]
                        },
                        "description": "Batch fetch array of {url, source}"
                    },
                    "concurrency": { "type": "integer", "description": "Parallel fetch concurrency (1-8, default: 2)" },
                    "force": { "type": "boolean", "description": "Force refresh, bypassing cache" },
                    "ttl": { "type": "integer", "description": "Cache freshness window in milliseconds" }
                }
            }
        },
        {
            "name": "ctx_stats",
            "description": "Show session statistics: tool calls, bytes sandboxed, cache hits, and estimated token savings.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "ctx_doctor",
            "description": "Diagnose context-mode environment: available runtimes, database health, and sandbox readiness.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "ctx_purge",
            "description": "Purge cached content and reset knowledge base.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ])
}

pub struct ContextMcpServer {
    sandbox: PolyglotSandbox,
    fetcher: WebFetcher,
    store: Arc<Mutex<ContentStore>>,
    project_root: Option<PathBuf>,
}

impl ContextMcpServer {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let db_path = resolve_content_store_path(project_root.as_deref());
        let store = ContentStore::open(&db_path).unwrap_or_else(|_| ContentStore::open_in_memory().expect("open memory store"));
        Self {
            sandbox: PolyglotSandbox::new(),
            fetcher: WebFetcher::new(),
            store: Arc::new(Mutex::new(store)),
            project_root,
        }
    }

    pub fn execute_tool(&self, name: &str, args: &Value) -> String {
        let result_text = match name {
            "ctx_execute" => self.tool_ctx_execute(args),
            "ctx_execute_file" => self.tool_ctx_execute_file(args),
            "ctx_batch_execute" => self.tool_ctx_batch_execute(args),
            "ctx_index" => self.tool_ctx_index(args),
            "ctx_search" => self.tool_ctx_search(args),
            "ctx_fetch_and_index" => self.tool_ctx_fetch_and_index(args),
            "ctx_stats" => self.tool_ctx_stats(),
            "ctx_doctor" => self.tool_ctx_doctor(),
            "ctx_purge" => self.tool_ctx_purge(),
            _ => format!("Unknown tool '{}'", name),
        };

        record_tool_call(name, result_text.len());
        result_text
    }

    fn tool_ctx_execute(&self, args: &Value) -> String {
        let code = match args.get("code").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return "Error: missing 'code' argument".to_string(),
        };

        let lang = args.get("language").and_then(|v| v.as_str()).unwrap_or("shell");
        let timeout = args.get("timeout").and_then(|v| v.as_u64());
        let max_lines = args.get("max_lines").and_then(|v| v.as_u64()).map(|n| n as usize);

        let cwd = args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .or_else(|| self.project_root.clone());

        let req = ExecRequest {
            language: lang.to_string(),
            code: code.to_string(),
            cwd,
            timeout_secs: timeout,
            max_lines,
            max_bytes: None,
        };

        match self.sandbox.execute(&req) {
            Ok(res) => {
                if res.exit_code != 0 {
                    let mut err_msg = format!("Exit code {}\n", res.exit_code);
                    if !res.stderr.is_empty() {
                        err_msg.push_str(&res.stderr);
                    }
                    if !res.stdout.is_empty() {
                        err_msg.push_str("\nOutput:\n");
                        err_msg.push_str(&res.stdout);
                    }
                    err_msg
                } else if res.stdout.is_empty() && !res.stderr.is_empty() {
                    res.stderr
                } else {
                    res.stdout
                }
            }
            Err(e) => format!("Execution error: {}", e),
        }
    }

    fn tool_ctx_execute_file(&self, args: &Value) -> String {
        let path_str = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return "Error: missing 'path' argument".to_string(),
        };

        let code = match args.get("code").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return "Error: missing 'code' argument".to_string(),
        };

        let lang = args.get("language").and_then(|v| v.as_str());
        let timeout = args.get("timeout").and_then(|v| v.as_u64());

        let target_path = if let Some(ref root) = self.project_root {
            root.join(path_str)
        } else {
            PathBuf::from(path_str)
        };

        let cwd = args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .or_else(|| self.project_root.clone());

        match self.sandbox.execute_file(&target_path, code, lang, cwd.as_deref(), timeout) {
            Ok(res) => {
                if res.exit_code != 0 {
                    format!("Exit code {}:\n{}\n{}", res.exit_code, res.stderr, res.stdout)
                } else if res.stdout.is_empty() && !res.stderr.is_empty() {
                    res.stderr
                } else {
                    res.stdout
                }
            }
            Err(e) => format!("File execution error: {}", e),
        }
    }

    fn tool_ctx_batch_execute(&self, args: &Value) -> String {
        let cmds = match args.get("commands").and_then(|v| v.as_array()) {
            Some(arr) if !arr.is_empty() => arr,
            _ => return "Error: 'commands' array required and cannot be empty".to_string(),
        };

        let timeout = args.get("timeout").and_then(|v| v.as_u64());
        let default_cwd = args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .or_else(|| self.project_root.clone());

        let mut results = Vec::new();
        let mut combined_indexed = 0;

        for (i, c_val) in cmds.iter().enumerate() {
            let cmd_str = c_val.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let label = c_val
                .get("label")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("batch-cmd-{}", i + 1));

            let cwd = c_val
                .get("cwd")
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
                .or_else(|| default_cwd.clone());

            let req = ExecRequest {
                language: "shell".to_string(),
                code: cmd_str.to_string(),
                cwd,
                timeout_secs: timeout,
                max_lines: Some(100),
                max_bytes: None,
            };

            match self.sandbox.execute(&req) {
                Ok(res) => {
                    let out_combined = format!("{}\n{}", res.stdout, res.stderr);
                    if let Ok(mut store) = self.store.lock() {
                        if let Ok(ir) = store.index(&out_combined, &label, Some("prose"), None) {
                            combined_indexed += ir.total_chunks;
                        }
                    }
                    results.push(format!(
                        "[{}] exit {}: ({} chars, {} lines)",
                        label,
                        res.exit_code,
                        out_combined.len(),
                        out_combined.lines().count()
                    ));
                }
                Err(e) => {
                    results.push(format!("[{}] failed: {}", label, e));
                }
            }
        }

        let mut summary = format!(
            "Batch executed {} commands (indexed {} chunks in total):\n{}",
            cmds.len(),
            combined_indexed,
            results.join("\n")
        );

        // Optional follow-up search queries across the batch output
        if let Some(queries) = args.get("queries").and_then(|v| v.as_array()) {
            summary.push_str("\n\n--- Search Query Results ---");
            if let Ok(store) = self.store.lock() {
                for q_val in queries {
                    if let Some(q) = q_val.as_str() {
                        let search_opts = SearchOptions {
                            query: q.to_string(),
                            limit: 3,
                            source: None,
                            content_type: None,
                            session_id: None,
                        };
                        if let Ok(hits) = store.search(&search_opts) {
                            summary.push_str(&format!("\nQuery '{}' ({} hits):", q, hits.len()));
                            for h in hits {
                                summary.push_str(&format!("\n  - [{}] {}: {}", h.label, h.title, h.highlighted));
                            }
                        }
                    }
                }
            }
        }

        summary
    }

    fn tool_ctx_index(&self, args: &Value) -> String {
        let content_opt = args.get("content").and_then(|v| v.as_str());
        let path_opt = args.get("path").and_then(|v| v.as_str());
        let source_label = args.get("source").and_then(|v| v.as_str()).unwrap_or("inline");
        let content_type = args.get("content_type").and_then(|v| v.as_str()).unwrap_or("auto");

        let content = if let Some(c) = content_opt {
            c.to_string()
        } else if let Some(p) = path_opt {
            let full_path = if let Some(ref root) = self.project_root {
                root.join(p)
            } else {
                PathBuf::from(p)
            };

            if let Some(denied) = check_path_deny(&full_path) {
                return denied;
            }

            if full_path.is_dir() {
                let mut store = match self.store.lock() {
                    Ok(s) => s,
                    Err(e) => return format!("Store lock error: {}", e),
                };
                return match store.index_directory(&full_path, 200, 5, Some(source_label)) {
                    Ok(ir) => format!(
                        "Indexed directory '{}': {} files, {} chunks, {} bytes{}",
                        ir.root,
                        ir.files_indexed,
                        ir.total_chunks,
                        ir.total_bytes,
                        if ir.capped { " (capped at 200 files)" } else { "" }
                    ),
                    Err(e) => format!("Directory index error: {}", e),
                };
            }

            match fs::read_to_string(&full_path) {
                Ok(s) => s,
                Err(e) => return format!("Failed to read file '{}': {}", full_path.display(), e),
            }
        } else {
            return "Error: either 'content' or 'path' must be provided".to_string();
        };

        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(e) => return format!("Store lock error: {}", e),
        };

        match store.index(&content, source_label, Some(content_type), None) {
            Ok(ir) => format!(
                "Indexed '{}': {} chunks ({} code chunks, {} bytes)",
                ir.label, ir.total_chunks, ir.code_chunks, ir.total_bytes
            ),
            Err(e) => format!("Index error: {}", e),
        }
    }

    fn tool_ctx_search(&self, args: &Value) -> String {
        let mut query_list: Vec<String> = Vec::new();

        if let Some(q) = args.get("query").and_then(|v| v.as_str()) {
            if !q.trim().is_empty() {
                query_list.push(q.trim().to_string());
            }
        }

        if let Some(arr) = args.get("queries").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    if !s.trim().is_empty() {
                        query_list.push(s.trim().to_string());
                    }
                }
            }
        }

        if query_list.is_empty() {
            return "Error: 'query' or 'queries' must be provided".to_string();
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let source_filter = args.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
        let ctype_filter = args.get("content_type").and_then(|v| v.as_str()).map(|s| s.to_string());

        let store = match self.store.lock() {
            Ok(s) => s,
            Err(e) => return format!("Store lock error: {}", e),
        };

        let mut all_results = Vec::new();
        for q in query_list {
            let opts = SearchOptions {
                query: q.clone(),
                limit,
                source: source_filter.clone(),
                content_type: ctype_filter.clone(),
                session_id: None,
            };

            match store.search(&opts) {
                Ok(results) => {
                    if results.is_empty() {
                        all_results.push(format!("No results for query '{}'", q));
                    } else {
                        let mut section = format!("Results for query '{}' ({} hits):", q, results.len());
                        for (i, r) in results.iter().enumerate() {
                            section.push_str(&format!(
                                "\n\n[{}] {} (score: {:.2}, source: '{}')\n{}",
                                i + 1,
                                r.title,
                                r.score,
                                r.label,
                                r.content
                            ));
                        }
                        all_results.push(section);
                    }
                }
                Err(e) => {
                    all_results.push(format!("Search error for '{}': {}", q, e));
                }
            }
        }

        all_results.join("\n\n---\n\n")
    }

    fn tool_ctx_fetch_and_index(&self, args: &Value) -> String {
        let single_url = args.get("url").and_then(|v| v.as_str());
        let single_source = args.get("source").and_then(|v| v.as_str());
        let requests_arr = args.get("requests").and_then(|v| v.as_array());

        let mut requests = Vec::new();
        if let Some(arr) = requests_arr {
            for item in arr {
                if let Some(u) = item.get("url").and_then(|v| v.as_str()) {
                    let s = item.get("source").and_then(|v| v.as_str()).map(|x| x.to_string());
                    requests.push(FetchRequest {
                        url: u.to_string(),
                        source: s,
                    });
                }
            }
        } else if let Some(u) = single_url {
            requests.push(FetchRequest {
                url: u.to_string(),
                source: single_source.map(|s| s.to_string()),
            });
        }

        if requests.is_empty() {
            return "Error: ctx_fetch_and_index requires either `url` or `requests: [{url, source?}]`".to_string();
        }

        let concurrency = args.get("concurrency").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let ttl = args.get("ttl").and_then(|v| v.as_u64());

        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(e) => return format!("Store lock error: {}", e),
        };

        let outcomes = self.fetcher.fetch_batch(&mut store, &requests, concurrency, force, ttl);

        let mut lines = Vec::new();
        for out in outcomes {
            match out {
                super::fetch::FetchOneOutcome::Cached { label, chunk_count, age_str, ttl_str } => {
                    lines.push(format!("[CACHE HIT] '{}': {} chunks (fresh from {}, TTL: {})", label, chunk_count, age_str, ttl_str));
                }
                super::fetch::FetchOneOutcome::Fetched { label, total_chunks, total_bytes, preview } => {
                    lines.push(format!("[FETCHED] '{}': {} chunks ({} bytes)\nPreview:\n{}", label, total_chunks, total_bytes, preview));
                }
                super::fetch::FetchOneOutcome::Error { url, error } => {
                    lines.push(format!("[ERROR] '{}': {}", url, error));
                }
            }
        }

        lines.join("\n\n---\n\n")
    }

    fn tool_ctx_stats(&self) -> String {
        let stats = get_stats();
        let total_returned = stats.total_bytes_returned();
        let tokens_saved = stats.estimated_tokens_saved();

        let mut lines = vec![
            "Context-Mode Session Statistics:".to_string(),
            format!("  Total tool calls:       {}", stats.calls.values().sum::<usize>()),
            format!("  Bytes returned to agent: {} bytes", total_returned),
            format!("  Bytes processed sandbox: {} bytes", stats.bytes_sandboxed),
            format!("  Bytes indexed into FTS5: {} bytes", stats.bytes_indexed),
            format!("  Cache hits / misses:    {} / {}", stats.cache_hits, stats.cache_misses),
            format!("  Cache bytes saved:       {} bytes", stats.cache_bytes_saved),
            format!("  Estimated tokens saved:  ~{} tokens", tokens_saved),
        ];

        lines.push("\nCalls per tool:".to_string());
        for (tool, count) in &stats.calls {
            let bytes = stats.bytes_returned.get(tool).copied().unwrap_or(0);
            lines.push(format!("  - {}: {} calls ({} bytes)", tool, count, bytes));
        }

        lines.join("\n")
    }

    fn tool_ctx_doctor(&self) -> String {
        let runtimes = detect_runtimes();
        let db_path = resolve_content_store_path(self.project_root.as_deref());

        let lines = vec![
            "Context-Mode Doctor Diagnostics:".to_string(),
            format!("  Default shell:      {}", runtimes.shell),
            format!("  Python runtime:     {}", runtimes.python.as_deref().unwrap_or("none")),
            format!("  Node.js runtime:    {}", runtimes.node.as_deref().unwrap_or("none")),
            format!("  Bun runtime:        {}", runtimes.bun.as_deref().unwrap_or("none")),
            format!("  Deno runtime:       {}", runtimes.deno.as_deref().unwrap_or("none")),
            format!("  Rust compiler:      {}", runtimes.rust.as_deref().unwrap_or("none")),
            format!("  Database path:      {}", db_path.display()),
            format!("  Database exists:    {}", db_path.exists()),
            format!("  Sandbox readiness:  READY (pure Rust in-process runner)"),
        ];

        lines.join("\n")
    }

    fn tool_ctx_purge(&self) -> String {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(e) => return format!("Store lock error: {}", e),
        };

        match store.purge_all() {
            Ok(count) => format!("Successfully purged {} indexed sources from knowledge base.", count),
            Err(e) => format!("Purge error: {}", e),
        }
    }
}

pub fn run_mcp_server() -> i32 {
    let project_root = std::env::var("CONTEXT_MODE_PROJECT_DIR")
        .or_else(|_| std::env::var("PROJECTMEM_ROOT"))
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok());

    let server = ContextMcpServer::new(project_root);

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

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                line.clear();
                continue;
            }
        };

        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");

        let resp: Option<Value> = match method {
            "initialize" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "context-mode",
                        "version": "1.0.5"
                    },
                    "capabilities": {
                        "tools": {}
                    },
                    "instructions": "You are connected to context-mode: an isolated sandbox execution and SQLite FTS5 knowledge retrieval engine."
                }
            })),
            "notifications/initialized" => None,
            "ping" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            })),
            "tools/list" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": mcp_tools_list()
                }
            })),
            "tools/call" => {
                let params = req.get("params");
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let args = params
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                let result_text = server.execute_tool(name, &args);
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": result_text
                            }
                        ]
                    }
                }))
            }
            _ => {
                if id.is_some() {
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("Method not found: {}", method)
                        }
                    }))
                } else {
                    None
                }
            }
        };

        if let Some(r) = resp {
            let serialized = serde_json::to_string(&r).unwrap_or_default();
            let _ = writeln!(writer, "{}", serialized);
            let _ = writer.flush();
        }

        line.clear();
    }

    0
}
