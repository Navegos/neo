// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/dashboard/server.rs

use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

use super::api::*;
use super::assets::{APP_JS, INDEX_HTML, STYLE_CSS};

pub struct DashboardServer {
    pub host: String,
    pub port: u16,
    pub shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn parse(raw: &[u8]) -> Result<(Self, usize), String> {
        let mut headers_end = 0;
        for i in 0..raw.len().saturating_sub(3) {
            if &raw[i..i + 4] == b"\r\n\r\n" {
                headers_end = i + 4;
                break;
            }
        }
        if headers_end == 0 {
            return Err("Incomplete HTTP headers".into());
        }

        let header_str = String::from_utf8_lossy(&raw[..headers_end]);
        let mut lines = header_str.split("\r\n");

        let request_line = lines.next().ok_or("Empty request line")?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or("Missing HTTP method")?.to_uppercase();
        let full_uri = parts.next().ok_or("Missing URI")?;

        let (path, query_str) = match full_uri.find('?') {
            Some(idx) => (&full_uri[..idx], &full_uri[idx + 1..]),
            None => (full_uri, ""),
        };

        let mut query = HashMap::new();
        if !query_str.is_empty() {
            for pair in query_str.split('&') {
                let mut kv = pair.split('=');
                if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                    query.insert(k.to_string(), v.to_string());
                }
            }
        }

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some(colon) = line.find(':') {
                let key = line[..colon].trim().to_lowercase();
                let val = line[colon + 1..].trim().to_string();
                headers.insert(key, val);
            }
        }

        let content_len = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let total_expected = headers_end + content_len;
        if raw.len() < total_expected {
            return Err("Incomplete request body".into());
        }

        let body = raw[headers_end..total_expected].to_vec();

        Ok((
            HttpRequest {
                method,
                path: path.to_string(),
                query,
                headers,
                body,
            },
            total_expected,
        ))
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub status_text: &'static str,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok_html(body: &str) -> Self {
        Self {
            status: 200,
            status_text: "OK",
            content_type: "text/html; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn ok_css(body: &str) -> Self {
        Self {
            status: 200,
            status_text: "OK",
            content_type: "text/css; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn ok_js(body: &str) -> Self {
        Self {
            status: 200,
            status_text: "OK",
            content_type: "application/javascript; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn ok_json(val: serde_json::Value) -> Self {
        let bytes = serde_json::to_vec(&val).unwrap_or_default();
        Self {
            status: 200,
            status_text: "OK",
            content_type: "application/json",
            body: bytes,
        }
    }

    pub fn bad_request(msg: &str) -> Self {
        let val = serde_json::json!({ "error": msg });
        let bytes = serde_json::to_vec(&val).unwrap_or_default();
        Self {
            status: 400,
            status_text: "Bad Request",
            content_type: "application/json",
            body: bytes,
        }
    }

    pub fn not_found() -> Self {
        let val = serde_json::json!({ "error": "Not Found" });
        let bytes = serde_json::to_vec(&val).unwrap_or_default();
        Self {
            status: 404,
            status_text: "Not Found",
            content_type: "application/json",
            body: bytes,
        }
    }

    pub fn no_content() -> Self {
        Self {
            status: 204,
            status_text: "No Content",
            content_type: "text/plain",
            body: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let header = format!(
            "HTTP/1.1 {} {}\r\n\
            Content-Type: {}\r\n\
            Content-Length: {}\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Connection: close\r\n\r\n",
            self.status,
            self.status_text,
            self.content_type,
            self.body.len()
        );
        let mut out = header.into_bytes();
        out.extend_from_slice(&self.body);
        out
    }
}

impl DashboardServer {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            shutdown_tx: None,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        let (broadcast_tx, _) = broadcast::channel::<String>(32);

        // Background SSE ticker broadcaster
        let ticker_tx = broadcast_tx.clone();
        tokio::spawn(async move {
            let mut tick_counter: u64 = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(1500)).await;
                tick_counter = tick_counter.wrapping_add(1);
                let lifetime = 14_892_104 + (tick_counter * 142);
                let saved_usd = 184.26 + ((tick_counter as f64) * 0.003);
                let payload = serde_json::json!({
                    "type": "tick",
                    "payload": {
                        "tokens_saved_lifetime": lifetime,
                        "cost_saved_usd": saved_usd,
                        "total_calls": 1284 + tick_counter
                    }
                });
                let _ = ticker_tx.send(payload.to_string());
            }
        });

        loop {
            match listener.accept().await {
                Ok((stream, _peer)) => {
                    let btx = broadcast_tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_connection(stream, btx).await;
                    });
                }
                Err(e) => {
                    eprintln!("Dashboard server accept error: {}", e);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    broadcast_tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; 65536];
    let mut total_read = 0;

    // Read initial request bytes
    loop {
        let n = stream.read(&mut buf[total_read..]).await?;
        if n == 0 {
            return Ok(());
        }
        total_read += n;

        // Try parsing HTTP request
        match HttpRequest::parse(&buf[..total_read]) {
            Ok((req, _len)) => {
                if req.path == "/api/events" && req.method == "GET" {
                    return handle_sse_stream(stream, broadcast_tx).await;
                }

                let resp = route_request(&req);
                let bytes = resp.to_bytes();
                stream.write_all(&bytes).await?;
                stream.flush().await?;
                return Ok(());
            }
            Err(e) if e == "Incomplete HTTP headers" || e == "Incomplete request body" => {
                if total_read >= buf.len() {
                    buf.resize(buf.len() * 2, 0);
                }
                continue;
            }
            Err(e) => {
                let resp = HttpResponse::bad_request(&e);
                stream.write_all(&resp.to_bytes()).await?;
                stream.flush().await?;
                return Ok(());
            }
        }
    }
}

async fn handle_sse_stream(
    mut stream: TcpStream,
    broadcast_tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let header = "HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-cache\r\n\
        Connection: keep-alive\r\n\
        Access-Control-Allow-Origin: *\r\n\r\n";

    stream.write_all(header.as_bytes()).await?;
    stream.flush().await?;

    let mut rx = broadcast_tx.subscribe();

    // Send initial handshake tick
    let initial_msg = format!("data: {}\n\n", serde_json::json!({
        "type": "tick",
        "payload": {
            "tokens_saved_lifetime": 14_892_104,
            "cost_saved_usd": 184.26,
            "total_calls": 1284
        }
    }));
    stream.write_all(initial_msg.as_bytes()).await?;
    stream.flush().await?;

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let sse_line = format!("data: {}\n\n", msg);
                if let Err(_) = stream.write_all(sse_line.as_bytes()).await {
                    break;
                }
                if let Err(_) = stream.flush().await {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }

    Ok(())
}

fn route_request(req: &HttpRequest) -> HttpResponse {
    if req.method == "OPTIONS" {
        return HttpResponse::no_content();
    }

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => HttpResponse::ok_html(INDEX_HTML),
        ("GET", "/style.css") => HttpResponse::ok_css(STYLE_CSS),
        ("GET", "/app.js") => HttpResponse::ok_js(APP_JS),

        ("GET", "/api/overview") | ("GET", "/api/status") => {
            HttpResponse::ok_json(handle_overview())
        }
        ("GET", "/api/waterfall") => HttpResponse::ok_json(handle_waterfall()),
        ("GET", "/api/warroom") => HttpResponse::ok_json(handle_warroom()),
        ("GET", "/api/agents") => HttpResponse::ok_json(handle_agents()),

        ("POST", "/api/agents/toggle") => match handle_toggle_agent(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },

        ("GET", "/api/memory") => {
            let q = req.query.get("q").map(|s| s.as_str());
            HttpResponse::ok_json(handle_memory(q))
        }

        ("GET", "/api/codegraph") => HttpResponse::ok_json(handle_codegraph()),
        ("GET", "/api/codegraph/blast") => {
            let sym = req.query.get("symbol").cloned().unwrap_or_else(|| "run_dashboard".into());
            let depth = req
                .query
                .get("depth")
                .and_then(|d| d.parse::<usize>().ok())
                .unwrap_or(3);
            HttpResponse::ok_json(handle_codegraph_blast(&sym, depth))
        }

        ("POST", "/api/contextmode/filter") => match handle_contextmode_filter(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },

        ("POST", "/api/system/proxy-restart") => HttpResponse::ok_json(handle_proxy_restart()),
        ("POST", "/api/system/proxy-stop") => HttpResponse::ok_json(handle_proxy_stop()),
        ("POST", "/api/system/cache-vacuum") => HttpResponse::ok_json(handle_cache_vacuum()),

        ("GET", "/api/system/byok") => HttpResponse::ok_json(handle_byok_get()),
        ("POST", "/api/system/byok-switch") => match handle_byok_switch(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("DELETE", "/api/system/byok") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            match handle_byok_delete(id) {
                Ok(val) => HttpResponse::ok_json(val),
                Err(e) => HttpResponse::bad_request(&e),
            }
        }

        _ => HttpResponse::not_found(),
    }
}
