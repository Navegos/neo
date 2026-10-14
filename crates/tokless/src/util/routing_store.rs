// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/routing_store.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::paths::{ensure_dir, tokless_data_dir, write_file_atomic};
use super::routing_lock::acquire_routing_file_lock;
use super::routing_plan::RouteSelection;
use super::routing_state::{
    encode_routing_state, new_routing_token, parse_routing_state, valid_loopback_listener,
    valid_routing_id, valid_routing_token, validate_no_duplicate_json_keys,
    validate_routing_state, validate_routing_state_transition, LaneBinding, RoutingState,
    ROUTING_STATE_ABSENT, ROUTING_STATE_COMMITTED, ROUTING_STATE_VERSION,
};

const ROUTING_STATE_FILE: &str = "routing.state.json";
const ROUTING_OWNERSHIP_FILE: &str = "routing.ownership.json";
const ROUTING_LOCK_FILE: &str = "routing.lock";

static ROUTING_STORE_MU: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct RoutingLease {
    pub version: i32,
    pub coordinator_nonce: String,
    pub generation: u64,
    pub lane_id: String,
    pub lane_instance_nonce: String,
    pub listener: String,
    pub lease_nonce: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct RoutingOwnershipFile {
    pub version: i32,
    pub coordinator_nonce: String,
    #[serde(default)]
    pub owner_nonce: String,
    pub expires_at: String,
    #[serde(default)]
    pub leases: Vec<RoutingLease>,
}

pub fn routing_state_path() -> String {
    Path::new(&tokless_data_dir())
        .join(ROUTING_STATE_FILE)
        .to_string_lossy()
        .to_string()
}

pub fn routing_ownership_path() -> String {
    Path::new(&tokless_data_dir())
        .join(ROUTING_OWNERSHIP_FILE)
        .to_string_lossy()
        .to_string()
}

pub fn routing_lease_path() -> String {
    routing_ownership_path()
}

pub fn claim_routing_coordinator(
    nonce: &str,
    owner: &str,
    ttl: Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    match read_routing_ownership_unlocked() {
        Ok(current) => {
            if coordinator_live(&current, Utc::now()) {
                return Err("routing coordinator is already owned".into());
            }
        }
        Err(err) => {
            if Path::new(&routing_ownership_path()).exists() {
                return Err(format!("read coordinator claim: {}", err).into());
            }
        }
    }
    let expires = (Utc::now() + chrono::Duration::from_std(ttl)?).to_rfc3339();
    let claim = RoutingOwnershipFile {
        version: ROUTING_STATE_VERSION,
        coordinator_nonce: nonce.to_string(),
        owner_nonce: owner.to_string(),
        expires_at: expires,
        leases: Vec::new(),
    };
    write_routing_ownership_unlocked(&claim)
}

pub fn read_routing_ownership_unlocked()
-> Result<RoutingOwnershipFile, Box<dyn std::error::Error + Send + Sync>> {
    let raw = std::fs::read_to_string(routing_ownership_path())?;
    validate_no_duplicate_json_keys(&raw)?;
    let claim: RoutingOwnershipFile =
        serde_json::from_str(&raw).map_err(|_| "invalid routing ownership")?;
    if claim.version != ROUTING_STATE_VERSION
        || (!claim.coordinator_nonce.is_empty() && !valid_routing_id(&claim.coordinator_nonce))
        || (!claim.owner_nonce.is_empty() && !valid_routing_token(&claim.owner_nonce))
    {
        return Err("invalid routing ownership".into());
    }
    DateTime::parse_from_rfc3339(&claim.expires_at)?;
    let mut seen_lanes = HashSet::with_capacity(claim.leases.len());
    let mut seen_nonces = HashSet::with_capacity(claim.leases.len());
    for lease in &claim.leases {
        validate_routing_lease(lease)?;
        if lease.coordinator_nonce != claim.coordinator_nonce {
            return Err("routing lease coordinator mismatch".into());
        }
        if !seen_lanes.insert(lease.lane_id.as_str()) {
            return Err("duplicate routing lease lane".into());
        }
        if !seen_nonces.insert(lease.lease_nonce.as_str()) {
            return Err("duplicate routing lease nonce".into());
        }
    }
    Ok(claim)
}

pub fn write_routing_ownership_unlocked(
    ownership: &RoutingOwnershipFile,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let b = serde_json::to_string(ownership)?;
    write_file_atomic(routing_ownership_path(), &b)?;
    Ok(())
}

pub fn coordinator_live(claim: &RoutingOwnershipFile, now: DateTime<Utc>) -> bool {
    if let Ok(expires) = DateTime::parse_from_rfc3339(&claim.expires_at) {
        return expires.with_timezone(&Utc) > now;
    }
    false
}

pub fn renew_routing_coordinator(
    nonce: &str,
    owner: &str,
    ttl: Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = renew_routing_ownership(nonce, owner, ttl, None)?;
    Ok(())
}

pub fn renew_routing_ownership(
    nonce: &str,
    owner: &str,
    ttl: Duration,
    expected: Option<&[RoutingLease]>,
) -> Result<Vec<RoutingLease>, Box<dyn std::error::Error + Send + Sync>> {
    if ttl.is_zero() {
        return Err("coordinator lease TTL must be positive".into());
    }
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    let mut claim =
        read_routing_ownership_unlocked().map_err(|_| "coordinator claim is not owned")?;
    if claim.coordinator_nonce != nonce || claim.owner_nonce != owner {
        return Err("coordinator claim is not owned".into());
    }
    if let Some(exp) = expected {
        let mut by_nonce = HashMap::new();
        for l in &claim.leases {
            by_nonce.insert(l.lease_nonce.as_str(), l);
        }
        for l in exp {
            match by_nonce.get(l.lease_nonce.as_str()) {
                Some(persisted) if same_routing_lease(persisted, l) => {}
                _ => return Err("routing lease is not owned".into()),
            }
        }
    }
    claim.expires_at = (Utc::now() + chrono::Duration::from_std(ttl)?).to_rfc3339();
    let expires = claim.expires_at.clone();
    for l in &mut claim.leases {
        l.expires_at = expires.clone();
    }
    write_routing_ownership_unlocked(&claim)?;
    Ok(claim.leases)
}

pub fn release_routing_coordinator(
    nonce: &str,
    owner: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    let path = routing_ownership_path();
    if !Path::new(&path).exists() {
        return Ok(());
    }
    let claim = read_routing_ownership_unlocked()?;
    if claim.coordinator_nonce != nonce || claim.owner_nonce != owner {
        return Err("coordinator claim is owned by another process".into());
    }
    if let Err(err) = std::fs::remove_file(&path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(err.into());
        }
    }
    Ok(())
}

pub fn routing_coordinator_owned(nonce: &str, owner: &str) -> bool {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    if let Ok(_file_guard) = acquire_routing_store_lock() {
        return coordinator_claim_matches_unlocked(nonce, owner);
    }
    false
}

pub(crate) fn coordinator_claim_matches_unlocked(nonce: &str, owner: &str) -> bool {
    if let Ok(claim) = read_routing_ownership_unlocked() {
        claim.coordinator_nonce == nonce
            && claim.owner_nonce == owner
            && coordinator_live(&claim, Utc::now())
    } else {
        false
    }
}

pub fn authorize_routing_dispatch_for_lane(
    mut selection: RouteSelection,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<RoutingLease, Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    let ownership =
        read_routing_ownership_unlocked().map_err(|_| "coordinator claim is not owned")?;
    if ownership.coordinator_nonce != coordinator_nonce
        || ownership.owner_nonce != owner_nonce
        || !coordinator_live(&ownership, Utc::now())
    {
        return Err("coordinator claim is not owned".into());
    }
    let state = read_routing_state_unlocked().map_err(|_| "routing state is not committed")?;
    if state.state != ROUTING_STATE_COMMITTED {
        return Err("routing state is not committed".into());
    }
    for lease in ownership.leases {
        if lease.lane_id != selection.lane_id {
            continue;
        }
        selection.lease_nonce = lease.lease_nonce.clone();
        if lease_live(&lease, Utc::now())
            && lease_matches_state(&lease, &state)
            && same_lease_selection(&lease, &selection)
        {
            return Ok(lease);
        }
        return Err("routing lease is unavailable".into());
    }
    Err("routing lease is unavailable".into())
}

pub fn read_routing_state() -> Result<RoutingState, Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    read_routing_state_unlocked()
}

fn read_routing_state_unlocked() -> Result<RoutingState, Box<dyn std::error::Error + Send + Sync>> {
    match std::fs::read_to_string(routing_state_path()) {
        Ok(raw) => parse_routing_state(&raw),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(RoutingState {
            state: ROUTING_STATE_ABSENT.to_string(),
            ..Default::default()
        }),
        Err(e) => Err(e.into()),
    }
}

pub fn publish_routing_state(
    next: RoutingState,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    if !coordinator_nonce.is_empty()
        && !coordinator_claim_matches_unlocked(coordinator_nonce, owner_nonce)
    {
        return Err("coordinator claim is not owned".into());
    }
    let prev = read_routing_state_unlocked()?;
    validate_routing_state_transition(&prev, &next)?;
    let content = encode_routing_state(&next)?;
    write_file_atomic(routing_state_path(), &content)?;
    Ok(())
}

pub fn create_routing_lease(
    state: &RoutingState,
    lane: &LaneBinding,
    ttl: Duration,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<RoutingLease, Box<dyn std::error::Error + Send + Sync>> {
    validate_routing_state(state).map_err(|e| format!("invalid routing state: {}", e))?;
    if state.state != ROUTING_STATE_COMMITTED || state.generation == 0 {
        return Err("routing lease requires committed state".into());
    }
    if lane.generation != state.generation
        || lane.coordinator_nonce != state.coordinator_nonce
        || !lane.ready
    {
        return Err("lane does not belong to committed routing state".into());
    }
    if ttl.is_zero() {
        return Err("routing lease TTL must be positive".into());
    }
    let nonce = new_routing_token()?;
    let expires = (Utc::now() + chrono::Duration::from_std(ttl)?).to_rfc3339();
    let lease = RoutingLease {
        version: ROUTING_STATE_VERSION,
        coordinator_nonce: state.coordinator_nonce.clone(),
        generation: state.generation,
        lane_id: lane.id.clone(),
        lane_instance_nonce: lane.instance_nonce.clone(),
        listener: lane.listener.clone(),
        lease_nonce: nonce,
        expires_at: expires,
    };
    validate_routing_lease(&lease)?;

    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    if !coordinator_nonce.is_empty()
        && !coordinator_claim_matches_unlocked(coordinator_nonce, owner_nonce)
    {
        return Err("coordinator claim is not owned".into());
    }
    let current_state =
        read_routing_state_unlocked().map_err(|e| format!("read routing state: {}", e))?;
    if !same_routing_state(&current_state, state) {
        return Err("routing state is not currently published".into());
    }
    let Some(current_lane) = routing_lane(&current_state, &lane.id) else {
        return Err("lane is not currently published".into());
    };
    if !same_lane_identity(current_lane, lane) {
        return Err("lane is not currently published".into());
    }

    let ownership_path = routing_ownership_path();
    let mut ownership = if Path::new(&ownership_path).exists() {
        read_routing_ownership_unlocked().map_err(|e| format!("read routing ownership: {}", e))?
    } else {
        RoutingOwnershipFile {
            version: ROUTING_STATE_VERSION,
            coordinator_nonce: state.coordinator_nonce.clone(),
            ..Default::default()
        }
    };

    if !coordinator_nonce.is_empty()
        && (ownership.coordinator_nonce != coordinator_nonce
            || ownership.owner_nonce != owner_nonce
            || !coordinator_live(&ownership, Utc::now()))
    {
        return Err("coordinator claim is not owned".into());
    }

    for existing in &ownership.leases {
        if existing.lane_id == lease.lane_id
            && existing.generation == lease.generation
            && existing.coordinator_nonce == lease.coordinator_nonce
            && lease_live(existing, Utc::now())
        {
            return Err("routing lease already owned".into());
        }
    }

    ownership.leases.retain(|existing| {
        lease_live(existing, Utc::now())
            && existing.generation == current_state.generation
            && existing.coordinator_nonce == current_state.coordinator_nonce
    });
    ownership.version = ROUTING_STATE_VERSION;
    ownership.coordinator_nonce = state.coordinator_nonce.clone();
    if ownership.expires_at.is_empty() {
        ownership.expires_at = lease.expires_at.clone();
    }
    ownership.leases.push(lease.clone());
    write_routing_ownership_unlocked(&ownership)?;
    Ok(lease)
}

pub fn read_routing_lease(
    lane_id: Option<&str>,
) -> Result<RoutingLease, Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    let ownership = read_routing_ownership_unlocked()?;
    for lease in ownership.leases {
        if lane_id.is_none() || lane_id == Some(lease.lane_id.as_str()) {
            return Ok(lease);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "routing lease not found").into())
}

pub fn renew_routing_lease(
    lease: &RoutingLease,
    ttl: Duration,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<RoutingLease, Box<dyn std::error::Error + Send + Sync>> {
    if ttl.is_zero() {
        return Err("routing lease TTL must be positive".into());
    }
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    if !coordinator_nonce.is_empty()
        && !coordinator_claim_matches_unlocked(coordinator_nonce, owner_nonce)
    {
        return Err("coordinator claim is not owned".into());
    }
    let state = read_routing_state_unlocked().map_err(|_| "routing lease is not owned")?;
    let mut ownership =
        read_routing_ownership_unlocked().map_err(|_| "routing lease is not owned")?;
    let Some(pos) = ownership
        .leases
        .iter()
        .position(|c| c.lease_nonce == lease.lease_nonce)
    else {
        return Err("routing lease is not owned".into());
    };
    let mut current = ownership.leases[pos].clone();
    if !lease_live(&current, Utc::now()) || !lease_matches_state(&current, &state) {
        return Err("routing lease is not owned".into());
    }
    if !coordinator_nonce.is_empty()
        && (ownership.coordinator_nonce != coordinator_nonce
            || ownership.owner_nonce != owner_nonce)
    {
        return Err("coordinator claim is not owned".into());
    }
    current.expires_at = (Utc::now() + chrono::Duration::from_std(ttl)?).to_rfc3339();
    ownership.leases[pos] = current.clone();
    write_routing_ownership_unlocked(&ownership)?;
    Ok(current)
}

pub fn renew_routing_leases(
    leases: &[RoutingLease],
    ttl: Duration,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<Vec<RoutingLease>, Box<dyn std::error::Error + Send + Sync>> {
    renew_routing_ownership(coordinator_nonce, owner_nonce, ttl, Some(leases))
}

pub fn release_routing_leases(
    leases: &[RoutingLease],
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    let path = routing_ownership_path();
    if !Path::new(&path).exists() {
        return Ok(());
    }
    let mut ownership = read_routing_ownership_unlocked()?;
    if ownership.coordinator_nonce != coordinator_nonce || ownership.owner_nonce != owner_nonce {
        return Err("coordinator claim is not owned".into());
    }
    for l in leases {
        if let Some(pos) = ownership
            .leases
            .iter()
            .position(|persisted| same_routing_lease(persisted, l))
        {
            ownership.leases.remove(pos);
        }
    }
    write_routing_ownership_unlocked(&ownership)
}

pub fn release_routing_lease(
    lease: &RoutingLease,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    if !coordinator_nonce.is_empty()
        && !coordinator_claim_matches_unlocked(coordinator_nonce, owner_nonce)
    {
        return Err("coordinator claim is not owned".into());
    }
    let path = routing_ownership_path();
    if !Path::new(&path).exists() {
        return Ok(());
    }
    let mut ownership = read_routing_ownership_unlocked()?;
    let Some(pos) = ownership
        .leases
        .iter()
        .position(|c| c.lease_nonce == lease.lease_nonce)
    else {
        return Err("routing lease is owned by another coordinator".into());
    };
    if !same_routing_lease(&ownership.leases[pos], lease) {
        return Err("routing lease is owned by another coordinator".into());
    }
    ownership.leases.remove(pos);
    write_routing_ownership_unlocked(&ownership)
}

pub fn validate_routing_lease(
    lease: &RoutingLease,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if lease.version != ROUTING_STATE_VERSION
        || !valid_routing_id(&lease.coordinator_nonce)
        || lease.generation == 0
        || !valid_routing_id(&lease.lane_id)
        || !valid_routing_id(&lease.lane_instance_nonce)
        || !valid_routing_token(&lease.lease_nonce)
        || lease.listener.is_empty()
    {
        return Err("invalid routing lease".into());
    }
    if DateTime::parse_from_rfc3339(&lease.expires_at).is_err() {
        return Err("invalid routing lease expiry".into());
    }
    let port = listener_port(&lease.listener);
    if port <= 0 || !valid_loopback_listener(&lease.listener, port) {
        return Err("invalid routing lease listener".into());
    }
    Ok(())
}

pub fn lease_live(lease: &RoutingLease, now: DateTime<Utc>) -> bool {
    if let Ok(exp) = DateTime::parse_from_rfc3339(&lease.expires_at) {
        return exp.with_timezone(&Utc) > now;
    }
    false
}

pub fn listener_port(listener: &str) -> i32 {
    if let Some((_, p_str)) = listener.rsplit_once(':') {
        if let Ok(p) = p_str.parse::<i32>() {
            return p;
        }
    }
    0
}

pub fn routing_lane<'a>(state: &'a RoutingState, id: &str) -> Option<&'a LaneBinding> {
    state.lanes.iter().find(|l| l.id == id)
}

pub fn same_routing_state_identity(a: &RoutingState, b: &RoutingState) -> bool {
    a.state == b.state
        && a.generation == b.generation
        && a.coordinator_nonce == b.coordinator_nonce
        && a.commit_nonce == b.commit_nonce
}

pub fn same_routing_state(a: &RoutingState, b: &RoutingState) -> bool {
    if !same_routing_state_identity(a, b) || a.previous_generation != b.previous_generation {
        return false;
    }
    match (encode_routing_state(a), encode_routing_state(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

pub fn same_routing_lease(a: &RoutingLease, b: &RoutingLease) -> bool {
    a == b
}

pub fn same_lane_identity(a: &LaneBinding, b: &LaneBinding) -> bool {
    a == b
}

pub fn lease_matches_state(lease: &RoutingLease, state: &RoutingState) -> bool {
    if let Some(lane) = routing_lane(state, &lease.lane_id) {
        return state.state == ROUTING_STATE_COMMITTED
            && lease.generation == state.generation
            && lease.coordinator_nonce == state.coordinator_nonce
            && lane.instance_nonce == lease.lane_instance_nonce
            && lane.listener == lease.listener
            && lane.ready;
    }
    false
}

pub fn same_lease_selection(lease: &RoutingLease, selection: &RouteSelection) -> bool {
    lease.coordinator_nonce == selection.coordinator_nonce
        && lease.generation == selection.generation
        && lease.lane_id == selection.lane_id
        && lease.lane_instance_nonce == selection.lane_instance_nonce
        && lease.listener == selection.listener
        && lease.lease_nonce == selection.lease_nonce
}

fn acquire_routing_store_lock()
-> Result<super::routing_lock::FileLockGuard, Box<dyn std::error::Error + Send + Sync>> {
    ensure_dir(tokless_data_dir())?;
    let path = Path::new(&tokless_data_dir()).join(ROUTING_LOCK_FILE);
    acquire_routing_file_lock(path)
}
