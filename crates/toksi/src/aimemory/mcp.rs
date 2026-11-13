// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/mcp.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use std::io::{self, BufRead, Write};
use serde_json::{json, Value};

use crate::aimemory::handoff::{HandoffStore, NewHandoff};
use crate::aimemory::message::{MessageNature, MessageState, MessageStore, NewMessage};
use crate::aimemory::tiered::TieredPacker;
use crate::aimemory::wiki::{WikiPage, WikiStore};

pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "memory_query",
            "description": "Query knowledge wiki pages, notes, and active handoffs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search term or tag" },
                    "limit": { "type": "integer", "description": "Max results to return (default: 5)" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "memory_recent",
            "description": "List recently modified wiki pages, active handoffs, and pending messages.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "Max results to return (default: 5)" }
                }
            }
        },
        {
            "name": "memory_write_page",
            "description": "Create or overwrite an OKF v0.2 wiki knowledge page.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Page slug coordinate (e.g. 'architecture/state_machine')" },
                    "title": { "type": "string", "description": "Page title" },
                    "doc_type": { "type": "string", "description": "OKF concept type (Concept, Fact, Decision, Procedure, Note, Invariant)" },
                    "body": { "type": "string", "description": "Markdown body content" },
                    "abstract_text": { "type": "string", "description": "Brief one-line abstract" },
                    "summary_text": { "type": "string", "description": "Paragraph summary" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags or keywords" }
                },
                "required": ["slug", "title", "body"]
            }
        },
        {
            "name": "memory_read_page",
            "description": "Read an OKF wiki knowledge page by slug.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Page slug" }
                },
                "required": ["slug"]
            }
        },
        {
            "name": "memory_patch_page",
            "description": "Patch or append to an OKF wiki knowledge page.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Page slug" },
                    "patch_text": { "type": "string", "description": "Text to patch or append" },
                    "append": { "type": "boolean", "description": "True to append, false to replace body (default: true)" }
                },
                "required": ["slug", "patch_text"]
            }
        },
        {
            "name": "memory_delete_page",
            "description": "Delete a wiki knowledge page by slug.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Page slug" }
                },
                "required": ["slug"]
            }
        },
        {
            "name": "memory_restore_page",
            "description": "Restore or re-create an empty wiki knowledge page.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Page slug" },
                    "title": { "type": "string", "description": "Title" }
                },
                "required": ["slug"]
            }
        },
        {
            "name": "memory_handoff_offer",
            "description": "Offer a claim-once multi-agent context handoff.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Short handoff title" },
                    "task": { "type": "string", "description": "Specific task to execute" },
                    "from_agent": { "type": "string", "description": "Source agent identifier" },
                    "to_agent": { "type": "string", "description": "Target agent identifier or 'any'" },
                    "active_files": { "type": "array", "items": { "type": "string" }, "description": "Key files touched" },
                    "unresolved_issues": { "type": "array", "items": { "type": "string" }, "description": "Open issues or blockers" },
                    "next_steps": { "type": "array", "items": { "type": "string" }, "description": "Recommended next actions" },
                    "context_notes": { "type": "string", "description": "Additional context notes" }
                },
                "required": ["title", "task", "from_agent"]
            }
        },
        {
            "name": "memory_handoff_begin",
            "description": "Alias for memory_handoff_offer.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Handoff title" },
                    "task": { "type": "string", "description": "Handoff task" },
                    "from_agent": { "type": "string", "description": "From agent" }
                },
                "required": ["title", "task", "from_agent"]
            }
        },
        {
            "name": "memory_handoff_list",
            "description": "List active multi-agent handoffs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "agent": { "type": "string", "description": "Filter by target agent" }
                }
            }
        },
        {
            "name": "memory_handoff_claim",
            "description": "Atomically claim an offered handoff (claim-once state transition).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff ID" },
                    "agent": { "type": "string", "description": "Claiming agent identifier" }
                },
                "required": ["id", "agent"]
            }
        },
        {
            "name": "memory_handoff_accept",
            "description": "Alias for memory_handoff_claim.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff ID" },
                    "agent": { "type": "string", "description": "Claiming agent" }
                },
                "required": ["id", "agent"]
            }
        },
        {
            "name": "memory_handoff_resolve",
            "description": "Mark a claimed handoff as resolved with optional resolution notes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff ID" },
                    "agent": { "type": "string", "description": "Resolving agent" },
                    "note": { "type": "string", "description": "Resolution note" }
                },
                "required": ["id", "agent"]
            }
        },
        {
            "name": "memory_handoff_cancel",
            "description": "Cancel a pending or claimed handoff.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Handoff ID" },
                    "reason": { "type": "string", "description": "Cancellation reason" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "memory_message_send",
            "description": "Send a directed cross-project or cross-agent message to an inbox.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sender_project": { "type": "string", "description": "Sender project name (default: current)" },
                    "sender_agent": { "type": "string", "description": "Sender agent identifier" },
                    "recipient_project": { "type": "string", "description": "Recipient project name" },
                    "recipient_agent": { "type": "string", "description": "Target agent or 'any'" },
                    "body": { "type": "string", "description": "Message content" },
                    "nature": { "type": "string", "description": "measured | reported | inferred | unknown" },
                    "evidence": { "type": "string", "description": "Supporting evidence if measured" }
                },
                "required": ["recipient_project", "body"]
            }
        },
        {
            "name": "memory_message_list",
            "description": "List messages in an inbox or outbox.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "recipient_project": { "type": "string", "description": "Filter by recipient project" },
                    "state": { "type": "string", "description": "pending | claimed | cancelled" }
                }
            }
        },
        {
            "name": "memory_message_pop",
            "description": "Atomically claim and pop the oldest pending message for an agent.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "recipient_project": { "type": "string", "description": "Recipient project coordinate" },
                    "recipient_agent": { "type": "string", "description": "Popping agent identifier" }
                },
                "required": ["recipient_project", "recipient_agent"]
            }
        },
        {
            "name": "memory_message_cancel",
            "description": "Cancel a pending message before it is popped.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Message ID" },
                    "sender_agent": { "type": "string", "description": "Sender agent identifier" }
                },
                "required": ["id", "sender_agent"]
            }
        },
        {
            "name": "memory_status",
            "description": "Get summary telemetry and storage status of ai-memory wiki, handoffs, and messages.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "memory_briefing",
            "description": "Generate a progressive disclosure briefing (L0/L1/L2) based on headroom budget.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "token_budget": { "type": "integer", "description": "Max token budget (default: 600)" }
                }
            }
        },
        {
            "name": "memory_explore",
            "description": "Explore knowledge topics, concept structures, and active handoffs.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

pub fn handle_aimemory_tool_call(name: &str, args: &Value) -> (String, bool) {
    let wiki = WikiStore::open_default();
    let handoff = HandoffStore::open_default();
    let message = MessageStore::open_default();

    match name {
        "memory_query" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

            let mut matched_pages = Vec::new();
            if let Ok(pages) = wiki.list_pages() {
                for p in pages {
                    if p.slug.to_lowercase().contains(&query)
                        || p.title.to_lowercase().contains(&query)
                        || p.body.to_lowercase().contains(&query)
                        || p.tags.iter().any(|t| t.to_lowercase().contains(&query))
                    {
                        matched_pages.push(p);
                        if matched_pages.len() >= limit {
                            break;
                        }
                    }
                }
            }

            let mut matched_handoffs = Vec::new();
            if let Ok(handoffs) = handoff.list_active() {
                for h in handoffs {
                    if h.title.to_lowercase().contains(&query) || h.task.to_lowercase().contains(&query) {
                        matched_handoffs.push(h);
                        if matched_handoffs.len() >= limit {
                            break;
                        }
                    }
                }
            }

            let res = json!({
                "pages": matched_pages,
                "handoffs": matched_handoffs
            });
            (serde_json::to_string_pretty(&res).unwrap_or_default(), false)
        }
        "memory_recent" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let pages = wiki.list_pages().unwrap_or_default();
            let handoffs = handoff.list_active().unwrap_or_default();
            let messages = message.list(None, None).unwrap_or_default();

            let res = json!({
                "recent_pages": pages.into_iter().take(limit).collect::<Vec<_>>(),
                "active_handoffs": handoffs.into_iter().take(limit).collect::<Vec<_>>(),
                "recent_messages": messages.into_iter().rev().take(limit).collect::<Vec<_>>()
            });
            (serde_json::to_string_pretty(&res).unwrap_or_default(), false)
        }
        "memory_write_page" | "memory_wiki_write" => {
            let slug = match args.get("slug").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s,
                _ => return ("Error: 'slug' is required".to_string(), true),
            };
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or(slug);
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let doc_type = args.get("doc_type").and_then(|v| v.as_str()).map(String::from);
            let abstract_text = args.get("abstract_text").and_then(|v| v.as_str()).map(String::from);
            let summary_text = args.get("summary_text").and_then(|v| v.as_str()).map(String::from);
            let tags = args
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let page = WikiPage {
                slug: slug.to_string(),
                title: title.to_string(),
                doc_type,
                abstract_text,
                summary_text,
                tags,
                body: body.to_string(),
            };

            match wiki.write_page(&page) {
                Ok(_) => (format!("Wrote wiki page '{}'", slug), false),
                Err(e) => (format!("Error writing wiki page: {e}"), true),
            }
        }
        "memory_read_page" | "memory_wiki_read" => {
            let slug = match args.get("slug").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ("Error: 'slug' is required".to_string(), true),
            };

            match wiki.read_page(slug) {
                Ok(Some(page)) => (serde_json::to_string_pretty(&page).unwrap_or_default(), false),
                Ok(None) => (format!("Page '{}' not found", slug), true),
                Err(e) => (format!("Error reading wiki page: {e}"), true),
            }
        }
        "memory_wiki_list" => {
            match wiki.list_pages() {
                Ok(pages) => (serde_json::to_string_pretty(&pages).unwrap_or_default(), false),
                Err(e) => (format!("Error listing wiki pages: {e}"), true),
            }
        }
        "memory_patch_page" => {
            let slug = match args.get("slug").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ("Error: 'slug' is required".to_string(), true),
            };
            let patch_text = args.get("patch_text").and_then(|v| v.as_str()).unwrap_or("");
            let append = args.get("append").and_then(|v| v.as_bool()).unwrap_or(true);

            match wiki.patch_page(slug, patch_text, append) {
                Ok(page) => (serde_json::to_string_pretty(&page).unwrap_or_default(), false),
                Err(e) => (format!("Error patching page: {e}"), true),
            }
        }
        "memory_delete_page" => {
            let slug = match args.get("slug").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ("Error: 'slug' is required".to_string(), true),
            };

            match wiki.delete_page(slug) {
                Ok(true) => (format!("Deleted page '{}'", slug), false),
                Ok(false) => (format!("Page '{}' did not exist", slug), false),
                Err(e) => (format!("Error deleting page: {e}"), true),
            }
        }
        "memory_restore_page" => {
            let slug = match args.get("slug").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ("Error: 'slug' is required".to_string(), true),
            };
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or(slug);

            let page = WikiPage {
                slug: slug.to_string(),
                title: title.to_string(),
                doc_type: Some("Concept".into()),
                abstract_text: None,
                summary_text: None,
                tags: Vec::new(),
                body: String::new(),
            };

            match wiki.write_page(&page) {
                Ok(_) => (format!("Restored empty page '{}'", slug), false),
                Err(e) => (format!("Error restoring page: {e}"), true),
            }
        }
        "memory_handoff_offer" | "memory_handoff_begin" => {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("");
            let from_agent = args.get("from_agent").and_then(|v| v.as_str()).unwrap_or("antigravity");
            if title.is_empty() || task.is_empty() {
                return ("Error: 'title' and 'task' are required".to_string(), true);
            }

            let to_agent = args.get("to_agent").and_then(|v| v.as_str()).map(String::from);
            let active_files = args
                .get("active_files")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let unresolved_issues = args
                .get("unresolved_issues")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let next_steps = args
                .get("next_steps")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let context_notes = args.get("context_notes").and_then(|v| v.as_str()).map(String::from);

            let new_h = NewHandoff {
                title: title.to_string(),
                task: task.to_string(),
                from_agent: from_agent.to_string(),
                to_agent,
                active_files,
                unresolved_issues,
                next_steps,
                context_notes,
            };

            match handoff.offer(new_h) {
                Ok(h) => (serde_json::to_string_pretty(&h).unwrap_or_default(), false),
                Err(e) => (format!("Error offering handoff: {e}"), true),
            }
        }
        "memory_handoff_list" => {
            let agent_filter = args.get("agent").and_then(|v| v.as_str());
            let list_res = match agent_filter {
                Some(a) => handoff.pending_for_agent(a),
                None => handoff.list_active(),
            };

            match list_res {
                Ok(list) => (serde_json::to_string_pretty(&list).unwrap_or_default(), false),
                Err(e) => (format!("Error listing handoffs: {e}"), true),
            }
        }
        "memory_handoff_claim" | "memory_handoff_accept" => {
            let id = match args.get("id").and_then(|v| v.as_str()) {
                Some(i) => i,
                None => return ("Error: 'id' is required".to_string(), true),
            };
            let agent = match args.get("agent").and_then(|v| v.as_str()) {
                Some(a) => a,
                None => return ("Error: 'agent' is required".to_string(), true),
            };

            match handoff.claim(id, agent) {
                Ok(h) => (serde_json::to_string_pretty(&h).unwrap_or_default(), false),
                Err(e) => (format!("Error claiming handoff: {e}"), true),
            }
        }
        "memory_handoff_resolve" => {
            let id = match args.get("id").and_then(|v| v.as_str()) {
                Some(i) => i,
                None => return ("Error: 'id' is required".to_string(), true),
            };
            let agent = match args.get("agent").and_then(|v| v.as_str()) {
                Some(a) => a,
                None => return ("Error: 'agent' is required".to_string(), true),
            };
            let note = args.get("note").and_then(|v| v.as_str());

            match handoff.resolve(id, agent, note) {
                Ok(h) => (serde_json::to_string_pretty(&h).unwrap_or_default(), false),
                Err(e) => (format!("Error resolving handoff: {e}"), true),
            }
        }
        "memory_handoff_cancel" => {
            let id = match args.get("id").and_then(|v| v.as_str()) {
                Some(i) => i,
                None => return ("Error: 'id' is required".to_string(), true),
            };
            let reason = args.get("reason").and_then(|v| v.as_str());

            match handoff.cancel(id, reason) {
                Ok(h) => (serde_json::to_string_pretty(&h).unwrap_or_default(), false),
                Err(e) => (format!("Error cancelling handoff: {e}"), true),
            }
        }
        "memory_message_send" => {
            let recipient_project = match args.get("recipient_project").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return ("Error: 'recipient_project' is required".to_string(), true),
            };
            let body = match args.get("body").and_then(|v| v.as_str()) {
                Some(b) => b,
                None => return ("Error: 'body' is required".to_string(), true),
            };
            let sender_project = args.get("sender_project").and_then(|v| v.as_str()).unwrap_or("current");
            let sender_agent = args.get("sender_agent").and_then(|v| v.as_str()).unwrap_or("antigravity");
            let recipient_agent = args.get("recipient_agent").and_then(|v| v.as_str()).map(String::from);
            let nature = match args.get("nature").and_then(|v| v.as_str()) {
                Some("measured") => MessageNature::Measured,
                Some("reported") => MessageNature::Reported,
                Some("inferred") => MessageNature::Inferred,
                _ => MessageNature::Unknown,
            };
            let evidence = args.get("evidence").and_then(|v| v.as_str()).map(String::from);

            let new_m = NewMessage {
                sender_project: sender_project.to_string(),
                sender_agent: sender_agent.to_string(),
                recipient_project: recipient_project.to_string(),
                recipient_agent,
                body: body.to_string(),
                nature,
                evidence,
            };

            match message.send(new_m) {
                Ok(m) => (serde_json::to_string_pretty(&m).unwrap_or_default(), false),
                Err(e) => (format!("Error sending message: {e}"), true),
            }
        }
        "memory_message_list" => {
            let rp = args.get("recipient_project").and_then(|v| v.as_str());
            let st = args.get("state").and_then(|v| v.as_str()).and_then(|s| match s {
                "pending" => Some(MessageState::Pending),
                "claimed" => Some(MessageState::Claimed),
                "cancelled" => Some(MessageState::Cancelled),
                _ => None,
            });

            match message.list(rp, st) {
                Ok(list) => (serde_json::to_string_pretty(&list).unwrap_or_default(), false),
                Err(e) => (format!("Error listing messages: {e}"), true),
            }
        }
        "memory_message_pop" => {
            let rp = match args.get("recipient_project").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return ("Error: 'recipient_project' is required".to_string(), true),
            };
            let ra = match args.get("recipient_agent").and_then(|v| v.as_str()) {
                Some(a) => a,
                None => return ("Error: 'recipient_agent' is required".to_string(), true),
            };

            match message.pop(rp, ra) {
                Ok(Some(m)) => (serde_json::to_string_pretty(&m).unwrap_or_default(), false),
                Ok(None) => ("No pending messages".to_string(), false),
                Err(e) => (format!("Error popping message: {e}"), true),
            }
        }
        "memory_message_cancel" => {
            let id = match args.get("id").and_then(|v| v.as_str()) {
                Some(i) => i,
                None => return ("Error: 'id' is required".to_string(), true),
            };
            let sa = match args.get("sender_agent").and_then(|v| v.as_str()) {
                Some(a) => a,
                None => return ("Error: 'sender_agent' is required".to_string(), true),
            };

            match message.cancel(id, sa) {
                Ok(m) => (serde_json::to_string_pretty(&m).unwrap_or_default(), false),
                Err(e) => (format!("Error cancelling message: {e}"), true),
            }
        }
        "memory_status" => {
            let page_count = wiki.list_pages().map(|p| p.len()).unwrap_or(0);
            let handoff_count = handoff.list_active().map(|h| h.len()).unwrap_or(0);
            let msg_pending_count = message
                .list(None, Some(MessageState::Pending))
                .map(|m| m.len())
                .unwrap_or(0);

            let status = json!({
                "wiki_pages": page_count,
                "active_handoffs": handoff_count,
                "pending_messages": msg_pending_count,
                "wiki_dir": wiki.root_dir().to_string_lossy(),
                "handoff_dir": handoff.base_dir().to_string_lossy(),
                "message_dir": message.base_dir().to_string_lossy()
            });
            (serde_json::to_string_pretty(&status).unwrap_or_default(), false)
        }
        "memory_briefing" => {
            let token_budget = args.get("token_budget").and_then(|v| v.as_u64()).unwrap_or(600) as usize;
            let entries = wiki.to_tiered_entries().unwrap_or_default();
            let packed = TieredPacker::pack(&entries, token_budget);
            (serde_json::to_string_pretty(&packed).unwrap_or_default(), false)
        }
        "memory_explore" => {
            let pages = wiki.list_pages().unwrap_or_default();
            let handoffs = handoff.list_active().unwrap_or_default();
            let res = json!({
                "concepts": pages.iter().map(|p| &p.title).collect::<Vec<_>>(),
                "handoffs": handoffs.iter().map(|h| &h.title).collect::<Vec<_>>()
            });
            (serde_json::to_string_pretty(&res).unwrap_or_default(), false)
        }
        _ => (format!("Unknown tool: {name}"), true),
    }
}

pub fn run_mcp_server() -> i32 {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        if let Ok(req) = serde_json::from_str::<Value>(trimmed) {
            let id = req.get("id").cloned();
            let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");

            match method {
                "initialize" => {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "protocolVersion": "2024-11-05",
                            "capabilities": { "tools": {} },
                            "serverInfo": {
                                "name": "toksi-aimemory-mcp",
                                "version": env!("CARGO_PKG_VERSION")
                            }
                        }
                    });
                    let _ = writeln!(stdout, "{}", resp);
                    let _ = stdout.flush();
                }
                "notifications/initialized" => {}
                "tools/list" => {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "tools": tool_definitions()
                        }
                    });
                    let _ = writeln!(stdout, "{}", resp);
                    let _ = stdout.flush();
                }
                "tools/call" => {
                    let params = req.get("params").cloned().unwrap_or(Value::Null);
                    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let args = params.get("arguments").cloned().unwrap_or(json!({}));

                    let (content, is_error) = handle_aimemory_tool_call(name, &args);

                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": content }],
                            "isError": is_error
                        }
                    });
                    let _ = writeln!(stdout, "{}", resp);
                    let _ = stdout.flush();
                }
                _ => {
                    if let Some(req_id) = id {
                        let resp = json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "error": {
                                "code": -32601,
                                "message": format!("Method not found: {method}")
                            }
                        });
                        let _ = writeln!(stdout, "{}", resp);
                        let _ = stdout.flush();
                    }
                }
            }
        }

        line.clear();
    }

    0
}
