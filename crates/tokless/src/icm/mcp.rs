// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/icm/mcp.rs

use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use chrono::Utc;
use serde_json::{json, Value};

use crate::icm::recall::{recall_memories, RecallOptions};
use crate::icm::schema::{Importance, Memory};
use crate::icm::store::{compute_summary_hash, IcmStore, DEFAULT_DB_FILENAME};

pub fn resolve_icm_db_path() -> PathBuf {
    if let Ok(p) = env::var("ICM_DB_PATH") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(root) = env::var("PROJECTMEM_ROOT") {
        if !root.trim().is_empty() {
            return PathBuf::from(root).join(".tokless").join(DEFAULT_DB_FILENAME);
        }
    }
    if let Ok(cwd) = env::current_dir() {
        return cwd.join(".tokless").join(DEFAULT_DB_FILENAME);
    }
    PathBuf::from(".tokless").join(DEFAULT_DB_FILENAME)
}

pub fn run_mcp_server() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin_lock = stdin.lock();
    let mut stdout_lock = stdout.lock();

    let db_path = resolve_icm_db_path();
    let mut store = match IcmStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[icm-mcp] failed to open sqlite database {}: {}", db_path.display(), e);
            return 1;
        }
    };

    let mut line = String::new();
    while let Ok(bytes) = stdin_lock.read_line(&mut line) {
        if bytes == 0 {
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
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                let _ = writeln!(stdout_lock, "{}", serde_json::to_string(&err_resp).unwrap());
                let _ = stdout_lock.flush();
                line.clear();
                continue;
            }
        };

        let resp = handle_mcp_request(&mut store, &req);
        if let Some(r) = resp {
            let _ = writeln!(stdout_lock, "{}", serde_json::to_string(&r).unwrap());
            let _ = stdout_lock.flush();
        }

        line.clear();
    }
    0
}

fn handle_mcp_request(store: &mut IcmStore, req: &Value) -> Option<Value> {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "tokless-icm",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "notifications/initialized" => None,
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "icm_memory_store",
                        "description": "Store a memory item in the local SQLite BM25 knowledge base (zero-LLM token cost).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "topic": { "type": "string", "description": "Subject category or topic" },
                                "summary": { "type": "string", "description": "Crisp key takeaway or finding" },
                                "raw_excerpt": { "type": "string", "description": "Optional supporting context or snippet" },
                                "keywords": {
                                    "type": "array",
                                    "items": { "type": "string" },
                                    "description": "Index tags for search"
                                },
                                "importance": {
                                    "type": "string",
                                    "enum": ["critical", "high", "medium", "low"],
                                    "default": "medium"
                                }
                            },
                            "required": ["topic", "summary"]
                        }
                    },
                    {
                        "name": "icm_memory_recall",
                        "description": "Recall memories via SQLite FTS5 BM25 with sub-10ms latency and zero LLM cost.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string", "description": "Search query terms or topic" },
                                "limit": { "type": "integer", "default": 5, "description": "Max hits to return" },
                                "topic": { "type": "string", "description": "Optional topic filter" },
                                "max_tokens": { "type": "integer", "description": "Optional token budget cap" }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "icm_set_fact",
                        "description": "Set a persistent structured entity fact slot (atomic supersession).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "entity": { "type": "string", "description": "Target entity or namespace" },
                                "key": { "type": "string", "description": "Fact slot key" },
                                "value": { "type": "string", "description": "Fact value" },
                                "source": { "type": "string", "description": "Source origin" }
                            },
                            "required": ["entity", "key", "value"]
                        }
                    },
                    {
                        "name": "icm_get_fact",
                        "description": "Retrieve an active fact slot by entity and key.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "entity": { "type": "string", "description": "Target entity" },
                                "key": { "type": "string", "description": "Fact slot key" }
                            },
                            "required": ["entity", "key"]
                        }
                    },
                    {
                        "name": "icm_list_topics",
                        "description": "List all memory topics with counts and weights.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                ]
            }
        })),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            let result = dispatch_tool(store, name, &args);
            match result {
                Ok(content) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": content
                            }
                        ]
                    }
                })),
                Err(err_msg) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "isError": true,
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error: {}", err_msg)
                            }
                        ]
                    }
                })),
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {}", method) }
        })),
    }
}

fn dispatch_tool(store: &mut IcmStore, name: &str, args: &Value) -> Result<String, String> {
    match name {
        "icm_memory_store" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).ok_or("missing 'topic'")?;
            let summary = args.get("summary").and_then(|v| v.as_str()).ok_or("missing 'summary'")?;
            let raw_excerpt = args.get("raw_excerpt").and_then(|v| v.as_str()).map(String::from);
            let keywords = args.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let importance = args.get("importance")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .parse::<Importance>()
                .unwrap_or(Importance::Medium);

            let now = Utc::now().to_rfc3339();
            let hash = compute_summary_hash(topic, summary);
            let id = format!("mem_{}", &hash[..8]);

            let mem = Memory {
                id: id.clone(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_accessed: now,
                access_count: 0,
                weight: match importance {
                    Importance::Critical => 2.0,
                    Importance::High => 1.5,
                    Importance::Medium => 1.0,
                    Importance::Low => 0.5,
                },
                topic: topic.to_string(),
                summary: summary.to_string(),
                raw_excerpt,
                keywords,
                importance,
                source_type: "mcp".into(),
                source_data: None,
                related_ids: Vec::new(),
                summary_hash: Some(hash),
            };

            store.insert_or_update_memory(&mem).map_err(|e| e.to_string())?;
            Ok(format!("Memory stored successfully with id: {}", id))
        }
        "icm_memory_recall" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let topic = args.get("topic").and_then(|v| v.as_str()).map(String::from);
            let max_tokens = args.get("max_tokens").and_then(|v| v.as_u64()).map(|v| v as usize);

            let opts = RecallOptions {
                query: query.to_string(),
                limit,
                topic,
                min_weight: None,
                max_tokens,
            };

            let res = recall_memories(store, &opts).map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
        }
        "icm_set_fact" => {
            let entity = args.get("entity").and_then(|v| v.as_str()).ok_or("missing 'entity'")?;
            let key = args.get("key").and_then(|v| v.as_str()).ok_or("missing 'key'")?;
            let value = args.get("value").and_then(|v| v.as_str()).ok_or("missing 'value'")?;
            let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("mcp");

            let id = store.set_fact(entity, key, value, source).map_err(|e| e.to_string())?;
            Ok(format!("Fact set: {}::{} = {} (id: {})", entity, key, value, id))
        }
        "icm_get_fact" => {
            let entity = args.get("entity").and_then(|v| v.as_str()).ok_or("missing 'entity'")?;
            let key = args.get("key").and_then(|v| v.as_str()).ok_or("missing 'key'")?;

            let fact = store.get_fact(entity, key).map_err(|e| e.to_string())?;
            match fact {
                Some(f) => serde_json::to_string_pretty(&f).map_err(|e| e.to_string()),
                None => Ok(format!("Fact not found for {}::{}", entity, key)),
            }
        }
        "icm_list_topics" => {
            let topics = store.list_topics().map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&topics).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

