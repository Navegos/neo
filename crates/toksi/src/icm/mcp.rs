// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/mcp.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::io::{self, BufRead, Write};
use chrono::Utc;
use serde_json::{json, Value};

use crate::icm::recall::{recall_memories, RecallOptions};
use crate::icm::schema::{Importance, Memory};
use super::store::{compute_summary_hash, resolve_icm_db_path_checked, IcmStore};

pub fn run_mcp_server() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin_lock = stdin.lock();
    let mut stdout_lock = stdout.lock();

    let db_path = match resolve_icm_db_path_checked() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[icm-mcp] failed to resolve repo-only sqlite path: {}", e);
            return 1;
        }
    };

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
                let serialized = serde_json::to_string(&err_resp).unwrap_or_else(|_| "{}".to_string());
                let _ = writeln!(stdout_lock, "{}", serialized);
                let _ = stdout_lock.flush();
                line.clear();
                continue;
            }
        };

        if let Some(resp) = handle_mcp_request(&mut store, &req) {
            let serialized = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
            let _ = writeln!(stdout_lock, "{}", serialized);
            let _ = stdout_lock.flush();
        }

        line.clear();
    }

    0
}

pub fn handle_mcp_request(store: &mut IcmStore, req: &Value) -> Option<Value> {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");

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
                    "name": "toksi-icm",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "notifications/initialized" => None,
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tool_definitions()
            }
        })),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(json!({}));
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            let (content, is_error) = handle_tool_call(store, name, &args);

            if is_error {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": content
                    }
                }))
            } else {
                Some(json!({
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
                }))
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {}", method) }
        })),
    }
}

pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "icm_memory_store",
            "description": "Store information in ICM long-term memory. Use to save decisions, preferences, resolved errors.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Category or namespace (e.g. 'architecture', 'decisions')" },
                    "summary": { "type": "string", "description": "Distilled, self-contained summary of the memory" },
                    "content": { "type": "string", "description": "Alternative key for summary" },
                    "raw_excerpt": { "type": "string", "description": "Original raw code snippet or excerpt" },
                    "keywords": { "type": "array", "items": { "type": "string" }, "description": "Keywords for indexing" },
                    "importance": { "type": "string", "enum": ["critical", "high", "medium", "low"], "description": "Importance level" }
                },
                "required": ["topic"]
            }
        },
        {
            "name": "icm_memory_recall",
            "description": "Recall memories from ICM long-term memory using hybrid FTS5 BM25 and temporal RRF fusion.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query or natural language question" },
                    "topic": { "type": "string", "description": "Filter by exact topic" },
                    "limit": { "type": "integer", "description": "Maximum number of results to return (default: 5)" },
                    "max_tokens": { "type": "integer", "description": "Maximum token budget for recalled results" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "icm_memory_related",
            "description": "Find related memories linked to an existing memory ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Source memory ID" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "icm_memory_forget",
            "description": "Delete a memory entry by ID from ICM.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "ID of memory to delete" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "icm_forget",
            "description": "Alias for icm_memory_forget: delete a memory entry by ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "ID of memory to delete" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "icm_memory_forget_topic",
            "description": "Delete non-critical memories under a topic.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic to clear" }
                },
                "required": ["topic"]
            }
        },
        {
            "name": "icm_memory_update",
            "description": "Update an existing memory's summary or importance.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Memory ID" },
                    "summary": { "type": "string", "description": "New summary content" },
                    "importance": { "type": "string", "enum": ["critical", "high", "medium", "low"] }
                },
                "required": ["id"]
            }
        },
        {
            "name": "icm_memory_consolidate",
            "description": "Consolidate a topic by rolling up expendable memories past threshold.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic to consolidate" },
                    "threshold": { "type": "integer", "description": "Consolidation threshold (default: 10)" }
                },
                "required": ["topic"]
            }
        },
        {
            "name": "icm_consolidate",
            "description": "Alias for icm_memory_consolidate.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic to consolidate" },
                    "threshold": { "type": "integer", "description": "Consolidation threshold (default: 10)" }
                },
                "required": ["topic"]
            }
        },
        {
            "name": "icm_memory_list_topics",
            "description": "List all active topics in ICM with counts and average weights.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "icm_list_topics",
            "description": "Alias for icm_memory_list_topics.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "icm_memory_stats",
            "description": "Get global telemetry statistics on ICM database.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "icm_memory_health",
            "description": "Inspect topic health and distribution in ICM.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Optional topic filter" }
                }
            }
        },
        {
            "name": "icm_pin",
            "description": "Pin a memory entry: sets importance to critical and weight to 1.0 (immune to decay/consolidation).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Memory ID to pin" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "icm_set_fact",
            "description": "Store or supersede a structured fact slot (entity::key = value).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "entity": { "type": "string", "description": "Entity name (e.g. 'compiler', 'postgres')" },
                    "key": { "type": "string", "description": "Attribute name" },
                    "value": { "type": "string", "description": "Attribute value" },
                    "source": { "type": "string", "description": "Source attribution" }
                },
                "required": ["entity", "key", "value"]
            }
        },
        {
            "name": "icm_get_fact",
            "description": "Get the active value of a structured fact slot.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "entity": { "type": "string", "description": "Entity name" },
                    "key": { "type": "string", "description": "Attribute name" }
                },
                "required": ["entity", "key"]
            }
        },
        {
            "name": "icm_decay",
            "description": "Apply temporal decay across all non-critical memories.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "decay_factor": { "type": "number", "description": "Decay factor between 0.01 and 0.99 (default: 0.95)" }
                }
            }
        },
        {
            "name": "icm_learn",
            "description": "Extract learnings and patterns from text into high-importance long-term memory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic name" },
                    "content": { "type": "string", "description": "Learned pattern or resolution text" }
                },
                "required": ["topic", "content"]
            }
        },
        {
            "name": "icm_wake_up",
            "description": "Generate standing context brief combining critical memories, facts, and recent feedback.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project identifier" }
                }
            }
        },
        {
            "name": "icm_memoir_create",
            "description": "Create a new memoir domain concept collection.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Memoir name" },
                    "description": { "type": "string", "description": "Domain description" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "icm_memoir_list",
            "description": "List all registered memoirs.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "icm_memoir_show",
            "description": "Inspect memoir and its concepts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memoir": { "type": "string", "description": "Memoir name or ID" }
                },
                "required": ["memoir"]
            }
        },
        {
            "name": "icm_memoir_add_concept",
            "description": "Add a concept to a memoir.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memoir_id": { "type": "string", "description": "Memoir ID" },
                    "name": { "type": "string", "description": "Concept name" },
                    "definition": { "type": "string", "description": "Concept definition" },
                    "labels": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["memoir_id", "name", "definition"]
            }
        },
        {
            "name": "icm_memoir_refine",
            "description": "Refine an existing concept definition and increment revision.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "concept_id": { "type": "string", "description": "Concept ID" },
                    "new_definition": { "type": "string", "description": "Refined definition" }
                },
                "required": ["concept_id", "new_definition"]
            }
        },
        {
            "name": "icm_memoir_search",
            "description": "Search concepts within a memoir.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memoir_id": { "type": "string", "description": "Memoir ID" },
                    "query": { "type": "string", "description": "Search query" }
                },
                "required": ["memoir_id", "query"]
            }
        },
        {
            "name": "icm_memoir_search_all",
            "description": "Search concepts across all memoirs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "icm_memoir_link",
            "description": "Link two concepts with a named relationship.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string", "description": "Source concept ID" },
                    "target_id": { "type": "string", "description": "Target concept ID" },
                    "relation": { "type": "string", "description": "Relationship name (e.g. 'depends_on')" }
                },
                "required": ["source_id", "target_id", "relation"]
            }
        },
        {
            "name": "icm_memoir_inspect",
            "description": "Inspect full concept graph for a memoir.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memoir_id": { "type": "string", "description": "Memoir ID" }
                },
                "required": ["memoir_id"]
            }
        },
        {
            "name": "icm_memoir_export",
            "description": "Export memoir concept graph to JSON.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memoir_id": { "type": "string", "description": "Memoir ID" }
                },
                "required": ["memoir_id"]
            }
        },
        {
            "name": "icm_feedback_record",
            "description": "Record user feedback or corrections on agent output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic or task area" },
                    "context": { "type": "string", "description": "Context of interaction" },
                    "predicted": { "type": "string", "description": "Agent predicted output" },
                    "corrected": { "type": "string", "description": "User corrected output" },
                    "reason": { "type": "string", "description": "Why correction was needed" }
                },
                "required": ["topic", "context", "predicted", "corrected"]
            }
        },
        {
            "name": "icm_feedback_search",
            "description": "Search past feedback records using FTS5.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "topic": { "type": "string", "description": "Optional topic filter" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "icm_feedback_stats",
            "description": "Get statistics on recorded feedback.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "icm_transcript_start_session",
            "description": "Start a session for transcript logging.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "agent": { "type": "string", "description": "Agent identifier" },
                    "project": { "type": "string", "description": "Project identifier" }
                }
            }
        },
        {
            "name": "icm_transcript_record",
            "description": "Record a message into an active session transcript.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session ID" },
                    "role": { "type": "string", "description": "Message role ('user', 'assistant', 'system', 'tool')" },
                    "content": { "type": "string", "description": "Message text" }
                },
                "required": ["session_id", "role", "content"]
            }
        },
        {
            "name": "icm_transcript_search",
            "description": "Search verbatim transcripts using FTS5.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "icm_transcript_show",
            "description": "Retrieve messages from a session transcript.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session ID" }
                },
                "required": ["session_id"]
            }
        },
        {
            "name": "icm_transcript_stats",
            "description": "Get statistics on session transcripts.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "memory_handoff_offer",
            "description": "Offer context handoff to another agent",
            "inputSchema": { "type": "object", "properties": { "title": { "type": "string" }, "task": { "type": "string" }, "from_agent": { "type": "string" } }, "required": ["title", "task", "from_agent"] }
        },
        {
            "name": "memory_handoff_claim",
            "description": "Claim offered context handoff",
            "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "agent": { "type": "string" } }, "required": ["id", "agent"] }
        },
        {
            "name": "memory_handoff_accept",
            "description": "Alias for memory_handoff_claim",
            "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "agent": { "type": "string" } }, "required": ["id", "agent"] }
        },
        {
            "name": "memory_handoff_resolve",
            "description": "Resolve claimed context handoff",
            "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "agent": { "type": "string" } }, "required": ["id", "agent"] }
        },
        {
            "name": "memory_handoff_list",
            "description": "List active context handoffs",
            "inputSchema": { "type": "object", "properties": { "agent": { "type": "string" } } }
        },
        {
            "name": "memory_wiki_read",
            "description": "Read OKF wiki documentation page",
            "inputSchema": { "type": "object", "properties": { "slug": { "type": "string" } }, "required": ["slug"] }
        },
        {
            "name": "memory_wiki_write",
            "description": "Write OKF wiki documentation page",
            "inputSchema": { "type": "object", "properties": { "slug": { "type": "string" }, "title": { "type": "string" }, "body": { "type": "string" } }, "required": ["slug", "title", "body"] }
        },
        {
            "name": "memory_wiki_list",
            "description": "List all wiki pages",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

pub fn handle_tool_call(store: &mut IcmStore, name: &str, args: &Value) -> (String, bool) {
    match name {
        "icm_memory_store" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("general");
            let summary = args.get("summary").or_else(|| args.get("content")).and_then(|v| v.as_str()).unwrap_or("");
            if summary.is_empty() {
                return ("Error: 'summary' or 'content' is required".to_string(), true);
            }
            let raw_excerpt = args.get("raw_excerpt").and_then(|v| v.as_str()).map(|s| s.to_string());
            let keywords = args.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec![topic.to_string()]);
            let importance = args.get("importance")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(Importance::Medium);

            let now = Utc::now().to_rfc3339();
            let id = format!("mem_{:x}", rand_u64());

            let mem = Memory {
                id: id.clone(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_accessed: now,
                access_count: 0,
                weight: if importance == Importance::Critical { 1.0 } else { 0.8 },
                topic: topic.to_string(),
                summary: summary.to_string(),
                raw_excerpt,
                keywords,
                importance,
                source_type: "mcp".into(),
                source_data: None,
                related_ids: Vec::new(),
                summary_hash: Some(compute_summary_hash(topic, summary)),
                subproject_id: Some(store.subproject_id().to_string()),
            };

            match store.insert_or_update_memory(&mem) {
                Ok(_) => (format!("Memory stored successfully: '{}' under topic '{}'", id, topic), false),
                Err(e) => (format!("Error storing memory: {e}"), true),
            }
        }
        "icm_memory_recall" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let topic = args.get("topic").and_then(|v| v.as_str()).map(String::from);
            let max_tokens = args.get("max_tokens").and_then(|v| v.as_u64()).map(|t| t as usize);

            let opts = RecallOptions {
                query: query.to_string(),
                limit,
                topic,
                min_weight: None,
                max_tokens,
            };

            match recall_memories(store, &opts) {
                Ok(res) => {
                    let out = serde_json::to_string_pretty(&res).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error recalling memories: {e}"), true),
            }
        }
        "icm_memory_related" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match store.get_memory(id) {
                Ok(Some(mem)) => {
                    let out = serde_json::to_string_pretty(&mem.related_ids).unwrap_or_default();
                    (out, false)
                }
                Ok(None) => (format!("Memory '{}' not found", id), true),
                Err(e) => (format!("Error fetching related: {e}"), true),
            }
        }
        "icm_memory_forget" | "icm_forget" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match store.delete_memory(id) {
                Ok(true) => (format!("Deleted memory '{id}'"), false),
                Ok(false) => (format!("Memory '{id}' not found"), false),
                Err(e) => (format!("Error deleting memory: {e}"), true),
            }
        }
        "icm_memory_forget_topic" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("");
            match store.forget_topic(topic) {
                Ok(count) => (format!("Cleared {} non-critical memories under topic '{topic}'", count), false),
                Err(e) => (format!("Error forgetting topic: {e}"), true),
            }
        }
        "icm_memory_update" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match store.get_memory(id) {
                Ok(Some(mut mem)) => {
                    if let Some(s) = args.get("summary").and_then(|v| v.as_str()) {
                        mem.summary = s.to_string();
                        mem.summary_hash = Some(compute_summary_hash(&mem.topic, s));
                    }
                    if let Some(imp_str) = args.get("importance").and_then(|v| v.as_str()) {
                        if let Ok(imp) = imp_str.parse() {
                            mem.importance = imp;
                        }
                    }
                    mem.updated_at = Utc::now().to_rfc3339();
                    match store.insert_or_update_memory(&mem) {
                        Ok(_) => (format!("Updated memory '{id}'"), false),
                        Err(e) => (format!("Error updating memory: {e}"), true),
                    }
                }
                Ok(None) => (format!("Memory '{id}' not found"), true),
                Err(e) => (format!("Error: {e}"), true),
            }
        }
        "icm_memory_consolidate" | "icm_consolidate" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("");
            let threshold = args.get("threshold").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            match store.consolidate_topic(topic, threshold) {
                Ok(count) => (format!("Consolidated topic '{topic}': pruned {count} expendable memories"), false),
                Err(e) => (format!("Error consolidating: {e}"), true),
            }
        }
        "icm_memory_list_topics" | "icm_list_topics" => {
            match store.list_topics() {
                Ok(topics) => {
                    let out = serde_json::to_string_pretty(&topics).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error listing topics: {e}"), true),
            }
        }
        "icm_memory_stats" | "icm_stats" => {
            match store.stats() {
                Ok(s) => {
                    let out = serde_json::to_string_pretty(&s).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error getting stats: {e}"), true),
            }
        }
        "icm_memory_health" => {
            let topic = args.get("topic").and_then(|v| v.as_str());
            match store.health(topic) {
                Ok(h) => {
                    let out = serde_json::to_string_pretty(&h).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error getting health: {e}"), true),
            }
        }
        "icm_pin" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match store.pin_memory(id) {
                Ok(m) => (format!("Pinned memory '{}' (importance: critical, weight: 1.0)", m.id), false),
                Err(e) => (format!("Error pinning memory: {e}"), true),
            }
        }
        "icm_set_fact" => {
            let entity = args.get("entity").and_then(|v| v.as_str()).unwrap_or("");
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("mcp");

            if entity.is_empty() || key.is_empty() || value.is_empty() {
                return ("Error: entity, key, and value are required".to_string(), true);
            }

            match store.set_fact(entity, key, value, source) {
                Ok(fid) => (format!("Set fact '{}': {}::{} = {}", fid, entity, key, value), false),
                Err(e) => (format!("Error setting fact: {e}"), true),
            }
        }
        "icm_get_fact" => {
            let entity = args.get("entity").and_then(|v| v.as_str()).unwrap_or("");
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");

            match store.get_fact(entity, key) {
                Ok(Some(f)) => {
                    let out = serde_json::to_string_pretty(&f).unwrap_or_default();
                    (out, false)
                }
                Ok(None) => (format!("Fact {}::{} not found", entity, key), true),
                Err(e) => (format!("Error getting fact: {e}"), true),
            }
        }
        "icm_decay" => {
            let factor = args.get("decay_factor").and_then(|v| v.as_f64()).unwrap_or(0.95) as f32;
            match store.apply_decay(factor) {
                Ok(count) => (format!("Applied temporal decay (factor {:.2}) to {} non-critical memories", factor, count), false),
                Err(e) => (format!("Error applying decay: {e}"), true),
            }
        }
        "icm_learn" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("learnings");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            match store.learn_from_text(topic, content, "mcp-learn") {
                Ok(m) => (format!("Learned new memory '{}' under topic '{}'", m.id, m.topic), false),
                Err(e) => (format!("Error learning: {e}"), true),
            }
        }
        "icm_wake_up" => {
            let project = args.get("project").and_then(|v| v.as_str());
            match store.build_wake_up_brief(project) {
                Ok(brief) => (brief, false),
                Err(e) => (format!("Error building wake up brief: {e}"), true),
            }
        }
        "icm_memoir_create" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
            match store.create_memoir(name, description, 50) {
                Ok(m) => (format!("Created memoir '{}' (id: {})", m.name, m.id), false),
                Err(e) => (format!("Error creating memoir: {e}"), true),
            }
        }
        "icm_memoir_list" => {
            match store.list_memoirs() {
                Ok(list) => {
                    let out = serde_json::to_string_pretty(&list).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error listing memoirs: {e}"), true),
            }
        }
        "icm_memoir_show" => {
            let memoir = args.get("memoir").and_then(|v| v.as_str()).unwrap_or("");
            match store.get_memoir(memoir) {
                Ok(Some(m)) => {
                    let out = serde_json::to_string_pretty(&m).unwrap_or_default();
                    (out, false)
                }
                Ok(None) => (format!("Memoir '{}' not found", memoir), true),
                Err(e) => (format!("Error showing memoir: {e}"), true),
            }
        }
        "icm_memoir_add_concept" => {
            let memoir_id = args.get("memoir_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let definition = args.get("definition").and_then(|v| v.as_str()).unwrap_or("");
            let labels: Vec<String> = args.get("labels")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();

            match store.add_concept(memoir_id, name, definition, &labels, 0.8, &[]) {
                Ok(c) => (format!("Added concept '{}' (id: {}) to memoir '{}'", c.name, c.id, memoir_id), false),
                Err(e) => (format!("Error adding concept: {e}"), true),
            }
        }
        "icm_memoir_refine" => {
            let concept_id = args.get("concept_id").and_then(|v| v.as_str()).unwrap_or("");
            let new_def = args.get("new_definition").and_then(|v| v.as_str()).unwrap_or("");
            match store.refine_concept(concept_id, new_def, 0.9) {
                Ok(c) => (format!("Refined concept '{}' (rev {})", c.name, c.revision), false),
                Err(e) => (format!("Error refining concept: {e}"), true),
            }
        }
        "icm_memoir_search" => {
            let memoir_id = args.get("memoir_id").and_then(|v| v.as_str()).unwrap_or("");
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match store.search_concepts(memoir_id, query, 10) {
                Ok(concepts) => {
                    let out = serde_json::to_string_pretty(&concepts).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error searching concepts: {e}"), true),
            }
        }
        "icm_memoir_search_all" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match store.search_all_concepts(query, 10) {
                Ok(concepts) => {
                    let out = serde_json::to_string_pretty(&concepts).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error searching concepts: {e}"), true),
            }
        }
        "icm_memoir_link" => {
            let source_id = args.get("source_id").and_then(|v| v.as_str()).unwrap_or("");
            let target_id = args.get("target_id").and_then(|v| v.as_str()).unwrap_or("");
            let relation = args.get("relation").and_then(|v| v.as_str()).unwrap_or("related_to");
            match store.link_concepts(source_id, target_id, relation, 1.0) {
                Ok(l) => (format!("Linked {} -> {} ({})", l.source_id, l.target_id, l.relation), false),
                Err(e) => (format!("Error linking concepts: {e}"), true),
            }
        }
        "icm_memoir_inspect" => {
            let memoir_id = args.get("memoir_id").and_then(|v| v.as_str()).unwrap_or("");
            match store.inspect_memoir(memoir_id) {
                Ok(val) => {
                    let out = serde_json::to_string_pretty(&val).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error inspecting memoir: {e}"), true),
            }
        }
        "icm_memoir_export" => {
            let memoir_id = args.get("memoir_id").and_then(|v| v.as_str()).unwrap_or("");
            match store.export_memoir(memoir_id) {
                Ok(json_str) => (json_str, false),
                Err(e) => (format!("Error exporting memoir: {e}"), true),
            }
        }
        "icm_feedback_record" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("general");
            let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");
            let predicted = args.get("predicted").and_then(|v| v.as_str()).unwrap_or("");
            let corrected = args.get("corrected").and_then(|v| v.as_str()).unwrap_or("");
            let reason = args.get("reason").and_then(|v| v.as_str());

            match store.record_feedback(topic, context, predicted, corrected, reason, "mcp") {
                Ok(fb) => (format!("Recorded feedback '{}' on topic '{}'", fb.id, fb.topic), false),
                Err(e) => (format!("Error recording feedback: {e}"), true),
            }
        }
        "icm_feedback_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let topic = args.get("topic").and_then(|v| v.as_str());
            match store.search_feedback(query, topic, 10) {
                Ok(fb) => {
                    let out = serde_json::to_string_pretty(&fb).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error searching feedback: {e}"), true),
            }
        }
        "icm_feedback_stats" => {
            match store.feedback_stats() {
                Ok(stats) => {
                    let out = serde_json::to_string_pretty(&stats).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error getting feedback stats: {e}"), true),
            }
        }
        "icm_transcript_start_session" => {
            let agent = args.get("agent").and_then(|v| v.as_str()).unwrap_or("mcp");
            let project = args.get("project").and_then(|v| v.as_str());
            match store.create_session(agent, project, "{}") {
                Ok(s) => (format!("Started session '{}' for agent '{}'", s.id, s.agent), false),
                Err(e) => (format!("Error starting session: {e}"), true),
            }
        }
        "icm_transcript_record" => {
            let session_id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
            let role = args.get("role").and_then(|v| v.as_str()).unwrap_or("assistant");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            match store.record_message(session_id, role, content, None, None, "{}") {
                Ok(msg) => (format!("Recorded message '{}' in session '{}'", msg.id, session_id), false),
                Err(e) => (format!("Error recording message: {e}"), true),
            }
        }
        "icm_transcript_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match store.search_transcripts(query, 10) {
                Ok(messages) => {
                    let out = serde_json::to_string_pretty(&messages).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error searching transcripts: {e}"), true),
            }
        }
        "icm_transcript_show" => {
            let session_id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
            match store.get_session_messages(session_id) {
                Ok(messages) => {
                    let out = serde_json::to_string_pretty(&messages).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error retrieving session messages: {e}"), true),
            }
        }
        "icm_transcript_stats" => {
            match store.transcript_stats() {
                Ok(stats) => {
                    let out = serde_json::to_string_pretty(&stats).unwrap_or_default();
                    (out, false)
                }
                Err(e) => (format!("Error getting transcript stats: {e}"), true),
            }
        }
        // Backward-compatibility delegation to aimemory wiki and handoff tools
        "memory_wiki_read" | "memory_wiki_write" | "memory_wiki_list" |
        "memory_handoff_offer" | "memory_handoff_begin" | "memory_handoff_claim" |
        "memory_handoff_accept" | "memory_handoff_resolve" | "memory_handoff_cancel" |
        "memory_handoff_list" => {
            crate::aimemory::mcp::handle_aimemory_tool_call(name, args)
        }
        _ => (format!("Unknown tool: {}", name), true),
    }
}

fn rand_u64() -> u64 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    d.as_nanos() as u64
}
