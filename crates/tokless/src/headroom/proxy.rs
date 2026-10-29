// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/proxy.rs
// created: 2026-10-05
// lastModified: 2026-10-28

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::Ipv6Addr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};

use super::autostart::{disable_proxy_autostart, stop_proxy_supervisor_for_patch};
use super::byok_gateway::{
    byok_gateway_live, byok_routes_configured, byok_routes_path, start_byok_gateway,
    stop_byok_gateway,
};
use super::cloudcode_patch::apply_cloud_code_patch;
pub use super::copilot_daemon::ensure_copilot_proxy_up;
use super::process_identity::{
    identify_process, kill_process, process_gone, process_identity_supported, spawn_detached,
    ProcessIdentityInfo,
};
pub use super::watchdog::run_proxy_watchdog;
use super::watchdog::{
    clear_proxy_stop_request, proxy_stop_requested, request_proxy_stop, run_headroom_supervised,
};
use crate::util::byok::{byok_gateway_port, parse_byok_routes};
use crate::util::headroom::{headroom_bin, headroom_installed, headroom_paths_resolved};
use crate::util::headroom_proxy::{
    clear_headroom_proxy_runtime, headroom_proxy_openai_url, headroom_proxy_port,
    headroom_proxy_url, proxy_routing_enabled, read_proxy_runtime, save_headroom_proxy_runtime,
    ProxyRuntime,
};
use crate::util::logger::{with_quiet, L};
use crate::util::paths::{
    ensure_dir, exists, read_file_safe, tokless_data_dir, which, write_file_atomic, IS_WIN,
};

pub const PROXY_READY_TIMEOUT: Duration = Duration::from_secs(15);
pub const PROXY_STOP_TIMEOUT: Duration = Duration::from_secs(5);
pub const PROXY_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
pub const PROXY_POLL_INTERVAL: Duration = Duration::from_millis(300);
pub const PROXY_IDENTITY_TIMEOUT: Duration = Duration::from_secs(2);
pub const PROXY_START_LOCK_WAIT: Duration = Duration::from_secs(30);
pub const PROXY_START_LOCK_STALE: Duration = Duration::from_secs(60);
pub const DEFAULT_ANTHROPIC_TARGET_URL: &str = "https://api.anthropic.com";
pub const DEFAULT_OPENAI_TARGET_URL: &str = "https://api.openai.com";
pub const HEADROOM_UPSTREAM_RESOLVE_TIMEOUT: &str = "8";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProxyOwnership {
    pub pid: u32,
    pub executable: String,
    pub args: Vec<String>,
    pub start_fingerprint: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub allowed_base_urls: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProxySupervisedState {
    pub pid: u32,
    pub executable: String,
    pub args: Vec<String>,
    #[serde(rename = "start_fingerprint")]
    pub start: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub allowed_base_urls: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub manager_args: Vec<String>,
}

pub fn proxy_port() -> u16 {
    headroom_proxy_port()
}

pub fn proxy_url() -> String {
    headroom_proxy_url()
}

pub fn proxy_openai_url() -> String {
    headroom_proxy_openai_url()
}

pub fn proxy_ownership_file() -> String {
    proxy_files().0
}

/// Returns the provider hosts Headroom should reach.
pub fn real_upstream_urls() -> (String, String) {
    let mut anthropic = DEFAULT_ANTHROPIC_TARGET_URL.to_string();
    let mut openai = DEFAULT_OPENAI_TARGET_URL.to_string();

    if let Ok(v) = std::env::var("TOKLESS_HEADROOM_ANTHROPIC_URL") {
        if !v.trim().is_empty() {
            anthropic = v.trim().to_string();
        }
    } else if let Some(st) = read_proxy_runtime() {
        if !st.anthropic_api_url.is_empty() {
            anthropic = st.anthropic_api_url;
        }
    }

    if let Ok(v) = std::env::var("TOKLESS_HEADROOM_OPENAI_URL") {
        if !v.trim().is_empty() {
            openai = v.trim().to_string();
        }
    } else if let Some(st) = read_proxy_runtime() {
        if !st.openai_api_url.is_empty() {
            openai = st.openai_api_url;
        }
    }

    (anthropic, openai)
}

/// Returns the URLs passed to `headroom proxy --*-api-url`.
pub fn proxy_upstream_urls() -> (String, String) {
    real_upstream_urls()
}

/// Returns optional upstream overrides for the Gemini (`--gemini-api-url`)
/// and CloudCode (`--cloudcode-api-url`) provider routes.
pub fn proxy_upstream_gemini_urls() -> (String, String) {
    let gemini = if let Ok(v) = std::env::var("TOKLESS_HEADROOM_GEMINI_URL") {
        v.trim().to_string()
    } else if let Some(st) = read_proxy_runtime() {
        st.gemini_api_url
    } else {
        String::new()
    };

    let cloudcode = if let Ok(v) = std::env::var("TOKLESS_HEADROOM_CLOUDCODE_URL") {
        v.trim().to_string()
    } else if let Some(st) = read_proxy_runtime() {
        st.cloudcode_api_url
    } else {
        String::new()
    };

    (gemini, cloudcode)
}

pub fn proxy_args(port: u16) -> Vec<String> {
    let (anthropic, openai) = proxy_upstream_urls();
    let (gemini, cloudcode) = proxy_upstream_gemini_urls();
    proxy_args_for_urls(port, &anthropic, &openai, &gemini, &cloudcode)
}

pub fn proxy_args_for_urls(
    port: u16,
    anthropic: &str,
    openai: &str,
    gemini: &str,
    cloudcode: &str,
) -> Vec<String> {
    let mut args = vec![
        "proxy".to_string(),
        "--port".to_string(),
        port.to_string(),
        "--no-cache".to_string(),
        "--anthropic-api-url".to_string(),
        anthropic.to_string(),
        "--openai-api-url".to_string(),
        openai.to_string(),
    ];
    if !gemini.is_empty() {
        args.push("--gemini-api-url".to_string());
        args.push(gemini.to_string());
    }
    if !cloudcode.is_empty() {
        args.push("--cloudcode-api-url".to_string());
        args.push(cloudcode.to_string());
    }
    args
}

pub fn proxy_args_from_runtime(port: u16) -> Vec<String> {
    if let Some(st) = read_proxy_runtime() {
        if !st.anthropic_api_url.trim().is_empty() && !st.openai_api_url.trim().is_empty() {
            return proxy_args_for_urls(
                port,
                &st.anthropic_api_url,
                &st.openai_api_url,
                &st.gemini_api_url,
                &st.cloudcode_api_url,
            );
        }
    }
    proxy_args(port)
}

pub fn proxy_port_from_runtime() -> u16 {
    if let Some(st) = read_proxy_runtime() {
        if st.port > 0 {
            return st.port;
        }
    }
    proxy_port()
}

pub fn proxy_daemon_env() -> Vec<(String, String)> {
    const BLOCKED: &[&str] = &[
        "TOKLESS_PROXY_PROVIDER",
        "TOKLESS_HEADROOM_PROXY_PORT",
        "TOKLESS_HEADROOM_ANTHROPIC_URL",
        "TOKLESS_HEADROOM_OPENAI_URL",
        "TOKLESS_HEADROOM_GEMINI_URL",
        "TOKLESS_HEADROOM_CLOUDCODE_URL",
        "OPENAI_TARGET_API_URL",
        "ANTHROPIC_TARGET_API_URL",
        "GROK_MODELS_BASE_URL",
        "HEADROOM_UPSTREAM_RESOLVE_TIMEOUT_S",
        "HEADROOM_ALLOWED_BASE_URLS",
    ];

    let mut out: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| !BLOCKED.contains(&k.as_str()))
        .collect();

    let timeout = std::env::var("HEADROOM_UPSTREAM_RESOLVE_TIMEOUT_S")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| HEADROOM_UPSTREAM_RESOLVE_TIMEOUT.to_string());
    out.push(("HEADROOM_UPSTREAM_RESOLVE_TIMEOUT_S".to_string(), timeout));

    let allowed = headroom_allowed_base_urls();
    if !allowed.is_empty() {
        out.push(("HEADROOM_ALLOWED_BASE_URLS".to_string(), allowed));
    }
    out
}

pub fn canonical_allowlist_origin(
    raw: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("empty origin".into());
    }
    let Some((scheme_raw, after_scheme)) = raw.split_once("://") else {
        return Err(format!("origin missing scheme: {}", raw).into());
    };
    let scheme = scheme_raw.to_lowercase();
    if !matches!(scheme.as_str(), "http" | "https" | "ws" | "wss") {
        return Err(format!("unsupported scheme: {}", scheme).into());
    }
    if after_scheme.contains('#') {
        return Err(format!("origin must not contain fragment: {}", raw).into());
    }
    if after_scheme.contains('?') {
        return Err(format!("origin must not contain query string: {}", raw).into());
    }
    let (authority, path) = match after_scheme.find('/') {
        Some(idx) => (&after_scheme[..idx], &after_scheme[idx..]),
        None => (after_scheme, ""),
    };
    if !path.is_empty() && path != "/" {
        return Err(format!("origin must not contain path: {}", raw).into());
    }
    if authority.contains('@') {
        return Err(format!("origin must not contain userinfo: {}", raw).into());
    }
    if authority.ends_with(':') {
        return Err(format!("origin has empty port: {}", raw).into());
    }

    let (hostname, port_str) = if authority.starts_with('[') || authority.contains(']') {
        if !authority.starts_with('[') || !authority.contains(']') {
            return Err(format!("malformed IPv6 literal in origin: {}", raw).into());
        }
        let Some(close) = authority.find(']') else {
            return Err(format!("malformed IPv6 literal in origin: {}", raw).into());
        };
        let host_inner = &authority[1..close];
        let rest = &authority[close + 1..];
        if host_inner.is_empty() {
            return Err(format!("missing host in origin: {}", raw).into());
        }
        if host_inner.contains('%') {
            return Err(format!("IPv6 zone identifiers not allowed: {}", raw).into());
        }
        if host_inner.chars().any(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) {
            return Err(format!("host contains whitespace: {}", raw).into());
        }
        if host_inner.parse::<Ipv6Addr>().is_err() {
            return Err(format!("invalid IPv6 address: {}", raw).into());
        }
        let port = if rest.is_empty() {
            ""
        } else if let Some(p) = rest.strip_prefix(':') {
            p
        } else {
            return Err(format!("malformed IPv6 literal in origin: {}", raw).into());
        };
        (host_inner.to_lowercase(), port)
    } else {
        if authority.matches(':').count() > 1 {
            return Err(format!("invalid colon in hostname: {}", raw).into());
        }
        let (h, p) = match authority.split_once(':') {
            Some((h, p)) => (h, p),
            None => (authority, ""),
        };
        if h.is_empty() {
            return Err(format!("missing host in origin: {}", raw).into());
        }
        if h.contains('%') {
            return Err(format!("IPv6 zone identifiers not allowed: {}", raw).into());
        }
        if h.chars().any(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) {
            return Err(format!("host contains whitespace: {}", raw).into());
        }
        (h.to_lowercase(), p)
    };

    if !port_str.is_empty() {
        let p: i32 = port_str
            .parse()
            .map_err(|_| format!("invalid port in origin: {}", raw))?;
        if p <= 0 || p > 65535 {
            return Err(format!("invalid port in origin: {}", raw).into());
        }
    }

    let is_default_port = port_str.is_empty()
        || ((scheme == "http" || scheme == "ws") && port_str == "80")
        || ((scheme == "https" || scheme == "wss") && port_str == "443");

    if is_default_port {
        if hostname.contains(':') {
            Ok(format!("{}://[{}]", scheme, hostname))
        } else {
            Ok(format!("{}://{}", scheme, hostname))
        }
    } else if hostname.contains(':') {
        Ok(format!("{}://[{}]:{}", scheme, hostname, port_str))
    } else {
        Ok(format!("{}://{}:{}", scheme, hostname, port_str))
    }
}

pub fn headroom_allowed_base_urls() -> String {
    headroom_allowed_base_urls_checked().unwrap_or_default()
}

pub fn headroom_allowed_base_urls_checked() -> Result<String, Box<dyn std::error::Error + Send + Sync>>
{
    let mut allowed = BTreeSet::new();
    if let Ok(existing) = std::env::var("HEADROOM_ALLOWED_BASE_URLS") {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            for part in trimmed.split(',') {
                let p = part.trim();
                if p.is_empty() {
                    continue;
                }
                let canon = canonical_allowlist_origin(p).map_err(
                    |e| -> Box<dyn std::error::Error + Send + Sync> {
                        format!("invalid HEADROOM_ALLOWED_BASE_URLS entry {:?}: {}", p, e).into()
                    },
                )?;
                allowed.insert(canon);
            }
        }
    }

    let port_str = byok_gateway_port().to_string();
    for origin in [
        format!("http://127.0.0.1:{}", port_str),
        format!("http://localhost:{}", port_str),
    ] {
        if let Ok(canon) = canonical_allowlist_origin(&origin) {
            allowed.insert(canon);
        }
    }

    let routes_file = byok_routes_path();
    let routes = if let Some(raw) = read_file_safe(&routes_file) {
        match parse_byok_routes(&raw) {
            Ok(r) => r,
            Err(err) => {
                L.warn(&format!("reading BYOK routes for allowlist: {}", err));
                if exists(&routes_file) {
                    return Err(
                        format!("failed to parse configured BYOK route registry: {}", err).into(),
                    );
                }
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    for r in routes {
        if !r.upstream.is_empty() {
            if let Ok(u) = url::Url::parse(&r.upstream) {
                if let Some(host) = u.host_str() {
                    let origin_raw = match u.port() {
                        Some(p) => format!("{}://{}:{}", u.scheme(), host, p),
                        None => format!("{}://{}", u.scheme(), host),
                    };
                    if let Ok(canon) = canonical_allowlist_origin(&origin_raw) {
                        allowed.insert(canon);
                    } else {
                        L.warn(&format!("ignoring invalid BYOK upstream origin: {}", r.upstream));
                    }
                } else {
                    L.warn(&format!("ignoring unparseable BYOK upstream: {}", r.upstream));
                }
            } else {
                L.warn(&format!("ignoring unparseable BYOK upstream: {}", r.upstream));
            }
        }
    }

    let keys: Vec<String> = allowed.into_iter().collect();
    Ok(keys.join(","))
}

pub fn resolve_headroom_bin() -> String {
    let t = crate::util::paths::tokless_abs();
    if !t.is_empty() && exists(&t) {
        return t;
    }
    if let Ok(cur) = std::env::current_exe() {
        let s = cur.to_string_lossy().to_string();
        if exists(&s) {
            return s;
        }
    }
    if headroom_installed() {
        return headroom_bin();
    }
    which("tokless")
}

pub fn proxy_live_z(port: u16) -> bool {
    proxy_live_z_for_port_timeout(port, PROXY_PROBE_TIMEOUT)
}

pub fn proxy_live_z_timeout(timeout: Duration) -> bool {
    proxy_live_z_for_port_timeout(proxy_port(), timeout)
}

fn proxy_live_z_for_port_timeout(port: u16, timeout: Duration) -> bool {
    let url = format!("http://127.0.0.1:{}/livez", port);
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
    else {
        return false;
    };
    let Ok(resp) = client.get(&url).send() else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    #[derive(Deserialize)]
    struct LiveResp {
        #[serde(default)]
        service: String,
        #[serde(default)]
        status: String,
    }
    resp.json::<LiveResp>()
        .map(|l| l.service == "headroom-proxy" || l.service == "tokless-headroom-native" || l.status == "ok")
        .unwrap_or(false)
}

pub fn proxy_running() -> bool {
    proxy_live_z(proxy_port())
}

/// Starts the daemon when it is not already running.
/// Quiet so hook/MCP stdout (JSON protocols) stays clean.
pub fn ensure_proxy_up() {
    let _ = ensure_proxy_up_checked();
}

pub fn ensure_proxy_up_checked() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !proxy_routing_enabled() {
        return Ok(());
    }
    with_quiet(start_proxy)
}

pub fn run_proxy_foreground() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !proxy_routing_enabled() {
        return Ok(());
    }
    let port = proxy_port_from_runtime();
    if read_proxy_runtime().is_none() {
        persist_proxy_runtime(port)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy runtime record: {}", e).into() })?;
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(run_native_headroom_proxy(port))
}

pub async fn run_native_headroom_proxy(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::TcpListener;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    L.info(&format!("headroom native proxy listening on http://127.0.0.1:{}", port));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    let _ = serve_headroom_connection(stream).await;
                });
            }
            Err(_) => break,
        }
    }
    Ok(())
}

async fn serve_headroom_connection(mut stream: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(mut req) = super::byok_gateway::read_http_request(&mut stream).await? else {
        return Ok(());
    };

    let (path, _, _) = super::byok_gateway::split_uri_path_and_query(&req.raw_uri);

    // Health/live checks
    if path == "/livez" || path == "/healthz" || path == "/readyz" {
        let body = br#"{"service":"headroom-proxy","status":"ok"}"#;
        super::byok_gateway::write_http_response(&mut stream, 200, "OK", &[("Content-Type".to_string(), "application/json".to_string())], body).await?;
        return Ok(());
    }

    // Check BYOK routing header or path
    let has_byok_header = req.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(super::byok_gateway::BYOK_ROUTE_HEADER));
    if has_byok_header || path.starts_with("/byok/") {
        let (status, headers, body) = super::byok_gateway::handle_byok_gateway_request(
            &req.method,
            &req.raw_uri,
            &req.headers,
            &req.body,
            None,
        ).await;
        super::byok_gateway::write_http_response(&mut stream, status, super::byok_gateway::status_reason(status), &headers, &body).await?;
        return Ok(());
    }

    // Identify target URL based on route
    let (anthropic_url, openai_url) = real_upstream_urls();
    let (gemini_url, _) = proxy_upstream_gemini_urls();

    let target_base = if path.starts_with("/v1/messages") {
        anthropic_url
    } else if path.starts_with("/v1beta") || path.contains("gemini") {
        if !gemini_url.is_empty() { gemini_url } else { openai_url }
    } else {
        openai_url
    };

    // Preprocessing on POST JSON requests
    if req.method.eq_ignore_ascii_case("POST") && !req.body.is_empty() {
        if let Ok(mut json_val) = serde_json::from_slice::<serde_json::Value>(&req.body) {
            // 1. Dynamic Memory Governor (scaling L0 vs L1 vs L2 based on headroom)
            super::adaptive_mem::inject_adaptive_memory(&mut json_val);

            // 2. 1024-token prompt cache alignment
            if path.starts_with("/v1/messages") {
                super::cache_align::auto_place_anthropic_cache_control(&mut json_val);
            }
            super::cache_align::stabilize_cache_keys(&mut json_val);

            // 3. Compress conversation messages (logs, code, and JSON in tool outputs)
            if let Some(serde_json::Value::Array(messages)) = json_val.get_mut("messages") {
                for msg in messages {
                    let is_tool_role = msg.get("role").and_then(|r| r.as_str()) == Some("tool");
                    let tool_name = msg.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                    if let Some(serde_json::Value::String(content)) = msg.get_mut("content") {
                        if content.len() > 120 {
                            if is_tool_role {
                                *content = crate::rtk::compress_tool_output(&tool_name, content);
                            } else {
                                *content = super::compression::compress_content(content, None);
                            }
                        }
                    } else if let Some(serde_json::Value::Array(blocks)) = msg.get_mut("content") {
                        for block in blocks {
                            let is_tool_result =
                                block.get("type").and_then(|t| t.as_str()) == Some("tool_result");
                            let block_name = block.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                            if let Some(serde_json::Value::String(text)) = block.get_mut("text") {
                                if text.len() > 120 {
                                    if is_tool_result {
                                        *text = crate::rtk::compress_tool_output(&block_name, text);
                                    } else {
                                        *text = super::compression::compress_content(text, None);
                                    }
                                }
                            } else if let Some(serde_json::Value::String(content)) = block.get_mut("content") {
                                if content.len() > 120 {
                                    if is_tool_result {
                                        *content = crate::rtk::compress_tool_output(&block_name, content);
                                    } else {
                                        *content = super::compression::compress_content(content, None);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(serialized) = serde_json::to_vec(&json_val) {
                req.body = serialized;
            }
        }
    }

    // Forward to upstream target
    let (status, headers, body) = super::byok_gateway::forward_to_byok_target(
        &target_base,
        &req.method,
        &path,
        &req.headers,
        &req.body,
    ).await;

    super::byok_gateway::write_http_response(&mut stream, status, super::byok_gateway::status_reason(status), &headers, &body).await?;
    Ok(())
}

pub fn run_headroom_foreground(
    bin: &str,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if IS_WIN {
        return run_headroom_supervised(bin, args);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let pid = std::process::id();
        let identity = identify_process(pid)?;
        write_proxy_supervised_state(pid, bin, args, &[], &identity.start)?;
        let mut cmd = Command::new(bin);
        cmd.args(args);
        cmd.env_clear();
        for (k, v) in proxy_daemon_env() {
            cmd.env(k, v);
        }
        let err = cmd.exec();
        let _ = clear_proxy_supervised_state();
        return Err(err.into());
    }
    #[cfg(not(unix))]
    {
        run_headroom_supervised(bin, args)
    }
}

pub fn proxy_files() -> (String, String) {
    let root = headroom_paths_resolved().root;
    let pid_file = Path::new(&root)
        .join("proxy.pid")
        .to_string_lossy()
        .to_string();
    let log_file = Path::new(&root)
        .join("proxy.log")
        .to_string_lossy()
        .to_string();
    (pid_file, log_file)
}

pub fn proxy_supervised_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("proxy.supervised.json")
        .to_string_lossy()
        .to_string()
}

pub fn write_proxy_supervised_state(
    pid: u32,
    bin: &str,
    args: &[String],
    manager_args: &[String],
    start: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = ProxySupervisedState {
        pid,
        executable: bin.to_string(),
        args: args.to_vec(),
        start: start.to_string(),
        allowed_base_urls: headroom_allowed_base_urls(),
        manager_args: manager_args.to_vec(),
    };
    let data = serde_json::to_string(&state)?;
    write_file_atomic(proxy_supervised_file(), &data)?;
    Ok(())
}

pub fn clear_proxy_supervised_state() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match std::fs::remove_file(proxy_supervised_file()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn proxy_file_matches(path: &str, expected: Option<&str>) -> bool {
    let current = read_file_safe(path);
    current.as_deref() == expected
}

fn clear_proxy_state_with_runtime_if_unchanged(
    pid_file: &str,
    pid_raw: Option<&str>,
    supervised_raw: Option<&str>,
    clear_runtime: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !pid_file.is_empty() && !proxy_file_matches(pid_file, pid_raw) {
        return Err(
            "proxy ownership record changed during cleanup — refusing to remove replacement record"
                .into(),
        );
    }
    let supervised_path = proxy_supervised_file();
    if !proxy_file_matches(&supervised_path, supervised_raw) {
        return Err(
            "supervised proxy record changed during cleanup — refusing to remove replacement record"
                .into(),
        );
    }
    if pid_raw.is_some() {
        if let Err(e) = std::fs::remove_file(pid_file) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }
    }
    if supervised_raw.is_some() {
        if let Err(e) = std::fs::remove_file(&supervised_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }
    }
    if clear_runtime {
        clear_proxy_runtime()?;
    }
    Ok(())
}

pub fn proxy_supervised_args_match(port: u16) -> bool {
    let Some(raw) = read_file_safe(proxy_supervised_file()) else {
        return false;
    };
    let Ok(state) = serde_json::from_str::<ProxySupervisedState>(&raw) else {
        return false;
    };
    if state.pid == 0 || state.executable.is_empty() || state.start.is_empty() {
        return false;
    }
    let Ok(identity) = identify_process(state.pid) else {
        return false;
    };
    let check_args = if !state.manager_args.is_empty() {
        &state.manager_args
    } else {
        &state.args
    };
    identity.start == state.start
        && identity.matches(&state.executable, check_args)
        && state.args == proxy_args_from_runtime(port)
}

fn proxy_start_lock_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("proxy.start.lock")
        .to_string_lossy()
        .to_string()
}

fn proxy_lifecycle_lock_path() -> String {
    Path::new(&tokless_data_dir())
        .join("proxy.lifecycle.lock")
        .to_string_lossy()
        .to_string()
}

pub struct ProxyLockGuard {
    path: String,
    token: String,
}

impl ProxyLockGuard {
    pub fn release(self) {
        drop(self);
    }
}

impl Drop for ProxyLockGuard {
    fn drop(&mut self) {
        if let Ok(content) = std::fs::read_to_string(&self.path) {
            if content == self.token {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

pub fn acquire_proxy_start_lock() -> Result<ProxyLockGuard, Box<dyn std::error::Error + Send + Sync>>
{
    acquire_proxy_lock(&proxy_start_lock_path())
}

pub fn acquire_proxy_lifecycle_lock(
) -> Result<ProxyLockGuard, Box<dyn std::error::Error + Send + Sync>> {
    acquire_proxy_lock(&proxy_lifecycle_lock_path())
}

fn acquire_proxy_lock(
    path: &str,
) -> Result<ProxyLockGuard, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = Path::new(path).parent() {
        ensure_dir(parent)?;
    }
    let deadline = Instant::now() + PROXY_START_LOCK_WAIT;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let token = format!("{}-{}", std::process::id(), nanos);

    loop {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(mut f) => {
                if let Err(err) = f.write_all(token.as_bytes()) {
                    drop(f);
                    let _ = std::fs::remove_file(path);
                    return Err(err.into());
                }
                if let Err(err) = f.sync_all() {
                    drop(f);
                    let _ = std::fs::remove_file(path);
                    return Err(err.into());
                }
                return Ok(ProxyLockGuard {
                    path: path.to_string(),
                    token,
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if let Ok(meta) = std::fs::metadata(path) {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = SystemTime::now().duration_since(modified) {
                            if age > PROXY_START_LOCK_STALE {
                                let stale = format!("{}.stale.{}", path, nanos);
                                if std::fs::rename(path, &stale).is_ok() {
                                    let _ = std::fs::remove_file(&stale);
                                    continue;
                                }
                            }
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Err("timed out waiting for another proxy start to finish".into());
                }
                std::thread::sleep(PROXY_POLL_INTERVAL);
            }
            Err(err) => return Err(err.into()),
        }
    }
}

pub fn proxy_args_match_recorded(port: u16) -> bool {
    let (pid_file, _) = proxy_files();
    let Some(raw) = read_file_safe(&pid_file) else {
        return false;
    };
    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return false;
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.start_fingerprint.is_empty()
        || record.args.is_empty()
    {
        return false;
    }
    let Ok(identity) = identify_process(record.pid) else {
        return false;
    };
    if !identity.matches_record(&record) {
        return false;
    }
    let want = proxy_args_from_runtime(port);
    if record.args == want {
        return true;
    }
    record.args.len() > want.len() && record.args[record.args.len() - want.len()..] == want[..]
}

pub fn proxy_allowed_base_urls_match_recorded() -> bool {
    let (pid_file, _) = proxy_files();
    let Ok(current_allowed) = headroom_allowed_base_urls_checked() else {
        return false;
    };
    if let Some(raw) = read_file_safe(&pid_file) {
        let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
            return false;
        };
        if record.allowed_base_urls.is_empty() || record.allowed_base_urls != current_allowed {
            return false;
        }
    }
    if let Some(raw) = read_file_safe(proxy_supervised_file()) {
        let Ok(state) = serde_json::from_str::<ProxySupervisedState>(&raw) else {
            return false;
        };
        if state.allowed_base_urls.is_empty() || state.allowed_base_urls != current_allowed {
            return false;
        }
    }
    true
}

pub fn proxy_owned_process_live() -> bool {
    let (pid_file, _) = proxy_files();
    if let Some(raw) = read_file_safe(&pid_file) {
        if let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) {
            if record.pid > 0
                && !record.executable.is_empty()
                && !record.args.is_empty()
                && !record.start_fingerprint.is_empty()
            {
                if let Ok(identity) = identify_process(record.pid) {
                    if identity.matches_record(&record) {
                        return true;
                    }
                }
            }
        }
    }
    let Some(raw) = read_file_safe(proxy_supervised_file()) else {
        return false;
    };
    let Ok(state) = serde_json::from_str::<ProxySupervisedState>(&raw) else {
        return false;
    };
    if state.pid == 0
        || state.executable.is_empty()
        || state.args.is_empty()
        || state.start.is_empty()
    {
        return false;
    }
    let check_args = if !state.manager_args.is_empty() {
        &state.manager_args
    } else {
        &state.args
    };
    let Ok(identity) = identify_process(state.pid) else {
        return false;
    };
    identity.start == state.start && identity.matches(&state.executable, check_args)
}

pub fn start_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    headroom_allowed_base_urls_checked()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy allowlist configuration: {}", e).into() })?;

    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy start: {}", e).into() })?;

    let byok_needed = byok_routes_configured();
    let mut byok_started = false;
    if byok_needed {
        byok_started = !byok_gateway_live();
        start_byok_gateway()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("BYOK gateway start: {}", e).into() })?;
    }

    let result = start_proxy_locked();
    if result.is_err() && byok_started {
        let _ = stop_byok_gateway();
    }
    result
}

fn start_proxy_locked() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    cleanup_legacy_route_state();
    if apply_cloud_code_patch() {
        stop_proxy_supervisor_for_patch()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("stop proxy supervisor for patch: {}", e).into() })?;
        if proxy_owned_process_live() || proxy_running() {
            stop_headroom_daemon_for_handoff()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("restart proxy after patch: {}", e).into() })?;
        }
    }

    let port = proxy_port();
    let args = proxy_args(port);
    let (pid_file, log_file) = proxy_files();

    let stop_requested = proxy_stop_requested()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy stop state: {}", e).into() })?;
    if stop_requested {
        if proxy_owned_process_live() {
            return Err("headroom proxy stop is in progress".into());
        }
        clear_proxy_stop_request()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy stale stop state: {}", e).into() })?;
    }

    let running = proxy_running();
    let owned = proxy_owned_process_live();
    if running || owned {
        let args_match = proxy_args_match_recorded(port) || proxy_supervised_args_match(port);
        if args_match && proxy_allowed_base_urls_match_recorded() {
            persist_proxy_runtime(port)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy runtime record: {}", e).into() })?;
            L.sub(&format!("headroom proxy already running on {}", proxy_url()));
            return Ok(());
        }
        if !owned {
            return Err("headroom proxy is running without tokless ownership — refusing to replace".into());
        }
        if !args_match {
            return Err("headroom proxy is running with stale arguments — refusing to replace".into());
        }
        L.sub("headroom proxy allowlist updated — restarting proxy");
        stop_headroom_daemon()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("stopping stale headroom proxy: {}", e).into() })?;
        cleanup_legacy_route_state();
    }

    let bin = resolve_headroom_bin();
    if bin.is_empty() {
        return Err("headroom binary not found — install headroom first (run `tokless`)".into());
    }
    if !process_identity_supported() {
        return Err("headroom proxy lifecycle is unsupported on this platform: trustworthy process identity is unavailable".into());
    }
    if let Some(parent) = Path::new(&log_file).parent() {
        ensure_dir(parent)?;
    }

    let log = File::create(&log_file)?;
    let log_err = log.try_clone()?;

    let mut cmd = Command::new(&bin);
    cmd.args(&args);
    cmd.env_clear();
    for (k, v) in proxy_daemon_env() {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::from(log)).stderr(Stdio::from(log_err));

    let child = spawn_detached(&mut cmd)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy failed to start: {}", e).into() })?;
    let pid = child.id();

    let identity = match verify_identity_with_retry(pid, &bin, &args) {
        Ok(id) => id,
        Err((partial, err)) => {
            let msg: Box<dyn std::error::Error + Send + Sync> = format!(
                "headroom proxy startup identity unavailable; refusing to signal pid {}: {}",
                pid, err
            )
            .into();
            if partial.executable.is_empty() {
                return Err(cleanup_unverified_child(pid, msg));
            }
            return Err(msg);
        }
    };

    if let Err(err) = write_proxy_ownership(
        &pid_file,
        &ProxyOwnership {
            pid,
            executable: identity.executable.clone(),
            args: identity.args.clone(),
            start_fingerprint: identity.start.clone(),
            allowed_base_urls: headroom_allowed_base_urls(),
        },
    ) {
        return Err(rollback_proxy_with_identity(
            pid,
            &pid_file,
            Some(&identity),
            format!("headroom proxy ownership record: {}", err).into(),
        ));
    }

    L.sub(&format!(
        "headroom proxy on {} (semantic cache off; log: {})",
        proxy_url(),
        log_file
    ));

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    while Instant::now() < deadline {
        if let Ok(current) = identify_process(pid) {
            if proxy_live_z_timeout(PROXY_PROBE_TIMEOUT)
                && current.equal(&identity)
                && current.matches(&bin, &args)
            {
                if let Err(err) = persist_proxy_runtime(port) {
                    return Err(rollback_proxy_with_identity(
                        pid,
                        &pid_file,
                        Some(&identity),
                        format!("headroom proxy runtime record: {}", err).into(),
                    ));
                }
                L.ok("headroom proxy ready");
                return Ok(());
            }
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(rollback_proxy_with_identity(
        pid,
        &pid_file,
        Some(&identity),
        format!(
            "headroom proxy did not become ready within {:?} — see {}",
            PROXY_READY_TIMEOUT, log_file
        )
        .into(),
    ))
}

pub fn cleanup_unverified_child(
    pid: u32,
    cause: Box<dyn std::error::Error + Send + Sync>,
) -> Box<dyn std::error::Error + Send + Sync> {
    if let Err(err) = kill_process(pid) {
        return format!("{}; cleanup kill for pid {}: {}", cause, pid, err).into();
    }
    cause
}

pub fn restore_proxy_after_autostart_rollback(
    was_running: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !was_running {
        return Ok(());
    }
    start_proxy()
}

pub fn verify_identity_with_retry(
    pid: u32,
    bin: &str,
    args: &[String],
) -> Result<ProcessIdentityInfo, (ProcessIdentityInfo, Box<dyn std::error::Error + Send + Sync>)> {
    let mut last_err: Option<Box<dyn std::error::Error + Send + Sync>> = None;
    let deadline = Instant::now() + PROXY_IDENTITY_TIMEOUT;
    while Instant::now() < deadline {
        match identify_process(pid) {
            Ok(identity) => {
                if !identity.matches(bin, args) {
                    return Err((
                        identity,
                        "headroom proxy identity could not be verified".into(),
                    ));
                }
                return Ok(identity);
            }
            Err(err) => {
                last_err = Some(err);
                std::thread::sleep(PROXY_POLL_INTERVAL);
            }
        }
    }
    let err_msg = last_err
        .map(|e| e.to_string())
        .unwrap_or_else(|| "identity never surfaced".to_string());
    Err((
        ProcessIdentityInfo::default(),
        format!("headroom proxy identity could not be verified: {}", err_msg).into(),
    ))
}

fn provider_env_var() -> &'static str {
    "TOKLESS_PROXY_PROVIDER"
}

pub fn persist_proxy_runtime(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (anthropic, openai) = real_upstream_urls();
    let (gemini, cloudcode) = proxy_upstream_gemini_urls();
    let provider = if let Ok(v) = std::env::var(provider_env_var()) {
        v.trim().to_string()
    } else if let Some(st) = read_proxy_runtime() {
        st.provider
    } else {
        String::new()
    };

    let p = ProxyRuntime {
        port,
        anthropic_api_url: anthropic,
        openai_api_url: openai,
        gemini_api_url: gemini,
        cloudcode_api_url: cloudcode,
        provider,
    };
    save_headroom_proxy_runtime(&p)
}

pub fn stop_headroom_daemon() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stop_headroom_daemon_with_runtime(true)
}

pub fn stop_headroom_daemon_for_handoff() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stop_headroom_daemon_with_runtime(false)
}

/// Stops the daemon while retaining a managed supervisor configuration for
/// rollback to its prior inactive state.
pub fn stop_proxy_preserving_autostart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy rollback stop: {}", e).into() })?;
    stop_headroom_daemon_for_handoff()
}

fn stop_headroom_daemon_with_runtime(
    clear_runtime: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (pid_file, _) = proxy_files();
    let supervised_raw = read_file_safe(proxy_supervised_file());
    let Some(raw) = read_file_safe(&pid_file) else {
        return stop_supervised_daemon_with_runtime(clear_runtime);
    };
    let Ok(record) = serde_json::from_str::<ProxyOwnership>(&raw) else {
        return Err(format!(
            "invalid proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.args.is_empty()
        || record.start_fingerprint.is_empty()
    {
        return Err(format!(
            "invalid proxy ownership record {} — refusing to stop",
            pid_file
        )
        .into());
    }

    match identify_process(record.pid) {
        Err(_) => {
            if proxy_supervised_args_match(proxy_port_from_runtime()) {
                if !proxy_file_matches(&pid_file, Some(&raw))
                    || !proxy_file_matches(&proxy_supervised_file(), supervised_raw.as_deref())
                {
                    return Err("proxy ownership record changed during stale cleanup — refusing to remove replacement record".into());
                }
                stop_supervised_daemon_with_runtime(false)?;
                if read_file_safe(&pid_file).as_deref() != Some(raw.as_str()) {
                    return Err("proxy ownership record changed during stale cleanup — refusing to remove replacement record".into());
                }
                if let Err(err) = std::fs::remove_file(&pid_file) {
                    if err.kind() != std::io::ErrorKind::NotFound {
                        return Err(format!(
                            "remove stale proxy ownership record {}: {}",
                            pid_file, err
                        )
                        .into());
                    }
                }
                if clear_runtime {
                    clear_proxy_runtime()?;
                }
                return Ok(());
            }
            if process_gone(record.pid) {
                L.sub("headroom proxy already stopped — removing stale ownership record");
                clear_proxy_state_with_runtime_if_unchanged(
                    &pid_file,
                    Some(&raw),
                    supervised_raw.as_deref(),
                    clear_runtime,
                )
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!(
                        "headroom proxy already stopped but state could not be cleared: {}",
                        e
                    )
                    .into()
                })?;
                return Ok(());
            }
            return Err(format!(
                "proxy pid {} identity could not be verified — refusing to stop",
                record.pid
            )
            .into());
        }
        Ok(identity) => {
            if !identity.matches_record(&record) {
                if proxy_supervised_args_match(proxy_port_from_runtime()) {
                    if !proxy_file_matches(&pid_file, Some(&raw))
                        || !proxy_file_matches(&proxy_supervised_file(), supervised_raw.as_deref())
                    {
                        return Err("proxy ownership record changed during stale cleanup — refusing to remove replacement record".into());
                    }
                    stop_supervised_daemon_with_runtime(false)?;
                    if read_file_safe(&pid_file).as_deref() != Some(raw.as_str()) {
                        return Err("proxy ownership record changed during stale cleanup — refusing to remove replacement record".into());
                    }
                    if let Err(err) = std::fs::remove_file(&pid_file) {
                        if err.kind() != std::io::ErrorKind::NotFound {
                            return Err(format!(
                                "remove stale proxy ownership record {}: {}",
                                pid_file, err
                            )
                            .into());
                        }
                    }
                    if clear_runtime {
                        clear_proxy_runtime()?;
                    }
                    return Ok(());
                }
                return Err(format!(
                    "proxy pid {} identity could not be verified — refusing to stop",
                    record.pid
                )
                .into());
            }
        }
    }

    kill_process(record.pid)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("failed to stop headroom proxy pid {}: {}", record.pid, e).into() })?;

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(record.pid) && !proxy_live_z_timeout(PROXY_PROBE_TIMEOUT) {
            clear_proxy_state_with_runtime_if_unchanged(
                &pid_file,
                Some(&raw),
                supervised_raw.as_deref(),
                clear_runtime,
            )
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                format!("headroom proxy stopped but state could not be cleared: {}", e).into()
            })?;
            L.ok("headroom proxy stopped");
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(format!("headroom proxy did not stop (pid {})", record.pid).into())
}

fn stop_supervised_daemon_with_runtime(
    clear_runtime: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let supervised_path = proxy_supervised_file();
    let Some(raw) = read_file_safe(&supervised_path) else {
        return Ok(());
    };
    let Ok(state) = serde_json::from_str::<ProxySupervisedState>(&raw) else {
        return Err("invalid supervised proxy record — refusing to stop".into());
    };
    if state.pid == 0
        || state.executable.is_empty()
        || state.args.is_empty()
        || state.start.is_empty()
    {
        return Err("invalid supervised proxy record — refusing to stop".into());
    }

    let valid = identify_process(state.pid)
        .map(|id| id.start == state.start && id.matches(&state.executable, &state.args))
        .unwrap_or(false);
    if !valid {
        if process_gone(state.pid) {
            clear_proxy_state_with_runtime_if_unchanged("", None, Some(&raw), clear_runtime)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!(
                        "supervised proxy stopped but state could not be cleared: {}",
                        e
                    )
                    .into()
                })?;
            return Ok(());
        }
        return Err(format!(
            "supervised proxy pid {} identity could not be verified — refusing to stop",
            state.pid
        )
        .into());
    }

    kill_process(state.pid)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("failed to stop supervised proxy pid {}: {}", state.pid, e).into() })?;

    let deadline = Instant::now() + PROXY_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if process_gone(state.pid) && !proxy_live_z_timeout(PROXY_PROBE_TIMEOUT) {
            clear_proxy_state_with_runtime_if_unchanged("", None, Some(&raw), clear_runtime)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    format!(
                        "supervised proxy stopped but state could not be cleared: {}",
                        e
                    )
                    .into()
                })?;
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err(format!("supervised proxy did not stop (pid {})", state.pid).into())
}

pub fn stop_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _lock = acquire_proxy_start_lock()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy stop: {}", e).into() })?;

    let res = stop_proxy_locked();
    let _ = stop_byok_gateway();
    res
}

fn stop_proxy_locked() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    request_proxy_stop()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("headroom proxy stop request: {}", e).into() })?;

    if let Err(err) = disable_proxy_autostart() {
        cleanup_legacy_route_state();
        let _ = clear_proxy_stop_request();
        return Err(err);
    }

    let mut err = stop_headroom_daemon();
    if err.is_ok() {
        if let Err(stop_err) = clear_proxy_stop_request() {
            err = Err(stop_err);
        }
    }
    if err.is_ok() {
        if let Err(runtime_err) = clear_proxy_runtime() {
            err = Err(runtime_err);
        }
    }
    cleanup_legacy_route_state();
    err
}

pub fn clear_proxy_runtime() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    clear_headroom_proxy_runtime()?;
    Ok(())
}

/// Removes the retired per-key route proxy state.
pub fn cleanup_legacy_route_state() {
    let root = headroom_paths_resolved().root;
    let pid_file = Path::new(&root).join("proxy.route.pid");
    if let Some(raw) = read_file_safe(&pid_file) {
        if let Ok(pid) = raw.trim().parse::<u32>() {
            if pid > 0 && process_identity_supported() {
                if let Ok(identity) = identify_process(pid) {
                    if identity.args.first().map(|s| s.as_str()) == Some("__route-proxy-serve") {
                        let _ = kill_process(pid);
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_file);
    }
    let _ = std::fs::remove_file(Path::new(&root).join("proxy.routes.json"));
    let _ = std::fs::remove_file(Path::new(&root).join("proxy.route.log"));
}

pub fn write_proxy_ownership(
    path: &str,
    record: &ProxyOwnership,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data = serde_json::to_string(record)?;
    write_file_atomic(path, &data)?;
    Ok(())
}

pub fn rollback_proxy(
    pid: u32,
    pid_file: &str,
    cause: Box<dyn std::error::Error + Send + Sync>,
) -> Box<dyn std::error::Error + Send + Sync> {
    rollback_proxy_with_identity(pid, pid_file, None, cause)
}

pub fn rollback_proxy_with_identity(
    pid: u32,
    pid_file: &str,
    expected: Option<&ProcessIdentityInfo>,
    cause: Box<dyn std::error::Error + Send + Sync>,
) -> Box<dyn std::error::Error + Send + Sync> {
    let pid_raw = read_file_safe(pid_file);
    let supervised_path = proxy_supervised_file();
    let supervised_raw = read_file_safe(&supervised_path);

    if let Some(exp) = expected {
        if !exp.executable.is_empty() {
            match identify_process(pid) {
                Ok(current) if current.equal(exp) => {}
                _ => return cause,
            }
        }
    }

    if let Err(err) = kill_process(pid) {
        return format!("{}; rollback kill for pid {}: {}", cause, pid, err).into();
    }

    if !proxy_file_matches(pid_file, pid_raw.as_deref())
        || !proxy_file_matches(&supervised_path, supervised_raw.as_deref())
    {
        return format!(
            "{}; proxy ownership record changed during rollback — refusing to remove replacement record",
            cause
        )
        .into();
    }

    if pid_raw.is_some() {
        if let Err(err) = std::fs::remove_file(pid_file) {
            if err.kind() != std::io::ErrorKind::NotFound {
                return format!("{}; rollback record removal: {}", cause, err).into();
            }
        }
    }
    if supervised_raw.is_some() {
        if let Err(err) = std::fs::remove_file(&supervised_path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                return format!("{}; {}", cause, err).into();
            }
        }
    }
    cause
}
