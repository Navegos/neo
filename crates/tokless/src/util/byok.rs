use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::headroom::headroom_paths_resolved;
use super::paths::{ensure_dir, read_file_safe, write_file_atomic};
pub use super::routing_state::valid_byok_protocol;
use super::routing_state::new_routing_token;

static BYOK_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

fn with_byok_registry_lock<F, T>(f: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
{
    let root = headroom_paths_resolved().root;
    ensure_dir(&root)?;
    let lock_path = Path::new(&root).join("byok.routes.lock");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let token = format!("{}-{}", std::process::id(), nanos);

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                if let Err(err) = file.write_all(token.as_bytes()).and_then(|_| file.flush()) {
                    drop(file);
                    let _ = std::fs::remove_file(&lock_path);
                    return Err(err.into());
                }
                drop(file);
                let res = f();
                if let Ok(raw) = std::fs::read_to_string(&lock_path) {
                    if raw == token {
                        let _ = std::fs::remove_file(&lock_path);
                    }
                }
                return res;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if let Ok(meta) = std::fs::metadata(&lock_path) {
                    if let Ok(modified) = meta.modified() {
                        if modified.elapsed().unwrap_or_default() > Duration::from_secs(30) {
                            let _ = std::fs::remove_file(&lock_path);
                            continue;
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(err) => return Err(err.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BYOKRoute {
    pub id: String,
    pub token: String,
    pub protocol: String,
    pub upstream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BYOKRouteFile {
    pub routes: Vec<BYOKRoute>,
}

pub fn byok_routes_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("byok.routes.json")
        .to_string_lossy()
        .to_string()
}

pub fn byok_route_header(route: &BYOKRoute) -> String {
    format!("{}.{}", route.id, route.token)
}

pub fn byok_route_protocol(api: &str) -> &'static str {
    match api {
        "openai-completions" => "openai-chat",
        "openai-responses" => "openai-responses",
        "anthropic-messages" => "anthropic-messages",
        _ => "",
    }
}

pub fn valid_byok_id(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    value
        .chars()
        .all(|c| c == '.' || c == '-' || c == '_' || c == ':' || c.is_ascii_alphanumeric())
}

pub fn valid_byok_token(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    value
        .chars()
        .all(|c| c == '-' || c == '_' || c.is_ascii_alphanumeric())
}

pub fn is_loopback_host(host: &str) -> bool {
    let clean = host.trim_matches(['[', ']']);
    if clean.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Ok(ip) = clean.parse::<std::net::IpAddr>() {
        return ip.is_loopback();
    }
    false
}

pub fn valid_byok_upstream(upstream: &str) -> bool {
    let clean = upstream.trim().trim_end_matches('/');
    let Ok(url) = url::Url::parse(clean) else {
        return false;
    };
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    if host.is_empty() {
        return false;
    }
    let is_loopback = is_loopback_host(host);
    if url.scheme() == "https" {
        if is_loopback {
            return true;
        }
        let port = url.port();
        return port.is_none() || port == Some(443) || port == Some(8443);
    }
    url.scheme() == "http" && is_loopback
}

pub fn new_byok_route(
    id: &str,
    protocol: &str,
    upstream: &str,
) -> Result<BYOKRoute, Box<dyn std::error::Error + Send + Sync>> {
    let clean_upstream = upstream.trim().trim_end_matches('/');
    if !valid_byok_upstream(clean_upstream) {
        return Err("invalid BYOK upstream".into());
    }
    if !valid_byok_id(id) || !valid_byok_protocol(protocol) {
        return Err("invalid BYOK route identity".into());
    }
    let token = new_routing_token()?;
    Ok(BYOKRoute {
        id: id.to_string(),
        token,
        protocol: protocol.to_string(),
        upstream: clean_upstream.to_string(),
    })
}

pub fn parse_byok_routes(
    raw: &str,
) -> Result<Vec<BYOKRoute>, Box<dyn std::error::Error + Send + Sync>> {
    let f: BYOKRouteFile = serde_json::from_str(raw)?;
    Ok(f.routes)
}

pub fn encode_byok_routes(
    routes: &[BYOKRoute],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let f = BYOKRouteFile {
        routes: routes.to_vec(),
    };
    Ok(serde_json::to_string(&f)?)
}

pub fn save_byok_routes(
    routes: &[BYOKRoute],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = BYOK_REGISTRY_LOCK.lock().unwrap();
    with_byok_registry_lock(|| {
        let mut clean = Vec::with_capacity(routes.len());
        let mut seen = HashSet::with_capacity(routes.len());
        for route in routes {
            let id = route.id.trim().to_string();
            let token = route.token.trim().to_string();
            let protocol = route.protocol.trim().to_string();
            let upstream = route.upstream.trim().trim_end_matches('/').to_string();
            if !valid_byok_id(&id) || !valid_byok_token(&token) || !valid_byok_protocol(&protocol) {
                return Err(format!("invalid BYOK route {:?}", id).into());
            }
            new_byok_route(&id, &protocol, &upstream)?;
            if !seen.insert(id.clone()) {
                return Err(format!("route {:?}: duplicate ID", id).into());
            }
            clean.push(BYOKRoute {
                id,
                token,
                protocol,
                upstream,
            });
        }
        let content = encode_byok_routes(&clean)?;
        write_file_atomic(byok_routes_path(), &content)?;
        Ok(())
    })
}

pub fn read_byok_routes() -> Result<Vec<BYOKRoute>, Box<dyn std::error::Error + Send + Sync>> {
    let _guard = BYOK_REGISTRY_LOCK.lock().unwrap();
    with_byok_registry_lock(|| {
        if let Some(raw) = read_file_safe(byok_routes_path()) {
            parse_byok_routes(&raw)
        } else {
            Ok(Vec::new())
        }
    })
}

pub fn upsert_byok_route(
    id: &str,
    protocol: &str,
    upstream: &str,
) -> Result<(BYOKRoute, Option<BYOKRoute>), Box<dyn std::error::Error + Send + Sync>> {
    new_byok_route(id, protocol, upstream)?;
    let _guard = BYOK_REGISTRY_LOCK.lock().unwrap();
    with_byok_registry_lock(|| {
        let mut routes = if let Some(raw) = read_file_safe(byok_routes_path()) {
            parse_byok_routes(&raw)
                .map_err(|e| format!("invalid BYOK route registry: {}", e))?
        } else {
            Vec::new()
        };
        for r in &mut routes {
            if r.id == id {
                let previous = r.clone();
                r.protocol = protocol.to_string();
                r.upstream = upstream.trim().trim_end_matches('/').to_string();
                let result = r.clone();
                let content = encode_byok_routes(&routes)?;
                write_file_atomic(byok_routes_path(), &content)?;
                return Ok((result, Some(previous)));
            }
        }
        let new_route = new_byok_route(id, protocol, upstream)?;
        routes.push(new_route.clone());
        let content = encode_byok_routes(&routes)?;
        write_file_atomic(byok_routes_path(), &content)?;
        Ok((new_route, None))
    })
}

pub fn byok_gateway_port() -> u16 {
    if let Ok(raw) = std::env::var("TOKLESS_BYOK_PROXY_PORT") {
        if let Ok(p) = raw.trim().parse::<u16>() {
            if p > 0 {
                return p;
            }
        }
    }
    18787
}

pub fn byok_gateway_endpoint() -> String {
    format!("http://127.0.0.1:{}", byok_gateway_port())
}

pub fn read_byok_route(id: &str) -> Option<BYOKRoute> {
    read_byok_routes().ok()?.into_iter().find(|r| r.id == id)
}

pub fn delete_byok_route(id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = BYOK_REGISTRY_LOCK.lock().unwrap();
    with_byok_registry_lock(|| {
        let path = byok_routes_path();
        if !Path::new(&path).exists() {
            return Ok(());
        }
        let raw = std::fs::read_to_string(&path)?;
        let mut routes = parse_byok_routes(&raw)?;
        routes.retain(|r| r.id != id);
        let content = encode_byok_routes(&routes)?;
        write_file_atomic(path, &content)?;
        Ok(())
    })
}
