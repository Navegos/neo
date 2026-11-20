// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase17_6_gateway_accounting_verification.rs

#![deny(unsafe_code)]

use std::time::Duration;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use toksi::gateway::circuit_breaker::CircuitBreakerRegistry;
use toksi::gateway::providers::aggregators::{AggregatorProvider, BifrostProviderClient};
use toksi::gateway::providers::{ProviderClient, ProviderId};
use toksi::gateway::reconnect::{AutoHealingClientPool, NetworkStatus};
use toksi::gateway::router::{DataClass, QualityGate, RouteCandidate, Router, RoutingPolicy};
use toksi::gateway::server::spawn_isolated_gateway;
use toksi::headroom::byok_gateway::is_proto_or_connect_request;
use toksi::telemetry::collector::{snapshot_from_collector, MetricsCollector};
use toksi::telemetry::tokentracker::{
    extract_quota_windows, SseStreamTokenParser, TokenTrackerEngine,
};

#[test]
fn test_phase17_6_zero_idle_telemetry_and_quota_windows() {
    let engine = TokenTrackerEngine::new(16);
    assert_eq!(engine.lifetime_summary(), (0, 0, 0, 0, 0.0, 0.0));

    let collector = MetricsCollector::new();
    let snap = snapshot_from_collector(&collector);
    assert_eq!(snap.total_calls, 0);
    assert_eq!(snap.input_tokens, 0);
    assert_eq!(snap.output_tokens, 0);
    assert_eq!(snap.tokens_saved_lifetime, 0);
    assert_eq!(snap.cost_spent_usd, 0.0);

    let headers = vec![
        ("anthropic-ratelimit-requests-limit".to_string(), "100".to_string()),
        ("anthropic-ratelimit-requests-remaining".to_string(), "88".to_string()),
        ("x-ratelimit-limit-tokens-day".to_string(), "500000".to_string()),
        ("x-ratelimit-remaining-tokens".to_string(), "420000".to_string()),
        ("retry-after".to_string(), "45".to_string()),
    ];
    let windows = extract_quota_windows("anthropic", "default", &headers, 1_700_000_000);
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].window_kind, "day");
    assert_eq!(windows[0].remaining_requests, Some(88));
    assert_eq!(windows[0].remaining_tokens, Some(420000));
    assert_eq!(windows[0].reset_at_epoch, Some(1_700_000_045));
}

#[test]
fn test_phase17_6_7d_router_confidential_quality_gate_and_aggregators() {
    let router = Router::new();
    let breakers = CircuitBreakerRegistry::new(2, Duration::from_secs(60));
    let candidates = vec![
        RouteCandidate::for_provider(ProviderId::OpenAI, "gpt-4o"),
        RouteCandidate::for_provider(ProviderId::Ollama, "qwen2.5-coder"),
    ];

    let policy = RoutingPolicy {
        data_class: DataClass::Confidential,
        quality_gate: Some(QualityGate {
            min_capability_score: 0.70,
            require_non_empty: true,
            allow_tool_calls: true,
        }),
        ..Default::default()
    };

    let ranked = router.route_ranked(&candidates, &policy, Some(&breakers));
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].candidate.provider, ProviderId::Ollama);

    let agg = AggregatorProvider::new(ProviderId::OpenRouter);
    assert_eq!(agg.endpoint(), format!("{}/chat/completions", ProviderId::OpenRouter.effective_endpoint().trim_end_matches('/')));

    let bifrost = BifrostProviderClient::default();
    assert!(bifrost.endpoint().ends_with("/chat/completions"));
}

#[test]
fn test_phase17_6_reconnect_pool_reset_and_connectrpc_headers() {
    let pool = AutoHealingClientPool::new().expect("pool init");
    assert_eq!(pool.generation(), 1);
    assert_eq!(pool.status(), NetworkStatus::Healthy);

    let healed = pool.report_transport_error("Connection reset by peer (os error 10054)");
    assert!(healed);
    assert_eq!(pool.generation(), 2);
    assert_eq!(pool.total_reconnect_count(), 1);
    assert_eq!(pool.status(), NetworkStatus::Healthy);

    let headers = vec![
        ("Content-Type".to_string(), "application/connect+proto".to_string()),
        ("Connect-Protocol-Version".to_string(), "1".to_string()),
        ("TE".to_string(), "trailers".to_string()),
        ("Connection".to_string(), "keep-alive".to_string()),
        ("Keep-Alive".to_string(), "timeout=5".to_string()),
        ("Transfer-Encoding".to_string(), "chunked".to_string()),
    ];
    assert!(is_proto_or_connect_request("/google.internal.cloud.code.v1internal.CodeService/StreamGenerate", &headers));
    let safe = AutoHealingClientPool::build_connectrpc_safe_headers(&headers);
    assert_eq!(safe.len(), 3);
    assert!(safe.iter().any(|(k, _)| k.eq_ignore_ascii_case("connect-protocol-version")));
    assert!(safe.iter().any(|(k, _)| k.eq_ignore_ascii_case("te")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_phase17_6_gateway_real_dispatch_and_sse_accounting() {
    // 1. Spin up mock OpenAI SSE + JSON upstream
    let upstream_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind mock upstream");
    let upstream_port = upstream_listener.local_addr().expect("local addr").port();

    let upstream_task = tokio::spawn(async move {
        for _ in 0..2 {
            let Ok((mut sock, _)) = upstream_listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let req_str = String::from_utf8_lossy(&buf[..n]);
                if req_str.contains("\"stream\":true") {
                    let sse = concat!(
                        "data: {\"id\":\"chatcmpl-1\",\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n",
                        "data: {\"id\":\"chatcmpl-1\",\"choices\":[],\"usage\":{\"prompt_tokens\":12,\"completion_tokens\":4,\"prompt_tokens_details\":{\"cached_tokens\":3}}}\n\n",
                        "data: [DONE]\n\n"
                    );
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nx-ratelimit-remaining-requests: 99\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        sse.len(),
                        sse
                    );
                    let _ = sock.write_all(resp.as_bytes()).await;
                } else {
                    let body = json!({
                        "id": "chatcmpl-mock",
                        "object": "chat.completion",
                        "model": "gpt-4o",
                        "choices": [{
                            "index": 0,
                            "message": { "role": "assistant", "content": "ok-real" },
                            "finish_reason": "stop"
                        }],
                        "usage": { "prompt_tokens": 10, "completion_tokens": 2 }
                    })
                    .to_string();
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nx-ratelimit-remaining-requests: 98\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = sock.write_all(resp.as_bytes()).await;
                }
            });
        }
    });

    // 2. Spin up isolated AI Gateway on ephemeral port
    let gw_port = 29199;
    let mut gw_handle = spawn_isolated_gateway("127.0.0.1", gw_port).expect("spawn gateway");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::builder().no_proxy().build().expect("client");
    let upstream_url = format!("http://127.0.0.1:{}/v1/chat/completions", upstream_port);

    // Non-streaming dispatch
    let resp = client
        .post(format!("http://127.0.0.1:{}/v1/chat/completions", gw_port))
        .header("Authorization", "Bearer test-key")
        .header("x-toksi-upstream", &upstream_url)
        .json(&json!({
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "ping"}]
        }))
        .send()
        .await
        .expect("gateway non-stream");
    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.expect("json");
    assert_eq!(
        body["choices"][0]["message"]["content"].as_str(),
        Some("ok-real")
    );

    // Streaming SSE dispatch
    let sse_resp = client
        .post(format!("http://127.0.0.1:{}/v1/chat/completions", gw_port))
        .header("Authorization", "Bearer test-key")
        .header("x-toksi-upstream", &upstream_url)
        .json(&json!({
            "model": "gpt-4o",
            "stream": true,
            "messages": [{"role": "user", "content": "ping-sse"}]
        }))
        .send()
        .await
        .expect("gateway sse");
    assert_eq!(sse_resp.status().as_u16(), 200);
    let sse_text = sse_resp.text().await.expect("sse text");
    assert!(!sse_text.contains("AI Gateway operational"));
    let mut parser = SseStreamTokenParser::new();
    for line in sse_text.lines() {
        let _ = parser.feed_line(line);
    }
    let parsed = parser.total_usage();
    assert_eq!(parsed.prompt_tokens, 12);
    assert_eq!(parsed.completion_tokens, 4);
    assert_eq!(parsed.cached_tokens, 3);

    gw_handle.shutdown();
    upstream_task.abort();
}

#[test]
fn test_phase17_6_sse_anthropic_cache_creation_reasoning_and_gemini() {
    let mut parser = SseStreamTokenParser::new();
    let lines = [
        r#"data: {"type":"message_start","message":{"usage":{"input_tokens":120,"cache_creation_input_tokens":45,"cache_read_input_tokens":30}}}"#,
        r#"data: {"type":"message_delta","usage":{"output_tokens":25}}"#,
    ];
    for line in lines {
        let _ = parser.feed_line(line);
    }
    let u = parser.total_usage();
    assert_eq!(u.prompt_tokens, 120);
    assert_eq!(u.completion_tokens, 25);
    assert_eq!(u.cached_tokens, 30);
    assert_eq!(u.cache_creation_tokens, 45);

    let mut oai_parser = SseStreamTokenParser::new();
    let _ = oai_parser.feed_line(
        r#"data: {"usage":{"prompt_tokens":50,"completion_tokens":40,"completion_tokens_details":{"reasoning_tokens":18}}}"#,
    );
    let oai_u = oai_parser.total_usage();
    assert_eq!(oai_u.reasoning_tokens, 18);

    let mut gem_parser = SseStreamTokenParser::new();
    let _ = gem_parser.feed_line(
        r#"data: {"usageMetadata":{"promptTokenCount":80,"candidatesTokenCount":35,"thoughtsTokenCount":12,"cachedContentTokenCount":10}}"#,
    );
    let gem_u = gem_parser.total_usage();
    assert_eq!(gem_u.prompt_tokens, 80);
    assert_eq!(gem_u.completion_tokens, 35);
    assert_eq!(gem_u.reasoning_tokens, 12);
    assert_eq!(gem_u.cached_tokens, 10);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_phase17_6_connectrpc_binary_frame_roundtrip() {
    let upstream_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind upstream");
    let upstream_port = upstream_listener.local_addr().expect("local addr").port();

    // Binary ConnectRPC envelope: 1 byte flags (0x00) + 4 byte big-endian length (5) + 5 raw non-UTF8 bytes
    let expected_frame: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x05, 0x08, 0xff, 0xfe, 0x01, 0x2a];
    let upstream_frame = expected_frame.clone();

    let upstream_task = tokio::spawn(async move {
        if let Ok((mut sock, _)) = upstream_listener.accept().await {
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf).await;
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/connect+proto\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                upstream_frame.len()
            );
            let _ = sock.write_all(header.as_bytes()).await;
            let _ = sock.write_all(&upstream_frame).await;
        }
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind proxy");
    let proxy_port = proxy_listener.local_addr().expect("proxy addr").port();
    let upstream_base = format!("http://127.0.0.1:{}", upstream_port);

    let proxy_task = tokio::spawn(async move {
        if let Ok((mut client_stream, _)) = proxy_listener.accept().await {
            let mut raw = Vec::new();
            let mut buf = [0u8; 4096];
            let mut hdr_end = None;
            while hdr_end.is_none() {
                let n = client_stream.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&buf[..n]);
                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    hdr_end = Some(pos + 4);
                }
            }
            if let Some(end) = hdr_end {
                let head_str = String::from_utf8_lossy(&raw[..end]);
                let mut lines = head_str.lines();
                let req_line = lines.next().unwrap_or("");
                let mut parts = req_line.split_whitespace();
                let method = parts.next().unwrap_or("POST").to_string();
                let path = parts.next().unwrap_or("/").to_string();
                let mut headers = Vec::new();
                let mut content_len = 0usize;
                for line in lines {
                    if let Some((k, v)) = line.split_once(':') {
                        let key = k.trim().to_string();
                        let val = v.trim().to_string();
                        if key.eq_ignore_ascii_case("content-length") {
                            content_len = val.parse().unwrap_or(0);
                        }
                        headers.push((key, val));
                    }
                }
                while raw.len() < end + content_len {
                    let n = client_stream.read(&mut buf).await.unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    raw.extend_from_slice(&buf[..n]);
                }
                let body = raw[end..raw.len().min(end + content_len)].to_vec();
                let _ = toksi::headroom::byok_gateway::forward_streaming_to_byok_target(
                    &upstream_base,
                    &method,
                    &path,
                    None,
                    &headers,
                    &body,
                    &mut client_stream,
                )
                .await;
            }
        }
    });

    let client = reqwest::Client::builder().no_proxy().build().expect("client");
    let resp = client
        .post(format!(
            "http://127.0.0.1:{}/google.internal.cloud.code.v1internal.CodeService/StreamGenerate",
            proxy_port
        ))
        .header("Content-Type", "application/connect+proto")
        .header("Connect-Protocol-Version", "1")
        .body(expected_frame.clone())
        .send()
        .await
        .expect("connectrpc roundtrip");
    assert_eq!(resp.status().as_u16(), 200);
    let body = resp.bytes().await.expect("bytes");
    assert_eq!(body.as_ref(), expected_frame.as_slice());

    proxy_task.abort();
    upstream_task.abort();
}
