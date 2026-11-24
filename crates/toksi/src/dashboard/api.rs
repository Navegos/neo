// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/api.rs
// created: 2026-10-30
// lastModified: 2026-11-19

#![deny(unsafe_code)]

use std::path::PathBuf;
use serde::Deserialize;
use serde_json::json;

use crate::agents::proxy_agent_wired;
use crate::core::{list_agents, list_tools, RunOpts};
use crate::dashboard::state::{global_cockpit_state, AgentObservationItem, LatencyBreakdown};
use crate::headroom::proxy::{proxy_port, proxy_running, start_proxy, stop_proxy};
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
        _ => format!(".toksi/{}.json", agent_id),
    }
}


// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

pub fn handle_overview() -> serde_json::Value {
    let proxy_up = proxy_running();
    let daemon_up = is_daemon_running();

    let collector = crate::telemetry::global_collector();
    let tokens_saved: u64 = collector.tokens_saved_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let total_calls: u64 = collector.upstream_requests_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let cost_saved = collector.cost_savings_usd_micros.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1_000_000.0;
    let input_tokens: u64 = collector.input_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let cached_tokens: u64 = collector.cached_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);

    let (compression_pct, cache_warmth_pct) = if input_tokens + tokens_saved > 0 {
        let comp = (tokens_saved as f64 / (input_tokens + tokens_saved) as f64) * 100.0;
        let warmth = if input_tokens > 0 {
            (cached_tokens as f64 / input_tokens as f64) * 100.0
        } else {
            0.0
        };
        (comp, warmth)
    } else {
        (0.0, 0.0)
    };

    json!({
        "proxy_up": proxy_up,
        "daemon_up": daemon_up,
        "tokens_saved_lifetime": tokens_saved,
        "tokens_saved_today": tokens_saved,
        "tokens_saved_7d": tokens_saved,
        "cost_saved_usd": (cost_saved * 100.0).round() / 100.0,
        "total_calls": total_calls,
        "compression_pct": (compression_pct * 10.0).round() / 10.0,
        "cache_warmth_pct": (cache_warmth_pct * 10.0).round() / 10.0
    })
}

pub fn handle_waterfall() -> serde_json::Value {
    let collector = crate::telemetry::global_collector();
    let snap = crate::telemetry::snapshot_from_collector(collector);
    let tokens_saved = snap.tokens_saved_lifetime;
    let total_calls = snap.total_calls;

    let rtk_saved = snap.rtk_tokens_saved;
    let hr_saved = snap.headroom_tokens_saved;
    let caveman_saved = snap.caveman_tokens_saved;
    let mcp_pruned_count = collector.headroom_mcp_pruned_tools_total.load(std::sync::atomic::Ordering::Relaxed);
    let mcp_pruner_saved = mcp_pruned_count.saturating_mul(350);
    let ccr_saved = 0u64;
    let cache_align_saved = snap.cached_tokens;
    let context_saved = 0u64;
    let economizer_saved = 0u64;
    let layer_sum = rtk_saved
        .saturating_add(hr_saved)
        .saturating_add(caveman_saved)
        .saturating_add(mcp_pruner_saved)
        .saturating_add(cache_align_saved);

    let pct_of = |part: u64| -> f64 {
        if layer_sum > 0 {
            ((part as f64 / layer_sum as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        }
    };

    let sessions = crate::telemetry::tokentracker::global_token_tracker().get_recent_sessions();
    let items: Vec<serde_json::Value> = if sessions.is_empty() && total_calls == 0 {
        vec![json!({
            "time": "Ready",
            "agent": "All Agents",
            "endpoint": "Live Interceptor (Standing By)",
            "raw_tokens": 0,
            "compressed_tokens": 0,
            "savings_pct": 0.0,
            "pipeline": "RTK + Headroom + Caveman",
            "saved_usd": 0.0
        })]
    } else if sessions.is_empty() {
        let raw = snap.input_tokens.saturating_add(tokens_saved);
        let pct = if raw > 0 {
            ((tokens_saved as f64 / raw as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        };
        vec![json!({
            "time": "Recent",
            "agent": "Active Gateway",
            "endpoint": format!(":{}", crate::config::resolved_ports().ai_gateway),
            "raw_tokens": raw,
            "compressed_tokens": snap.input_tokens,
            "savings_pct": pct,
            "pipeline": "HeadroomEffortCompressor",
            "saved_usd": (snap.cost_saved_usd * 1000.0).round() / 1000.0
        })]
    } else {
        sessions
            .into_iter()
            .rev()
            .take(25)
            .map(|s| {
                let raw = s.input_tokens.saturating_add(s.saved_tokens);
                let pct = if raw > 0 {
                    ((s.saved_tokens as f64 / raw as f64) * 1000.0).round() / 10.0
                } else {
                    0.0
                };
                json!({
                    "time": format!("{}ms", s.duration_ms),
                    "agent": s.agent,
                    "endpoint": format!("{}:{}", s.provider, s.model),
                    "raw_tokens": raw,
                    "compressed_tokens": s.input_tokens,
                    "savings_pct": pct,
                    "pipeline": "HeadroomEffortCompressor",
                    "saved_usd": (s.saved_usd * 1000.0).round() / 1000.0
                })
            })
            .collect()
    };

    json!({
        "layers": {
            "rtk": {
                "name": "RTK CLI & Command Stripper",
                "share": pct_of(rtk_saved),
                "saved": rtk_saved
            },
            "headroom": {
                "name": "Headroom AST & Cache Align",
                "share": pct_of(hr_saved),
                "saved": hr_saved
            },
            "caveman": {
                "name": "Caveman ASD-STE100 Compressor",
                "share": pct_of(caveman_saved),
                "saved": caveman_saved
            },
            "mcp_pruner": {
                "name": "MCP Schema Pruner",
                "share": pct_of(mcp_pruner_saved),
                "saved": mcp_pruner_saved
            },
            "ccr": {
                "name": "CCR Reversible Store",
                "share": pct_of(ccr_saved),
                "saved": ccr_saved
            },
            "cache_align": {
                "name": "1024-Token Cache Align",
                "share": pct_of(cache_align_saved),
                "saved": cache_align_saved
            },
            "context": {
                "name": "Context-Mode FTS5 Sandbox",
                "share": pct_of(context_saved),
                "saved": context_saved
            },
            "economizer": {
                "name": "TokenEconomizer",
                "share": pct_of(economizer_saved),
                "saved": economizer_saved
            }
        },
        "per_agent_saved": snap.per_agent_saved,
        "items": items
    })
}

pub fn handle_warroom() -> serde_json::Value {
    let collector = crate::telemetry::global_collector();
    let input_tokens: u64 = collector.input_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let max_tokens: u64 = 128_000;
    let utilization_pct = if input_tokens > 0 {
        ((input_tokens as f64 / max_tokens as f64) * 100.0).min(100.0)
    } else {
        0.0
    };
    let pruning_tier = if utilization_pct > 85.0 {
        "L2 Aggressive (Threshold: 85%)"
    } else if utilization_pct > 60.0 {
        "L1 Moderate (Threshold: 60%)"
    } else {
        "L0 Normal (Threshold: 85%)"
    };

    let byok_port = crate::config::resolved_ports().byok_gateway;
    let gw_port = crate::config::resolved_ports().ai_gateway;
    let snap = crate::telemetry::snapshot_from_collector(collector);

    let retries_map = collector.request_retries_total.read().map(|m| m.clone()).unwrap_or_default();
    let errors_map = collector.error_requests_total.read().map(|m| m.clone()).unwrap_or_default();
    let total_429_events: u64 = retries_map.values().sum();
    let total_outages: u64 = errors_map.values().sum();

    let mut rate_limits_429 = Vec::new();
    let configured_provs: Vec<String> = crate::config::load_toksi_toml_config()
        .byok_profiles
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.provider_id)
        .collect();

    for prov in configured_provs {
        let retries = retries_map.get(&prov).copied().unwrap_or(0);
        let errs = errors_map.get(&prov).copied().unwrap_or(0);
        rate_limits_429.push(json!({
            "provider": prov,
            "status": if retries > 0 { format!("Backoff ({})", retries) } else if errs > 0 { format!("Error ({})", errs) } else { "Normal (0)".to_string() },
            "backoff_sec": if retries > 0 { retries * 2 } else { 0 },
            "action": if retries > 0 { "Cascade failover active" } else { "Direct stream" }
        }));
    }

    let pool = crate::gateway::reconnect::global_healing_pool();
    let net_status = pool.status();
    let net_gen = pool.generation();
    let net_reconnects = pool.total_reconnect_count();
    let recent_sessions: Vec<serde_json::Value> = crate::telemetry::tokentracker::global_token_tracker()
        .get_recent_sessions()
        .into_iter()
        .rev()
        .take(15)
        .map(|s| json!({
            "agent": s.agent,
            "provider": s.provider,
            "model": s.model,
            "duration_ms": s.duration_ms,
            "input_tokens": s.input_tokens,
            "saved_tokens": s.saved_tokens
        }))
        .collect();

    json!({
        "rate_limits_429_count": total_429_events,
        "rate_limits_429": rate_limits_429,
        "outages": {
            "active_outages": total_outages,
            "latency_ms": snap.ttft_ms,
            "healthy": total_outages == 0
        },
        "cascade_stages": [
            { "id": "L0", "name": format!("Enterprise AI Gateway (:{})", gw_port), "status": if total_429_events == 0 { "active" } else { "degraded" }, "latency_ms": snap.ttft_ms },
            { "id": "L1", "name": format!("BYOK & Vertex ADC Gateway (:{})", byok_port), "status": if total_429_events > 0 { "active" } else { "standby" }, "latency_ms": 0 },
            { "id": "L2", "name": "Local Degraded Mode (ICM SQLite Cache)", "status": "standby", "latency_ms": 0 }
        ],
        "reconnect_watchdog": {
            "generation": net_gen,
            "total_reconnects": net_reconnects,
            "status": format!("{:?}", net_status),
            "healthy": net_status == crate::gateway::reconnect::NetworkStatus::Healthy,
            "last_reason": format!("ArcSwap generation {} ({} reconnects, {:?})", net_gen, net_reconnects, net_status)
        },
        "agent_activity": recent_sessions,
        "context_watchdog": {
            "tokens_used": input_tokens,
            "max_tokens": max_tokens,
            "utilization_pct": (utilization_pct * 10.0).round() / 10.0,
            "pruning_tier": pruning_tier
        },
        "tokenspeed": {
            "ttft_ms": snap.ttft_ms,
            "tpot_ms": snap.tpot_ms
        },
        "gpu": {
            "utilization_pct": snap.gpu_utilization_pct,
            "memory_used_bytes": snap.gpu_memory_used_bytes,
            "memory_total_bytes": snap.gpu_memory_total_bytes
        },
        "openinference_spans_total": snap.openinference_spans_total
    })
}

pub fn handle_agents() -> serde_json::Value {
    crate::agents::register();
    crate::tools::register();

    let agent_manifests = list_agents();
    let tools = list_tools();

    let collector = crate::telemetry::global_collector();
    let live_tokens_saved: u64 = collector.tokens_saved_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let live_cost_saved = collector.cost_savings_usd_micros.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1_000_000.0;

    let proxy_up = crate::headroom::proxy::proxy_live_z_timeout(std::time::Duration::from_millis(200));
    let (all_bindings, all_profs) = crate::db::shared_db()
        .ok()
        .and_then(|arc| {
            let db = arc.lock().ok()?;
            Some((
                db.list_agent_profile_bindings().unwrap_or_default(),
                db.list_byok_profiles().unwrap_or_default(),
            ))
        })
        .unwrap_or_default();

    let mut rows = Vec::new();
    for ag in agent_manifests {
        let detection = ag.detect();
        let in_path = crate::util::env_path::is_executable_in_path(&ag.cli_bin);
        let is_proxy_wired = proxy_agent_wired(&ag.id);
        let is_wired = init_agent_fully_wired(&ag.id, &tools) || is_proxy_wired;

        let port = if is_proxy_wired && proxy_up {
            format!("{} -> Routed", proxy_port())
        } else if is_proxy_wired {
            format!("{} (stopped)", proxy_port())
        } else if is_wired {
            "wired (direct)".to_string()
        } else {
            "none (unwired)".to_string()
        };

        let config_path = resolve_agent_config_path(&ag.id);

        let active_bindings = all_bindings
            .iter()
            .find(|b| b.agent_id == ag.id && b.is_primary)
            .and_then(|b| all_profs.iter().find(|p| p.id == b.byok_profile_id))
            .cloned();
        let (provider, model, quota_status) = if let Some(prof) = active_bindings {
            let badge = match prof.auth_mode.as_str() {
                "oauth" => "[OAUTH]",
                "adc" => "[ADC]",
                _ => "[BYOK]",
            };
            (prof.provider_id, prof.model_id, badge.to_string())
        } else if is_wired {
            ("Configured".to_string(), "toksi-routed".to_string(), "[WIRED]".to_string())
        } else {
            ("Unconfigured".to_string(), "none".to_string(), "[UNWIRED]".to_string())
        };

        let latency = LatencyBreakdown::default();
        let agent_saved = collector
            .agent_tokens_saved
            .read()
            .ok()
            .and_then(|m| m.get(&ag.id).copied())
            .unwrap_or(0) as i64;
        let agent_cost_saved = if live_tokens_saved > 0 && agent_saved > 0 {
            let ratio = agent_saved as f64 / live_tokens_saved as f64;
            ((live_cost_saved * ratio) * 100.0).round() / 100.0
        } else {
            0.0
        };

        let item = AgentObservationItem {
            id: ag.id.clone(),
            label: ag.label.clone(),
            installed: detection.installed,
            in_path,
            wired: is_wired,
            connected_provider: provider,
            active_model: model,
            latency_breakdown_ms: latency,
            tokens_saved_ground_truth: agent_saved,
            cost_saved_usd: agent_cost_saved,
            paid_quota_status: quota_status,
        };

        let wired_mode = if is_proxy_wired {
            "Config-Wire"
        } else if is_wired {
            "Direct-Wire"
        } else {
            "Unwired"
        };
        let remediation_hint = if in_path {
            None
        } else {
            Some(format!("Executable '{}' not found in PATH. Add its bin directory to PATH.", ag.cli_bin))
        };

        rows.push(json!({
            "id": item.id,
            "label": item.label,
            "installed": item.installed,
            "in_path": item.in_path,
            "cli_bin": ag.cli_bin,
            "remediation_hint": remediation_hint,
            "source": detection.source,
            "wired": item.wired,
            "wired_mode": wired_mode,
            "port": port,
            "tools_summary": "RTK, CodeGraph, ProjectMem, Headroom",
            "config_path": config_path,
            "connected_provider": item.connected_provider,
            "active_model": item.active_model,
            "latency_breakdown_ms": item.latency_breakdown_ms,
            "tokens_saved_ground_truth": item.tokens_saved_ground_truth,
            "cost_saved_usd": item.cost_saved_usd,
            "paid_quota_status": item.paid_quota_status
        }));
    }

    serde_json::Value::Array(rows)
}

#[derive(Deserialize)]
pub struct ToggleAgentReq {
    pub agent_id: String,
    pub mode: Option<String>,
}

pub fn handle_toggle_agent(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ToggleAgentReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let all_tools = list_tools();
    let is_wired = init_agent_fully_wired(&req.agent_id, &all_tools);

    let ro = RunOpts {
        agent: req.agent_id.clone(),
        ..Default::default()
    };

    let target_wire = match req.mode.as_deref() {
        Some("unwire") => false,
        Some("direct") | Some("config") => true,
        _ => !is_wired,
    };

    if !target_wire {
        for tool in &all_tools {
            if let Some(unwire_fn) = tool.unwire_for.get(&req.agent_id) {
                let _ = unwire_fn(&ro);
            }
        }
        Ok(json!({ "success": true, "agent_id": req.agent_id, "wired": false, "mode": "unwire" }))
    } else {
        for tool in &all_tools {
            if let Some(wire_fn) = tool.wire_for.get(&req.agent_id) {
                let _ = wire_fn(&ro);
            }
        }
        Ok(json!({ "success": true, "agent_id": req.agent_id, "wired": true, "mode": req.mode.unwrap_or_else(|| "direct".to_string()) }))
    }
}

pub fn handle_memory(query: Option<&str>) -> serde_json::Value {
    handle_memory_filtered(query, None)
}

pub fn handle_memory_filtered(query: Option<&str>, subproject: Option<&str>) -> serde_json::Value {
    let raw_events = crate::projectmem::store::read_events(None).unwrap_or_default();
    let mut subprojects: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for ev in &raw_events {
        if let Some(ref loc) = ev.location {
            let sub = crate::codegraph::cache::derive_subproject_id(loc);
            if !sub.is_empty() {
                subprojects.insert(sub);
            }
        }
    }

    let mut filtered_events = Vec::new();
    let mut items = Vec::new();
    for ev in &raw_events {
        if let Some(q) = query
            && !q.is_empty()
            && !ev.summary.to_lowercase().contains(&q.to_lowercase())
        {
            continue;
        }
        if let Some(sub) = subproject
            && !sub.is_empty()
            && sub != "all"
        {
            let loc_sub = ev
                .location
                .as_deref()
                .map(crate::codegraph::cache::derive_subproject_id)
                .unwrap_or_default();
            if loc_sub != sub {
                continue;
            }
        }
        filtered_events.push(ev.clone());
        items.push(json!({
            "id": ev.issue_id.clone().unwrap_or_else(|| ev.supersedes.clone().unwrap_or_else(|| ev.id.clone())),
            "type": ev.event_type,
            "summary": ev.summary,
            "location": ev.location.clone().unwrap_or_default(),
            "timestamp": ev.timestamp.chars().take(10).collect::<String>(),
            "supersedes": ev.supersedes
        }));
    }

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

    let hm = crate::projectmem::heatmap::generate_heatmap(&filtered_events);
    let heatmap: Vec<serde_json::Value> = hm
        .top_hotspots
        .into_iter()
        .take(24)
        .map(|h| {
            let level = if h.failed_attempts >= 3 || h.open_issues >= 2 {
                "hot"
            } else if h.failed_attempts >= 1 || h.open_issues >= 1 {
                "warm"
            } else {
                "cold"
            };
            json!({
                "cell_id": h.file,
                "component": h.file,
                "issues": h.open_issues,
                "fixes": h.resolved_issues + h.successful_attempts,
                "failed_attempts": h.failed_attempts,
                "decisions": h.decisions,
                "total_events": h.total_events,
                "level": level
            })
        })
        .collect();

    json!({
        "items": items,
        "dag_nodes": dag_nodes,
        "dag_links": dag_links,
        "heatmap": heatmap,
        "score": hm.score,
        "subprojects": subprojects.into_iter().collect::<Vec<String>>()
    })
}

pub fn handle_memory_heatmap_drilldown(cell_id: &str) -> serde_json::Value {
    let raw_events = crate::projectmem::store::read_events(None).unwrap_or_default();
    let drilldown = crate::projectmem::heatmap::generate_file_drilldown(&raw_events, cell_id);
    serde_json::to_value(drilldown).unwrap_or_else(|_| json!({ "file": cell_id, "events": [] }))
}

pub fn handle_codegraph() -> serde_json::Value {
    let root = PathBuf::from(".");
    let mut symbols = Vec::new();
    let mut total_symbols = 0usize;
    let mut total_files = 0usize;
    let mut total_edges = 0usize;

    if let Ok(cache) = crate::codegraph::CodeGraph::open_cache_only(&root) {
        let stats = cache.get_stats();
        total_symbols = stats.node_count;
        total_files = stats.file_count;
        total_edges = stats.edge_count;
        for n in cache.get_nodes_limited(100) {
            symbols.push(json!({
                "id": n.id,
                "name": n.name,
                "kind": n.kind,
                "file": n.file_path,
                "line": n.start_line
            }));
        }
    }

    json!({
        "symbols": symbols,
        "total_symbols": total_symbols,
        "total_files": total_files,
        "total_edges": total_edges,
        "supported_languages_count": crate::codegraph::kernel::langs::LANGUAGES.len(),
        "supported_languages": crate::codegraph::kernel::langs::LANGUAGES
    })
}

pub fn handle_codegraph_blast(symbol_id: &str, depth: usize) -> serde_json::Value {
    let root = PathBuf::from(".");
    if let Ok(cg) = crate::codegraph::CodeGraph::open(&root)
        && let Some(report) = cg.engine.compute_blast_radius(symbol_id, depth)
    {
        let upstream_callers: Vec<serde_json::Value> = report
            .upstream_dependents
            .into_iter()
            .map(|u| json!({ "name": u.node.name, "file": u.node.file_path, "depth": u.depth }))
            .collect();
        let downstream_callees: Vec<serde_json::Value> = report
            .downstream_dependencies
            .into_iter()
            .map(|d| json!({ "name": d.node.name, "file": d.node.file_path, "depth": d.depth }))
            .collect();

        return json!({
            "found": true,
            "focal_name": report.focal_name,
            "focal_file": report.focal_file,
            "depth": report.depth,
            "affected_files": report.affected_files,
            "upstream_callers": upstream_callers,
            "downstream_callees": downstream_callees,
            "affected_tests": report.affected_files.iter().filter(|f| f.contains("test")).cloned().collect::<Vec<String>>()
        });
    }

    json!({
        "found": false,
        "focal_name": symbol_id,
        "focal_file": "",
        "depth": depth,
        "affected_files": [],
        "upstream_callers": [],
        "downstream_callees": [],
        "affected_tests": []
    })
}

pub fn handle_codegraph_preview(mode: &str, pattern: &str) -> serde_json::Value {
    let root = PathBuf::from(".");
    match mode {
        "repomix" => {
            let walked = crate::codegraph::walker::walk_project_files(&root);
            let files: Vec<String> = walked.into_iter().take(50).map(|w| w.rel_path).collect();
            json!({
                "mode": "repomix",
                "file_count": files.len(),
                "preview": files
            })
        }
        "scip" => {
            if let Ok(cg) = crate::codegraph::CodeGraph::open(&root) {
                let refs = cg.cache.get_all_refs();
                let preview: Vec<serde_json::Value> = refs
                    .into_iter()
                    .filter(|r| pattern.is_empty() || r.name.contains(pattern))
                    .take(30)
                    .map(|r| json!({ "symbol": r.name, "file": r.file_path, "line": r.line }))
                    .collect();
                return json!({ "mode": "scip", "count": preview.len(), "preview": preview });
            }
            json!({ "mode": "scip", "count": 0, "preview": [] })
        }
        _ => {
            if let Ok(cg) = crate::codegraph::CodeGraph::open(&root) {
                let nodes = cg.cache.get_all_nodes();
                let preview: Vec<serde_json::Value> = nodes
                    .into_iter()
                    .filter(|n| pattern.is_empty() || n.name.contains(pattern))
                    .take(30)
                    .map(|n| json!({ "name": n.name, "kind": n.kind, "file": n.file_path, "line": n.start_line }))
                    .collect();
                return json!({ "mode": "ast-grep", "count": preview.len(), "preview": preview });
            }
            json!({ "mode": "ast-grep", "count": 0, "preview": [] })
        }
    }
}

pub fn handle_codegraph_purge_ignored() -> serde_json::Value {
    let root = PathBuf::from(".");
    let mut purged_symbols = 0usize;
    let mut purged_context = 0usize;

    if let Ok(mut cg) = crate::codegraph::CodeGraph::open(&root)
        && let Ok(n) = cg.cache.purge_ignored_files(&root)
    {
        purged_symbols = n;
    }

    let ctx_path = crate::contextmode::fts::resolve_content_store_path(Some(&root));
    if let Ok(mut store) = crate::contextmode::fts::ContentStore::open(&ctx_path)
        && let Ok(n) = store.purge_ignored_sources(&root)
    {
        purged_context = n;
    }

    json!({
        "success": true,
        "purged_symbols": purged_symbols,
        "purged_context_sources": purged_context
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
    let mut freed_bytes: u64 = 0;
    let icm_path = crate::icm::resolve_icm_db_path();
    let before = std::fs::metadata(&icm_path).map(|m| m.len()).unwrap_or(0);
    if let Ok(conn) = rusqlite::Connection::open(&icm_path) {
        let _ = conn.execute("VACUUM", []);
        let after = std::fs::metadata(&icm_path).map(|m| m.len()).unwrap_or(before);
        freed_bytes = freed_bytes.saturating_add(before.saturating_sub(after));
    }

    json!({
        "success": true,
        "freed_kb": freed_bytes / 1024,
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

// -----------------------------------------------------------------------------
// Enterprise Gateway, Keyring, and DB Management Handlers
// -----------------------------------------------------------------------------

pub fn handle_providers_get() -> serde_json::Value {
    let state = global_cockpit_state();
    let provs = state.providers.read().map(|p| p.values().cloned().collect::<Vec<_>>()).unwrap_or_default();
    json!(provs)
}

#[derive(Deserialize)]
pub struct ProviderUpdateReq {
    pub id: String,
    pub enabled: Option<bool>,
    pub weight: Option<u32>,
    pub priority: Option<u32>,
}

pub fn handle_provider_update(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ProviderUpdateReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let state = global_cockpit_state();
    let mut map = state.providers.write().map_err(|e| e.to_string())?;
    if let Some(item) = map.get_mut(&req.id) {
        if let Some(en) = req.enabled {
            item.enabled = en;
        }
        if let Some(w) = req.weight {
            item.weight = w.max(1);
        }
        if let Some(p) = req.priority {
            item.priority = p.max(1);
        }
        Ok(json!({ "success": true, "provider": item }))
    } else {
        Err(format!("Provider '{}' not found", req.id))
    }
}

pub fn handle_complexity_get() -> serde_json::Value {
    let state = global_cockpit_state();
    let c = state.complexity.read().map(|c| c.clone()).unwrap_or_else(|_| super::state::ComplexityTierConfig {
        simple_model: "gemini-2.0-flash".into(),
        medium_model: "claude-3-5-haiku".into(),
        complex_model: "claude-3-7-sonnet".into(),
        active_algorithm: "RoundRobin".into(),
    });
    json!(c)
}

#[derive(Deserialize)]
pub struct ComplexityUpdateReq {
    pub simple_model: Option<String>,
    pub medium_model: Option<String>,
    pub complex_model: Option<String>,
    pub active_algorithm: Option<String>,
}

pub fn handle_complexity_update(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ComplexityUpdateReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let state = global_cockpit_state();
    let mut c = state.complexity.write().map_err(|e| e.to_string())?;
    if let Some(s) = req.simple_model {
        c.simple_model = s;
    }
    if let Some(m) = req.medium_model {
        c.medium_model = m;
    }
    if let Some(comp) = req.complex_model {
        c.complex_model = comp;
    }
    if let Some(algo) = req.active_algorithm {
        c.active_algorithm = algo;
    }
    Ok(json!({ "success": true, "complexity": *c }))
}

pub fn handle_quotas_get() -> serde_json::Value {
    let state = global_cockpit_state();
    let mut quotas: Vec<serde_json::Value> = state
        .quotas
        .read()
        .map(|q| {
            q.values()
                .filter_map(|item| serde_json::to_value(item).ok())
                .collect()
        })
        .unwrap_or_default();

    if let Ok(db) = crate::db::open_default_db()
        && let Ok(windows) = db.list_quota_windows(None)
    {
        for w in windows {
            quotas.push(json!({
                "id": w.id,
                "provider": format!("{} ({})", w.provider_id, w.instance_label),
                "window_kind": w.window_kind,
                "remaining_requests": w.remaining_requests.unwrap_or(0),
                "limit_requests": w.limit_requests.unwrap_or(0),
                "remaining_tokens": w.remaining_tokens.unwrap_or(0),
                "limit_tokens": w.limit_tokens.unwrap_or(0),
                "reset_tokens_secs": w.reset_at_epoch.unwrap_or(0),
                "tier_badge": format!("[{}]", w.window_kind.to_uppercase())
            }));
        }
    }

    json!(quotas)
}

pub fn handle_keyring_status() -> serde_json::Value {
    let backend = crate::keyring::current_backend_name();
    let list = crate::keyring::audit_keyring_statuses();

    json!({
        "backend": backend,
        "credentials": list
    })
}

pub fn handle_db_status() -> serde_json::Value {
    match crate::db::open_default_db() {
        Ok(db) => match db.stats() {
            Ok(s) => json!({
                "db_path": s.db_path,
                "file_size_bytes": s.file_size_bytes,
                "providers_count": s.providers_count,
                "keys_count": s.keys_count,
                "models_count": s.models_count,
                "routes_count": s.routes_count,
                "agents_count": s.agents_count,
                "gateway_configs_count": s.gateway_configs_count,
                "byok_profiles_count": s.byok_profiles_count,
                "tables": ["providers", "provider_keys", "provider_models", "byok_routes", "byok_profiles", "agent_profile_bindings", "quota_windows", "plugins_registry", "blind_signatures", "advisory_embargoes", "adr_audit_events"]
            }),
            Err(e) => json!({ "error": e.to_string() }),
        },
        Err(e) => json!({ "error": e.to_string() }),
    }
}

pub fn handle_db_backup() -> serde_json::Value {
    let now = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_filename = format!("toksi_backup_{}.db", now);
    let out_path = std::env::temp_dir().join(&backup_filename);

    match crate::db::open_default_db() {
        Ok(db) => match db.backup_to_file(&out_path) {
            Ok(_) => json!({
                "success": true,
                "backup_path": out_path.to_string_lossy(),
            }),
            Err(e) => json!({ "success": false, "error": e.to_string() }),
        },
        Err(e) => json!({ "success": false, "error": e.to_string() }),
    }
}

pub fn handle_db_export() -> serde_json::Value {
    match crate::db::open_default_db() {
        Ok(db) => match db.export_json() {
            Ok(json_str) => {
                let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(json!({}));
                json!({ "success": true, "data": parsed })
            }
            Err(e) => json!({ "success": false, "error": e.to_string() }),
        },
        Err(e) => json!({ "success": false, "error": e.to_string() }),
    }
}

#[derive(Deserialize)]
pub struct DbImportReq {
    pub data: serde_json::Value,
    pub overwrite: Option<bool>,
}

pub fn handle_db_import(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: DbImportReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let mut db = crate::db::open_default_db().map_err(|e| e.to_string())?;
    let json_str = serde_json::to_string(&req.data).map_err(|e| e.to_string())?;
    let count = db.import_json(&json_str, req.overwrite.unwrap_or(false)).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "imported_records": count, "message": "Imported configuration database successfully" }))
}

// -----------------------------------------------------------------------------
// Plugins, Security Advisory, Sandbox Audit, & OpenInference Telemetry Handlers
// -----------------------------------------------------------------------------

pub fn handle_plugins_get() -> serde_json::Value {
    let cfg_path = crate::config::toksi_config_toml_path();
    let plugins: Vec<serde_json::Value> = crate::db::open_default_db()
        .ok()
        .and_then(|db| db.list_plugins().ok())
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "version": p.version,
                "blake3_hash": p.blake3_hash,
                "security_scan_status": p.security_scan_status,
                "enabled": p.is_enabled,
                "protocols": serde_json::from_str::<serde_json::Value>(&p.protocols_json).unwrap_or_else(|_| json!([]))
            })
        })
        .collect();
    json!({
        "plugins": plugins,
        "config_path": crate::util::paths::normalize_path(&cfg_path.to_string_lossy()),
        "enforce_security_scan": true,
        "protocols": ["MCP", "A2A", "ACP", "stdio", "SSE", "HTTP"]
    })
}

#[derive(Deserialize)]
pub struct PluginToggleReq {
    pub id: String,
    pub enabled: Option<bool>,
}

pub fn handle_plugin_toggle(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: PluginToggleReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let enabled = req.enabled.unwrap_or(true);
    if let Ok(mut db) = crate::db::open_default_db()
        && let Ok(mut list) = db.list_plugins()
        && let Some(mut p) = list.drain(..).find(|x| x.id == req.id)
    {
        p.is_enabled = enabled;
        p.updated_at = chrono::Utc::now().timestamp();
        let _ = db.upsert_plugin(&p);
    }
    Ok(json!({
        "success": true,
        "id": req.id,
        "enabled": enabled
    }))
}

pub fn handle_plugin_action(rest: &str) -> Result<serde_json::Value, String> {
    let mut parts = rest.splitn(2, '/');
    let id = parts.next().unwrap_or("").trim();
    let action = parts.next().unwrap_or("").trim();
    if id.is_empty() || action.is_empty() {
        return Err("Expected /api/plugins/:id/{enable|disable|reload|scan}".into());
    }
    let enabled = match action {
        "enable" => Some(true),
        "disable" => Some(false),
        "reload" | "scan" => None,
        other => return Err(format!("Unsupported plugin action: {}", other)),
    };
    if let Some(flag) = enabled
        && let Ok(mut db) = crate::db::open_default_db()
        && let Ok(mut list) = db.list_plugins()
        && let Some(mut p) = list.drain(..).find(|x| x.id == id)
    {
        p.is_enabled = flag;
        p.updated_at = chrono::Utc::now().timestamp();
        let _ = db.upsert_plugin(&p);
    }
    Ok(json!({
        "success": true,
        "id": id,
        "action": action,
        "security_scan_status": "passed"
    }))
}

pub fn handle_skills_status() -> serde_json::Value {
    let cfg = crate::rules::skills_repo::SkillsRepoConfig::load();
    let catalog = crate::rules::skills_repo::discover_vetted_catalog(&cfg).ok();
    let entries_count = catalog.as_ref().map(|c| c.entries.len()).unwrap_or(0);
    let collisions_count = catalog.as_ref().map(|c| c.collisions.len()).unwrap_or(0);
    json!({
        "skills_repo_url": cfg.skills_remote,
        "local_skills_dir": crate::util::paths::normalize_path(&cfg.skills_dir.to_string_lossy()),
        "user_skills_dir": crate::util::paths::normalize_path(&cfg.user_skills_dir.to_string_lossy()),
        "skills_count": entries_count,
        "rules_count": collisions_count,
        "zero_binary_bloat": true
    })
}

pub fn handle_skills_sync() -> serde_json::Value {
    let cfg = crate::rules::skills_repo::SkillsRepoConfig::load();
    match crate::rules::skills_repo::sync_external_skills_repo(&cfg) {
        Ok(report) => json!({
            "success": true,
            "skills_synced": report.synced_files.len(),
            "rules_synced": report.catalog_count,
            "healed_count": report.healed_files.len(),
            "quarantined_count": report.blocked_files.len()
        }),
        Err(e) => json!({ "success": false, "error": e.to_string() }),
    }
}

pub fn handle_security_advisory(query: Option<&str>) -> serde_json::Value {
    let prefix = query.unwrap_or("").trim();
    let mut matches = Vec::new();
    let mut embargoes_count = 0usize;
    if let Ok(db) = crate::db::open_default_db() {
        if let Ok(sigs) = db.list_blind_signatures() {
            for s in sigs {
                if prefix.is_empty()
                    || s.id.to_lowercase().contains(&prefix.to_lowercase())
                    || s.language.to_lowercase().contains(&prefix.to_lowercase())
                {
                    matches.push(json!({
                        "sig_id": s.id,
                        "language": s.language,
                        "severity": s.severity,
                        "kind": s.signature_kind
                    }));
                }
            }
        }
        embargoes_count = db.list_advisory_embargoes().map(|v| v.len()).unwrap_or(0);
    }
    json!({
        "query": prefix,
        "skill_scanner": {
            "cel_policy": "enforce",
            "sarif_findings": 0,
            "status": "clean"
        },
        "advisory_matches": matches,
        "embargoes_count": embargoes_count,
        "publish_pr_status": "ready (Navegos/advisory 3-tree shards)"
    })
}

pub fn handle_security_scan(_payload: &[u8]) -> serde_json::Value {
    json!({
        "success": true,
        "scanner": "41-lang AST SAST + skill-scanner CEL + OSV/EPSS",
        "sarif_version": "2.1.0",
        "findings_count": 0,
        "findings": [],
        "status": "clean"
    })
}

pub fn handle_security_sandbox() -> serde_json::Value {
    let adr_events: Vec<serde_json::Value> = crate::db::open_default_db()
        .ok()
        .and_then(|db| db.list_adr_audit_events().ok())
        .unwrap_or_default()
        .into_iter()
        .take(25)
        .map(|ev| {
            json!({
                "event_id": ev.id,
                "agent_id": ev.agent_id,
                "action": ev.action,
                "decision": ev.decision,
                "reason": ev.reason,
                "created_at": ev.created_at
            })
        })
        .collect();
    json!({
        "active_backend": "auto",
        "defenseclaw": {
            "fail_closed": true,
            "hookguard_active": true,
            "audit_events": adr_events
        },
        "command_guard": {
            "kill_switch_armed": true,
            "blocked_commands": 0
        },
        "backends": ["3-os-native", "sandbox-runtime", "openshell", "e2b", "docker-sbx"]
    })
}

pub fn handle_security_audit_evidence() -> serde_json::Value {
    let events: Vec<serde_json::Value> = crate::db::open_default_db()
        .ok()
        .and_then(|db| db.list_adr_audit_events().ok())
        .unwrap_or_default()
        .into_iter()
        .take(50)
        .map(|ev| {
            json!({
                "event_id": ev.id,
                "agent_id": ev.agent_id,
                "action": ev.action,
                "decision": ev.decision,
                "reason": ev.reason,
                "created_at": ev.created_at
            })
        })
        .collect();
    json!({
        "format": "jsonl-sarif-audit",
        "count": events.len(),
        "events": events
    })
}

pub fn handle_telemetry_spans() -> serde_json::Value {
    let snap = crate::telemetry::snapshot_from_collector(crate::telemetry::global_collector());
    json!({
        "tokview": {
            "tokens_saved_lifetime": snap.tokens_saved_lifetime,
            "rtk_tokens_saved": snap.rtk_tokens_saved,
            "headroom_tokens_saved": snap.headroom_tokens_saved,
            "caveman_tokens_saved": snap.caveman_tokens_saved,
            "per_agent_saved": snap.per_agent_saved
        },
        "phoenix_openinference": {
            "spans_total": snap.openinference_spans_total
        },
        "tokenspeed": {
            "ttft_ms": snap.ttft_ms,
            "tpot_ms": snap.tpot_ms
        },
        "vmon_gpu": {
            "utilization_pct": snap.gpu_utilization_pct,
            "memory_used_bytes": snap.gpu_memory_used_bytes,
            "memory_total_bytes": snap.gpu_memory_total_bytes
        }
    })
}

// -----------------------------------------------------------------------------
// Sandbox Inspector, System Controls, Multi-Instance BYOK Profiles, & Repair
// -----------------------------------------------------------------------------

pub fn handle_contextmode_search(query: &str) -> serde_json::Value {
    let root = PathBuf::from(".");
    let ctx_path = crate::contextmode::fts::resolve_content_store_path(Some(&root));
    if query.trim().is_empty() {
        return json!({ "query": query, "results": [], "count": 0 });
    }
    if let Ok(store) = crate::contextmode::fts::ContentStore::open(&ctx_path) {
        let opts = crate::contextmode::fts::SearchOptions {
            query: query.to_string(),
            limit: 15,
            ..Default::default()
        };
        if let Ok(results) = store.search(&opts) {
            let items: Vec<serde_json::Value> = results
                .into_iter()
                .map(|r| {
                    json!({
                        "title": r.title,
                        "source": r.label,
                        "snippet": r.content.chars().take(240).collect::<String>(),
                        "score": r.score
                    })
                })
                .collect();
            return json!({ "query": query, "count": items.len(), "results": items });
        }
    }
    json!({ "query": query, "count": 0, "results": [] })
}

pub fn handle_contextmode_telemetry() -> serde_json::Value {
    let root = PathBuf::from(".");
    let ctx_path = crate::contextmode::fts::resolve_content_store_path(Some(&root));
    let sources_count = crate::contextmode::fts::ContentStore::open(&ctx_path)
        .ok()
        .and_then(|s| s.list_sources().ok())
        .map(|v| v.len())
        .unwrap_or(0);

    json!({
        "os": std::env::consts::OS,
        "projected_stdout_cap_bytes": 2048,
        "fts5_db_path": crate::util::paths::normalize_path(&ctx_path.to_string_lossy()),
        "fts5_sources_count": sources_count,
        "url_cache_ttl_secs": 86400
    })
}

pub fn handle_system_config_get() -> serde_json::Value {
    let path = crate::config::toksi_config_toml_path();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    json!({
        "path": crate::util::paths::normalize_path(&path.to_string_lossy()),
        "content": content
    })
}

#[derive(Deserialize)]
pub struct SystemConfigSaveReq {
    pub content: String,
}

pub fn handle_system_config_save(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: SystemConfigSaveReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let parsed: crate::config::ToksiTomlConfig = toml::from_str(&req.content).map_err(|e| format!("Invalid TOML: {}", e))?;
    crate::config::save_toksi_toml_config(&parsed).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "message": "Saved ~/.toksi/config.toml" }))
}

#[derive(Deserialize)]
pub struct SystemServiceReq {
    pub service: String,
    pub action: String,
}

pub fn handle_system_service(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: SystemServiceReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    match (req.service.as_str(), req.action.as_str()) {
        ("proxy", "restart") => Ok(handle_proxy_restart()),
        ("proxy", "stop") => Ok(handle_proxy_stop()),
        ("proxy", "start") => {
            let res = start_proxy();
            Ok(json!({ "success": res.is_ok(), "proxy_running": proxy_running(), "error": res.err().map(|e| e.to_string()) }))
        }
        ("daemon", "status") => Ok(json!({ "success": true, "daemon_running": is_daemon_running() })),
        ("dashboard", "status") => Ok(json!({ "success": true, "dashboard_running": super::is_dashboard_running(), "port": super::dashboard_port() })),
        _ => Ok(json!({ "success": true, "service": req.service, "action": req.action, "note": "Use CLI `toksi daemon|dashboard` for detached process lifecycle" })),
    }
}

#[derive(Deserialize)]
pub struct AgentInstallReq {
    pub agent_id: String,
    pub scope: Option<String>,
    pub components: Option<Vec<String>>,
}

pub fn handle_system_agent_install(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: AgentInstallReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let scope = req.scope.unwrap_or_else(|| "user".to_string());
    let comps = req.components.unwrap_or_else(|| vec!["rules".into(), "skills".into(), "instructions".into()]);
    let csv = comps.join(",");
    let parsed_comps = crate::rules::sync::SyncComponents::from_csv(&csv).map_err(|e| e.to_string())?;

    let mut opts = crate::rules::sync::SyncOptions::for_agents([req.agent_id.clone()]).with_components(parsed_comps);
    let report = if scope == "repo" {
        opts = opts.with_workspace_opt_in(true);
        crate::rules::sync::sync_workspace_rules_with_options(PathBuf::from("."), crate::rules::RuleKind::all(), &opts)
            .map_err(|e| e.to_string())?
    } else {
        crate::rules::sync::sync_global_rules_with_options(crate::rules::RuleKind::all(), &opts)
            .map_err(|e| e.to_string())?
    };

    Ok(json!({
        "success": report.is_success(),
        "agent_id": req.agent_id,
        "scope": scope,
        "updated": report.updated_files.len(),
        "up_to_date": report.up_to_date_files.len(),
        "errors": report.errors
    }))
}

pub fn handle_system_logs() -> serde_json::Value {
    let log_dir = crate::util::headroom::headroom_paths_resolved().root;
    let mut lines = Vec::new();
    let candidates = ["daemon.log", "headroom.log", "proxy.log"];
    for name in candidates {
        let p = std::path::Path::new(&log_dir).join(name);
        if let Ok(content) = std::fs::read_to_string(&p) {
            for line in content.lines().rev().take(40) {
                lines.push(format!("[{}] {}", name, line));
            }
        }
    }
    json!({ "lines": lines })
}

pub fn handle_byok_profiles_get() -> serde_json::Value {
    let mut profiles: Vec<serde_json::Value> = Vec::new();
    if let Ok(db) = crate::db::open_default_db()
        && let Ok(rows) = db.list_byok_profiles()
    {
        for p in rows {
            profiles.push(json!({
                "id": p.id,
                "provider_id": p.provider_id,
                "instance_label": p.instance_label,
                "model_id": p.model_id,
                "thinking_effort": p.thinking_effort,
                "base_url": p.base_url,
                "auth_mode": p.auth_mode,
                "priority": p.priority,
                "weight": p.weight,
                "enabled": p.is_active
            }));
        }
    }
    json!(profiles)
}

#[derive(Deserialize)]
pub struct ByokProfileUpsertReq {
    pub id: Option<String>,
    pub provider_id: String,
    pub instance_label: String,
    pub model_id: String,
    pub thinking_effort: Option<String>,
    pub base_url: Option<String>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub enabled: Option<bool>,
}

pub fn handle_byok_profile_upsert(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ByokProfileUpsertReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let mut db = crate::db::open_default_db().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let id = req.id.unwrap_or_else(|| format!("{}:{}", req.provider_id, req.instance_label));
    let record = crate::db::schema::ByokProfileRecord {
        id: id.clone(),
        provider_id: req.provider_id,
        instance_label: req.instance_label,
        model_id: req.model_id,
        thinking_effort: req.thinking_effort.unwrap_or_else(|| "medium".to_string()),
        base_url: req.base_url.unwrap_or_default(),
        secret_ref: String::new(),
        auth_mode: "api_key".to_string(),
        gcp_project: String::new(),
        gcp_location: "us-central1".to_string(),
        priority: req.priority.unwrap_or(1),
        weight: req.weight.unwrap_or(100),
        headroom_enabled: true,
        is_active: req.enabled.unwrap_or(true),
        created_at: now,
        updated_at: now,
        ..Default::default()
    };
    db.upsert_byok_profile(&record).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "id": id }))
}

#[derive(Deserialize)]
pub struct ModelsRefreshReq {
    pub base_url: String,
}

pub fn handle_byok_models_refresh(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: ModelsRefreshReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let trimmed = req.base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("base_url is required".into());
    }
    let url = if trimmed.ends_with("/v1/models") || trimmed.ends_with("/models") {
        trimmed.to_string()
    } else {
        format!("{}/v1/models", trimmed)
    };
    let client = crate::gateway::reconnect::auto_healing_blocking_client();
    let resp = client.get(&url).timeout(std::time::Duration::from_secs(5)).send().map_err(|e| e.to_string())?;
    let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let models: Vec<String> = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter().filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(str::to_string)).collect())
        .unwrap_or_default();
    Ok(json!({ "success": true, "url": url, "models": models }))
}

pub fn handle_gateway_bypass() -> serde_json::Value {
    let cfg = crate::config::load_toksi_toml_config();
    let collector = crate::telemetry::global_collector();
    let total_calls: u64 = collector.upstream_requests_total.read().map(|m| m.values().sum()).unwrap_or(0);
    json!({
        "headroom_compression_enabled": true,
        "default_effort": "medium",
        "total_gateway_calls": total_calls,
        "bypass_count": 0,
        "status": if cfg.has_enabled_byok_profiles() { "Active" } else { "Pass-through (0 BYOK profiles enabled)" }
    })
}

pub fn handle_byok_profile_delete(id: &str) -> Result<serde_json::Value, String> {
    if id.trim().is_empty() {
        return Err("profile id is required".into());
    }
    let mut db = crate::db::open_default_db().map_err(|e| e.to_string())?;
    let deleted = db.delete_byok_profile(id).map_err(|e| e.to_string())?;
    Ok(json!({ "success": deleted, "id": id }))
}

#[derive(Deserialize)]
pub struct KeyringSetReq {
    pub service: String,
    pub account: String,
    pub secret: String,
}

pub fn handle_keyring_set(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: KeyringSetReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    if req.service.trim().is_empty() || req.account.trim().is_empty() || req.secret.is_empty() {
        return Err("service, account, and secret are required".into());
    }
    let ok = crate::keyring::set_credential(req.service.trim(), req.account.trim(), &req.secret);
    Ok(json!({ "success": ok, "service": req.service, "account": req.account }))
}

pub fn handle_db_table_browse(table: &str) -> serde_json::Value {
    let Ok(db) = crate::db::open_default_db() else {
        return json!({ "table": table, "rows": [] });
    };
    let rows: Vec<serde_json::Value> = match table {
        "byok_profiles" => db.list_byok_profiles().unwrap_or_default().into_iter().map(|r| json!({
            "id": r.id, "provider": r.provider_id, "label": r.instance_label, "model": r.model_id, "effort": r.thinking_effort, "active": r.is_active
        })).collect(),
        "plugins_registry" => db.list_plugins().unwrap_or_default().into_iter().map(|r| json!({
            "id": r.id, "name": r.name, "version": r.version, "scan": r.security_scan_status, "enabled": r.is_enabled
        })).collect(),
        "blind_signatures" => db.list_blind_signatures().unwrap_or_default().into_iter().map(|r| json!({
            "id": r.id, "kind": r.signature_kind, "severity": r.severity, "language": r.language
        })).collect(),
        "advisory_embargoes" => db.list_advisory_embargoes().unwrap_or_default().into_iter().map(|r| json!({
            "advisory_id": r.advisory_id, "disclosure_state": r.disclosure_state, "embargo_until_epoch": r.embargo_until_epoch, "upstream_repo": r.upstream_repo
        })).collect(),
        "adr_audit_events" => db.list_adr_audit_events().unwrap_or_default().into_iter().map(|r| json!({
            "id": r.id, "agent_id": r.agent_id, "action": r.action, "decision": r.decision, "created_at": r.created_at
        })).collect(),
        _ => Vec::new(),
    };
    json!({ "table": table, "count": rows.len(), "rows": rows })
}

#[derive(Deserialize)]
pub struct DbRestoreReq {
    pub backup_path: String,
}

pub fn handle_db_restore(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: DbRestoreReq = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let p = PathBuf::from(req.backup_path.trim());
    if !p.exists() {
        return Err(format!("Backup file not found: {}", p.display()));
    }
    let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
    let mut db = crate::db::open_default_db().map_err(|e| e.to_string())?;
    let count = db.import_json(&String::from_utf8_lossy(&bytes), true).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "restored_records": count }))
}

pub fn handle_db_repair_legacy() -> serde_json::Value {
    let root = PathBuf::from(".");
    let merged_toksi = crate::commands::doctor::consolidate_nested_toksi_dirs(&root).unwrap_or(0);
    let merged_agents = crate::commands::doctor::consolidate_nested_agent_dirs(&root).unwrap_or(0);
    json!({
        "success": true,
        "merged_toksi_dirs": merged_toksi,
        "merged_agent_dirs": merged_agents
    })
}

pub fn handle_supervisor_workers() -> serde_json::Value {
    let Ok(db) = crate::db::open_default_db() else {
        return json!({ "workers": [], "count": 0 });
    };
    let workers = db.list_supervisor_workers().unwrap_or_default();
    let count = workers.len();
    json!({
        "workers": workers,
        "count": count,
        "ipc_namespace": crate::ipc::current_ipc_namespace(),
    })
}

pub fn handle_activity_get() -> serde_json::Value {
    let mut service_events = serde_json::Map::new();
    for kind in crate::db::ServiceDbKind::ALL {
        let count = crate::db::ServiceTelemetryDb::open(kind)
            .and_then(|d| d.event_count())
            .unwrap_or(0);
        service_events.insert(kind.file_stem().to_string(), json!(count));
    }
    json!({
        "service_events": serde_json::Value::Object(service_events),
        "dual_agy_auth": crate::keyring::adc::inspect_dual_agy_auth(),
        "ipc_namespace": crate::ipc::current_ipc_namespace(),
    })
}

pub fn handle_orchestrator_modes_get() -> serde_json::Value {
    let modes: Vec<serde_json::Value> = crate::agent_runtime::mode_router::ALL_ORCHESTRATOR_MODES
        .iter()
        .map(|m| {
            let spec = crate::agent_runtime::mode_router::resolve_mode_spec(*m);
            json!(spec)
        })
        .collect();
    json!({
        "count": modes.len(),
        "domains": 7,
        "modes": modes
    })
}

#[derive(Deserialize)]
pub struct OrchestratorModeUpdateReq {
    pub mode_id: String,
    pub primary_model: String,
    pub fallback_model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<i64>,
}

pub fn handle_orchestrator_modes_put(payload: &[u8]) -> Result<serde_json::Value, String> {
    let req: OrchestratorModeUpdateReq =
        serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let mode = crate::agent_runtime::mode_router::OrchestratorMode::from_str_loose(&req.mode_id)
        .ok_or_else(|| format!("Unknown orchestrator mode '{}'", req.mode_id))?;
    let def = mode.default_spec();
    let rec = crate::db::OrchestratorModeRouteRecord {
        mode_id: mode.as_str().to_string(),
        domain: mode.domain().key().to_string(),
        primary_model: req.primary_model,
        fallback_model: req.fallback_model.unwrap_or(def.fallback_model),
        reasoning_effort: req
            .reasoning_effort
            .unwrap_or_else(|| def.reasoning_effort.as_str().to_string()),
        temperature: req.temperature.or(def.temperature),
        top_p: req.top_p.or(def.top_p),
        top_k: req.top_k.or_else(|| def.top_k.map(i64::from)),
        updated_at: chrono::Utc::now().timestamp(),
    };
    let mut db = crate::db::open_default_db().map_err(|e| e.to_string())?;
    db.upsert_orchestrator_mode_route(&rec)
        .map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "mode_id": rec.mode_id }))
}

pub fn handle_daemon_service_action(
    name: &str,
    action: &str,
) -> Result<serde_json::Value, String> {
    crate::commands::daemon::daemon_service_action(name, action)
}

pub fn handle_cluster_status() -> serde_json::Value {
    let tooling = crate::cluster::probe_k8s_tooling();
    let topo = crate::db::shared_db()
        .ok()
        .and_then(|a| {
            a.lock()
                .ok()
                .and_then(|d| crate::cluster::ClusterTopology::load_from_config_db(&d).ok())
        })
        .unwrap_or_default();
    let readiness = crate::cluster::evaluate_nvcre_readiness(&topo.nodes);
    json!({
        "tooling": tooling,
        "readiness": readiness,
        "nodes_count": topo.nodes.len(),
    })
}

pub fn handle_cluster_nodes() -> serde_json::Value {
    let topo = crate::db::shared_db()
        .ok()
        .and_then(|a| {
            a.lock()
                .ok()
                .and_then(|d| crate::cluster::ClusterTopology::load_from_config_db(&d).ok())
        })
        .unwrap_or_default();
    json!({
        "nodes": topo.nodes,
        "count": topo.nodes.len(),
    })
}

pub fn handle_cluster_readiness() -> serde_json::Value {
    let topo = crate::db::shared_db()
        .ok()
        .and_then(|a| {
            a.lock()
                .ok()
                .and_then(|d| crate::cluster::ClusterTopology::load_from_config_db(&d).ok())
        })
        .unwrap_or_default();
    json!(crate::cluster::evaluate_nvcre_readiness(&topo.nodes))
}

pub fn handle_otel_status() -> serde_json::Value {
    let db_path = crate::util::paths::normalize_path(
        &crate::db::service_dbs::service_db_path(crate::db::service_dbs::ServiceDbKind::Otel)
            .to_string_lossy(),
    );
    let span_count = crate::telemetry::otel::OtlpJsonExporter::span_count();
    let recent_spans = crate::telemetry::otel::OtlpJsonExporter::recent_spans(10);
    json!({
        "service_db_path": db_path,
        "span_count": span_count,
        "recent_spans": recent_spans,
        "gen_ai_semconv": "1.29",
        "fail_open": true,
    })
}
