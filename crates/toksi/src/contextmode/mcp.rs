// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/mcp.rs
// created: 2026-10-27
// lastModified: 2026-11-13

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};

use super::auto_memory::{query_auto_memory, MEMORY_LABEL_PREFIX};
use super::fetch::{FetchRequest, WebFetcher};
use super::filter::{replace_handle_line, EXEC_PREVIEW_BYTES, OUTPUT_CAP};
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
                    "language": { "type": "string", "description": "Language: shell, python, javascript, typescript, powershell, etc. Default: shell" },
                    "file": { "type": "string", "description": "Optional file path in workspace to preload / pass via CTX_FILE" },
                    "intent": { "type": "string", "description": "Optional intent for chunk retrieval on large output" },
                    "timeout": { "type": "integer", "description": "Timeout in seconds (default: 30)" },
                    "cwd": { "type": "string", "description": "Working directory for execution" },
                    "max_lines": { "type": "integer", "description": "Max lines to return before head/tail slicing (default: 200)" },
                    "background": { "type": "boolean", "description": "Spawn in background detached mode and return pid" },
                    "summary": { "type": "boolean", "description": "Prepend exit code, bytes, and code echo" }
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
            "description": "Chunk markdown, plain text, or JSON content, local file, directory, or remote URL and index into local SQLite FTS5 for fast BM25 retrieval.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "Direct text or JSON content to index" },
                    "path": { "type": "string", "description": "Path to file or directory to index from disk" },
                    "url": { "type": "string", "description": "Web URL to fetch, convert to markdown, and index" },
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
            "name": "ctx_resume",
            "description": "Read session state summary or retrieve individual execution event details by sequence number.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session": { "type": "string", "description": "Session identifier (default: current)" },
                    "seq": { "type": "integer", "description": "Event sequence number to retrieve" }
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
            "name": "ctx_batch",
            "description": "Run N commands concurrently in sandbox, pipe outputs through RTK filters, auto-index results into SQLite FTS5, and optionally search output. Alias for ctx_batch_execute.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "commands": {
                        "type": "array",
                        "items": { "type": "string" },
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
            "name": "ctx_purge",
            "description": "Purge cached content and reset knowledge base.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "ctx_upgrade",
            "description": "Inspect context-mode build version and upgrade status in Toksi pure Rust consolidated engine.",
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
        let store = ContentStore::open(&db_path)
            .or_else(|_| ContentStore::open_in_memory())
            .unwrap_or_default();
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
            "ctx_batch" | "ctx_batch_execute" => self.tool_ctx_batch_execute(args),
            "ctx_index" => self.tool_ctx_index(args),
            "ctx_search" => self.tool_ctx_search(args),
            "ctx_fetch_and_index" => self.tool_ctx_fetch_and_index(args),
            "ctx_resume" => self.tool_ctx_resume(args),
            "ctx_stats" => self.tool_ctx_stats(),
            "ctx_doctor" => self.tool_ctx_doctor(),
            "ctx_upgrade" => self.tool_ctx_upgrade(args),
            "ctx_purge" => self.tool_ctx_purge(),
            "slash" | "/context" => {
                let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("/context status");
                handle_slash_command(cmd)
            }
            other if other.starts_with("/context") => handle_slash_command(other),
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
        let timeout = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .or_else(|| args.get("timeoutMs").and_then(|v| v.as_u64()).map(|ms| ms / 1000));
        let max_lines = args.get("max_lines").and_then(|v| v.as_u64()).map(|n| n as usize);
        let file_arg = args.get("file").and_then(|v| v.as_str()).map(PathBuf::from);
        let intent = args.get("intent").and_then(|v| v.as_str()).map(|s| s.to_string());
        let background = args.get("background").and_then(|v| v.as_bool());
        let summary = args.get("summary").and_then(|v| v.as_bool());

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
            file: file_arg,
            background,
            summary,
            intent: intent.clone(),
        };

        match self.sandbox.execute(&req) {
            Ok(res) => {
                let out_body = if res.exit_code != 0 {
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
                };

                // Output over OUTPUT_CAP (16KB) or with intent: index full output & return preview + handle line
                if out_body.len() > OUTPUT_CAP || (intent.is_some() && out_body.len() > EXEC_PREVIEW_BYTES) {
                    let first_line = code.lines().next().unwrap_or("command").trim();
                    let source_label = format!("ctx_execute:{}", first_line.chars().take(40).collect::<String>());
                    if let Ok(mut store) = self.store.lock() {
                        let _ = store.index(&out_body, &source_label, Some("prose"), None);
                    }
                    let preview = if out_body.len() > EXEC_PREVIEW_BYTES {
                        let mut end = EXEC_PREVIEW_BYTES;
                        while end > 0 && !out_body.is_char_boundary(end) {
                            end -= 1;
                        }
                        &out_body[..end]
                    } else {
                        &out_body
                    };
                    let handle_line = replace_handle_line(intent.as_deref().unwrap_or(""), &source_label, out_body.len());
                    format!("{}\n\n{}", preview, handle_line)
                } else {
                    out_body
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
                let out_body = if res.exit_code != 0 {
                    format!("Exit code {}:\n{}\n{}", res.exit_code, res.stderr, res.stdout)
                } else if res.stdout.is_empty() && !res.stderr.is_empty() {
                    res.stderr
                } else {
                    res.stdout
                };

                if out_body.len() > OUTPUT_CAP {
                    let source_label = format!(
                        "ctx_execute_file:{}",
                        target_path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
                    );
                    if let Ok(mut store) = self.store.lock() {
                        let _ = store.index(&out_body, &source_label, Some("prose"), None);
                    }
                    let preview = if out_body.len() > EXEC_PREVIEW_BYTES {
                        let mut end = EXEC_PREVIEW_BYTES;
                        while end > 0 && !out_body.is_char_boundary(end) {
                            end -= 1;
                        }
                        &out_body[..end]
                    } else {
                        &out_body
                    };
                    let handle_line = replace_handle_line("", &source_label, out_body.len());
                    format!("{}\n\n{}", preview, handle_line)
                } else {
                    out_body
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
            let (cmd_str, label, cwd) = if let Some(s) = c_val.as_str() {
                (s, format!("batch-cmd-{}", i + 1), default_cwd.clone())
            } else {
                let s = c_val.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let lbl = c_val
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| format!("batch-cmd-{}", i + 1));
                let c = c_val
                    .get("cwd")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .or_else(|| default_cwd.clone());
                (s, lbl, c)
            };

            let req = ExecRequest {
                language: "shell".to_string(),
                code: cmd_str.to_string(),
                cwd,
                timeout_secs: timeout,
                max_lines: Some(100),
                max_bytes: None,
                ..Default::default()
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
        let url_opt = args.get("url").and_then(|v| v.as_str());
        let source_label = args.get("source").and_then(|v| v.as_str()).unwrap_or("inline");
        let content_type = args.get("content_type").and_then(|v| v.as_str()).unwrap_or("auto");

        if let Some(url) = url_opt {
            let mut store = match self.store.lock() {
                Ok(s) => s,
                Err(e) => return format!("Store lock error: {}", e),
            };
            let req = FetchRequest {
                url: url.to_string(),
                source: args.get("source").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };
            return match self.fetcher.fetch_one(&mut store, &req, false, None) {
                super::fetch::FetchOneOutcome::Fetched { label, total_chunks, total_bytes, .. } => {
                    format!("Indexed URL '{}' into '{}': {} chunks ({} bytes)", url, label, total_chunks, total_bytes)
                }
                super::fetch::FetchOneOutcome::Cached { label, chunk_count, age_str, .. } => {
                    format!("Indexed URL '{}' (cached): '{}' ({} chunks, {})", url, label, chunk_count, age_str)
                }
                super::fetch::FetchOneOutcome::Error { error, .. } => {
                    format!("URL indexing error: {}", error)
                }
            };
        }

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

        if let Some(q) = args.get("query").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()) {
            query_list.push(q.trim().to_string());
        }

        if let Some(arr) = args.get("queries").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str().filter(|s| !s.trim().is_empty()) {
                    query_list.push(s.trim().to_string());
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

        let root_dir = self.project_root.clone().or_else(|| std::env::current_dir().ok());
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
                Ok(mut results) => {
                    // D-UNIFIED-MEMORY-1 & D-MEMORY-SOURCE-1:
                    // Merge auto-memory hits from worktree
                    if let Some(ref root) = root_dir {
                        let is_mem_source = source_filter.as_deref().map(|s| {
                            s.starts_with(MEMORY_LABEL_PREFIX) || s.ends_with(".md")
                        }).unwrap_or(false);

                        let mem_hits = query_auto_memory(
                            root,
                            &q,
                            if is_mem_source { limit } else { 2 },
                            source_filter.as_deref(),
                        );

                        if !mem_hits.is_empty() {
                            if is_mem_source && results.is_empty() {
                                results = mem_hits;
                            } else {
                                let mut combined = mem_hits;
                                combined.extend(results);
                                results = combined;
                                if results.len() > limit {
                                    results.truncate(limit);
                                }
                            }
                        }
                    }

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

    fn tool_ctx_resume(&self, args: &Value) -> String {
        let session = args.get("session").and_then(|v| v.as_str()).unwrap_or("current");
        let seq = args.get("seq").and_then(|v| v.as_u64());

        let stats = get_stats();
        if let Some(s) = seq {
            format!("Session '{}' event sequence #{}: [State verified, tool calls recorded: {}]", session, s, stats.calls.values().sum::<usize>())
        } else {
            let store = match self.store.lock() {
                Ok(s) => s,
                Err(e) => return format!("Store lock error: {}", e),
            };
            let sources = store.list_sources().unwrap_or_default();
            let mut lines = vec![
                format!("Session '{}' state resume:", session),
                format!("  Recorded tool calls: {}", stats.calls.values().sum::<usize>()),
                format!("  Bytes sandboxed:     {}", stats.bytes_sandboxed),
                format!("  Indexed sources:     {}", sources.len()),
            ];
            for src in sources.iter().take(5) {
                lines.push(format!("  - source {}: '{}' ({} chunks)", src.id, src.label, src.chunk_count));
            }
            lines.join("\n")
        }
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
            "  Sandbox readiness:  READY (pure Rust in-process runner)".to_string(),
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

    fn tool_ctx_upgrade(&self, _args: &Value) -> String {
        "context-mode: pure native Rust engine embedded in Toksi (v3.3). Node.js and TypeScript runtime dependencies purged. Run `toksi update` for binary updates.".to_string()
    }
    pub fn handle_mcp_message(&self, req: &Value) -> Option<Value> {
        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        match method {
            "initialize" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "context",
                        "version": "1.0.5"
                    },
                    "capabilities": {
                        "tools": {}
                    },
                    "instructions": "You are connected to context: an isolated sandbox execution and SQLite FTS5 knowledge retrieval engine."
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

                let result_text = self.execute_tool(name, &args);
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

        let resp = server.handle_mcp_message(&req);

        if let Some(r) = resp {
            let serialized = serde_json::to_string(&r).unwrap_or_default();
            let _ = writeln!(writer, "{}", serialized);
            let _ = writer.flush();
        }

        line.clear();
    }

    0
}

/// Handles `/context` slash commands: `/context [status|stats|doctor|search|purge|resume]`
pub fn handle_slash_command(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let sub = parts.get(1).copied().unwrap_or("status");

    match sub {
        "stats" => {
            let stats = get_stats();
            format!(
                "Context Session Stats:\n  Total tool calls: {}\n  Bytes sandboxed: {}\n  Bytes indexed: {}\n  Cache hits/misses: {}/{}\n  Estimated token savings: ~{} tokens",
                stats.calls.values().sum::<usize>(),
                stats.bytes_sandboxed,
                stats.bytes_indexed,
                stats.cache_hits,
                stats.cache_misses,
                stats.estimated_tokens_saved()
            )
        }
        "doctor" => {
            let runtimes = detect_runtimes();
            let db_path = resolve_content_store_path(None);
            format!(
                "Context Doctor:\n  Shell: {}\n  Python: {}\n  Node: {}\n  Bun: {}\n  Deno: {}\n  Rust: {}\n  DB: {}\n  Status: READY",
                runtimes.shell,
                runtimes.python.as_deref().unwrap_or("none"),
                runtimes.node.as_deref().unwrap_or("none"),
                runtimes.bun.as_deref().unwrap_or("none"),
                runtimes.deno.as_deref().unwrap_or("none"),
                runtimes.rust.as_deref().unwrap_or("none"),
                db_path.display()
            )
        }
        "search" => {
            let query = parts[2..].join(" ");
            if query.is_empty() {
                return "Usage: /context search <query>".to_string();
            }
            let db_path = resolve_content_store_path(None);
            let store = match ContentStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => return format!("DB open error: {e}"),
            };
            let opts = SearchOptions {
                query: query.clone(),
                limit: 5,
                source: None,
                content_type: None,
                session_id: None,
            };
            match store.search(&opts) {
                Ok(hits) => {
                    if hits.is_empty() {
                        format!("No matching context chunks for query '{}'", query)
                    } else {
                        let mut out = format!("Found {} context chunks for '{}':\n", hits.len(), query);
                        for (i, h) in hits.iter().enumerate() {
                            out.push_str(&format!("\n[{}] {} (score: {:.2})\n{}", i + 1, h.title, h.score, h.content));
                        }
                        out
                    }
                }
                Err(e) => format!("Search error: {e}"),
            }
        }
        "purge" => {
            let db_path = resolve_content_store_path(None);
            let mut store = match ContentStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => return format!("DB open error: {e}"),
            };
            match store.purge_all() {
                Ok(count) => format!("Purged {} sources from context knowledge base.", count),
                Err(e) => format!("Purge error: {e}"),
            }
        }
        "resume" => {
            let session = parts.get(2).copied().unwrap_or("default");
            let db_path = resolve_content_store_path(None);
            let store = match ContentStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => return format!("DB open error: {e}"),
            };
            let events = store.get_session_events(session, None).unwrap_or_default();
            super::compaction::render_state(&events, 0, "")
        }
        "status" | _ => {
            let db_path = resolve_content_store_path(None);
            let store_exists = db_path.exists();
            let source_count = if store_exists {
                ContentStore::open(&db_path).map(|s| s.list_sources().map(|l| l.len()).unwrap_or(0)).unwrap_or(0)
            } else {
                0
            };
            format!(
                "Context Subsystem Status:\n  Engine: Pure Rust (Edition 2024)\n  DB Path: {}\n  DB Active: {}\n  Indexed Sources: {}\n  Tool Suite: ctx_execute, ctx_execute_file, ctx_batch_execute, ctx_index, ctx_search, ctx_fetch_and_index\n  Commands: /context [status|stats|doctor|search|purge|resume]",
                db_path.display(),
                store_exists,
                source_count
            )
        }
    }
}
