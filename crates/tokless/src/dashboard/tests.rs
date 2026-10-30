// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/dashboard/tests.rs

use super::api::*;
use super::assets::*;
use super::server::*;

#[test]
fn test_embedded_assets_presence() {
    assert!(!INDEX_HTML.is_empty(), "index.html must not be empty");
    assert!(INDEX_HTML.contains("tokless"), "index.html must contain brand");
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
    assert!(APP_JS.contains("ToklessApp"), "app.js must define ToklessApp class");
}

#[test]
fn test_http_request_parsing() {
    // 1. Simple GET request
    let raw_get = b"GET /api/overview HTTP/1.1\r\nHost: 127.0.0.1:9099\r\n\r\n";
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

    let nodes = mem["dag_nodes"].as_array().expect("dag_nodes array");
    assert!(!nodes.is_empty());
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
    assert!(res["freed_kb"].as_u64().unwrap_or(0) > 0);
}
