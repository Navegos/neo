use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

pub const ROUTING_STATE_VERSION: i32 = 1;

pub const ROUTING_STATE_ABSENT: &str = "absent";
pub const ROUTING_STATE_PREPARING: &str = "preparing";
pub const ROUTING_STATE_COMMITTED: &str = "committed";
pub const ROUTING_STATE_REVOKING: &str = "revoking";
pub const ROUTING_STATE_STOPPED: &str = "stopped";
pub const ROUTING_STATE_DEGRADED: &str = "degraded";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RoutingState {
    pub version: i32,
    pub generation: u64,
    pub previous_generation: u64,
    pub state: String,
    pub coordinator_nonce: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub commit_nonce: String,
    #[serde(default)]
    pub routes: Vec<RouteBinding>,
    #[serde(default)]
    pub lanes: Vec<LaneBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RouteBinding {
    pub id: String,
    pub token: String,
    pub profile_id: String,
    pub surface_id: String,
    pub provider_instance_id: String,
    pub protocol: String,
    pub lane_id: String,
    pub lane_instance_nonce: String,
    pub coordinator_nonce: String,
    pub upstream_id: String,
    pub config_fingerprint: String,
    pub generation: u64,
    pub revoked: bool,
    #[serde(skip_serializing_if = "is_zero_u64", default)]
    pub revoked_generation: u64,
}

fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LaneBinding {
    pub id: String,
    pub instance_nonce: String,
    pub provider_instance_id: String,
    pub protocol: String,
    pub upstream_id: String,
    pub listener: String,
    pub port: i32,
    pub generation: u64,
    pub coordinator_nonce: String,
    pub ready: bool,
}

pub fn new_routing_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut b = [0u8; 32];
    use std::time::SystemTime;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_nanos();
    for (i, byte) in b.iter_mut().enumerate() {
        let v = (now.wrapping_add((i as u128).wrapping_mul(0x9e3779b97f4a7c15))) ^ ((i as u128) << 32);
        *byte = (v ^ (v >> 8) ^ (v >> 16)) as u8;
    }
    // Encode with unpadded base64url
    let mut s = String::new();
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut buf: u32 = 0;
    let mut bits = 0;
    for &byte in &b {
        buf = (buf << 8) | (byte as u32);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            s.push(CHARS[((buf >> bits) & 0x3f) as usize] as char);
        }
    }
    if bits > 0 {
        s.push(CHARS[((buf << (6 - bits)) & 0x3f) as usize] as char);
    }
    Ok(s)
}

pub fn valid_routing_id(val: &str) -> bool {
    if val.is_empty() || val.len() > 256 {
        return false;
    }
    val.chars().all(|c| {
        c == '.' || c == '-' || c == '_' || c == ':' || c.is_ascii_alphanumeric()
    })
}

pub fn valid_routing_token(val: &str) -> bool {
    val.len() == 43 && val.chars().all(|c| c == '-' || c == '_' || c.is_ascii_alphanumeric())
}

pub fn valid_config_fingerprint(val: &str) -> bool {
    val.len() == 64 && val.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn valid_byok_protocol(protocol: &str) -> bool {
    matches!(protocol, "openai-chat" | "openai-responses" | "anthropic-messages")
}

pub fn valid_loopback_listener(listener: &str, port: i32) -> bool {
    if let Some((host, p_str)) = listener.rsplit_once(':') {
        let clean_host = host.trim_matches(['[', ']']);
        if (clean_host == "127.0.0.1" || clean_host == "::1") && p_str == port.to_string() && !listener.contains('/') {
            return true;
        }
    }
    false
}

pub fn validate_routing_state(state: &RoutingState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match state.state.as_str() {
        ROUTING_STATE_ABSENT => {
            if state.version != 0 || state.generation != 0 || state.previous_generation != 0 || !state.coordinator_nonce.is_empty() || !state.commit_nonce.is_empty() || !state.routes.is_empty() || !state.lanes.is_empty() {
                return Err("absent routing state must be empty".into());
            }
            return Ok(());
        }
        ROUTING_STATE_PREPARING | ROUTING_STATE_COMMITTED | ROUTING_STATE_REVOKING | ROUTING_STATE_STOPPED | ROUTING_STATE_DEGRADED => {}
        _ => return Err(format!("invalid routing state {:?}", state.state).into()),
    }
    if state.version != ROUTING_STATE_VERSION {
        return Err(format!("unsupported routing state version {}", state.version).into());
    }
    if state.generation == 0 || state.previous_generation > state.generation || !valid_routing_id(&state.coordinator_nonce) {
        return Err("invalid routing state generation or coordinator".into());
    }
    if state.state == ROUTING_STATE_COMMITTED && !valid_routing_id(&state.commit_nonce) {
        return Err("committed routing state has invalid commit nonce".into());
    }
    if state.state != ROUTING_STATE_COMMITTED && !state.commit_nonce.is_empty() {
        return Err("non-committed routing state has commit nonce".into());
    }
    if state.state == ROUTING_STATE_COMMITTED && (state.routes.is_empty() || state.lanes.is_empty()) {
        return Err("committed routing state is empty".into());
    }

    let mut lanes: HashMap<&str, &LaneBinding> = HashMap::new();
    let mut ports: HashSet<i32> = HashSet::new();
    let mut backends: HashSet<String> = HashSet::new();
    let mut lane_providers: HashSet<&str> = HashSet::new();

    for lane in &state.lanes {
        if !valid_routing_id(&lane.id) || !valid_routing_id(&lane.instance_nonce) {
            return Err("invalid lane identity".into());
        }
        if lanes.contains_key(lane.id.as_str()) {
            return Err(format!("duplicate lane {:?}", lane.id).into());
        }
        if !valid_routing_id(&lane.provider_instance_id) || !valid_routing_id(&lane.upstream_id) || !valid_byok_protocol(&lane.protocol) || lane.coordinator_nonce != state.coordinator_nonce {
            return Err(format!("lane {:?} has invalid backend identity", lane.id).into());
        }
        if lane.generation != state.generation || lane.port <= 0 || lane.port > 65535 || !valid_loopback_listener(&lane.listener, lane.port) {
            return Err(format!("lane {:?} has invalid generation or listener", lane.id).into());
        }
        if ports.contains(&lane.port) {
            return Err(format!("duplicate lane port {}", lane.port).into());
        }
        let backend = format!("{}\0{}\0{}", lane.provider_instance_id, lane.protocol, lane.upstream_id);
        if backends.contains(&backend) {
            return Err(format!("duplicate lane backend {:?}", lane.provider_instance_id).into());
        }
        if lane_providers.contains(lane.provider_instance_id.as_str()) {
            return Err(format!("duplicate lane provider instance {:?}", lane.provider_instance_id).into());
        }
        ports.insert(lane.port);
        backends.insert(backend);
        lane_providers.insert(&lane.provider_instance_id);
        lanes.insert(&lane.id, lane);
    }

    let mut route_ids: HashSet<&str> = HashSet::new();
    let mut tokens: HashSet<&str> = HashSet::new();
    let mut providers: HashSet<&str> = HashSet::new();

    for route in &state.routes {
        if !valid_routing_id(&route.id) || !valid_routing_token(&route.token) {
            return Err("invalid route identity".into());
        }
        if route_ids.contains(route.id.as_str()) {
            return Err(format!("duplicate route {:?}", route.id).into());
        }
        if tokens.contains(route.token.as_str()) {
            return Err("duplicate route token".into());
        }
        if !valid_routing_id(&route.profile_id)
            || !valid_routing_id(&route.surface_id)
            || !valid_routing_id(&route.provider_instance_id)
            || !valid_byok_protocol(&route.protocol)
            || !valid_routing_id(&route.lane_id)
            || !valid_routing_id(&route.lane_instance_nonce)
            || route.coordinator_nonce != state.coordinator_nonce
            || !valid_routing_id(&route.upstream_id)
            || !valid_config_fingerprint(&route.config_fingerprint)
        {
            return Err(format!("route {:?} has invalid identity", route.id).into());
        }
        if route.generation != state.generation {
            return Err(format!("route {:?} has stale generation", route.id).into());
        }
        if route.revoked && (route.revoked_generation == 0 || route.revoked_generation > state.generation)
            || !route.revoked && route.revoked_generation != 0
        {
            return Err(format!("route {:?} has invalid revocation state", route.id).into());
        }
        let lane = match lanes.get(route.lane_id.as_str()) {
            Some(&l) => l,
            None => return Err(format!("route {:?} does not bind exactly to lane", route.id).into()),
        };
        if lane.instance_nonce != route.lane_instance_nonce
            || lane.coordinator_nonce != route.coordinator_nonce
            || lane.provider_instance_id != route.provider_instance_id
            || lane.protocol != route.protocol
            || lane.upstream_id != route.upstream_id
        {
            return Err(format!("route {:?} does not bind exactly to lane", route.id).into());
        }
        if providers.contains(route.provider_instance_id.as_str()) {
            return Err(format!("duplicate provider instance {:?}", route.provider_instance_id).into());
        }
        if state.state == ROUTING_STATE_COMMITTED && (route.revoked || !lane.ready) {
            return Err(format!("committed route {:?} is revoked or lane is not ready", route.id).into());
        }
        if state.state != ROUTING_STATE_COMMITTED && !route.revoked {
            return Err(format!("non-committed route {:?} is active", route.id).into());
        }
        route_ids.insert(&route.id);
        tokens.insert(&route.token);
        providers.insert(&route.provider_instance_id);
    }
    Ok(())
}

pub fn validate_routing_state_transition(
    previous: &RoutingState,
    next: &RoutingState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    validate_routing_state(previous)?;
    validate_routing_state(next)?;
    if previous.state == ROUTING_STATE_ABSENT {
        if next.previous_generation != 0 || next.state != ROUTING_STATE_PREPARING {
            return Err("initial routing state must enter preparing".into());
        }
        return Ok(());
    }
    if previous.coordinator_nonce != next.coordinator_nonce {
        return Err("routing coordinator changed".into());
    }
    if next.previous_generation != previous.generation || next.generation <= previous.generation {
        return Err("routing generation is not monotonic".into());
    }

    let allowed = match previous.state.as_str() {
        ROUTING_STATE_PREPARING => &[ROUTING_STATE_COMMITTED, ROUTING_STATE_DEGRADED, ROUTING_STATE_STOPPED][..],
        ROUTING_STATE_COMMITTED => &[ROUTING_STATE_PREPARING, ROUTING_STATE_REVOKING, ROUTING_STATE_DEGRADED, ROUTING_STATE_STOPPED][..],
        ROUTING_STATE_REVOKING => &[ROUTING_STATE_STOPPED, ROUTING_STATE_PREPARING, ROUTING_STATE_DEGRADED][..],
        ROUTING_STATE_DEGRADED => &[ROUTING_STATE_PREPARING, ROUTING_STATE_STOPPED][..],
        ROUTING_STATE_STOPPED => &[ROUTING_STATE_PREPARING][..],
        _ => &[][..],
    };
    if !allowed.contains(&next.state.as_str()) {
        return Err(format!("invalid routing state transition {:?} to {:?}", previous.state, next.state).into());
    }
    Ok(())
}

pub fn parse_routing_state(raw: &str) -> Result<RoutingState, Box<dyn std::error::Error + Send + Sync>> {
    let state: RoutingState = serde_json::from_str(raw)?;
    validate_routing_state(&state)?;
    Ok(state)
}

pub fn encode_routing_state(state: &RoutingState) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    validate_routing_state(state)?;
    Ok(serde_json::to_string(state)?)
}
