use std::sync::Arc;
use std::time::Duration;

use crate::util::routing_coordinator::RoutingCoordinator;
use crate::util::routing_plan::{
    route_protocol_for_path, RouteRequest, RouteSelection, RoutingSnapshot,
};

pub const ROUTER_ROUTE_HEADER: &str = "X-Tokless-Route";

pub struct Router {
    coordinator: Option<Arc<RoutingCoordinator>>,
    snapshot: Option<RoutingSnapshot>,
    owned: Option<Arc<dyn Fn(&RouteSelection) -> bool + Send + Sync>>,
}

impl Router {
    pub fn new<F>(snapshot: RoutingSnapshot, owned: F) -> Self
    where
        F: Fn(&RouteSelection) -> bool + Send + Sync + 'static,
    {
        Self {
            coordinator: None,
            snapshot: Some(snapshot),
            owned: Some(Arc::new(owned)),
        }
    }

    pub fn from_coordinator(coordinator: Arc<RoutingCoordinator>) -> Self {
        Self {
            coordinator: Some(coordinator),
            snapshot: None,
            owned: None,
        }
    }

    pub fn from_snapshot(snapshot: RoutingSnapshot) -> Self {
        Self {
            coordinator: None,
            snapshot: Some(snapshot),
            owned: None,
        }
    }

    pub fn resolve_route_request(
        &self,
        credential: &str,
        path: &str,
        method: &str,
    ) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>> {
        self.resolve_full_request(credential, method, path, "", "")
    }

    pub fn resolve_full_request(
        &self,
        credential: &str,
        method: &str,
        path: &str,
        raw_path: &str,
        raw_query: &str,
    ) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>> {
        let protocol = route_protocol_for_path(path).ok_or("unsupported route protocol")?;
        let req = RouteRequest {
            credential: credential.to_string(),
            protocol: protocol.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            raw_path: raw_path.to_string(),
            raw_query: raw_query.to_string(),
        };

        let selection = if let Some(ref coord) = self.coordinator {
            coord.resolve_and_authorize(req)?
        } else if let Some(ref snap) = self.snapshot {
            if let Some(ref owned_cb) = self.owned {
                let cb = owned_cb.clone();
                snap.resolve_request(req, move |sel| cb(sel))?
            } else {
                snap.resolve_request(req, |_| true)?
            }
        } else {
            return Err("router has no active snapshot or coordinator".into());
        };

        validate_router_selection(&selection)?;
        Ok(selection)
    }

    pub async fn handle_http_request(
        &self,
        method: &str,
        raw_uri: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> (u16, Vec<(String, String)>, Vec<u8>) {
        let path_and_query = if let Some(idx) = raw_uri.find("://") {
            let after_scheme = &raw_uri[idx + 3..];
            match after_scheme.find('/') {
                Some(slash) => &after_scheme[slash..],
                None => "/",
            }
        } else {
            raw_uri
        };
        let (path, raw_query) = match path_and_query.split_once('?') {
            Some((p, q)) => (p, q),
            None => (path_and_query, ""),
        };
        let raw_path = if path.contains('%') { path } else { "" };

        if route_protocol_for_path(path).is_none() {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"unsupported route protocol\n".to_vec(),
            );
        }

        let credentials: Vec<&str> = headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case(ROUTER_ROUTE_HEADER))
            .map(|(_, v)| v.as_str())
            .collect();
        if credentials.len() != 1 {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"route rejected\n".to_vec(),
            );
        }

        let Ok(selection) =
            self.resolve_full_request(credentials[0], method, path, raw_path, raw_query)
        else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"route rejected\n".to_vec(),
            );
        };

        if let Some(ref coord) = self.coordinator {
            if coord.authorize(&selection).is_err() {
                return (
                    502,
                    vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                    b"route lease unavailable\n".to_vec(),
                );
            }
        }

        let target_url = format!("http://{}{}", selection.listener, path);
        let Ok(client) = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(60))
            .build()
        else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"route backend unavailable\n".to_vec(),
            );
        };

        let req_method =
            reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST);
        let mut builder = client.request(req_method, &target_url);
        for (k, v) in headers {
            if k.eq_ignore_ascii_case(ROUTER_ROUTE_HEADER)
                || k.eq_ignore_ascii_case("host")
                || k.eq_ignore_ascii_case("content-length")
                || k.eq_ignore_ascii_case("connection")
            {
                continue;
            }
            builder = builder.header(k.as_str(), v.as_str());
        }
        builder = builder.body(body.to_vec());

        let Ok(resp) = builder.send().await else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"route backend unavailable\n".to_vec(),
            );
        };

        let status = resp.status().as_u16();
        let mut resp_headers = Vec::new();
        for (k, v) in resp.headers() {
            if let Ok(v_str) = v.to_str() {
                let k_str = k.as_str();
                if k_str.eq_ignore_ascii_case("content-length")
                    || k_str.eq_ignore_ascii_case("transfer-encoding")
                {
                    continue;
                }
                resp_headers.push((k_str.to_string(), v_str.to_string()));
            }
        }
        let Ok(resp_body) = resp.bytes().await else {
            return (
                502,
                vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())],
                b"route backend unavailable\n".to_vec(),
            );
        };
        (status, resp_headers, resp_body.to_vec())
    }
}

pub fn new_router<F>(snapshot: RoutingSnapshot, owned: F) -> Router
where
    F: Fn(&RouteSelection) -> bool + Send + Sync + 'static,
{
    Router::new(snapshot, owned)
}

pub fn new_coordinator_router(coordinator: Arc<RoutingCoordinator>) -> Router {
    Router::from_coordinator(coordinator)
}

pub fn router_address(selection: &RouteSelection) -> String {
    let Some((host_raw, raw_port)) = selection.listener.rsplit_once(':') else {
        return String::new();
    };
    let Ok(port) = raw_port.parse::<i32>() else {
        return String::new();
    };
    if port <= 0 || port > 65535 || port.to_string() != raw_port {
        return String::new();
    }
    let host = if host_raw.starts_with('[') && host_raw.ends_with(']') {
        &host_raw[1..host_raw.len() - 1]
    } else {
        host_raw
    };
    if host != "127.0.0.1" && host != "::1" {
        return String::new();
    }
    let joined = if host.contains(':') {
        format!("[{}]:{}", host, raw_port)
    } else {
        format!("{}:{}", host, raw_port)
    };
    if joined != selection.listener {
        return String::new();
    }
    selection.listener.clone()
}

pub fn validate_router_selection(
    selection: &RouteSelection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if router_address(selection).is_empty()
        || selection.route_id.is_empty()
        || selection.lane_id.is_empty()
        || selection.generation == 0
    {
        return Err("invalid route selection".into());
    }
    Ok(())
}
