// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/routing_state.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

pub const ROUTING_STATE_VERSION: i32 = 1;

pub const ROUTING_STATE_ABSENT: &str = "absent";
pub const ROUTING_STATE_PREPARING: &str = "preparing";
pub const ROUTING_STATE_COMMITTED: &str = "committed";
pub const ROUTING_STATE_REVOKING: &str = "revoking";
pub const ROUTING_STATE_STOPPED: &str = "stopped";
pub const ROUTING_STATE_DEGRADED: &str = "degraded";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn new_routing_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut b = [0u8; 32];
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    for chunk_idx in 0..4 {
        let seq = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
        let rs = RandomState::new();
        let mut hasher = rs.build_hasher();
        hasher.write_u128(now);
        hasher.write_u64(seq);
        hasher.write_usize(chunk_idx);
        let val = hasher.finish().to_le_bytes();
        b[chunk_idx * 8..(chunk_idx + 1) * 8].copy_from_slice(&val);
    }

    let mut s = String::with_capacity(43);
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
    val.chars()
        .all(|c| c == '.' || c == '-' || c == '_' || c == ':' || c.is_ascii_alphanumeric())
}

fn b64url_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    }
}

pub fn valid_routing_token(val: &str) -> bool {
    if val.len() != 43 {
        return false;
    }
    let bytes = val.as_bytes();
    for &b in bytes {
        if b64url_val(b).is_none() {
            return false;
        }
    }
    // 32 bytes = 256 bits; 42 * 6 = 252 bits, leaving 4 bits in the 43rd char (bottom 2 bits must be 0 in strict mode)
    if let Some(last) = b64url_val(bytes[42]) {
        (last & 0b11) == 0
    } else {
        false
    }
}

pub fn valid_config_fingerprint(val: &str) -> bool {
    val.len() == 64 && val.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn valid_byok_protocol(protocol: &str) -> bool {
    matches!(
        protocol,
        "openai-chat" | "openai-responses" | "anthropic-messages"
    )
}

pub fn valid_loopback_listener(listener: &str, port: i32) -> bool {
    if port <= 0 || port > 65535 || listener.contains('/') {
        return false;
    }
    listener == format!("127.0.0.1:{}", port) || listener == format!("[::1]:{}", port)
}

pub fn validate_routing_state(
    state: &RoutingState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match state.state.as_str() {
        ROUTING_STATE_ABSENT => {
            if state.version != 0
                || state.generation != 0
                || state.previous_generation != 0
                || !state.coordinator_nonce.is_empty()
                || !state.commit_nonce.is_empty()
                || !state.routes.is_empty()
                || !state.lanes.is_empty()
            {
                return Err("absent routing state must be empty".into());
            }
            return Ok(());
        }
        ROUTING_STATE_PREPARING
        | ROUTING_STATE_COMMITTED
        | ROUTING_STATE_REVOKING
        | ROUTING_STATE_STOPPED
        | ROUTING_STATE_DEGRADED => {}
        _ => return Err(format!("invalid routing state {:?}", state.state).into()),
    }
    if state.version != ROUTING_STATE_VERSION {
        return Err(format!("unsupported routing state version {}", state.version).into());
    }
    if state.generation == 0
        || state.previous_generation > state.generation
        || !valid_routing_id(&state.coordinator_nonce)
    {
        return Err("invalid routing state generation or coordinator".into());
    }
    if state.state == ROUTING_STATE_COMMITTED && !valid_routing_id(&state.commit_nonce) {
        return Err("committed routing state has invalid commit nonce".into());
    }
    if state.state != ROUTING_STATE_COMMITTED && !state.commit_nonce.is_empty() {
        return Err("non-committed routing state has commit nonce".into());
    }
    if state.state == ROUTING_STATE_COMMITTED
        && (state.routes.is_empty() || state.lanes.is_empty())
    {
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
        if !valid_routing_id(&lane.provider_instance_id)
            || !valid_routing_id(&lane.upstream_id)
            || !valid_byok_protocol(&lane.protocol)
            || lane.coordinator_nonce != state.coordinator_nonce
        {
            return Err(format!("lane {:?} has invalid backend identity", lane.id).into());
        }
        if lane.generation != state.generation
            || lane.port <= 0
            || lane.port > 65535
            || !valid_loopback_listener(&lane.listener, lane.port)
        {
            return Err(format!("lane {:?} has invalid generation or listener", lane.id).into());
        }
        if ports.contains(&lane.port) {
            return Err(format!("duplicate lane port {}", lane.port).into());
        }
        let backend = format!(
            "{}\0{}\0{}",
            lane.provider_instance_id, lane.protocol, lane.upstream_id
        );
        if backends.contains(&backend) {
            return Err(format!("duplicate lane backend {:?}", lane.provider_instance_id).into());
        }
        if lane_providers.contains(lane.provider_instance_id.as_str()) {
            return Err(format!(
                "duplicate lane provider instance {:?}",
                lane.provider_instance_id
            )
            .into());
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
        if route.revoked
            && (route.revoked_generation == 0 || route.revoked_generation > state.generation)
            || !route.revoked && route.revoked_generation != 0
        {
            return Err(format!("route {:?} has invalid revocation state", route.id).into());
        }
        let lane = match lanes.get(route.lane_id.as_str()) {
            Some(&l) => l,
            None => {
                return Err(format!("route {:?} does not bind exactly to lane", route.id).into())
            }
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
            return Err(format!(
                "duplicate provider instance {:?}",
                route.provider_instance_id
            )
            .into());
        }
        if state.state == ROUTING_STATE_COMMITTED && (route.revoked || !lane.ready) {
            return Err(
                format!("committed route {:?} is revoked or lane is not ready", route.id).into(),
            );
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
    validate_routing_state(previous)
        .map_err(|e| format!("invalid previous routing state: {}", e))?;
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
        ROUTING_STATE_PREPARING => {
            &[ROUTING_STATE_COMMITTED, ROUTING_STATE_DEGRADED, ROUTING_STATE_STOPPED][..]
        }
        ROUTING_STATE_COMMITTED => &[
            ROUTING_STATE_PREPARING,
            ROUTING_STATE_REVOKING,
            ROUTING_STATE_DEGRADED,
            ROUTING_STATE_STOPPED,
        ][..],
        ROUTING_STATE_REVOKING => {
            &[ROUTING_STATE_STOPPED, ROUTING_STATE_PREPARING, ROUTING_STATE_DEGRADED][..]
        }
        ROUTING_STATE_DEGRADED => &[ROUTING_STATE_PREPARING, ROUTING_STATE_STOPPED][..],
        ROUTING_STATE_STOPPED => &[ROUTING_STATE_PREPARING][..],
        _ => &[][..],
    };
    if !allowed.contains(&next.state.as_str()) {
        return Err(format!(
            "invalid routing state transition {:?} to {:?}",
            previous.state, next.state
        )
        .into());
    }
    Ok(())
}

pub(crate) fn validate_no_duplicate_json_keys(
    raw: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    struct NoDupValue;

    impl<'de> serde::de::Deserialize<'de> for NoDupValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = NoDupValue;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("any valid JSON value without duplicate object keys")
                }
                fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_string<E>(self, _: String) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_unit<E>(self) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_none<E>(self) -> Result<Self::Value, E> {
                    Ok(NoDupValue)
                }
                fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::de::Deserializer<'de>,
                {
                    serde::de::Deserialize::deserialize(deserializer)
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    while seq.next_element::<NoDupValue>()?.is_some() {}
                    Ok(NoDupValue)
                }
                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut keys = HashSet::new();
                    while let Some((k, _)) = map.next_entry::<String, NoDupValue>()? {
                        if !keys.insert(k) {
                            return Err(serde::de::Error::custom(
                                "duplicate or invalid JSON object key",
                            ));
                        }
                    }
                    Ok(NoDupValue)
                }
            }
            deserializer.deserialize_any(Visitor)
        }
    }

    let _: NoDupValue = serde_json::from_str(raw)?;
    Ok(())
}

pub fn parse_routing_state(
    raw: &str,
) -> Result<RoutingState, Box<dyn std::error::Error + Send + Sync>> {
    validate_no_duplicate_json_keys(raw)?;
    let state: RoutingState = serde_json::from_str(raw)?;
    validate_routing_state(&state)?;
    Ok(state)
}

pub fn encode_routing_state(
    state: &RoutingState,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    validate_routing_state(state)?;
    Ok(serde_json::to_string(state)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_no_duplicate_json_keys() {
        let valid = r#"{"a": 1, "b": 2}"#;
        assert!(validate_no_duplicate_json_keys(valid).is_ok());

        let dup = r#"{"a": 1, "a": 2}"#;
        assert!(validate_no_duplicate_json_keys(dup).is_err());
    }

    #[test]
    fn test_valid_byok_protocols() {
        assert!(valid_byok_protocol("openai-chat"));
        assert!(valid_byok_protocol("anthropic-messages"));
        assert!(!valid_byok_protocol("unknown-proto"));
    }

    #[test]
    fn test_routing_state_transitions() {
        let prev_absent = RoutingState {
            state: ROUTING_STATE_ABSENT.to_string(),
            ..Default::default()
        };
        let next_prep = RoutingState {
            version: ROUTING_STATE_VERSION,
            generation: 1,
            previous_generation: 0,
            state: ROUTING_STATE_PREPARING.to_string(),
            coordinator_nonce: "coord_1".to_string(),
            ..Default::default()
        };
        assert!(validate_routing_state_transition(&prev_absent, &next_prep).is_ok());

        let bad_next = RoutingState {
            version: ROUTING_STATE_VERSION,
            generation: 1,
            previous_generation: 1,
            state: ROUTING_STATE_COMMITTED.to_string(),
            coordinator_nonce: "coord_1".to_string(),
            ..Default::default()
        };
        assert!(validate_routing_state_transition(&prev_absent, &bad_next).is_err());
    }
}
