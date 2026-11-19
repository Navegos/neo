// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/tests.rs
// created: 2026-10-30
// lastModified: 2026-11-18

use super::api::*;
use super::assets::*;
use super::server::*;

#[test]
fn test_embedded_assets_presence() {
    assert!(!INDEX_HTML.is_empty(), "index.html must not be empty");
    assert!(INDEX_HTML.contains("toksi"), "index.html must contain brand");
    assert!(INDEX_HTML.contains("tab-waterfall"), "index.html must contain waterfall tab");
    assert!(INDEX_HTML.contains("tab-warroom"), "index.html must contain warroom tab");
    assert!(INDEX_HTML.contains("tab-fleet"), "index.html must contain fleet tab");
    assert!(INDEX_HTML.contains("tab-memory"), "index.html must contain memory tab");
    assert!(INDEX_HTML.contains("tab-codegraph"), "index.html must contain codegraph tab");
    assert!(INDEX_HTML.contains("tab-sandbox"), "index.html must contain sandbox tab");
    assert!(INDEX_HTML.contains("tab-system"), "index.html must contain system tab");

    assert!(!STYLE_CSS.is_empty(), "style.css must not be empty");
    assert!(STYLE_CSS.contains("--bg-dark"), "style.css must define dark variables");

    assert!(!APP_JS.is_empty(), "app.js must not be empty");
    assert!(APP_JS.contains("ToksiApp"), "app.js must define ToksiApp class");
    assert!(APP_JS.contains("loadGatewayControls"), "app.js must load gateway controls");
    assert!(APP_JS.contains("loadKeyringAndDb"), "app.js must load keyring and db");
    assert!(APP_JS.contains("openHeatmapDrilldown"), "app.js must support clickable heatmap drilldown");

    assert!(!INDEX_HTML.contains("5,658,990"), "index.html must not contain fake waterfall numbers");
    assert!(!INDEX_HTML.contains("42,500 / 128,000"), "index.html must not contain fake context utilization");
    assert!(!INDEX_HTML.contains("1.8s"), "index.html must not contain fake DeepSeek throttle");
    assert!(!INDEX_HTML.contains("[PAID]"), "must not contain [PAID] badge");
    assert!(!INDEX_HTML.contains("[DISQUALIFIED-FREE]"), "must not contain [DISQUALIFIED-FREE] badge");
}

#[test]
fn test_http_request_parsing() {
    // 1. Simple GET request
    let raw_get = b"GET /api/overview HTTP/1.1\r\nHost: 127.0.0.1:9100\r\n\r\n";
    let (req, len) = HttpRequest::parse(raw_get).expect("GET request parse failed");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/api/overview");
    assert_eq!(len, raw_get.len());

    // 2. GET with query string
    let raw_query = b"GET /api/memory?q=sqlite&depth=2 HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let (req, _) = HttpRequest::parse(raw_query).expect("GET with query parse failed");
    assert_eq!(req.path, "/api/memory");
    assert_eq!(req.query.get("q").map(|s| s.as_str()), Some("sqlite"));
    assert_eq!(req.query.get("depth").map(|s| s.as_str()), Some("2"));

    // 3. POST with body and Content-Length
    let body_data = b"{\"agent_id\":\"claude\"}";
    let raw_post = format!(
        "POST /api/agents/toggle HTTP/1.1\r\n\
        Content-Type: application/json\r\n\
        Content-Length: {}\r\n\r\n\
        {}",
        body_data.len(),
        std::str::from_utf8(body_data).unwrap()
    );
    let (req, _) = HttpRequest::parse(raw_post.as_bytes()).expect("POST request parse failed");
    assert_eq!(req.method, "POST");
    assert_eq!(req.path, "/api/agents/toggle");
    assert_eq!(req.body, body_data);
}

#[test]
fn test_http_response_generation() {
    let resp = HttpResponse::ok_html("<h1>Hello</h1>");
    let bytes = resp.to_bytes();
    let str_resp = String::from_utf8_lossy(&bytes);
    assert!(str_resp.starts_with("HTTP/1.1 200 OK"));
    assert!(str_resp.contains("Content-Type: text/html; charset=utf-8"));
    assert!(str_resp.contains("<h1>Hello</h1>"));

    let json_resp = HttpResponse::ok_json(serde_json::json!({ "status": "ok" }));
    let json_bytes = json_resp.to_bytes();
    let json_str = String::from_utf8_lossy(&json_bytes);
    assert!(json_str.contains("HTTP/1.1 200 OK"));
    assert!(json_str.contains("\"status\":\"ok\""));

    let not_found = HttpResponse::not_found();
    let not_found_bytes = not_found.to_bytes();
    let not_found_str = String::from_utf8_lossy(&not_found_bytes);
    assert!(not_found_str.contains("HTTP/1.1 404 Not Found"));
}

#[test]
fn test_api_overview_and_waterfall() {
    let overview = handle_overview();
    assert!(overview.get("tokens_saved_lifetime").is_some());
    assert!(overview.get("cost_saved_usd").is_some());
    assert!(overview.get("compression_pct").is_some());

    let waterfall = handle_waterfall();
    assert!(waterfall.get("layers").is_some());
    let layers = &waterfall["layers"];
    assert!(layers.get("rtk").is_some());
    assert!(layers.get("headroom").is_some());
    assert!(layers.get("caveman").is_some());

    let items = waterfall["items"].as_array().expect("items should be an array");
    assert!(!items.is_empty());
}

#[test]
fn test_api_warroom_telemetry() {
    let warroom = handle_warroom();
    assert!(warroom.get("rate_limits_429").is_some());
    assert!(warroom.get("cascade_stages").is_some());
    assert!(warroom.get("context_watchdog").is_some());
}

#[test]
fn test_api_agents_fleet_17_coverage() {
    let agents = handle_agents();
    let arr = agents.as_array().expect("agents must be an array");
    assert!(arr.len() >= 17, "All 17 agent harnesses must be registered and represented");
}

#[test]
fn test_api_memory_and_dag() {
    let mem = handle_memory(None);
    assert!(mem.get("dag_nodes").is_some());
    assert!(mem.get("dag_links").is_some());
    assert!(mem.get("heatmap").is_some());
    assert!(mem.get("subprojects").is_some());

    assert!(mem["dag_nodes"].is_array(), "dag_nodes must be an array");
    let dd = handle_memory_heatmap_drilldown("crates/toksi/src/dashboard/mod.rs");
    assert!(dd.get("events").is_some());
}

#[test]
fn test_api_codegraph_and_blast() {
    let cg = handle_codegraph();
    assert!(cg.get("symbols").is_some());

    let blast = handle_codegraph_blast("run_dashboard", 3);
    assert_eq!(blast["depth"], 3);
    assert!(blast.get("affected_files").is_some());
    assert!(blast.get("upstream_callers").is_some());
}

#[test]
fn test_api_contextmode_filter() {
    let payload = serde_json::json!({
        "raw_output": "line 1\nline 2\n\x1b[31merror\x1b[0m: sample\n",
        "cmd": "test",
        "max_lines": 10,
        "max_bytes": 1024
    });
    let bytes = serde_json::to_vec(&payload).unwrap();
    let res = handle_contextmode_filter(&bytes).expect("filter must succeed");
    assert!(res.get("filtered").is_some());
    assert!(res.get("savings_pct").is_some());
}

#[test]
fn test_api_cache_vacuum() {
    let res = handle_cache_vacuum();
    assert_eq!(res["success"], true);
    assert!(res["freed_kb"].is_u64());
}

#[test]
fn test_api_providers_controls() {
    let provs = handle_providers_get();
    let arr = provs.as_array().expect("providers must be array");
    assert!(!arr.is_empty());

    let update_payload = serde_json::json!({
        "id": "anthropic",
        "weight": 150,
        "priority": 2
    });
    let bytes = serde_json::to_vec(&update_payload).unwrap();
    let res = handle_provider_update(&bytes).expect("update provider must succeed");
    assert_eq!(res["success"], true);
    assert_eq!(res["provider"]["weight"], 150);
    assert_eq!(res["provider"]["priority"], 2);
}

#[test]
fn test_api_complexity_router() {
    let comp = handle_complexity_get();
    assert!(comp.get("active_algorithm").is_some());

    let update_payload = serde_json::json!({
        "active_algorithm": "LeastLatency",
        "complex_model": "claude-3-7-sonnet"
    });
    let bytes = serde_json::to_vec(&update_payload).unwrap();
    let res = handle_complexity_update(&bytes).expect("update complexity must succeed");
    assert_eq!(res["success"], true);
    assert_eq!(res["complexity"]["active_algorithm"], "LeastLatency");
}

#[test]
fn test_api_quotas_and_keyring() {
    let quotas = handle_quotas_get();
    assert!(quotas.is_array(), "quotas must be JSON array");

    let keyring = handle_keyring_status();
    assert!(keyring.get("backend").is_some());
    assert!(keyring.get("credentials").is_some());

    let plugins = handle_plugins_get();
    assert!(plugins.get("plugins").is_some());
    let advisory = handle_security_advisory(Some("CVE-2026"));
    assert_eq!(advisory["query"], "CVE-2026");
    let sandbox = handle_security_sandbox();
    assert_eq!(sandbox["defenseclaw"]["fail_closed"], true);
    let spans = handle_telemetry_spans();
    assert!(spans.get("tokview").is_some());
    assert!(spans.get("tokenspeed").is_some());
    assert!(spans.get("vmon_gpu").is_some());
}

#[test]
fn test_api_db_status_and_export() {
    let status = handle_db_status();
    println!("DB status response: {:?}", status);
    assert!(status.is_object());

    let export = handle_db_export();
    assert_eq!(export["success"], true);

    let cg_prev = handle_codegraph_preview("ast-grep", "");
    assert_eq!(cg_prev["mode"], "ast-grep");
    let ctx_tel = handle_contextmode_telemetry();
    assert_eq!(ctx_tel["projected_stdout_cap_bytes"], 2048);
    let bypass = handle_gateway_bypass();
    assert_eq!(bypass["headroom_compression_enabled"], true);
    let cfg = handle_system_config_get();
    assert!(cfg.get("path").is_some());
    let profiles = handle_byok_profiles_get();
    assert!(profiles.is_array());
}
