// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/byok_gateway.rs
// created: 2026-10-05
// lastModified: 2026-10-28

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

use super::autostart::stop_byok_gateway_supervisor;
use super::process_identity::{
    identify_process, kill_process, process_gone, spawn_detached, ProcessIdentityInfo,
};
use super::proxy::verify_identity_with_retry;
use super::responses_compat::{flatten_responses_tools, transform_sse_bytes};
pub use crate::util::byok::BYOKRoute;
use crate::util::byok::{is_loopback_host, valid_byok_id};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::logger::L;
use crate::util::paths::{exists, read_file_safe, tokless_abs, write_file_atomic};
use crate::util::routing_coordinator::RoutingCoordinator;
use crate::util::routing_plan::{route_path_for_protocol, RouteRequest};
use crate::util::routing_state::{
    new_routing_token, validate_routing_state, LaneBinding, RouteBinding, RoutingState,
    ROUTING_STATE_COMMITTED, ROUTING_STATE_PREPARING, ROUTING_STATE_STOPPED, ROUTING_STATE_VERSION,
};
use crate::util::routing_store::read_routing_state;

pub const BYOK_ROUTE_HEADER: &str = "X-Tokless-Route";
pub const BYOK_GATEWAY_SERVE_ARG: &str = "__byok-gateway-serve";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ByokGatewayOwnership {
    pub pid: u32,
    pub executable: String,
    pub args: Vec<String>,
    #[serde(rename = "start_fingerprint")]
    pub start: String,
}

#[derive(Debug, Clone, Default)]
pub struct ByokRouteTable {
    pub root: String,
    pub generation: u64,
    pub items: HashMap<String, BYOKRoute>,
    pub by_name: HashMap<String, BYOKRoute>,
}

static ACTIVE_BYOK_ROUTES: RwLock<Option<ByokRouteTable>> = RwLock::new(None);
static BYOK_RELOAD_ERR: RwLock<Option<String>> = RwLock::new(None);
static BYOK_LOAD_MU: Mutex<()> = Mutex::new(());
static BYOK_GATEWAY_LIFECYCLE: Mutex<()> = Mutex::new(());
static BYOK_GATEWAY_SHUTDOWN: Mutex<Option<watch::Sender<bool>>> = Mutex::new(None);

pub fn byok_routes_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("byok.routes.json")
        .to_string_lossy()
        .to_string()
}

pub fn byok_gateway_pid_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("byok.gateway.pid")
        .to_string_lossy()
        .to_string()
}

pub fn byok_gateway_args() -> Vec<String> {
    vec![BYOK_GATEWAY_SERVE_ARG.to_string()]
}

pub fn byok_gateway_owned() -> (ByokGatewayOwnership, bool) {
    let Some(raw) = read_file_safe(byok_gateway_pid_path()) else {
        return (ByokGatewayOwnership::default(), false);
    };
    let Ok(record) = serde_json::from_str::<ByokGatewayOwnership>(&raw) else {
        return (ByokGatewayOwnership::default(), false);
    };
    if record.pid == 0
        || record.executable.is_empty()
        || record.start.is_empty()
        || record.args.is_empty()
    {
        return (ByokGatewayOwnership::default(), false);
    }
    let Ok(identity) = identify_process(record.pid) else {
        return (ByokGatewayOwnership::default(), false);
    };
    if identity.start != record.start || !identity.matches(&record.executable, &record.args) {
        return (ByokGatewayOwnership::default(), false);
    }
    (record, true)
}

/// Records the running gateway process itself so ownership holds whether
/// spawned by the CLI or supervised by a service manager.
pub fn persist_byok_gateway_ownership(
    identity: &ProcessIdentityInfo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let record = ByokGatewayOwnership {
        pid: std::process::id(),
        executable: identity.executable.clone(),
        args: identity.args.clone(),
        start: identity.start.clone(),
    };
    let b = serde_json::to_string(&record)?;
    write_file_atomic(byok_gateway_pid_path(), &b)?;
    Ok(())
}

pub fn save_byok_routes(
    routes: &[BYOKRoute],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    crate::util::byok::save_byok_routes(routes)?;
    load_byok_routes()
}

fn store_empty_route_table(current_root: String, reload_err: String) {
    let mut active = ACTIVE_BYOK_ROUTES.write().unwrap();
    let generation = active.as_ref().map(|t| t.generation + 1).unwrap_or(1);
    *active = Some(ByokRouteTable {
        root: current_root,
        generation,
        items: HashMap::new(),
        by_name: HashMap::new(),
    });
    let mut err_guard = BYOK_RELOAD_ERR.write().unwrap();
    *err_guard = Some(reload_err);
}

pub fn load_byok_routes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = BYOK_LOAD_MU.lock().unwrap();
    let current_root = headroom_paths_resolved().root;
    let routes_file = byok_routes_path();

    if !exists(&routes_file) {
        store_empty_route_table(current_root, "BYOK route registry not found".to_string());
        return Ok(());
    }

    let routes = match crate::util::byok::read_byok_routes() {
        Ok(r) => r,
        Err(err) => {
            let msg = format!("invalid BYOK route registry: {}", err);
            store_empty_route_table(current_root, msg.clone());
            return Err(msg.into());
        }
    };

    let mut items: HashMap<String, BYOKRoute> = HashMap::with_capacity(routes.len());
    let mut by_name: HashMap<String, BYOKRoute> = HashMap::with_capacity(routes.len());

    for mut route in routes {
        route.id = route.id.trim().to_string();
        route.token = route.token.trim().to_string();
        route.protocol = route.protocol.trim().to_string();
        route.upstream = route.upstream.trim().trim_end_matches('/').to_string();

        if let Err(err) = validate_byok_route(&route) {
            let msg = format!("route {:?}: {}", route.id, err);
            store_empty_route_table(current_root, msg.clone());
            return Err(msg.into());
        }
        if items.contains_key(&route.id) {
            let msg = format!("route {:?}: duplicate ID", route.id);
            store_empty_route_table(current_root, msg.clone());
            return Err(msg.into());
        }
        items.insert(route.id.clone(), route.clone());
        let name = match route.id.find(':') {
            Some(idx) => &route.id[idx + 1..],
            None => &route.id,
        };
        by_name
            .entry(name.to_string())
            .or_insert_with(|| route.clone());
    }

    {
        let mut active = ACTIVE_BYOK_ROUTES.write().unwrap();
        let generation = active.as_ref().map(|t| t.generation + 1).unwrap_or(1);
        *active = Some(ByokRouteTable {
            root: current_root,
            generation,
            items,
            by_name,
        });
    }
    {
        let mut err_guard = BYOK_RELOAD_ERR.write().unwrap();
        *err_guard = None;
    }
    Ok(())
}

/// Reports whether a valid route registry has entries.
pub fn byok_routes_configured() -> bool {
    let needs_load = ACTIVE_BYOK_ROUTES.read().unwrap().is_none();
    if needs_load && load_byok_routes().is_err() {
        return false;
    }
    ACTIVE_BYOK_ROUTES
        .read()
        .unwrap()
        .as_ref()
        .map(|tbl| !tbl.items.is_empty())
        .unwrap_or(false)
}

pub fn lookup_byok_route(id: &str) -> Option<BYOKRoute> {
    let needs_load = ACTIVE_BYOK_ROUTES.read().unwrap().is_none();
    if needs_load && load_byok_routes().is_err() {
        return None;
    }
    let guard = ACTIVE_BYOK_ROUTES.read().unwrap();
    let tbl = guard.as_ref()?;
    let clean_id = id.trim();
    tbl.items
        .get(clean_id)
        .or_else(|| tbl.by_name.get(clean_id))
        .cloned()
}

pub fn configured_byok_routes() -> Vec<BYOKRoute> {
    let needs_load = ACTIVE_BYOK_ROUTES.read().unwrap().is_none();
    if needs_load && load_byok_routes().is_err() {
        return Vec::new();
    }
    let guard = ACTIVE_BYOK_ROUTES.read().unwrap();
    let Some(tbl) = guard.as_ref() else {
        return Vec::new();
    };
    let mut routes: Vec<BYOKRoute> = tbl.items.values().cloned().collect();
    routes.sort_by(|a, b| a.id.cmp(&b.id));
    routes
}

/// Loopback endpoint used by compatible BYOK clients.
pub fn byok_gateway_url() -> String {
    format!("http://127.0.0.1:{}", byok_gateway_port())
}

pub fn byok_gateway_port() -> u16 {
    crate::util::byok::byok_gateway_port()
}

pub fn valid_byok_upstream(raw: &str) -> bool {
    crate::util::byok::valid_byok_upstream(raw)
}

pub fn valid_byok_token(value: &str) -> bool {
    if value.is_empty() || value.contains('.') || value.contains(':') {
        return false;
    }
    value
        .chars()
        .all(|c| c == '-' || c == '_' || c.is_ascii_alphanumeric())
}

pub fn validate_byok_route(
    route: &BYOKRoute,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !valid_byok_id(&route.id) || !valid_byok_token(&route.token) || route.protocol.is_empty() {
        return Err("missing ID, token, or protocol".into());
    }
    if !valid_byok_upstream(&route.upstream) {
        return Err("upstream must be an HTTPS URL without credentials".into());
    }
    match route.protocol.as_str() {
        "openai-chat" | "openai-responses" | "anthropic-messages" => Ok(()),
        other => Err(format!("unsupported protocol {:?}", other).into()),
    }
}

pub fn is_public_upstream_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            if v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_multicast()
                || v4.is_broadcast()
            {
                return false;
            }
            let octets = v4.octets();
            // 100.64.0.0/10 (Carrier-grade NAT)
            if octets[0] == 100 && (octets[1] & 0xC0) == 64 {
                return false;
            }
            // 198.18.0.0/15 (Benchmarking)
            if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
                return false;
            }
            // 0.0.0.0/8
            if octets[0] == 0 {
                return false;
            }
            true
        }
        IpAddr::V6(v6) => {
            if let Some(mapped_v4) = v6.to_ipv4_mapped() {
                return is_public_upstream_ip(IpAddr::V4(mapped_v4));
            }
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return false;
            }
            let segs = v6.segments();
            // fc00::/7 (Unique Local)
            if (segs[0] & 0xfe00) == 0xfc00 {
                return false;
            }
            // fe80::/10 (Link-Local Unicast)
            if (segs[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            true
        }
    }
}

pub fn route_parts(raw: &str) -> Option<(&str, &str)> {
    let idx = raw.rfind('.')?;
    if idx == 0 || idx >= raw.len() - 1 {
        return None;
    }
    Some((&raw[..idx], &raw[idx + 1..]))
}

fn is_valid_virtual_provider_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 128 {
        return false;
    }
    id.bytes()
        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

pub fn parse_virtual_path_prefix(path: &str) -> (String, String, bool) {
    for prefix in ["/tokless/byok/", "/byok/"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            let Some(slash_idx) = rest.find('/') else {
                return (String::new(), String::new(), false);
            };
            let prov = &rest[..slash_idx];
            let remaining = &rest[slash_idx..];
            if !is_valid_virtual_provider_id(prov) {
                return (String::new(), String::new(), false);
            }
            return (prov.to_string(), remaining.to_string(), true);
        }
    }
    (String::new(), String::new(), false)
}

pub fn byok_route_path(protocol: &str, path: &str) -> bool {
    route_path_for_protocol(protocol, path)
}

pub fn byok_target_path(target: &str, incoming: &str) -> String {
    let target = target.trim_end_matches('/');
    let incoming = incoming.trim_end_matches('/');
    if target.is_empty() || target == "/" {
        return incoming.to_string();
    }
    if target == "/v1" && incoming.starts_with("/v1") {
        if incoming == "/v1" {
            return "/".to_string();
        }
        return incoming[3..].to_string();
    }
    if target.ends_with("/v1") && incoming.starts_with("/v1") {
        if incoming == "/v1" {
            return target.to_string();
        }
        return format!("{}{}", target, &incoming[3..]);
    }
    if let Some(rest) = incoming.strip_prefix(target) {
        if rest.is_empty() {
            return "/".to_string();
        }
        if rest.starts_with('/') {
            return rest.to_string();
        }
    }
    incoming.to_string()
}

pub fn byok_gateway_live() -> bool {
    let url = format!("{}/livez", byok_gateway_url());
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(300))
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
    }
    resp.json::<LiveResp>()
        .map(|l| l.service == "tokless-byok-gateway")
        .unwrap_or(false)
}

pub fn start_byok_gateway() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = BYOK_GATEWAY_LIFECYCLE.lock().unwrap();
    if BYOK_GATEWAY_SHUTDOWN.lock().unwrap().is_some() {
        return Ok(());
    }
    let (_, owned) = byok_gateway_owned();
    if owned && byok_gateway_live() {
        return Ok(());
    }
    load_byok_routes()?;
    if BYOK_GATEWAY_SHUTDOWN.lock().unwrap().is_some() {
        return Ok(());
    }
    if byok_gateway_live() {
        return Err(format!(
            "BYOK gateway port {} is occupied by an unowned process",
            byok_gateway_port()
        )
        .into());
    }
    let self_exe = tokless_abs();
    let mut cmd = Command::new(&self_exe);
    cmd.arg(BYOK_GATEWAY_SERVE_ARG);
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let child = spawn_detached(&mut cmd)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("start BYOK gateway: {}", e).into() })?;
    let pid = child.id();

    let identity = match verify_identity_with_retry(pid, &self_exe, &byok_gateway_args()) {
        Ok(id) => id,
        Err((_, err)) => {
            let _ = kill_process(pid);
            return Err(err);
        }
    };

    let record = ByokGatewayOwnership {
        pid,
        executable: identity.executable,
        args: identity.args,
        start: identity.start,
    };
    let b = serde_json::to_string(&record)?;
    if let Err(err) = write_file_atomic(byok_gateway_pid_path(), &b) {
        let _ = kill_process(pid);
        return Err(err.into());
    }

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if byok_gateway_live() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let _ = std::fs::remove_file(byok_gateway_pid_path());
    let _ = kill_process(pid);
    Err("BYOK gateway did not become ready".into())
}

pub fn stop_byok_gateway() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = BYOK_GATEWAY_LIFECYCLE.lock().unwrap();
    if let Some(tx) = BYOK_GATEWAY_SHUTDOWN.lock().unwrap().take() {
        let _ = tx.send(true);
        return Ok(());
    }
    let (record, ok) = byok_gateway_owned();
    if !ok {
        if !byok_gateway_live() {
            let _ = std::fs::remove_file(byok_gateway_pid_path());
            return Ok(());
        }
        return Err("BYOK gateway is running but ownership cannot be verified".into());
    }
    if let Err(err) = stop_byok_gateway_supervisor() {
        L.warn(&format!("stop BYOK gateway supervisor: {}", err));
    }
    let _ = kill_process(record.pid);
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if process_gone(record.pid) && !byok_gateway_live() {
            let _ = std::fs::remove_file(byok_gateway_pid_path());
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(format!("BYOK gateway did not stop (pid {})", record.pid).into())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// Pure-std SHA-256 implementation for config fingerprints and file watching
fn sha256_digest(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

async fn append_byok_bindings(
    state: &mut RoutingState,
    routes: &[BYOKRoute],
    ready: bool,
) -> Result<Vec<TcpListener>, Box<dyn std::error::Error + Send + Sync>> {
    let mut listeners = Vec::with_capacity(routes.len());
    for route in routes {
        let lane_listener = TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = lane_listener.local_addr()?;
        let port = local_addr.port() as i32;
        let listener_str = local_addr.to_string();
        listeners.push(lane_listener);

        let lane_id = format!("byok-lane-{}", route.id);
        let instance_nonce = new_routing_token()?;
        let instance = format!("byok-instance-{}", instance_nonce);
        let fp_input = format!("{}\0{}", route.protocol, route.upstream);
        let fp = hex_encode(&sha256_digest(fp_input.as_bytes()));

        state.lanes.push(LaneBinding {
            id: lane_id.clone(),
            instance_nonce: instance.clone(),
            provider_instance_id: instance.clone(),
            protocol: route.protocol.clone(),
            upstream_id: route.id.clone(),
            listener: listener_str,
            port,
            generation: state.generation,
            coordinator_nonce: state.coordinator_nonce.clone(),
            ready,
        });

        let revoked_generation = if ready { 0 } else { state.generation };
        state.routes.push(RouteBinding {
            id: route.id.clone(),
            token: route.token.clone(),
            profile_id: route.id.clone(),
            surface_id: "byok".to_string(),
            provider_instance_id: instance,
            protocol: route.protocol.clone(),
            lane_id,
            lane_instance_nonce: state.lanes.last().unwrap().instance_nonce.clone(),
            coordinator_nonce: state.coordinator_nonce.clone(),
            upstream_id: route.id.clone(),
            config_fingerprint: fp,
            generation: state.generation,
            revoked: !ready,
            revoked_generation,
        });
    }
    Ok(listeners)
}

async fn build_byok_routing_states(
    coordinator_nonce: &str,
    generation: u64,
    previous_generation: u64,
    routes: &[BYOKRoute],
) -> Result<(RoutingState, RoutingState, Vec<TcpListener>), Box<dyn std::error::Error + Send + Sync>>
{
    let mut preparing = RoutingState {
        version: ROUTING_STATE_VERSION,
        generation,
        previous_generation,
        state: ROUTING_STATE_PREPARING.to_string(),
        coordinator_nonce: coordinator_nonce.to_string(),
        commit_nonce: String::new(),
        routes: Vec::new(),
        lanes: Vec::new(),
    };
    let listeners = append_byok_bindings(&mut preparing, routes, false).await?;
    let mut committed = preparing.clone();
    committed.generation += 1;
    committed.previous_generation = preparing.generation;
    committed.state = ROUTING_STATE_COMMITTED.to_string();
    committed.commit_nonce = new_routing_token()?;
    for lane in &mut committed.lanes {
        lane.generation = committed.generation;
        lane.ready = true;
    }
    for route in &mut committed.routes {
        route.generation = committed.generation;
        route.revoked = false;
        route.revoked_generation = 0;
    }
    validate_routing_state(&committed)?;
    Ok((preparing, committed, listeners))
}

async fn run_byok_watcher(
    routes_path: String,
    mut stop_rx: watch::Receiver<bool>,
    interval: Duration,
) {
    let mut last_hash: Option<[u8; 32]> = None;
    match std::fs::read(&routes_path) {
        Ok(initial) => {
            last_hash = Some(sha256_digest(&initial));
        }
        Err(err) => {
            let msg = if err.kind() == std::io::ErrorKind::NotFound {
                "BYOK route registry not found".to_string()
            } else {
                format!("failed reading BYOK routes: {}", err)
            };
            *BYOK_RELOAD_ERR.write().unwrap() = Some(msg);
            let _ = load_byok_routes();
        }
    }

    let mut ticker = tokio::time::interval(interval);
    loop {
        tokio::select! {
            _ = stop_rx.changed() => {
                return;
            }
            _ = ticker.tick() => {
                match std::fs::read(&routes_path) {
                    Ok(data) => {
                        let h = sha256_digest(&data);
                        if last_hash != Some(h) {
                            last_hash = Some(h);
                            let _ = load_byok_routes();
                        }
                    }
                    Err(err) => {
                        last_hash = None;
                        let msg = if err.kind() == std::io::ErrorKind::NotFound {
                            "BYOK route registry not found".to_string()
                        } else {
                            format!("failed reading BYOK routes: {}", err)
                        };
                        *BYOK_RELOAD_ERR.write().unwrap() = Some(msg);
                        let _ = load_byok_routes();
                    }
                }
            }
        }
    }
}

pub async fn run_byok_gateway_serve() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    load_byok_routes()?;
    let coordinator = RoutingCoordinator::new(Duration::from_secs(30))
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("start routing coordinator: {}", e).into() })?;

    let routes = configured_byok_routes();
    if routes.is_empty() {
        let _ = coordinator.close();
        return Err("no BYOK routes configured".into());
    }

    let mut current = coordinator
        .current()
        .ok_or("routing snapshot unavailable")?;

    let current_state = read_routing_state()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("read routing state: {}", e).into() })?;
    if current_state.state == ROUTING_STATE_PREPARING {
        let mut stopped = current_state.clone();
        stopped.generation += 1;
        stopped.previous_generation = current_state.generation;
        stopped.state = ROUTING_STATE_STOPPED.to_string();
        stopped.commit_nonce.clear();
        for lane in &mut stopped.lanes {
            lane.generation = stopped.generation;
            lane.ready = false;
        }
        for route in &mut stopped.routes {
            route.generation = stopped.generation;
            route.revoked = true;
            route.revoked_generation = stopped.generation;
        }
        coordinator
            .publish(stopped)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("recover stale BYOK preparation: {}", e).into() })?;
        current = coordinator
            .current()
            .ok_or("routing snapshot unavailable after recovery")?;
    }

    let (preparing, committed, lane_listeners) = build_byok_routing_states(
        &coordinator.nonce,
        current.generation() + 1,
        current.generation(),
        &routes,
    )
    .await?;

    let mut targets = Vec::with_capacity(routes.len());
    for route in &routes {
        let parsed = url::Url::parse(&route.upstream)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid BYOK upstream: {}", e).into() })?;
        targets.push(parsed);
    }

    coordinator
        .publish(preparing)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("publish BYOK routing preparation: {}", e).into() })?;

    let (stop_tx, stop_rx) = watch::channel(false);
    *BYOK_GATEWAY_SHUTDOWN.lock().unwrap() = Some(stop_tx.clone());

    let mut lane_addrs = Vec::with_capacity(lane_listeners.len());
    for (lane_listener, target_url) in lane_listeners.into_iter().zip(targets.into_iter()) {
        let addr = lane_listener.local_addr()?;
        lane_addrs.push(addr);
        let mut lane_stop = stop_rx.clone();
        let target_str = target_url.to_string();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = lane_stop.changed() => break,
                    accept_res = lane_listener.accept() => {
                        if let Ok((stream, _)) = accept_res {
                            let target = target_str.clone();
                            tokio::spawn(async move {
                                let _ = serve_proxy_connection(stream, &target).await;
                            });
                        } else {
                            break;
                        }
                    }
                }
            }
        });
    }

    for addr in &lane_addrs {
        tokio::time::timeout(Duration::from_secs(1), TcpStream::connect(addr))
            .await
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "BYOK lane not ready: timeout".into() })?
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("BYOK lane not ready: {}", e).into() })?;
    }

    coordinator
        .publish(committed.clone())
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("publish BYOK routing state: {}", e).into() })?;

    for lane in &committed.lanes {
        coordinator
            .acquire(&committed, lane)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("acquire BYOK lane {}: {}", lane.id, e).into() })?;
    }

    let port = byok_gateway_port();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    L.info(&format!("BYOK gateway listening on {}", addr));

    if let Ok(identity) = identify_process(std::process::id()) {
        let _ = persist_byok_gateway_ownership(&identity);
    }

    let watcher_stop = stop_rx.clone();
    tokio::spawn(run_byok_watcher(
        byok_routes_path(),
        watcher_stop,
        Duration::from_millis(100),
    ));

    let mut gateway_stop = stop_rx.clone();
    loop {
        tokio::select! {
            _ = gateway_stop.changed() => {
                break;
            }
            accept_res = listener.accept() => {
                match accept_res {
                    Ok((stream, _)) => {
                        let coord = coordinator.clone();
                        tokio::spawn(async move {
                            let _ = serve_gateway_connection(stream, Some(coord)).await;
                        });
                    }
                    Err(_) => break,
                }
            }
        }
    }

    let _ = stop_tx.send(true);
    *BYOK_GATEWAY_SHUTDOWN.lock().unwrap() = None;
    let _ = coordinator.close();
    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP Server & Upstream Forwarding with Responses API Compatibility
// ---------------------------------------------------------------------------

pub(crate) struct ParsedHttpRequest {
    pub(crate) method: String,
    pub(crate) raw_uri: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

pub(crate) async fn read_http_request(
    stream: &mut TcpStream,
) -> Result<Option<ParsedHttpRequest>, Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = Vec::with_capacity(4096);
    let mut temp = [0u8; 4096];
    let header_end: usize;
    loop {
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            if buf.is_empty() {
                return Ok(None);
            }
            return Err("unexpected EOF reading HTTP headers".into());
        }
        buf.extend_from_slice(&temp[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = pos + 4;
            break;
        }
        if buf.len() > 64 * 1024 {
            return Err("HTTP header too large".into());
        }
    }

    let header_str = String::from_utf8_lossy(&buf[..header_end - 4]);
    let mut lines = header_str.split("\r\n");
    let req_line = lines.next().ok_or("missing request line")?;
    let mut parts = req_line.split_whitespace();
    let method = parts.next().ok_or("missing method")?.to_string();
    let raw_uri = parts.next().ok_or("missing URI")?.to_string();

    let mut headers = Vec::new();
    let mut content_length: usize = 0;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().to_string();
            if key.eq_ignore_ascii_case("content-length") {
                content_length = val.parse::<usize>().unwrap_or(0);
            }
            headers.push((key, val));
        }
    }

    let mut body = buf[header_end..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&temp[..n]);
    }
    if content_length > 0 && body.len() > content_length {
        body.truncate(content_length);
    }

    Ok(Some(ParsedHttpRequest {
        method,
        raw_uri,
        headers,
        body,
    }))
}

pub(crate) async fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut head = format!("HTTP/1.1 {} {}\r\n", status, reason);
    let mut has_content_len = false;
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("transfer-encoding") {
            continue;
        }
        if k.eq_ignore_ascii_case("content-length") {
            has_content_len = true;
        }
        head.push_str(&format!("{}: {}\r\n", k, v));
    }
    if !has_content_len {
        head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    head.push_str("Connection: close\r\n\r\n");
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    Ok(())
}

pub(crate) fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

pub(crate) fn split_uri_path_and_query(raw_uri: &str) -> (String, String, String) {
    // Strip scheme://host if absolute URI in request line
    let path_and_query = if let Some(idx) = raw_uri.find("://") {
        let after_scheme = &raw_uri[idx + 3..];
        match after_scheme.find('/') {
            Some(slash) => &after_scheme[slash..],
            None => "/",
        }
    } else {
        raw_uri
    };
    let (raw_path_part, raw_query) = match path_and_query.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (path_and_query.to_string(), String::new()),
    };
    let raw_path = if raw_path_part.contains('%') {
        raw_path_part.clone()
    } else {
        String::new()
    };
    (raw_path_part, raw_path, raw_query)
}

async fn serve_gateway_connection(
    mut stream: TcpStream,
    coordinator: Option<Arc<RoutingCoordinator>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(req) = read_http_request(&mut stream).await? else {
        return Ok(());
    };
    let (status, headers, body) = handle_byok_gateway_request(
        &req.method,
        &req.raw_uri,
        &req.headers,
        &req.body,
        coordinator.as_ref(),
    )
    .await;
    write_http_response(&mut stream, status, status_reason(status), &headers, &body).await
}

async fn serve_proxy_connection(
    mut stream: TcpStream,
    target_base: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Some(req) = read_http_request(&mut stream).await? else {
        return Ok(());
    };
    let (path, _, _) = split_uri_path_and_query(&req.raw_uri);
    let (status, headers, body) =
        forward_to_byok_target(target_base, &req.method, &path, &req.headers, &req.body).await;
    write_http_response(&mut stream, status, status_reason(status), &headers, &body).await
}

pub async fn handle_byok_gateway_request(
    method: &str,
    raw_uri: &str,
    headers: &[(String, String)],
    body: &[u8],
    coordinator: Option<&Arc<RoutingCoordinator>>,
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let (mut path, raw_path, raw_query) = split_uri_path_and_query(raw_uri);

    if path == "/livez" {
        let err_opt = BYOK_RELOAD_ERR.read().unwrap().clone();
        if let Some(err_str) = err_opt {
            let quoted = serde_json::to_string(&err_str).unwrap_or_else(|_| "\"error\"".to_string());
            let body_str = format!(
                r#"{{"service":"tokless-byok-gateway","status":"degraded","error":{}}}"#,
                quoted
            );
            return (
                503,
                vec![("Content-Type".to_string(), "application/json".to_string())],
                body_str.into_bytes(),
            );
        }
        return (
            200,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            br#"{"service":"tokless-byok-gateway","status":"ok"}"#.to_vec(),
        );
    }

    let credentials: Vec<&str> = headers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case(BYOK_ROUTE_HEADER))
        .map(|(_, v)| v.as_str())
        .collect();

    if credentials.len() > 1 {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"duplicate route header\n".to_vec(),
        );
    }

    let (prov_id, remaining_path, is_virtual) = parse_virtual_path_prefix(&path);
    let route: BYOKRoute;
    let cred: String;

    if is_virtual {
        let Some(found) = lookup_byok_route(&prov_id) else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"unknown BYOK route\n".to_vec(),
            );
        };
        route = found;
        if credentials.len() == 1 {
            let Some((h_id, h_token)) = route_parts(credentials[0]) else {
                return (
                    502,
                    vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                    b"unknown BYOK route\n".to_vec(),
                );
            };
            let clean_h_id = match h_id.find(':') {
                Some(idx) => &h_id[idx + 1..],
                None => h_id,
            };
            if (clean_h_id != prov_id && h_id != route.id)
                || !constant_time_eq(h_token.as_bytes(), route.token.as_bytes())
            {
                return (
                    502,
                    vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                    b"unknown BYOK route\n".to_vec(),
                );
            }
        }
        path = remaining_path;
        cred = format!("{}.{}", route.id, route.token);
    } else {
        if credentials.len() != 1 {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"unknown BYOK route\n".to_vec(),
            );
        }
        let Some((id, token)) = route_parts(credentials[0]) else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"unknown BYOK route\n".to_vec(),
            );
        };
        let Some(found) = lookup_byok_route(id) else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"unknown BYOK route\n".to_vec(),
            );
        };
        if !constant_time_eq(token.as_bytes(), found.token.as_bytes()) {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"unknown BYOK route\n".to_vec(),
            );
        }
        route = found;
        cred = credentials[0].to_string();
    }

    if !method.eq_ignore_ascii_case("POST") {
        return (
            405,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"method not allowed\n".to_vec(),
        );
    }

    if !raw_query.is_empty()
        || !raw_path.is_empty()
        || path.contains('%')
        || !route_path_for_protocol(&route.protocol, &path)
    {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"non-canonical route path\n".to_vec(),
        );
    }

    let mut proxy_target = route.upstream.clone();
    if let Some(coord) = coordinator {
        let req_obj = RouteRequest {
            credential: cred,
            protocol: route.protocol.clone(),
            method: method.to_uppercase(),
            path: path.clone(),
            raw_path,
            raw_query,
        };
        match coord.resolve_and_authorize(req_obj) {
            Ok(selection) => {
                proxy_target = format!("http://{}", selection.listener);
            }
            Err(_) => {
                return (
                    502,
                    vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                    b"coordinator authorization failed\n".to_vec(),
                );
            }
        }
    }

    forward_to_byok_target(&proxy_target, method, &path, headers, body).await
}

fn check_upstream_host_allowed(target_url: &url::Url) -> Result<(), String> {
    let Some(host) = target_url.host_str() else {
        return Err("missing upstream host".to_string());
    };
    if is_loopback_host(host) {
        return Ok(());
    }
    let port = target_url.port_or_known_default().unwrap_or(443);
    let clean_host = host.trim_matches(['[', ']']);
    if let Ok(ip) = clean_host.parse::<IpAddr>() {
        if !is_public_upstream_ip(ip) {
            return Err("upstream address is not publicly routable".to_string());
        }
        return Ok(());
    }
    let addrs: Vec<SocketAddr> = (clean_host, port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .collect();
    if addrs.iter().any(|a| is_public_upstream_ip(a.ip())) {
        Ok(())
    } else {
        Err("upstream address is not publicly routable".to_string())
    }
}

pub async fn forward_to_byok_target(
    target_base: &str,
    method: &str,
    incoming_path: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let Ok(mut target_url) = url::Url::parse(target_base) else {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"invalid BYOK upstream\n".to_vec(),
        );
    };

    if check_upstream_host_allowed(&target_url).is_err() {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"BYOK upstream unavailable\n".to_vec(),
        );
    }

    let merged_path = byok_target_path(target_url.path(), incoming_path);
    target_url.set_path(&merged_path);
    target_url.set_query(None);

    let is_responses_post = method.eq_ignore_ascii_case("POST")
        && (merged_path == "/v1/responses"
            || merged_path == "/responses"
            || incoming_path == "/v1/responses"
            || incoming_path == "/responses");

    let mut outgoing_body = body.to_vec();
    let mut ns_map = HashMap::new();
    if is_responses_post && !body.is_empty() {
        let (rewritten, m, ok) = flatten_responses_tools(body);
        if ok {
            outgoing_body = rewritten;
            ns_map = m;
        }
    }

    let Ok(client) = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(60))
        .build()
    else {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"BYOK upstream unavailable\n".to_vec(),
        );
    };

    let req_method = reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST);
    let mut req_builder = client.request(req_method, target_url);

    for (k, v) in headers {
        if k.eq_ignore_ascii_case(BYOK_ROUTE_HEADER)
            || k.eq_ignore_ascii_case("host")
            || k.eq_ignore_ascii_case("content-length")
            || k.eq_ignore_ascii_case("connection")
        {
            continue;
        }
        req_builder = req_builder.header(k.as_str(), v.as_str());
    }
    req_builder = req_builder.body(outgoing_body);

    let Ok(resp) = req_builder.send().await else {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"BYOK upstream unavailable\n".to_vec(),
        );
    };

    let status = resp.status().as_u16();
    let mut resp_headers = Vec::new();
    let mut is_event_stream = false;
    for (k, v) in resp.headers() {
        let k_str = k.as_str().to_string();
        let Ok(v_str) = v.to_str() else {
            continue;
        };
        if k_str.eq_ignore_ascii_case("content-type") && v_str.starts_with("text/event-stream") {
            is_event_stream = true;
        }
        if k_str.eq_ignore_ascii_case("content-length")
            || k_str.eq_ignore_ascii_case("transfer-encoding")
        {
            continue;
        }
        resp_headers.push((k_str, v_str.to_string()));
    }

    let Ok(raw_resp_bytes) = resp.bytes().await else {
        return (
            502,
            vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
            b"BYOK upstream unavailable\n".to_vec(),
        );
    };

    let final_body = if is_responses_post && status == 200 && is_event_stream {
        transform_sse_bytes(&raw_resp_bytes, ns_map)
    } else {
        raw_resp_bytes.to_vec()
    };

    (status, resp_headers, final_body)
}
