// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/reconnect.rs
// created: 2026-11-09
// lastModified: 2026-11-16

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::util::logger::L;

/// Safe atomic-swap container for lock-free-style client pool replacement (`#![deny(unsafe_code)]`).
pub struct ArcSwap<T> {
    inner: RwLock<Arc<T>>,
}

impl<T> ArcSwap<T> {
    pub fn new(val: Arc<T>) -> Self {
        Self {
            inner: RwLock::new(val),
        }
    }

    pub fn load(&self) -> Arc<T> {
        self.inner
            .read()
            .map(|g| Arc::clone(&g))
            .unwrap_or_else(|poisoned| Arc::clone(&poisoned.into_inner()))
    }

    pub fn store(&self, val: Arc<T>) {
        if let Ok(mut g) = self.inner.write() {
            *g = val;
        }
    }

    pub fn swap(&self, val: Arc<T>) -> Arc<T> {
        match self.inner.write() {
            Ok(mut g) => std::mem::replace(&mut *g, val),
            Err(poisoned) => {
                let mut g = poisoned.into_inner();
                std::mem::replace(&mut *g, val)
            }
        }
    }
}

/// Domain errors for network healing and reconnection watchdog.
#[derive(Debug, thiserror::Error)]
pub enum NetworkHealingError {
    #[error("HTTP client initialization failed: {0}")]
    ClientBuildError(#[from] reqwest::Error),
    #[error("Network connection lock poisoned")]
    LockPoisoned,
    #[error("Socket probe timeout: {0}")]
    ProbeTimeout(String),
    #[error("IO error during network health check: {0}")]
    IoError(#[from] std::io::Error),
}

/// Operational status of the upstream network connectivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkStatus {
    Healthy,
    Degraded,
    Disconnected,
    Reconnecting,
}

/// Auto-healing client pool and network reconnection watchdog.
///
/// Automatically monitors upstream TCP transport, detects connection drops,
/// VPN disconnects, DNS invalidation, OS sleep/wake transitions, or network interface toggles,
/// and atomically swaps `ArcSwap<reqwest::Client>` pools to prevent dormant lockout states.
pub struct AutoHealingClientPool {
    generation: AtomicU64,
    consecutive_transport_failures: AtomicU32,
    total_reconnect_count: AtomicU64,
    last_reconnect_epoch_secs: AtomicU64,
    last_wall_epoch_secs: AtomicU64,
    start_instant: Instant,
    last_monotonic_secs: AtomicU64,
    status: RwLock<NetworkStatus>,
    async_client: ArcSwap<reqwest::Client>,
    loopback_async_client: ArcSwap<reqwest::Client>,
    blocking_client: RwLock<Option<reqwest::blocking::Client>>,
    last_known_interface_hash: AtomicU64,
}

impl AutoHealingClientPool {
    /// Creates a new auto-healing client pool configured with aggressive socket keep-alive
    /// and short idle pool TTLs to eliminate hanging connections.
    pub fn new() -> Result<Self, NetworkHealingError> {
        let async_client = Self::build_async_client()?;
        let loopback_async_client = Self::build_loopback_async_client()?;
        let initial_hash = Self::compute_network_interface_signature();

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Self {
            generation: AtomicU64::new(1),
            consecutive_transport_failures: AtomicU32::new(0),
            total_reconnect_count: AtomicU64::new(0),
            last_reconnect_epoch_secs: AtomicU64::new(now_secs),
            last_wall_epoch_secs: AtomicU64::new(now_secs),
            start_instant: Instant::now(),
            last_monotonic_secs: AtomicU64::new(0),
            status: RwLock::new(NetworkStatus::Healthy),
            async_client: ArcSwap::new(Arc::new(async_client)),
            loopback_async_client: ArcSwap::new(Arc::new(loopback_async_client)),
            blocking_client: RwLock::new(None),
            last_known_interface_hash: AtomicU64::new(initial_hash),
        })
    }

    /// Builds a fresh `reqwest::Client` with bounded idle socket timeouts and keep-alives.
    fn build_async_client() -> Result<reqwest::Client, NetworkHealingError> {
        reqwest::Client::builder()
            .pool_idle_timeout(Some(Duration::from_secs(15)))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(NetworkHealingError::from)
    }

    /// Builds a fresh loopback `reqwest::Client` bypassing `HTTP_PROXY`/`HTTPS_PROXY`.
    fn build_loopback_async_client() -> Result<reqwest::Client, NetworkHealingError> {
        reqwest::Client::builder()
            .no_proxy()
            .pool_idle_timeout(Some(Duration::from_secs(15)))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(NetworkHealingError::from)
    }

    /// Builds a fresh `reqwest::blocking::Client` with bounded idle socket timeouts and keep-alives.
    fn build_blocking_client() -> Result<reqwest::blocking::Client, NetworkHealingError> {
        reqwest::blocking::Client::builder()
            .pool_idle_timeout(Some(Duration::from_secs(15)))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(NetworkHealingError::from)
    }

    /// Computes a hash signature of the current primary network interface / route.
    /// Uses UDP socket connect without sending data to determine local routing path.
    pub fn compute_network_interface_signature() -> u64 {
        let mut hasher = DefaultHasher::new();

        // Query local address resolved for public DNS route
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0")
            && socket.connect("1.1.1.1:53").is_ok()
            && let Ok(local_addr) = socket.local_addr()
        {
            local_addr.hash(&mut hasher);
        }

        // Include hostname hash
        if let Ok(h) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
            h.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Returns the current client pool generation counter.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Returns total auto-healing reconnect events performed.
    pub fn total_reconnect_count(&self) -> u64 {
        self.total_reconnect_count.load(Ordering::Relaxed)
    }

    /// Returns the current network status.
    pub fn status(&self) -> NetworkStatus {
        self.status
            .read()
            .map(|s| *s)
            .unwrap_or(NetworkStatus::Healthy)
    }

    /// Returns a cloned handle to the active `reqwest::Client`.
    pub fn get_async_client(&self) -> Result<reqwest::Client, NetworkHealingError> {
        Ok((*self.async_client.load()).clone())
    }

    /// Returns a cloned handle to the active loopback `reqwest::Client` (bypasses HTTP_PROXY).
    pub fn get_loopback_async_client(&self) -> Result<reqwest::Client, NetworkHealingError> {
        Ok((*self.loopback_async_client.load()).clone())
    }

    /// Returns a cloned handle to the active `reqwest::blocking::Client`.
    pub fn get_blocking_client(&self) -> Result<reqwest::blocking::Client, NetworkHealingError> {
        if let Some(existing) = self
            .blocking_client
            .read()
            .map_err(|_| NetworkHealingError::LockPoisoned)?
            .clone()
        {
            return Ok(existing);
        }

        let built = if tokio::runtime::Handle::try_current().is_ok() {
            std::thread::spawn(Self::build_blocking_client)
                .join()
                .map_err(|_| NetworkHealingError::LockPoisoned)??
        } else {
            Self::build_blocking_client()?
        };

        let mut lock = self
            .blocking_client
            .write()
            .map_err(|_| NetworkHealingError::LockPoisoned)?;
        if let Some(ref existing) = *lock {
            return Ok(existing.clone());
        }
        *lock = Some(built.clone());
        Ok(built)
    }

    /// Invalidates stale connection pools, reconstructs fresh HTTP clients with clean DNS,
    /// and resets consecutive failure counters without requiring process restarts.
    pub fn invalidate_and_reconnect(&self, reason: &str) -> Result<u64, NetworkHealingError> {
        {
            if let Ok(mut st) = self.status.write() {
                *st = NetworkStatus::Reconnecting;
            }
        }

        let new_async = Self::build_async_client()?;
        let new_loopback = Self::build_loopback_async_client()?;

        let _old_async = self.async_client.swap(Arc::new(new_async));
        let _old_loopback = self.loopback_async_client.swap(Arc::new(new_loopback));

        {
            let mut blocking_lock = self.blocking_client.write().map_err(|_| NetworkHealingError::LockPoisoned)?;
            if let Some(old_blocking) = blocking_lock.take() {
                if tokio::runtime::Handle::try_current().is_ok() {
                    std::thread::spawn(move || drop(old_blocking));
                } else {
                    drop(old_blocking);
                }
            }
        }

        let new_gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.total_reconnect_count.fetch_add(1, Ordering::Relaxed);
        self.consecutive_transport_failures.store(0, Ordering::Relaxed);

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_reconnect_epoch_secs.store(now_secs, Ordering::Relaxed);

        {
            if let Ok(mut st) = self.status.write() {
                *st = NetworkStatus::Healthy;
            }
        }

        L.info(&format!(
            "[HEAL] Network reconnect auto-healing triggered: {}. Refreshed upstream connection pools (generation {}).",
            reason, new_gen
        ));

        Ok(new_gen)
    }

    /// Evaluates whether an error description indicates connection reset, broken pipe,
    /// DNS failure, interface drop, or socket timeout.
    pub fn is_network_reset_error(err_str: &str) -> bool {
        let s = err_str.to_ascii_lowercase();
        s.contains("reset by peer")
            || s.contains("broken pipe")
            || s.contains("connection refused")
            || s.contains("connection reset")
            || s.contains("connection closed")
            || s.contains("connection aborted")
            || s.contains("network is down")
            || s.contains("network is unreachable")
            || s.contains("host is unreachable")
            || s.contains("timed out")
            || s.contains("timeout")
            || s.contains("dns error")
            || s.contains("failed to lookup address")
            || s.contains("name resolution")
            || s.contains("unexpected eof")
            || s.contains("pool closed")
            || s.contains("dispatch task is gone")
    }

    /// Reports a transport error to the watchdog. If the error indicates a dead socket
    /// or connection reset, immediately triggers pool invalidation and self-healing.
    pub fn report_transport_error(&self, error_desc: &str) -> bool {
        if Self::is_network_reset_error(error_desc) {
            let failures = self.consecutive_transport_failures.fetch_add(1, Ordering::Relaxed) + 1;
            {
                if let Ok(mut st) = self.status.write() {
                    *st = NetworkStatus::Degraded;
                }
            }

            // Immediately heal on transport drop/reset
            if failures >= 1 {
                let _ = self.invalidate_and_reconnect(error_desc);
                return true;
            }
        }
        false
    }

    /// Probes reachability against a lightweight upstream TCP target.
    pub fn probe_network_connectivity(&self, target_host_port: &str, timeout: Duration) -> bool {
        let addrs = match target_host_port.to_socket_addrs() {
            Ok(iter) => iter.collect::<Vec<SocketAddr>>(),
            Err(_) => return false,
        };

        for addr in addrs {
            if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                return true;
            }
        }

        false
    }

    /// Checks if the local network interface signature has changed (e.g. Wi-Fi <-> VPN switch).
    /// If changed, automatically invalidates stale connection pools.
    pub fn check_and_heal_interface_change(&self) -> bool {
        let current_hash = Self::compute_network_interface_signature();
        let old_hash = self.last_known_interface_hash.load(Ordering::Relaxed);

        if current_hash != old_hash && current_hash != 0 {
            self.last_known_interface_hash.store(current_hash, Ordering::Relaxed);
            let _ = self.invalidate_and_reconnect("network interface toggle or route change detected");
            return true;
        }

        false
    }

    /// Detects OS sleep/wake transitions by comparing wall-clock jump against monotonic elapsed time.
    pub fn check_and_heal_sleep_wake(&self) -> bool {
        let now_wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let prev_wall = self.last_wall_epoch_secs.swap(now_wall, Ordering::Relaxed);
        let now_mono = self.start_instant.elapsed().as_secs();
        let prev_mono = self.last_monotonic_secs.swap(now_mono, Ordering::Relaxed);

        let wall_delta = now_wall.saturating_sub(prev_wall);
        let mono_delta = now_mono.saturating_sub(prev_mono);
        if wall_delta > mono_delta.saturating_add(5) {
            let _ = self.invalidate_and_reconnect("OS sleep/wake clock jump detected");
            return true;
        }
        false
    }

    /// Preserves HTTP/2 ConnectRPC framing & streaming headers cleanly (`#0033`/`#0034`)
    /// while stripping HTTP/1.1 hop-by-hop headers that corrupt multiplexed streams.
    pub fn build_connectrpc_safe_headers(headers: &[(String, String)]) -> Vec<(String, String)> {
        headers
            .iter()
            .filter(|(k, _)| {
                let lower = k.to_ascii_lowercase();
                !matches!(
                    lower.as_str(),
                    "connection" | "keep-alive" | "transfer-encoding" | "upgrade" | "http2-settings" | "proxy-connection"
                )
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Spawns a background asynchronous watchdog loop that periodically inspects
    /// sleep/wake transitions, interface changes, and connection health.
    pub fn start_watchdog_loop(self: Arc<Self>, probe_interval: Duration) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(probe_interval);
            loop {
                interval.tick().await;

                // 1. Check OS sleep/wake transitions
                self.check_and_heal_sleep_wake();

                // 2. Check network interface toggles
                self.check_and_heal_interface_change();

                // 3. If status was degraded, probe and restore if healthy
                if self.status() == NetworkStatus::Degraded
                    && (self.probe_network_connectivity("1.1.1.1:53", Duration::from_secs(1))
                        || self.probe_network_connectivity("8.8.8.8:53", Duration::from_secs(1)))
                {
                    let _ = self.invalidate_and_reconnect("network connectivity restored after degradation");
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Global Singleton & Helper Functions
// ---------------------------------------------------------------------------

static GLOBAL_HEALING_POOL: OnceLock<AutoHealingClientPool> = OnceLock::new();

/// Returns a reference to the global `AutoHealingClientPool` singleton.
pub fn global_healing_pool() -> &'static AutoHealingClientPool {
    GLOBAL_HEALING_POOL.get_or_init(|| {
        AutoHealingClientPool::new().unwrap_or_else(|e| {
            eprintln!("[WARN] AutoHealingClientPool fallback init: {}", e);
            // Fallback pool creation
            let async_client = reqwest::Client::new();
            let loopback_async_client = reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            AutoHealingClientPool {
                generation: AtomicU64::new(1),
                consecutive_transport_failures: AtomicU32::new(0),
                total_reconnect_count: AtomicU64::new(0),
                last_reconnect_epoch_secs: AtomicU64::new(0),
                last_wall_epoch_secs: AtomicU64::new(0),
                start_instant: Instant::now(),
                last_monotonic_secs: AtomicU64::new(0),
                status: RwLock::new(NetworkStatus::Healthy),
                async_client: ArcSwap::new(Arc::new(async_client)),
                loopback_async_client: ArcSwap::new(Arc::new(loopback_async_client)),
                blocking_client: RwLock::new(None),
                last_known_interface_hash: AtomicU64::new(0),
            }
        })
    })
}

/// Retrieves the active `reqwest::Client` with auto-healed socket connection pools.
pub fn auto_healing_async_client() -> reqwest::Client {
    global_healing_pool()
        .get_async_client()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Retrieves the active loopback `reqwest::Client` (no proxy) with auto-healed pools.
pub fn auto_healing_loopback_async_client() -> reqwest::Client {
    global_healing_pool()
        .get_loopback_async_client()
        .unwrap_or_else(|_| {
            reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new())
        })
}

/// Retrieves the active `reqwest::blocking::Client` with auto-healed socket connection pools.
pub fn auto_healing_blocking_client() -> reqwest::blocking::Client {
    global_healing_pool()
        .get_blocking_client()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

/// Reports an upstream transport error to the global watchdog for auto-healing.
pub fn report_network_transport_error(error_desc: &str) -> bool {
    global_healing_pool().report_transport_error(error_desc)
}

/// Manually triggers a global connection pool invalidation and reconnect.
pub fn force_network_pool_reconnect(reason: &str) -> Result<u64, NetworkHealingError> {
    global_healing_pool().invalidate_and_reconnect(reason)
}

/// Spawns the background OS reconnect watchdog loop for `global_healing_pool()`.
pub fn start_global_watchdog(probe_interval: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let pool = global_healing_pool();
        let mut interval = tokio::time::interval(probe_interval);
        loop {
            interval.tick().await;
            pool.check_and_heal_sleep_wake();
            pool.check_and_heal_interface_change();
            if pool.status() == NetworkStatus::Degraded
                && (pool.probe_network_connectivity("1.1.1.1:53", Duration::from_secs(1))
                    || pool.probe_network_connectivity("8.8.8.8:53", Duration::from_secs(1)))
            {
                let _ = pool.invalidate_and_reconnect("network connectivity restored after degradation");
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healing_pool_initialization() {
        let pool = match AutoHealingClientPool::new() {
            Ok(p) => p,
            Err(e) => panic!("AutoHealingClientPool initialization failed: {}", e),
        };

        assert_eq!(pool.generation(), 1);
        assert_eq!(pool.total_reconnect_count(), 0);
        assert_eq!(pool.status(), NetworkStatus::Healthy);

        let async_client = pool.get_async_client();
        assert!(async_client.is_ok());

        let blocking_client = pool.get_blocking_client();
        assert!(blocking_client.is_ok());
    }

    #[test]
    fn test_invalidate_and_reconnect_increments_generation() {
        let pool = match AutoHealingClientPool::new() {
            Ok(p) => p,
            Err(e) => panic!("AutoHealingClientPool initialization failed: {}", e),
        };

        assert_eq!(pool.generation(), 1);

        let gen2 = match pool.invalidate_and_reconnect("VPN reconnect test") {
            Ok(g) => g,
            Err(e) => panic!("invalidate_and_reconnect failed: {}", e),
        };

        assert_eq!(gen2, 2);
        assert_eq!(pool.generation(), 2);
        assert_eq!(pool.total_reconnect_count(), 1);
        assert_eq!(pool.status(), NetworkStatus::Healthy);
    }

    #[test]
    fn test_network_reset_error_detection() {
        assert!(AutoHealingClientPool::is_network_reset_error("Connection reset by peer (os error 10054)"));
        assert!(AutoHealingClientPool::is_network_reset_error("Broken pipe (os error 32)"));
        assert!(AutoHealingClientPool::is_network_reset_error("operation timed out"));
        assert!(AutoHealingClientPool::is_network_reset_error("failed to lookup address information: Name resolution failed"));
        assert!(AutoHealingClientPool::is_network_reset_error("dispatch task is gone"));

        assert!(!AutoHealingClientPool::is_network_reset_error("HTTP 401 Unauthorized"));
        assert!(!AutoHealingClientPool::is_network_reset_error("Invalid JSON body"));
    }

    #[test]
    fn test_report_transport_error_triggers_reconnect() {
        let pool = match AutoHealingClientPool::new() {
            Ok(p) => p,
            Err(e) => panic!("AutoHealingClientPool initialization failed: {}", e),
        };

        assert_eq!(pool.generation(), 1);
        let healed = pool.report_transport_error("Connection reset by peer");
        assert!(healed);
        assert_eq!(pool.generation(), 2);
        assert_eq!(pool.total_reconnect_count(), 1);
    }

    #[test]
    fn test_global_singleton_access() {
        let client = auto_healing_async_client();
        let _ = client.get("http://127.0.0.1:9099/healthz");

        let blocking = auto_healing_blocking_client();
        let _ = blocking.get("http://127.0.0.1:9099/healthz");

        let generation_num = global_healing_pool().generation();
        assert!(generation_num >= 1);
    }

    #[test]
    fn test_arc_swap_and_connectrpc_header_preservation() {
        let headers = vec![
            ("content-type".to_string(), "application/connect+proto".to_string()),
            ("connect-protocol-version".to_string(), "1".to_string()),
            ("te".to_string(), "trailers".to_string()),
            ("Connection".to_string(), "keep-alive".to_string()),
            ("Transfer-Encoding".to_string(), "chunked".to_string()),
        ];
        let safe = AutoHealingClientPool::build_connectrpc_safe_headers(&headers);
        assert_eq!(safe.len(), 3);
        assert!(safe.iter().any(|(k, _)| k == "content-type"));
        assert!(safe.iter().any(|(k, _)| k == "connect-protocol-version"));
        assert!(safe.iter().any(|(k, _)| k == "te"));

        let swap = ArcSwap::new(Arc::new(10u64));
        assert_eq!(*swap.load(), 10);
        let prev = swap.swap(Arc::new(20));
        assert_eq!(*prev, 10);
        assert_eq!(*swap.load(), 20);
    }
}
