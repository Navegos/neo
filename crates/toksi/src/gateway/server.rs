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
    crate::util::ports::resolved_ports().ai_gateway
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
            let (status, response) = handle_openai_chat_completions(&req, &orchestrator).await;
            let status_text = if status < 400 { "OK" } else { "Bad Gateway" };
            write_http_response(
                &mut stream,
                status,
                status_text,
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

        // 3b. Drop-in OpenAI Responses API: /v1/responses
        if path == "/v1/responses" && req.method == "POST" {
            let (status, response) = handle_openai_responses(&req, &orchestrator).await;
            let status_text = if status < 400 { "OK" } else { "Bad Gateway" };
            write_http_response(
                &mut stream,
                status,
                status_text,
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

        // 3c. Drop-in OpenAI Embeddings API: /v1/embeddings
        if path == "/v1/embeddings" && req.method == "POST" {
            let (status, response) = handle_openai_embeddings(&req, &orchestrator).await;
            let status_text = if status < 400 { "OK" } else { "Bad Gateway" };
            write_http_response(
                &mut stream,
                status,
                status_text,
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

        // 3d. Zero-cost model catalog endpoint: /v1/models
        if path == "/v1/models" && req.method == "GET" {
            let response = handle_models_list();
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
            let (status, response) = handle_anthropic_messages(&req, &orchestrator).await;
            let status_text = if status < 400 { "OK" } else { "Bad Gateway" };
            write_http_response(
                &mut stream,
                status,
                status_text,
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
            let (status, response) = handle_gemini_models(&path, &req, &orchestrator).await;
            let status_text = if status < 400 { "OK" } else { "Bad Gateway" };
            write_http_response(
                &mut stream,
                status,
                status_text,
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

fn header_value<'a>(req: &'a ParsedHttpRequest, name: &str) -> Option<&'a str> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn extract_auth_key(req: &ParsedHttpRequest) -> Option<String> {
    if let Some(auth) = header_value(req, "authorization") {
        let stripped = auth
            .strip_prefix("Bearer ")
            .or_else(|| auth.strip_prefix("bearer "))
            .unwrap_or(auth)
            .trim();
        if !stripped.is_empty() {
            return Some(stripped.to_string());
        }
    }
    if let Some(k) = header_value(req, "x-api-key")
        .or_else(|| header_value(req, "x-goog-api-key"))
        .or_else(|| header_value(req, "api-key"))
    {
        let trimmed = k.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn infer_provider_from_model(model: &str, default_provider: crate::gateway::providers::ProviderId) -> crate::gateway::providers::ProviderId {
    use crate::gateway::providers::ProviderId;
    let m = model.to_lowercase();
    if m.starts_with("claude") {
        ProviderId::Anthropic
    } else if m.starts_with("gemini") {
        ProviderId::Gemini
    } else if m.starts_with("grok") {
        ProviderId::XAI
    } else if m.starts_with("deepseek") {
        ProviderId::DeepSeek
    } else if m.starts_with("mistral") || m.starts_with("codestral") {
        ProviderId::Mistral
    } else if m.starts_with("llama") || m.starts_with("qwen") || m.starts_with("ollama/") {
        ProviderId::Ollama
    } else if m.starts_with("command") {
        ProviderId::Cohere
    } else if m.starts_with("gpt") || m.starts_with("o1") || m.starts_with("o3") || m.starts_with("o4") {
        ProviderId::OpenAI
    } else {
        default_provider
    }
}

fn error_json_bytes(message: &str, err_type: &str) -> Vec<u8> {
    let body = json!({
        "error": {
            "message": message,
            "type": err_type,
        }
    });
    serde_json::to_vec(&body).unwrap_or_default()
}

fn resolve_candidates_for_request(
    req: &ParsedHttpRequest,
    model: &str,
    default_provider: crate::gateway::providers::ProviderId,
) -> Vec<crate::gateway::router::RouteCandidate> {
    use crate::gateway::providers::ProviderId;
    use crate::gateway::router::RouteCandidate;
    if let Some(explicit) = header_value(req, "x-toksi-provider").and_then(ProviderId::from_name) {
        return vec![RouteCandidate::for_provider(explicit, model)];
    }
    let primary = infer_provider_from_model(model, default_provider);
    vec![RouteCandidate::for_provider(primary, model)]
}

async fn handle_openai_chat_completions(
    req: &ParsedHttpRequest,
    orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> (u16, Vec<u8>) {
    let mut payload: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return (400, error_json_bytes(&format!("Invalid JSON payload: {}", e), "invalid_request_error")),
    };
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("gpt-4o")
        .to_string();
    let vk = header_value(req, crate::gateway::governance::TOKSI_VK_HEADER);
    let session_id = header_value(req, "x-toksi-session").unwrap_or("gateway-default");
    let explicit_key = extract_auth_key(req);
    let explicit_upstream = header_value(req, "x-toksi-upstream");
    let candidates = resolve_candidates_for_request(req, &model, crate::gateway::providers::ProviderId::OpenAI);
    let policy = crate::gateway::router::RoutingPolicy::default();

    match orchestrator
        .dispatch_with_cascade_async(
            session_id,
            vk,
            explicit_key.as_deref(),
            explicit_upstream,
            &candidates,
            &policy,
            &mut payload,
        )
        .await
    {
        Ok((_, unified)) => {
            if !unified.raw.is_null() {
                (200, serde_json::to_vec(&unified.raw).unwrap_or_default())
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let resp = json!({
                    "id": unified.id,
                    "object": "chat.completion",
                    "created": now,
                    "model": unified.model,
                    "choices": [{
                        "index": 0,
                        "message": { "role": unified.role, "content": unified.content },
                        "finish_reason": unified.finish_reason.unwrap_or_else(|| "stop".to_string()),
                    }],
                    "usage": {
                        "prompt_tokens": unified.input_tokens,
                        "completion_tokens": unified.output_tokens,
                        "total_tokens": unified.input_tokens + unified.output_tokens
                    }
                });
                (200, serde_json::to_vec(&resp).unwrap_or_default())
            }
        }
        Err(e) => (502, error_json_bytes(&e, "upstream_gateway_error")),
    }
}

async fn handle_openai_responses(
    req: &ParsedHttpRequest,
    orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> (u16, Vec<u8>) {
    let raw_body: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return (400, error_json_bytes(&format!("Invalid JSON payload: {}", e), "invalid_request_error")),
    };
    let mut chat_payload = crate::gateway::compat::responses_to_chat_completions(&raw_body);
    let model = chat_payload
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("gpt-4o")
        .to_string();
    let vk = header_value(req, crate::gateway::governance::TOKSI_VK_HEADER);
    let session_id = header_value(req, "x-toksi-session").unwrap_or("gateway-responses");
    let explicit_key = extract_auth_key(req);
    let explicit_upstream = header_value(req, "x-toksi-upstream");
    let candidates = resolve_candidates_for_request(req, &model, crate::gateway::providers::ProviderId::OpenAI);

    match orchestrator
        .dispatch_with_cascade_async(
            session_id,
            vk,
            explicit_key.as_deref(),
            explicit_upstream,
            &candidates,
            &crate::gateway::router::RoutingPolicy::default(),
            &mut chat_payload,
        )
        .await
    {
        Ok((_, unified)) => {
            let source = if unified.raw.is_null() {
                json!({
                    "id": unified.id,
                    "model": unified.model,
                    "choices": [{ "message": { "content": unified.content } }],
                    "usage": { "prompt_tokens": unified.input_tokens, "completion_tokens": unified.output_tokens }
                })
            } else {
                unified.raw
            };
            let formatted = crate::gateway::compat::chat_to_responses_json(&source, &model);
            (200, serde_json::to_vec(&formatted).unwrap_or_default())
        }
        Err(e) => (502, error_json_bytes(&e, "upstream_gateway_error")),
    }
}

async fn handle_openai_embeddings(
    req: &ParsedHttpRequest,
    orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> (u16, Vec<u8>) {
    use crate::gateway::providers::{ProviderCategory, ProviderId};
    let payload: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return (400, error_json_bytes(&format!("Invalid JSON payload: {}", e), "invalid_request_error")),
    };
    let provider = header_value(req, "x-toksi-provider")
        .and_then(ProviderId::from_name)
        .unwrap_or(ProviderId::OpenAI);
    let key = extract_auth_key(req)
        .or_else(|| orchestrator.key_manager.get_available_key(provider));
    let key_str = match key {
        Some(k) => k,
        None if provider.category() == ProviderCategory::Local => String::new(),
        None => {
            return (
                502,
                error_json_bytes(
                    &format!("No API key configured for embeddings provider {}", provider.as_str()),
                    "authentication_error",
                ),
            );
        }
    };

    let endpoint = header_value(req, "x-toksi-upstream")
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!("{}/embeddings", provider.effective_endpoint().trim_end_matches('/'))
        });
    let Ok(client) = crate::gateway::reconnect::global_healing_pool().get_async_client() else {
        return (502, error_json_bytes("Failed to acquire upstream HTTP client", "upstream_gateway_error"));
    };

    let mut builder = client.post(&endpoint).json(&payload);
    if !key_str.is_empty() {
        builder = builder.bearer_auth(&key_str);
    }
    match builder.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let bytes = resp.bytes().await.unwrap_or_default();
            (status, bytes.to_vec())
        }
        Err(e) => (502, error_json_bytes(&e.to_string(), "upstream_gateway_error")),
    }
}

fn handle_models_list() -> Vec<u8> {
    let data: Vec<Value> = crate::gateway::providers::ProviderId::ALL
        .iter()
        .map(|p| {
            json!({
                "id": p.as_str(),
                "object": "model",
                "owned_by": format!("{:?}", p.category()).to_lowercase(),
                "endpoint": p.effective_endpoint(),
            })
        })
        .collect();
    serde_json::to_vec(&json!({ "object": "list", "data": data })).unwrap_or_default()
}

async fn handle_anthropic_messages(
    req: &ParsedHttpRequest,
    orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> (u16, Vec<u8>) {
    let mut payload: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return (400, error_json_bytes(&format!("Invalid JSON payload: {}", e), "invalid_request_error")),
    };
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("claude-3-7-sonnet-20250219")
        .to_string();
    let vk = header_value(req, crate::gateway::governance::TOKSI_VK_HEADER);
    let session_id = header_value(req, "x-toksi-session").unwrap_or("gateway-anthropic");
    let explicit_key = extract_auth_key(req);
    let explicit_upstream = header_value(req, "x-toksi-upstream");
    let candidates = resolve_candidates_for_request(req, &model, crate::gateway::providers::ProviderId::Anthropic);

    match orchestrator
        .dispatch_with_cascade_async(
            session_id,
            vk,
            explicit_key.as_deref(),
            explicit_upstream,
            &candidates,
            &crate::gateway::router::RoutingPolicy::default(),
            &mut payload,
        )
        .await
    {
        Ok((_, unified)) => {
            if !unified.raw.is_null() {
                (200, serde_json::to_vec(&unified.raw).unwrap_or_default())
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let resp = json!({
                    "id": if unified.id.is_empty() { format!("msg_{}", now) } else { unified.id },
                    "type": "message",
                    "role": "assistant",
                    "content": [{ "type": "text", "text": unified.content }],
                    "model": unified.model,
                    "stop_reason": unified.finish_reason.unwrap_or_else(|| "end_turn".to_string()),
                    "usage": {
                        "input_tokens": unified.input_tokens,
                        "output_tokens": unified.output_tokens
                    }
                });
                (200, serde_json::to_vec(&resp).unwrap_or_default())
            }
        }
        Err(e) => (502, error_json_bytes(&e, "upstream_gateway_error")),
    }
}

async fn handle_gemini_models(
    path: &str,
    req: &ParsedHttpRequest,
    orchestrator: &Arc<GatewayProxyOrchestrator>,
) -> (u16, Vec<u8>) {
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
        return (200, serde_json::to_vec(&resp).unwrap_or_default());
    }

    let raw_body: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return (400, error_json_bytes(&format!("Invalid JSON payload: {}", e), "invalid_request_error")),
    };
    let model = path
        .trim_start_matches("/v1beta/models/")
        .split(':')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("gemini-2.5-flash")
        .to_string();

    // Normalize Gemini `contents` to `messages` so Headroom & cascade can run uniformly
    let messages: Vec<Value> = raw_body
        .get("contents")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|c| {
                    let role = c.get("role").and_then(Value::as_str).unwrap_or("user");
                    let text = c
                        .get("parts")
                        .and_then(Value::as_array)
                        .and_then(|p| p.first())
                        .and_then(|p| p.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    json!({ "role": if role == "model" { "assistant" } else { role }, "content": text })
                })
                .collect()
        })
        .unwrap_or_default();

    let mut chat_payload = json!({ "model": model, "messages": messages });
    let vk = header_value(req, crate::gateway::governance::TOKSI_VK_HEADER);
    let session_id = header_value(req, "x-toksi-session").unwrap_or("gateway-gemini");
    let explicit_key = extract_auth_key(req);
    let explicit_upstream = header_value(req, "x-toksi-upstream");
    let candidates = resolve_candidates_for_request(req, &model, crate::gateway::providers::ProviderId::Gemini);

    match orchestrator
        .dispatch_with_cascade_async(
            session_id,
            vk,
            explicit_key.as_deref(),
            explicit_upstream,
            &candidates,
            &crate::gateway::router::RoutingPolicy::default(),
            &mut chat_payload,
        )
        .await
    {
        Ok((_, unified)) => {
            if !unified.raw.is_null() && unified.raw.get("candidates").is_some() {
                (200, serde_json::to_vec(&unified.raw).unwrap_or_default())
            } else {
                let resp = json!({
                    "candidates": [{
                        "content": {
                            "parts": [{ "text": unified.content }],
                            "role": "model"
                        },
                        "finishReason": "STOP"
                    }],
                    "usageMetadata": {
                        "promptTokenCount": unified.input_tokens,
                        "candidatesTokenCount": unified.output_tokens,
                        "totalTokenCount": unified.input_tokens + unified.output_tokens
                    }
                });
                (200, serde_json::to_vec(&resp).unwrap_or_default())
            }
        }
        Err(e) => (502, error_json_bytes(&e, "upstream_gateway_error")),
    }
}
