use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use chrono::DateTime;
use tokio::sync::watch;

use super::routing_plan::{RouteRequest, RouteSelection, RoutingSnapshot};
use super::routing_state::{new_routing_token, LaneBinding, RoutingState};
use super::routing_store::{
    claim_routing_coordinator, create_routing_lease, lease_live, lease_matches_state,
    read_routing_state, release_routing_coordinator, release_routing_lease,
    release_routing_leases, renew_routing_ownership, routing_coordinator_owned,
    same_routing_lease, RoutingLease,
};

fn lease_expires_after(a: &RoutingLease, b: &RoutingLease) -> bool {
    match (
        DateTime::parse_from_rfc3339(&a.expires_at),
        DateTime::parse_from_rfc3339(&b.expires_at),
    ) {
        (Ok(ea), Ok(eb)) => ea > eb,
        _ => false,
    }
}

pub struct RoutingCoordinator {
    pub nonce: String,
    pub owner: String,
    pub lease_ttl: Duration,
    leases: Arc<RwLock<HashMap<String, RoutingLease>>>,
    snapshot: Arc<RwLock<Option<RoutingSnapshot>>>,
    active: Arc<AtomicBool>,
    shutdown_tx: watch::Sender<bool>,
}

impl RoutingCoordinator {
    pub async fn new(ttl: Duration) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        if ttl.is_zero() {
            return Err("coordinator lease TTL must be positive".into());
        }
        let state = read_routing_state()?;
        let nonce = if !state.coordinator_nonce.is_empty() {
            state.coordinator_nonce.clone()
        } else {
            new_routing_token()?
        };
        let owner = new_routing_token()?;
        let snapshot = RoutingSnapshot::new(state)?;

        claim_routing_coordinator(&nonce, &owner, ttl)?;

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let active = Arc::new(AtomicBool::new(true));
        let leases = Arc::new(RwLock::new(HashMap::new()));
        let snapshot_arc = Arc::new(RwLock::new(Some(snapshot)));

        let coord = Arc::new(RoutingCoordinator {
            nonce: nonce.clone(),
            owner: owner.clone(),
            lease_ttl: ttl,
            leases: leases.clone(),
            snapshot: snapshot_arc,
            active: active.clone(),
            shutdown_tx,
        });

        let heartbeat_active = active.clone();
        let heartbeat_leases = leases.clone();
        let heartbeat_nonce = nonce.clone();
        let heartbeat_owner = owner.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::cmp::max(ttl / 3, Duration::from_millis(1)));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !heartbeat_active.load(Ordering::Relaxed) {
                            return;
                        }
                        let current_leases: Vec<RoutingLease> = {
                            let guard = heartbeat_leases.read().unwrap();
                            guard.values().cloned().collect()
                        };
                        match renew_routing_ownership(&heartbeat_nonce, &heartbeat_owner, ttl, Some(&current_leases)) {
                            Ok(renewed) => {
                                let mut guard = heartbeat_leases.write().unwrap();
                                for l in renewed {
                                    guard.insert(l.lane_id.clone(), l);
                                }
                            }
                            Err(_) => {
                                if !routing_coordinator_owned(&heartbeat_nonce, &heartbeat_owner) {
                                    heartbeat_active.store(false, Ordering::Relaxed);
                                    return;
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        return;
                    }
                }
            }
        });

        Ok(coord)
    }

    pub fn nonce(&self) -> &str {
        &self.nonce
    }

    pub fn current(&self) -> Option<RoutingSnapshot> {
        let guard = self.snapshot.read().unwrap();
        guard.clone()
    }

    pub fn publish(
        &self,
        state: RoutingState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.active.load(Ordering::Relaxed) {
            return Err("coordinator is fenced".into());
        }
        let snap = RoutingSnapshot::new(state.clone())?;
        if state.coordinator_nonce != self.nonce {
            return Err("routing state coordinator mismatch".into());
        }
        super::routing_store::publish_routing_state(state.clone(), &self.nonce, &self.owner)?;
        {
            let mut l_guard = self.leases.write().unwrap();
            l_guard.retain(|_, v| v.generation == state.generation);
        }
        {
            let mut s_guard = self.snapshot.write().unwrap();
            *s_guard = Some(snap);
        }
        Ok(())
    }

    pub fn authorize(
        &self,
        selection: &RouteSelection,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.active.load(Ordering::Relaxed) {
            return Err("coordinator is fenced".into());
        }
        let snap = self.current().ok_or("routing snapshot unavailable")?;
        if selection.generation != snap.generation() {
            return Err("routing lease is stale".into());
        }
        let guard = self.leases.read().unwrap();
        let lease = guard
            .get(&selection.lane_id)
            .ok_or("routing lease is unavailable")?;
        if lease.coordinator_nonce != self.nonce
            || lease.generation != selection.generation
            || lease.listener != selection.listener
            || lease.lane_instance_nonce != selection.lane_instance_nonce
            || lease.lease_nonce != selection.lease_nonce
            || !lease_live(lease, chrono::Utc::now())
            || !lease_matches_state(lease, &snap.state)
        {
            return Err("routing lease is unavailable".into());
        }
        if !self.active.load(Ordering::Relaxed) {
            return Err("coordinator is fenced".into());
        }
        Ok(())
    }

    pub fn authorize_request(
        &self,
        request: RouteRequest,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _ = self.resolve_and_authorize(request)?;
        Ok(())
    }

    pub fn resolve_and_authorize(
        &self,
        request: RouteRequest,
    ) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>> {
        let snap = self.current().ok_or("routing snapshot unavailable")?;
        let mut selection = snap.resolve_request(request, |sel| {
            let guard = self.leases.read().unwrap();
            if let Some(lease) = guard.get(&sel.lane_id) {
                lease.coordinator_nonce == self.nonce
                    && lease.generation == sel.generation
                    && lease.listener == sel.listener
                    && lease_live(lease, chrono::Utc::now())
            } else {
                false
            }
        })?;
        {
            let guard = self.leases.read().unwrap();
            if let Some(lease) = guard.get(&selection.lane_id) {
                selection.lease_nonce = lease.lease_nonce.clone();
            }
        }
        self.authorize(&selection)?;
        Ok(selection)
    }

    pub fn acquire(
        &self,
        state: &RoutingState,
        lane: &LaneBinding,
    ) -> Result<RoutingLease, Box<dyn std::error::Error + Send + Sync>> {
        if !self.active.load(Ordering::Relaxed) {
            return Err("coordinator is fenced".into());
        }
        if state.coordinator_nonce != self.nonce {
            return Err("routing state coordinator mismatch".into());
        }
        let lease =
            create_routing_lease(state, lane, self.lease_ttl, &self.nonce, &self.owner)?;
        let mut guard = self.leases.write().unwrap();
        let should_insert = match guard.get(&lease.lane_id) {
            Some(existing) => lease_expires_after(&lease, existing),
            None => true,
        };
        if should_insert {
            guard.insert(lease.lane_id.clone(), lease.clone());
        }
        Ok(lease)
    }

    pub fn renew(
        &self,
        lease: &RoutingLease,
    ) -> Result<RoutingLease, Box<dyn std::error::Error + Send + Sync>> {
        if !self.active.load(Ordering::Relaxed) {
            return Err("coordinator is fenced".into());
        }
        {
            let guard = self.leases.read().unwrap();
            match guard.get(&lease.lane_id) {
                Some(owned)
                    if same_routing_lease(owned, lease)
                        && lease.coordinator_nonce == self.nonce => {}
                _ => return Err("coordinator does not own lease".into()),
            }
        }
        let renewed = renew_routing_ownership(
            &self.nonce,
            &self.owner,
            self.lease_ttl,
            Some(std::slice::from_ref(lease)),
        )?;
        for current in renewed {
            if current.lane_id == lease.lane_id {
                let mut guard = self.leases.write().unwrap();
                let should_insert = match guard.get(&lease.lane_id) {
                    Some(existing) => lease_expires_after(&current, existing),
                    None => true,
                };
                if should_insert {
                    guard.insert(lease.lane_id.clone(), current.clone());
                }
                return Ok(current);
            }
        }
        Err("routing lease is unavailable".into())
    }

    pub fn release(
        &self,
        lease: &RoutingLease,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.active.load(Ordering::Relaxed) {
            return Err("coordinator is fenced".into());
        }
        {
            let guard = self.leases.read().unwrap();
            match guard.get(&lease.lane_id) {
                Some(owned)
                    if same_routing_lease(owned, lease)
                        && lease.coordinator_nonce == self.nonce => {}
                _ => return Err("coordinator does not own lease".into()),
            }
        }
        release_routing_lease(lease, &self.nonce, &self.owner)?;
        let mut guard = self.leases.write().unwrap();
        guard.remove(&lease.lane_id);
        Ok(())
    }

    pub fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.active.store(false, Ordering::Relaxed);
        let _ = self.shutdown_tx.send(true);
        let current_leases: Vec<RoutingLease> = {
            let mut guard = self.leases.write().unwrap();
            let list = guard.values().cloned().collect();
            guard.clear();
            list
        };
        let l_res = release_routing_leases(&current_leases, &self.nonce, &self.owner);
        let c_res = release_routing_coordinator(&self.nonce, &self.owner);
        l_res.and(c_res)
    }
}
