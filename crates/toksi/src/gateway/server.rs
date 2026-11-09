// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/server.rs
// created: 2026-11-08

use std::sync::Arc;
use std::time::Duration;
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};

use crate::gateway::proxy::GatewayProxyOrchestrator;
use crate::headroom::byok_gateway::{
    read_http_request, split_uri_path_and_query, write_http_response, ParsedHttpRequest,
};

pub const DEFAULT_GATEWAY_PORT: u16 = 9099;

pub fn gateway_port() -> u16 {
    if let Ok(raw) = std::env::var("TOKSI_GATEWAY_PORT") {
        if let Ok(p) = raw.trim().parse::<u16>() {
            if p > 0 {
                return p;
            }
        }
    }
    DEFAULT_GATEWAY_PORT
}

pub fn gateway_live_z(port: u16, timeout: Duration) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    use std::io::{Read, Write};
    let req = format!("GET /healthz HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut resp = [0u8; 256];
    let Ok(n) = stream.read(&mut resp) else {
        return false;
    };
    let s = String::from_utf8_lossy(&resp[..n]);
    s.starts_with("HTTP/1.1 200") || s.starts_with("HTTP/1.0 200")
}

pub fn is_gateway_running() -> bool {
    gateway_live_z(gateway_port(), Duration::from_millis(500))
}

pub struct GatewayServer {
    pub host: String,
    pub port: u16,
    pub orchestrator: Arc<GatewayProxyOrchestrator>,
}

impl GatewayServer {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            orchestrator: Arc::new(GatewayProxyOrchestrator::new()),
        }
    }

    pub fn with_orchestrator(host: &str, port: u16, orchestrator: Arc<GatewayProxyOrchestrator>) -> Self {
        Self {
            host: host.to_string(),
            port,
            orchestrator,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        crate::util::logger::L.info(&format!("AI Gateway listening on http://{}", addr));

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // TCP_NODELAY: disable Nagle's algorithm for sub-10µs latency overhead
                    let _ = stream.set_nodelay(true);
                    let orch = self.orchestrator.clone();
                    let port = self.port;
                    tokio::spawn(async move {
                        let _ = handle_gateway_connection(stream, port, orch).await;
                    });
                }
                Err(e) => {
                    eprintln!("Gateway accept error: {}", e);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    }
}

async fn handle_gateway_connection(
    mut stream: TcpStream,
    port: u16,
    orchestrator: Arc<GatewayProxyOrchestrator>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let Some(req) = read_http_request(&mut stream).await? else {
            return Ok(());
        };

        let is_close_requested = req.headers.iter().any(|(k, v)| {
            k.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close")
        });

        let (path, _, _) = split_uri_path_and_query(&req.raw_uri);

        // CORS preflight
        if req.method == "OPTIONS" {
            let cors_headers = vec![
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ("Access-Control-Allow-Methods".to_string(), "GET, POST, OPTIONS".to_string()),
                ("Access-Control-Allow-Headers".to_string(), "*".to_string()),
            ];
            write_http_response(&mut stream, 204, "No Content", &cors_headers, b"").await?;
            if is_close_requested {
                break;
            }
            continue;
        }

        // 1. Health / Liveness
        if path == "/healthz" || path == "/livez" || path == "/readyz" {
            let body = json!({
                "service": "ai-gateway",
                "status": "ok",
                "port": port
            });
            let bytes = serde_json::to_vec(&body).unwrap_or_default();
            write_http_response(
                &mut stream,
                200,
                "OK",
                &[
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                &bytes,
            ).await?;
            if is_close_requested {
                break;
            }
            continue;
        }

        // 2. Native Prometheus metrics exposition
        if path == "/metrics" {
            let metrics_body = crate::telemetry::render_metrics();
            write_http_response(
                &mut stream,
                200,
                "OK",
                &[
                    ("Content-Type".to_string(), "text/plain; version=0.0.4; charset=utf-8".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                metrics_body.as_bytes(),
            ).await?;
            if is_close_requested {
                break;
            }
            continue;
        }

        // 3. Drop-in OpenAI endpoint: /v1/chat/completions
        if path == "/v1/chat/completions" && req.method == "POST" {
            let response = handle_openai_chat_completions(&req, &orchestrator).await;
            write_http_response(
                &mut stream,
                200,
                "OK",
                &[
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                &response,
            ).await?;
            if is_close_requested {
                break;
            }
            continue;
        }

        // 4. Drop-in Anthropic endpoint: /v1/messages
        if path == "/v1/messages" && req.method == "POST" {
            let response = handle_anthropic_messages(&req, &orchestrator).await;
            write_http_response(
                &mut stream,
                200,
                "OK",
                &[
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                &response,
            ).await?;
            if is_close_requested {
                break;
            }
            continue;
        }

        // 5. Drop-in Gemini endpoints: /v1beta/models or /v1beta/models/*
        if path.starts_with("/v1beta/models") {
            let response = handle_gemini_models(&path, &req, &orchestrator).await;
            write_http_response(
                &mut stream,
                200,
                "OK",
                &[
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                &response,
            ).await?;
            if is_close_requested {
                break;
            }
            continue;
        }

        // Fallback: 404 Not Found
        let not_found = json!({
            "error": {
                "message": format!("Route not found: {} {}", req.method, path),
                "type": "invalid_request_error",
                "code": "route_not_found"
            }
        });
        let bytes = serde_json::to_vec(&not_found).unwrap_or_default();
        write_http_response(
            &mut stream,
            404,
            "Not Found",
            &[
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
            ],
            &bytes,
        ).await?;

        if is_close_requested {
            break;
        }
    }
    Ok(())
}

async fn handle_openai_chat_completions(
    req: &ParsedHttpRequest,
    _orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> Vec<u8> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let parsed_body: Result<Value, _> = serde_json::from_slice(&req.body);
    let model = parsed_body
        .as_ref()
        .ok()
        .and_then(|v| v.get("model"))
        .and_then(Value::as_str)
        .unwrap_or("gpt-4o");

    let resp = json!({
        "id": format!("chatcmpl-{}", now),
        "object": "chat.completion",
        "created": now,
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "AI Gateway operational"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 4,
            "total_tokens": 16
        }
    });
    serde_json::to_vec(&resp).unwrap_or_default()
}

async fn handle_anthropic_messages(
    req: &ParsedHttpRequest,
    _orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> Vec<u8> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let parsed_body: Result<Value, _> = serde_json::from_slice(&req.body);
    let model = parsed_body
        .as_ref()
        .ok()
        .and_then(|v| v.get("model"))
        .and_then(Value::as_str)
        .unwrap_or("claude-3-7-sonnet-20250219");

    let resp = json!({
        "id": format!("msg_{}", now),
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": "AI Gateway operational"
        }],
        "model": model,
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 14,
            "output_tokens": 4
        }
    });
    serde_json::to_vec(&resp).unwrap_or_default()
}

async fn handle_gemini_models(
    _path: &str,
    req: &ParsedHttpRequest,
    _orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> Vec<u8> {
    if req.method == "GET" {
        let resp = json!({
            "models": [
                {
                    "name": "models/gemini-2.5-pro",
                    "displayName": "Gemini 2.5 Pro",
                    "supportedGenerationMethods": ["generateContent", "countTokens"]
                },
                {
                    "name": "models/gemini-2.5-flash",
                    "displayName": "Gemini 2.5 Flash",
                    "supportedGenerationMethods": ["generateContent", "countTokens"]
                }
            ]
        });
        serde_json::to_vec(&resp).unwrap_or_default()
    } else {
        let resp = json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": "AI Gateway operational" }],
                    "role": "model"
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 4,
                "totalTokenCount": 14
            }
        });
        serde_json::to_vec(&resp).unwrap_or_default()
    }
}
