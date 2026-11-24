// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/server.rs
// created: 2026-10-30
// lastModified: 2026-11-16

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

use super::api::*;
use super::assets::{APP_JS, INDEX_HTML, STYLE_CSS};
use super::sse::{handle_sse_stream, start_sse_ticker};

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

    pub fn ok_plain(body: &str) -> Self {
        Self {
            status: 200,
            status_text: "OK",
            content_type: "text/plain; version=0.0.4; charset=utf-8",
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

pub struct IsolatedDashboardHandle {
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    join_handle: Option<std::thread::JoinHandle<()>>,
    pub thread_name: String,
    pub port: u16,
}

impl IsolatedDashboardHandle {
    pub fn is_finished(&self) -> bool {
        self.join_handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true)
    }

    pub fn shutdown(&mut self) {
        let _ = self.shutdown_tx.send(true);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for IsolatedDashboardHandle {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
    }
}

/// Spawns Port 9100 Web Dashboard Cockpit in its own isolated OS thread & Tokio reactor
/// strictly separated from Port 9099 inference threads so slow browser downloads or SSE
/// clients cannot induce TCP backpressure on streaming LLM inference.
pub fn spawn_isolated_dashboard(
    host: &str,
    port: u16,
) -> std::io::Result<IsolatedDashboardHandle> {
    let (shutdown_tx, _shutdown_rx) = tokio::sync::watch::channel(false);
    let host_owned = host.to_string();
    let thread_name = format!("toksi-dashboard-{}", port);
    let tx_for_server = shutdown_tx.clone();

    let join_handle = std::thread::Builder::new()
        .name(thread_name.clone())
        .spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("toksi-dashboard-cockpit")
                .enable_all()
                .build()
            else {
                crate::util::logger::L.err("Failed to initialize isolated Dashboard Tokio runtime");
                return;
            };
            rt.block_on(async move {
                let mut server = DashboardServer::new(&host_owned, port);
                server.shutdown_tx = Some(tx_for_server);
                if let Err(e) = server.run().await {
                    crate::util::logger::L.warn(&format!("Dashboard server error: {}", e));
                }
            });
        })?;

    Ok(IsolatedDashboardHandle {
        shutdown_tx,
        join_handle: Some(join_handle),
        thread_name,
        port,
    })
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

        // Background SSE ticker broadcaster broadcasting ground-truth session metrics
        let _ticker_handle = start_sse_ticker(broadcast_tx.clone());

        let mut shutdown_rx = self.shutdown_tx.as_ref().map(|tx| tx.subscribe());
        loop {
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream, _peer)) => {
                            let btx = broadcast_tx.clone();
                            tokio::spawn(async move {
                                let _ = handle_connection(stream, btx).await;
                            });
                        }
                        Err(e) => {
                            crate::util::logger::L.warn(&format!("Dashboard server accept error: {}", e));
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    }
                }
                _ = async {
                    if let Some(ref mut rx) = shutdown_rx {
                        let _ = rx.changed().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    break;
                }
            }
        }
        Ok(())
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    broadcast_tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; 65536];
    let mut total_read = 0;

    // Read initial request bytes with bounded timeout so stalled browser sockets cannot hang tasks
    loop {
        let n = match tokio::time::timeout(Duration::from_secs(10), stream.read(&mut buf[total_read..])).await {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Ok(()),
        };
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
                let _ = tokio::time::timeout(Duration::from_secs(10), async {
                    stream.write_all(&bytes).await?;
                    stream.flush().await
                })
                .await;
                return Ok(());
            }
            Err(e) if e == "Incomplete HTTP headers" || e == "Incomplete request body" => {
                if total_read >= buf.len() {
                    if buf.len() >= 4 * 1024 * 1024 {
                        return Err("Request too large".into());
                    }
                    buf.resize(buf.len() * 2, 0);
                }
                continue;
            }
            Err(e) => {
                let resp = HttpResponse::bad_request(&e);
                let _ = tokio::time::timeout(Duration::from_secs(5), async {
                    stream.write_all(&resp.to_bytes()).await?;
                    stream.flush().await
                })
                .await;
                return Ok(());
            }
        }
    }
}

fn route_request(req: &HttpRequest) -> HttpResponse {
    if req.method == "OPTIONS" {
        return HttpResponse::no_content();
    }

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => HttpResponse::ok_html(INDEX_HTML),
        ("GET", "/style.css") => HttpResponse::ok_css(STYLE_CSS),
        ("GET", "/app.js") => HttpResponse::ok_js(APP_JS),
        ("GET", "/metrics") => HttpResponse::ok_plain(&crate::telemetry::render_metrics()),

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

        ("GET", "/api/memory") | ("GET", "/api/memory/dag") => {
            let q = req.query.get("q").map(|s| s.as_str());
            let sub = req.query.get("subproject").map(|s| s.as_str());
            HttpResponse::ok_json(handle_memory_filtered(q, sub))
        }
        ("GET", p) if p.starts_with("/api/memory/heatmap/") => {
            let cell_id = p.trim_start_matches("/api/memory/heatmap/");
            let decoded = cell_id.replace("%2F", "/").replace("%2f", "/");
            HttpResponse::ok_json(handle_memory_heatmap_drilldown(&decoded))
        }

        ("GET", "/api/codegraph") | ("GET", "/api/graph/explore") => HttpResponse::ok_json(handle_codegraph()),
        ("GET", "/api/codegraph/blast") | ("GET", "/api/graph/impact") => {
            let sym = req
                .query
                .get("symbol")
                .or_else(|| req.query.get("sym"))
                .cloned()
                .unwrap_or_else(|| "run_dashboard".into());
            let depth = req
                .query
                .get("depth")
                .and_then(|d| d.parse::<usize>().ok())
                .unwrap_or(3);
            HttpResponse::ok_json(handle_codegraph_blast(&sym, depth))
        }
        ("GET", "/api/codegraph/preview") => {
            let mode = req.query.get("mode").map(|s| s.as_str()).unwrap_or("ast-grep");
            let pattern = req.query.get("pattern").map(|s| s.as_str()).unwrap_or("");
            HttpResponse::ok_json(handle_codegraph_preview(mode, pattern))
        }
        ("POST", "/api/codegraph/purge-ignored") => HttpResponse::ok_json(handle_codegraph_purge_ignored()),

        ("POST", "/api/contextmode/filter") => match handle_contextmode_filter(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("GET", "/api/contextmode/search") => {
            let q = req.query.get("q").map(|s| s.as_str()).unwrap_or("");
            HttpResponse::ok_json(handle_contextmode_search(q))
        }
        ("GET", "/api/contextmode/telemetry") => HttpResponse::ok_json(handle_contextmode_telemetry()),

        ("POST", "/api/system/proxy-restart") => HttpResponse::ok_json(handle_proxy_restart()),
        ("POST", "/api/system/proxy-stop") => HttpResponse::ok_json(handle_proxy_stop()),
        ("POST", "/api/system/cache-vacuum") => HttpResponse::ok_json(handle_cache_vacuum()),
        ("GET", "/api/system/config") => HttpResponse::ok_json(handle_system_config_get()),
        ("POST", "/api/system/config") => match handle_system_config_save(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("POST", "/api/system/service") => match handle_system_service(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("POST", "/api/system/agent-install") => match handle_system_agent_install(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("GET", "/api/system/logs") => HttpResponse::ok_json(handle_system_logs()),
        ("GET", "/api/supervisor/workers") => HttpResponse::ok_json(handle_supervisor_workers()),
        ("GET", "/api/activity") => HttpResponse::ok_json(handle_activity_get()),
        ("GET", "/api/orchestrator/modes") => HttpResponse::ok_json(handle_orchestrator_modes_get()),
        ("PUT", "/api/orchestrator/modes") => match handle_orchestrator_modes_put(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("POST", p) if p.starts_with("/api/daemon/service/") => {
            let rest = p.trim_start_matches("/api/daemon/service/");
            if let Some((name, action)) = rest.split_once('/') {
                match handle_daemon_service_action(name, action) {
                    Ok(val) => HttpResponse::ok_json(val),
                    Err(e) => HttpResponse::bad_request(&e),
                }
            } else {
                HttpResponse::bad_request("Expected /api/daemon/service/:name/:action")
            }
        }

        ("GET", "/api/providers") => HttpResponse::ok_json(handle_providers_get()),
        ("POST", "/api/providers/update") => match handle_provider_update(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },

        ("GET", "/api/complexity") => HttpResponse::ok_json(handle_complexity_get()),
        ("POST", "/api/complexity/update") => match handle_complexity_update(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },

        ("GET", "/api/quotas") => HttpResponse::ok_json(handle_quotas_get()),
        ("GET", "/api/keyring/status") => HttpResponse::ok_json(handle_keyring_status()),
        ("POST", "/api/keyring/set") => match handle_keyring_set(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("GET", "/api/plugins") => HttpResponse::ok_json(handle_plugins_get()),
        ("POST", "/api/plugins/toggle") => match handle_plugin_toggle(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("POST", p) if p.starts_with("/api/plugins/") => {
            let rest = p.trim_start_matches("/api/plugins/");
            match handle_plugin_action(rest) {
                Ok(val) => HttpResponse::ok_json(val),
                Err(e) => HttpResponse::bad_request(&e),
            }
        }
        ("GET", "/api/skills/status") => HttpResponse::ok_json(handle_skills_status()),
        ("POST", "/api/skills/sync") => HttpResponse::ok_json(handle_skills_sync()),
        ("GET", "/api/security/advisory") => {
            let q = req.query.get("q").map(|s| s.as_str());
            HttpResponse::ok_json(handle_security_advisory(q))
        }
        ("POST", "/api/security/scan") => HttpResponse::ok_json(handle_security_scan(&req.body)),
        ("GET", "/api/security/sandbox") => HttpResponse::ok_json(handle_security_sandbox()),
        ("GET", "/api/security/audit-evidence") => HttpResponse::ok_json(handle_security_audit_evidence()),
        ("GET", "/api/telemetry/spans") => HttpResponse::ok_json(handle_telemetry_spans()),

        ("GET", "/api/db/status") => HttpResponse::ok_json(handle_db_status()),
        ("GET", "/api/db/table") => {
            let name = req.query.get("name").map(|s| s.as_str()).unwrap_or("byok_profiles");
            HttpResponse::ok_json(handle_db_table_browse(name))
        }
        ("POST", "/api/db/backup") => HttpResponse::ok_json(handle_db_backup()),
        ("POST", "/api/db/restore") => match handle_db_restore(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("GET", "/api/db/export") | ("POST", "/api/db/export") => HttpResponse::ok_json(handle_db_export()),
        ("POST", "/api/db/import") => match handle_db_import(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("POST", "/api/db/repair-legacy") => HttpResponse::ok_json(handle_db_repair_legacy()),

        ("GET", "/api/byok/profiles") => HttpResponse::ok_json(handle_byok_profiles_get()),
        ("POST", "/api/byok/profiles") | ("PUT", "/api/byok/profiles") => match handle_byok_profile_upsert(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("DELETE", "/api/byok/profiles") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            match handle_byok_profile_delete(id) {
                Ok(val) => HttpResponse::ok_json(val),
                Err(e) => HttpResponse::bad_request(&e),
            }
        }
        ("POST", "/api/byok/models-refresh") => match handle_byok_models_refresh(&req.body) {
            Ok(val) => HttpResponse::ok_json(val),
            Err(e) => HttpResponse::bad_request(&e),
        },
        ("GET", "/api/gateway/bypass") => HttpResponse::ok_json(handle_gateway_bypass()),

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
