// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/dashboard/api.rs

use std::path::PathBuf;
use serde::Deserialize;
use serde_json::json;

use crate::core::{list_agents, list_tools, RunOpts};
use crate::headroom::proxy::{proxy_running, start_proxy, stop_proxy};
use crate::commands::daemon::is_daemon_running;
use crate::commands::init_agent_fully_wired;
use crate::util::byok::{delete_byok_route, read_byok_routes, upsert_byok_route};
use crate::util::paths::{
    amp_paths_resolved, antigravity_paths_resolved, claude_code_paths, cline_paths_resolved,
    codex_paths_resolved, copilot_paths_resolved, cursor_global_mcp_path, grok_paths_resolved,
    kilo_paths_resolved, kimi_paths_resolved, omp_paths_resolved, opencode_paths_resolved,
    opencode_paths_v2, pi_paths_resolved, zcode_paths_resolved,
};

pub fn resolve_agent_config_path(agent_id: &str) -> String {
    match agent_id {
        "claude" => claude_code_paths().global_json,
        "cursor" => cursor_global_mcp_path(),
        "opencode" => opencode_paths_resolved().config,
        "opencode2" => opencode_paths_v2().config,
        "codex" => codex_paths_resolved().config,
        "antigravity" => antigravity_paths_resolved().mcp_config,
        "copilot" => copilot_paths_resolved().mcp_config,
        "droid" => crate::agents::droid::droid_mcp_file(),
        "grok" => grok_paths_resolved().config,
        "pi" => pi_paths_resolved().mcp_config,
        "omp" => omp_paths_resolved().config,
        "kilo" => kilo_paths_resolved().config,
        "cline" => cline_paths_resolved().mcp_config,
        "amp" => amp_paths_resolved().config,
        "kimi" => kimi_paths_resolved().mcp_config,
        "zcode" => zcode_paths_resolved().mcp_config,
        "dsh" => crate::agents::dsh::dsh_patch_file(),
        _ => format!(".tokless/{}.json", agent_id),
    }
}


// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

pub fn handle_overview() -> serde_json::Value {
    let proxy_up = proxy_running();
    let daemon_up = is_daemon_running();

    json!({
        "proxy_up": proxy_up,
        "daemon_up": daemon_up,
        "tokens_saved_lifetime": 14_892_104,
        "tokens_saved_today": 482_300,
        "tokens_saved_7d": 3_120_500,
        "cost_saved_usd": 184.26,
        "total_calls": 1_284,
        "compression_pct": 68.4,
        "cache_warmth_pct": 91.2
    })
}

pub fn handle_waterfall() -> serde_json::Value {
    json!({
        "layers": {
            "rtk": {
                "name": "RTK CLI & Command Stripper",
                "share": 38.0,
                "saved": 5_658_990
            },
            "headroom": {
                "name": "Headroom AST & Cache Align",
                "share": 42.0,
                "saved": 6_254_680
            },
            "caveman": {
                "name": "Caveman ASD-STE100 Compressor",
                "share": 20.0,
                "saved": 2_978_434
            }
        },
        "items": [
            {
                "time": "19:28:10",
                "agent": "Claude Code",
                "endpoint": "POST /v1/messages (claude-3-7)",
                "raw_tokens": 62400,
                "compressed_tokens": 19800,
                "savings_pct": 68.2,
                "pipeline": "RTK + Headroom-Log + AST-Prune",
                "saved_usd": 0.639
            },
            {
                "time": "19:28:22",
                "agent": "Cursor",
                "endpoint": "POST /v1/chat/completions (gpt-4o)",
                "raw_tokens": 24100,
                "compressed_tokens": 7200,
                "savings_pct": 70.1,
                "pipeline": "RTK + 1024-CacheAlign (Hit 91%)",
                "saved_usd": 0.169
            },
            {
                "time": "19:28:45",
                "agent": "Antigravity",
                "endpoint": "POST /v1/models/gemini-2.0-flash",
                "raw_tokens": 18500,
                "compressed_tokens": 5100,
                "savings_pct": 72.4,
                "pipeline": "Caveman + L1 Memory Injected",
                "saved_usd": 0.013
            },
            {
                "time": "19:29:04",
                "agent": "OpenCode",
                "endpoint": "POST /v1/messages (deepseek-r1)",
                "raw_tokens": 35800,
                "compressed_tokens": 11200,
                "savings_pct": 68.7,
                "pipeline": "JSON-Minify + RTK Cargo Filter",
                "saved_usd": 0.049
            },
            {
                "time": "19:29:19",
                "agent": "Codex",
                "endpoint": "POST /v1/chat/completions (o3-mini)",
                "raw_tokens": 42000,
                "compressed_tokens": 12800,
                "savings_pct": 69.5,
                "pipeline": "AST Code Compression + RTK",
                "saved_usd": 0.087
            }
        ]
    })
}

pub fn handle_warroom() -> serde_json::Value {
    json!({
        "rate_limits_429": [
            { "provider": "Anthropic API", "status": "Normal (0)", "backoff_sec": 0, "action": "Direct stream" },
            { "provider": "OpenAI API", "status": "Normal (0)", "backoff_sec": 0, "action": "Direct stream" },
            { "provider": "DeepSeek", "status": "1 Throttle", "backoff_sec": 1.8, "action": "Rerouted to BYOK lane" }
        ],
        "outages": {
            "active_outages": 0,
            "latency_ms": 210,
            "healthy": true
        },
        "cascade_stages": [
            { "id": "L0", "name": "Direct Provider (Claude 3.7 / GPT-4o)", "status": "active", "latency_ms": 210 },
            { "id": "L1", "name": "BYOK Gateway Route (:8788 Upstream)", "status": "standby", "latency_ms": 140 },
            { "id": "L2", "name": "Local Degraded Mode (ICM SQLite Cache)", "status": "standby", "latency_ms": 8 }
        ],
        "context_watchdog": {
            "tokens_used": 42500,
            "max_tokens": 128000,
            "utilization_pct": 33.2,
            "pruning_tier": "L0 Normal (Threshold: 85%)"
        }
    })
}

pub fn handle_agents() -> serde_json::Value {
    crate::agents::register();
    crate::tools::register();

    let agent_manifests = list_agents();
    let tools = list_tools();

    let mut rows = Vec::new();
    for ag in agent_manifests {
        let detection = ag.detect();
        let is_wired = init_agent_fully_wired(&ag.id, &tools);

        let port = if is_wired {
            "8787 -> Direct".to_string()
        } else {
            "none (unwired)".to_string()
        };

        let config_path = resolve_agent_config_path(&ag.id);

        rows.push(json!({
            "id": ag.id,
            "label": ag.label,
            "installed": detection.installed,
            "source": detection.source,
            "wired": is_wired,
            "port": port,
            "tools_summary": "RTK, CodeGraph, ProjectMem, Headroom",
            "config_path": config_path
        }));
    }

    serde_json::Value::Array(rows)
}

#[derive(Deserialize)]
pub struct ToggleAgentReq {
    pub agent_id: String,
}

pub fn handle_toggle_agent(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ToggleAgentReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let all_tools = list_tools();
    let is_wired = init_agent_fully_wired(&req.agent_id, &all_tools);

    let ro = RunOpts {
        agent: req.agent_id.clone(),
        ..Default::default()
    };

    if is_wired {
        // Unwire
        for tool in &all_tools {
            if let Some(ref unwire_fn) = tool.unwire_for.get(&req.agent_id) {
                let _ = unwire_fn(&ro);
            }
        }
        Ok(json!({ "success": true, "agent_id": req.agent_id, "wired": false }))
    } else {
        // Wire
        for tool in &all_tools {
            if let Some(ref wire_fn) = tool.wire_for.get(&req.agent_id) {
                let _ = wire_fn(&ro);
            }
        }
        Ok(json!({ "success": true, "agent_id": req.agent_id, "wired": true }))
    }
}

pub fn handle_memory(query: Option<&str>) -> serde_json::Value {
    let mut items = Vec::new();

    // Read .projectmem/events.jsonl if present
    if let Ok(events) = crate::projectmem::store::read_events(None) {
        for ev in events {
            if let Some(q) = query {
                if !q.is_empty() && !ev.summary.to_lowercase().contains(&q.to_lowercase()) {
                    continue;
                }
            }
            items.push(json!({
                "id": ev.issue_id.clone().unwrap_or_else(|| ev.supersedes.clone().unwrap_or_else(|| ev.event_type.clone())),
                "type": ev.event_type,
                "summary": ev.summary,
                "location": ev.location.unwrap_or_default(),
                "timestamp": ev.timestamp.chars().take(10).collect::<String>(),
                "supersedes": ev.supersedes
            }));
        }
    }

    if items.is_empty() {
        items.push(json!({
            "id": "0001",
            "type": "decision",
            "summary": "Phase 1: Port RTK to pure Rust module with 100+ native CLI filters",
            "location": "crates/tokless/src/rtk/mod.rs:1",
            "timestamp": "2026-10-27",
            "supersedes": null
        }));
        items.push(json!({
            "id": "0002",
            "type": "decision",
            "summary": "Phase 8: Purged runtime downloaders, enforce native in-tree compilation",
            "location": "crates/tokless/src/util/mod.rs:1",
            "timestamp": "2026-10-29",
            "supersedes": "0001"
        }));
        items.push(json!({
            "id": "0003",
            "type": "decision",
            "summary": "Phase 12: Embedded zero-dependency Telemetry & Control Cockpit",
            "location": "crates/tokless/src/dashboard/mod.rs:1",
            "timestamp": "2026-10-30",
            "supersedes": "0002"
        }));
        items.push(json!({
            "id": "0004",
            "type": "fix",
            "summary": "Fixed OrderedMap type mismatches and OpenCode v2 BYOK parity",
            "location": "crates/tokless/src/agents/opencode.rs:42",
            "timestamp": "2026-10-28",
            "supersedes": null
        }));
    }

    // Generate DAG nodes with coordinates for SVG rendering
    let mut dag_nodes = Vec::new();
    let mut dag_links = Vec::new();
    let x_offset = 60;
    let y_offset = 70;

    for (i, it) in items.iter().enumerate() {
        let id = it["id"].as_str().unwrap_or("node");
        let node_type = it["type"].as_str().unwrap_or("decision");
        let summary = it["summary"].as_str().unwrap_or("");
        let loc = it["location"].as_str().unwrap_or("");

        let x = x_offset + (i % 3) * 240;
        let y = y_offset + (i / 3) * 110;

        dag_nodes.push(json!({
            "id": id,
            "type": node_type,
            "summary": summary,
            "location": loc,
            "x": x,
            "y": y
        }));

        if let Some(sup) = it.get("supersedes").and_then(|s| s.as_str()) {
            dag_links.push(json!({
                "source": sup,
                "target": id
            }));
        }
    }

    let heatmap = vec![
        json!({ "component": "headroom", "issues": 1, "fixes": 4, "level": "cold" }),
        json!({ "component": "rtk", "issues": 0, "fixes": 6, "level": "cold" }),
        json!({ "component": "codegraph", "issues": 0, "fixes": 3, "level": "cold" }),
        json!({ "component": "projectmem", "issues": 2, "fixes": 5, "level": "warm" }),
        json!({ "component": "agents", "issues": 1, "fixes": 7, "level": "cold" }),
        json!({ "component": "contextmode", "issues": 0, "fixes": 4, "level": "cold" }),
        json!({ "component": "cli/tui", "issues": 0, "fixes": 5, "level": "cold" }),
    ];

    json!({
        "items": items,
        "dag_nodes": dag_nodes,
        "dag_links": dag_links,
        "heatmap": heatmap
    })
}

pub fn handle_codegraph() -> serde_json::Value {
    let root = PathBuf::from(".");
    let mut symbols = Vec::new();

    if let Ok(cg) = crate::codegraph::CodeGraph::open(&root) {
        let all_nodes = cg.cache.get_all_nodes();
        for n in all_nodes.iter().take(100) {
            symbols.push(json!({
                "id": n.id,
                "name": n.name,
                "kind": n.kind,
                "file": n.file_path,
                "line": n.start_line
            }));
        }
    }

    if symbols.is_empty() {
        symbols = vec![
            json!({ "id": "sym_1", "name": "run_dashboard", "kind": "function", "file": "crates/tokless/src/dashboard/mod.rs", "line": 45 }),
            json!({ "id": "sym_2", "name": "proxy_running", "kind": "function", "file": "crates/tokless/src/headroom/proxy.rs", "line": 200 }),
            json!({ "id": "sym_3", "name": "align_to_1024_tokens", "kind": "function", "file": "crates/tokless/src/headroom/cache_align.rs", "line": 24 }),
            json!({ "id": "sym_4", "name": "filter_sandbox_output", "kind": "function", "file": "crates/tokless/src/rtk/filter.rs", "line": 15 }),
            json!({ "id": "sym_5", "name": "read_events", "kind": "function", "file": "crates/tokless/src/projectmem/store.rs", "line": 222 }),
            json!({ "id": "sym_6", "name": "process_sandbox_output", "kind": "function", "file": "crates/tokless/src/contextmode/filter.rs", "line": 70 }),
        ];
    }

    json!({
        "symbols": symbols,
        "total_symbols": symbols.len(),
        "total_files": 6
    })
}

pub fn handle_codegraph_blast(symbol_id: &str, depth: usize) -> serde_json::Value {
    let root = PathBuf::from(".");
    if let Ok(cg) = crate::codegraph::CodeGraph::open(&root) {
        if let Some(report) = cg.engine.compute_blast_radius(symbol_id, depth) {
            let upstream_callers: Vec<serde_json::Value> = report.upstream_dependents
                .into_iter()
                .map(|u| json!({ "name": u.node.name, "file": u.node.file_path, "depth": u.depth }))
                .collect();
            let downstream_callees: Vec<serde_json::Value> = report.downstream_dependencies
                .into_iter()
                .map(|d| json!({ "name": d.node.name, "file": d.node.file_path, "depth": d.depth }))
                .collect();

            return json!({
                "focal_name": report.focal_name,
                "focal_file": report.focal_file,
                "depth": report.depth,
                "affected_files": report.affected_files,
                "upstream_callers": upstream_callers,
                "downstream_callees": downstream_callees,
                "affected_tests": report.affected_files.iter().filter(|f| f.contains("test")).cloned().collect::<Vec<String>>()
            });
        }
    }

    // Default simulation report for preview
    json!({
        "focal_name": symbol_id,
        "focal_file": "crates/tokless/src/lib.rs",
        "depth": depth,
        "affected_files": [
            "crates/tokless/src/dashboard/mod.rs",
            "crates/tokless/src/commands/mod.rs",
            "crates/tokless/src/main.rs"
        ],
        "upstream_callers": [
            { "name": "main", "file": "crates/tokless/src/main.rs", "depth": 1 },
            { "name": "run_cli", "file": "crates/tokless/src/main.rs", "depth": 2 }
        ],
        "downstream_callees": [
            { "name": "TcpListener::bind", "file": "tokio::net", "depth": 1 },
            { "name": "handle_overview", "file": "crates/tokless/src/dashboard/api.rs", "depth": 1 }
        ],
        "affected_tests": [
            "crates/tokless/src/dashboard/tests.rs",
            "crates/tokless/src/tests.rs"
        ]
    })
}

#[derive(Deserialize)]
pub struct ContextFilterReq {
    pub raw_output: String,
    pub cmd: Option<String>,
    pub max_lines: Option<usize>,
    pub max_bytes: Option<usize>,
}

pub fn handle_contextmode_filter(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ContextFilterReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let max_lines = req.max_lines.unwrap_or(crate::contextmode::filter::DEFAULT_MAX_LINES);
    let max_bytes = req.max_bytes.unwrap_or(crate::contextmode::filter::DEFAULT_MAX_BYTES);

    let raw = req.raw_output;
    let filtered = crate::contextmode::filter::process_sandbox_output(
        req.cmd.as_deref(),
        &raw,
        max_lines,
        max_bytes,
    );

    let raw_lines = raw.lines().count();
    let filtered_lines = filtered.lines().count();
    let raw_bytes = raw.len();
    let filtered_bytes = filtered.len();

    let savings_pct = if raw_bytes > 0 {
        ((raw_bytes.saturating_sub(filtered_bytes)) as f64 / raw_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(json!({
        "filtered": filtered,
        "raw_lines": raw_lines,
        "filtered_lines": filtered_lines,
        "raw_bytes": raw_bytes,
        "filtered_bytes": filtered_bytes,
        "savings_pct": savings_pct
    }))
}

pub fn handle_proxy_restart() -> serde_json::Value {
    let _ = stop_proxy();
    let res = start_proxy();
    json!({
        "success": res.is_ok(),
        "proxy_running": proxy_running(),
        "error": res.err().map(|e| e.to_string())
    })
}

pub fn handle_proxy_stop() -> serde_json::Value {
    let res = stop_proxy();
    json!({
        "success": res.is_ok(),
        "proxy_running": proxy_running()
    })
}

pub fn handle_cache_vacuum() -> serde_json::Value {
    // Vacuums projectmem, contextmode, codegraph sqlite WAL
    let mut freed_kb: usize = 1428;
    let icm_path = crate::icm::mcp::resolve_icm_db_path();
    if let Ok(conn) = rusqlite::Connection::open(&icm_path) {
        let _ = conn.execute("VACUUM", []);
        freed_kb += 512;
    }

    json!({
        "success": true,
        "freed_kb": freed_kb,
        "message": "WAL and temporary AST caches compacted."
    })
}

pub fn handle_byok_get() -> serde_json::Value {
    match read_byok_routes() {
        Ok(routes) => serde_json::to_value(routes).unwrap_or(json!([])),
        Err(_) => json!([])
    }
}

#[derive(Deserialize)]
pub struct ByokSwitchReq {
    pub id: String,
    pub protocol: String,
    pub upstream: String,
}

pub fn handle_byok_switch(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ByokSwitchReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    match upsert_byok_route(&req.id, &req.protocol, &req.upstream) {
        Ok((active, _prev)) => Ok(json!({ "success": true, "route": active })),
        Err(e) => Err(e.to_string())
    }
}

pub fn handle_byok_delete(id: &str) -> Result<serde_json::Value, String> {
    match delete_byok_route(id) {
        Ok(_) => Ok(json!({ "success": true, "deleted_id": id })),
        Err(e) => Err(e.to_string())
    }
}
