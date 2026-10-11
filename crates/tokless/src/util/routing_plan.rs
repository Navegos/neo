use super::routing_state::{
    valid_byok_protocol, valid_routing_id, valid_routing_token, validate_routing_state,
    RoutingState, ROUTING_STATE_COMMITTED,
};
use super::routing_store::{
    lease_live, lease_matches_state, validate_routing_lease, RoutingLease,
};

#[derive(Debug, Clone)]
pub struct RoutingSnapshot {
    pub state: RoutingState,
}

impl RoutingSnapshot {
    pub fn new(state: RoutingState) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        validate_routing_state(&state)?;
        Ok(RoutingSnapshot { state })
    }

    pub fn generation(&self) -> u64 {
        self.state.generation
    }

    pub fn resolve_request<F>(
        &self,
        request: RouteRequest,
        ownership: F,
    ) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&RouteSelection) -> bool,
    {
        if request.method != "POST" {
            return Err("route method not allowed".into());
        }
        if !request.raw_query.is_empty()
            || !request.raw_path.is_empty()
            || request.path.contains('%')
        {
            return Err("non-canonical route path".into());
        }
        if !route_path_for_protocol(&request.protocol, &request.path) {
            return Err("route protocol or path mismatch".into());
        }
        let selection = resolve_route(&self.state, &request.credential, &request.protocol)?;
        if !ownership(&selection) {
            return Err("route listener ownership unavailable".into());
        }
        Ok(selection)
    }

    pub fn resolve_request_with_lease<F>(
        &self,
        request: RouteRequest,
        lease: &RoutingLease,
        ownership: F,
    ) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&RouteSelection) -> bool,
    {
        validate_routing_lease(lease)?;
        if !lease_live(lease, chrono::Utc::now()) {
            return Err("routing lease is unavailable".into());
        }
        let mut selection = self.resolve_request(request, |sel| {
            if !lease_matches_state(lease, &self.state)
                || lease.lane_id != sel.lane_id
                || lease.listener != sel.listener
            {
                return false;
            }
            let mut with_nonce = sel.clone();
            with_nonce.lease_nonce = lease.lease_nonce.clone();
            ownership(&with_nonce)
        })?;
        selection.lease_nonce = lease.lease_nonce.clone();
        Ok(selection)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RouteRequest {
    pub credential: String,
    pub protocol: String,
    pub method: String,
    pub path: String,
    pub raw_path: String,
    pub raw_query: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteSelection {
    pub route_id: String,
    pub lane_id: String,
    pub lane_instance_nonce: String,
    pub coordinator_nonce: String,
    pub upstream_id: String,
    pub listener: String,
    pub protocol: String,
    pub generation: u64,
    pub lease_nonce: String,
}

pub fn route_protocol_for_path(path: &str) -> Option<&'static str> {
    match path {
        "/v1/chat/completions" | "/chat/completions" => Some("openai-chat"),
        "/v1/responses" | "/responses" => Some("openai-responses"),
        "/v1/messages" | "/messages" => Some("anthropic-messages"),
        _ => None,
    }
}

pub fn route_path_for_protocol(protocol: &str, path: &str) -> bool {
    route_protocol_for_path(path) == Some(protocol)
}

pub fn route_protocol_path(protocol: &str, path: &str) -> bool {
    route_path_for_protocol(protocol, path)
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

pub fn resolve_route(
    state: &RoutingState,
    credential: &str,
    protocol: &str,
) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>> {
    validate_routing_state(state).map_err(|e| format!("invalid routing state: {}", e))?;
    if state.state != ROUTING_STATE_COMMITTED {
        return Err("routing state is not committed".into());
    }
    if !valid_byok_protocol(protocol) {
        return Err("invalid request protocol".into());
    }
    let idx = match credential.rfind('.') {
        Some(i) if i > 0 && i < credential.len() - 1 => i,
        _ => return Err("malformed route credential".into()),
    };
    let id = &credential[..idx];
    let token = &credential[idx + 1..];
    if !valid_routing_id(id) || !valid_routing_token(token) || token.contains('.') {
        return Err("malformed route credential".into());
    }

    for route in &state.routes {
        if route.id != id || !constant_time_eq(route.token.as_bytes(), token.as_bytes()) {
            continue;
        }
        if route.revoked || route.generation != state.generation || route.protocol != protocol {
            return Err("route credential is stale or protocol mismatched".into());
        }
        for lane in &state.lanes {
            if lane.id == route.lane_id
                && lane.instance_nonce == route.lane_instance_nonce
                && lane.coordinator_nonce == route.coordinator_nonce
                && lane.provider_instance_id == route.provider_instance_id
                && lane.protocol == route.protocol
                && lane.upstream_id == route.upstream_id
                && lane.generation == state.generation
                && lane.ready
            {
                return Ok(RouteSelection {
                    route_id: route.id.clone(),
                    lane_id: lane.id.clone(),
                    lane_instance_nonce: lane.instance_nonce.clone(),
                    coordinator_nonce: lane.coordinator_nonce.clone(),
                    upstream_id: lane.upstream_id.clone(),
                    listener: lane.listener.clone(),
                    protocol: lane.protocol.clone(),
                    generation: state.generation,
                    lease_nonce: String::new(),
                });
            }
        }
        return Err("route lane is unavailable".into());
    }
    Err("unknown route credential".into())
}
