use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::paths::{ensure_dir, tokless_data_dir, write_file_atomic};
use super::routing_lock::acquire_routing_file_lock;
use super::routing_state::{
    new_routing_token, parse_routing_state, valid_loopback_listener, valid_routing_id,
    valid_routing_token, validate_routing_state_transition, LaneBinding, RoutingState,
    ROUTING_STATE_ABSENT, ROUTING_STATE_COMMITTED, ROUTING_STATE_VERSION,
};

const ROUTING_STATE_FILE: &str = "routing.state.json";
const ROUTING_OWNERSHIP_FILE: &str = "routing.ownership.json";
const ROUTING_LOCK_FILE: &str = "routing.lock";

static ROUTING_STORE_MU: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
pub struct RoutingOwnershipFile {
    pub version: i32,
    pub coordinator_nonce: String,
    pub owner_nonce: String,
    pub expires_at: String,
    #[serde(default)]
    pub leases: Vec<RoutingLease>,
}

pub fn routing_state_path() -> String {
    Path::new(&tokless_data_dir()).join(ROUTING_STATE_FILE).to_string_lossy().to_string()
}

pub fn routing_ownership_path() -> String {
    Path::new(&tokless_data_dir()).join(ROUTING_OWNERSHIP_FILE).to_string_lossy().to_string()
}

pub fn routing_lease_path() -> String {
    routing_ownership_path()
}

pub fn claim_routing_coordinator(nonce: &str, owner: &str, ttl: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    if let Ok(current) = read_routing_ownership_unlocked() {
        if coordinator_live(&current, Utc::now()) {
            return Err("routing coordinator is already owned".into());
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

pub fn read_routing_ownership_unlocked() -> Result<RoutingOwnershipFile, Box<dyn std::error::Error + Send + Sync>> {
    let raw = std::fs::read_to_string(routing_ownership_path())?;
    let claim: RoutingOwnershipFile = serde_json::from_str(&raw)?;
    if claim.version != ROUTING_STATE_VERSION {
        return Err("invalid routing ownership version".into());
    }
    if !claim.coordinator_nonce.is_empty() && !valid_routing_id(&claim.coordinator_nonce) {
        return Err("invalid routing ownership coordinator".into());
    }
    if !claim.owner_nonce.is_empty() && !valid_routing_token(&claim.owner_nonce) {
        return Err("invalid routing ownership owner".into());
    }
    for lease in &claim.leases {
        validate_routing_lease(lease)?;
        if lease.coordinator_nonce != claim.coordinator_nonce {
            return Err("routing lease coordinator mismatch".into());
        }
    }
    Ok(claim)
}

pub fn write_routing_ownership_unlocked(ownership: &RoutingOwnershipFile) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let mut claim = read_routing_ownership_unlocked()?;
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
                Some(persisted) if *persisted == l => {}
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

pub fn release_routing_coordinator(nonce: &str, owner: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    match read_routing_ownership_unlocked() {
        Ok(claim) => {
            if claim.coordinator_nonce != nonce || claim.owner_nonce != owner {
                return Err("coordinator claim is owned by another process".into());
            }
            let _ = std::fs::remove_file(routing_ownership_path());
        }
        Err(_) => return Ok(()),
    }
    Ok(())
}

pub fn routing_coordinator_owned(nonce: &str, owner: &str) -> bool {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    if let Ok(_file_guard) = acquire_routing_store_lock() {
        if let Ok(claim) = read_routing_ownership_unlocked() {
            return claim.coordinator_nonce == nonce
                && claim.owner_nonce == owner
                && coordinator_live(&claim, Utc::now());
        }
    }
    false
}

pub fn read_routing_state() -> Result<RoutingState, Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
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
    if !coordinator_nonce.is_empty() {
        let claim = read_routing_ownership_unlocked()?;
        if claim.coordinator_nonce != coordinator_nonce || claim.owner_nonce != owner_nonce || !coordinator_live(&claim, Utc::now()) {
            return Err("coordinator claim is not owned".into());
        }
    }
    let prev = match std::fs::read_to_string(routing_state_path()) {
        Ok(raw) => parse_routing_state(&raw)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => RoutingState {
            state: ROUTING_STATE_ABSENT.to_string(),
            ..Default::default()
        },
        Err(e) => return Err(e.into()),
    };
    validate_routing_state_transition(&prev, &next)?;
    let content = serde_json::to_string(&next)?;
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
    if state.state != ROUTING_STATE_COMMITTED || state.generation == 0 {
        return Err("routing lease requires committed state".into());
    }
    if lane.generation != state.generation || lane.coordinator_nonce != state.coordinator_nonce || !lane.ready {
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
    let mut ownership = read_routing_ownership_unlocked().unwrap_or(RoutingOwnershipFile {
        version: ROUTING_STATE_VERSION,
        coordinator_nonce: state.coordinator_nonce.clone(),
        ..Default::default()
    });
    if !coordinator_nonce.is_empty()
        && (ownership.coordinator_nonce != coordinator_nonce
            || ownership.owner_nonce != owner_nonce
            || !coordinator_live(&ownership, Utc::now()))
    {
        return Err("coordinator claim is not owned".into());
    }
    ownership.leases.retain(|existing| {
        lease_live(existing, Utc::now())
            && existing.generation == state.generation
            && existing.coordinator_nonce == state.coordinator_nonce
    });
    ownership.leases.push(lease.clone());
    write_routing_ownership_unlocked(&ownership)?;
    Ok(lease)
}

pub fn release_routing_leases(
    leases: &[RoutingLease],
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _mem_guard = ROUTING_STORE_MU.lock().unwrap();
    let _file_guard = acquire_routing_store_lock()?;
    let mut ownership = match read_routing_ownership_unlocked() {
        Ok(o) => o,
        Err(_) => return Ok(()),
    };
    if ownership.coordinator_nonce != coordinator_nonce || ownership.owner_nonce != owner_nonce {
        return Err("coordinator claim is not owned".into());
    }
    for l in leases {
        ownership.leases.retain(|persisted| persisted.lease_nonce != l.lease_nonce);
    }
    write_routing_ownership_unlocked(&ownership)
}

pub fn release_routing_lease(
    lease: &RoutingLease,
    coordinator_nonce: &str,
    owner_nonce: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    release_routing_leases(&[lease.clone()], coordinator_nonce, owner_nonce)
}

pub fn validate_routing_lease(lease: &RoutingLease) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

pub fn lease_matches_state(lease: &RoutingLease, state: &RoutingState) -> bool {
    if let Some(lane) = state.lanes.iter().find(|l| l.id == lease.lane_id) {
        return state.state == ROUTING_STATE_COMMITTED
            && lease.generation == state.generation
            && lease.coordinator_nonce == state.coordinator_nonce
            && lane.instance_nonce == lease.lane_instance_nonce
            && lane.listener == lease.listener
            && lane.ready;
    }
    false
}

fn acquire_routing_store_lock() -> Result<super::routing_lock::FileLockGuard, Box<dyn std::error::Error + Send + Sync>> {
    ensure_dir(tokless_data_dir())?;
    let path = Path::new(&tokless_data_dir()).join(ROUTING_LOCK_FILE);
    acquire_routing_file_lock(path)
}
