// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/ipc/network_manager.rs
// created: 2026-11-25

//! Single `NetworkService` Network Manager (`toksi _worker network-manager`) owning all
//! daemon TCP listening sockets to prevent `WSAEADDRINUSE` (`os error 10048`) port
//! collisions and dispatching inbound frames over DirectWire IPC to service/tool workers.

#![deny(unsafe_code)]

use std::collections::HashSet;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

use crate::commands::daemon::DaemonServicePlan;
use crate::config::PortsConfig;
use crate::headroom::byok_gateway::{read_http_request, status_reason, write_http_response};

use super::client::send_frame_with_timeout;
use super::frame::{IpcFrame, IpcOpcode};
use super::transport::WorkerRole;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRouteBinding {
    pub port: u16,
    pub service_role: WorkerRole,
    pub protocol: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkServiceRouteTable {
    bindings: Vec<PortRouteBinding>,
}

impl NetworkServiceRouteTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the canonical route table for a daemon service plan.
    /// `:9100` (`Port9100`) remains detached unless explicitly enabled in `plan.dashboard`.
    pub fn from_daemon_plan(plan: &DaemonServicePlan, ports: &PortsConfig) -> Self {
        let mut table = Self::new();
        let candidates = [
            (ports.headroom_proxy, WorkerRole::Port8787, "http/sse", plan.headroom_proxy),
            (ports.grok_oauth_proxy, WorkerRole::Port8788, "http/sse", plan.grok_oauth_proxy),
            (
                ports.copilot_vscode_proxy,
                WorkerRole::Port8789,
                "http/sse",
                plan.copilot_vscode_proxy,
            ),
            (
                ports.copilot_cli_proxy,
                WorkerRole::Port8790,
                "http/sse",
                plan.copilot_cli_proxy,
            ),
            (
                ports.protocol_bus,
                WorkerRole::Port9098,
                "mcp/a2a/acp/anp/quic/ide",
                plan.protocol_bus,
            ),
            (ports.ai_gateway, WorkerRole::Port9099, "http/sse/otel", plan.ai_gateway),
            (ports.byok_gateway, WorkerRole::Port18787, "http/sse/adc", plan.byok_gateway),
        ];
        for (port, role, proto, enabled) in candidates {
            let _ = table.claim_port(PortRouteBinding {
                port,
                service_role: role,
                protocol: proto.to_string(),
                enabled,
            });
        }
        table
    }

    /// Claims exclusive ownership of `binding.port`; fails fast on duplicate port bind
    /// attempts before any OS socket bind runs.
    pub fn claim_port(&mut self, binding: PortRouteBinding) -> Result<(), String> {
        if let Some(existing) = self.bindings.iter().find(|b| b.port == binding.port) {
            return Err(format!(
                "port {} already owned by '{}'; refusing duplicate bind (prevents WSAEADDRINUSE/10048)",
                binding.port,
                existing.service_role.as_str()
            ));
        }
        self.bindings.push(binding);
        Ok(())
    }

    pub fn bindings(&self) -> &[PortRouteBinding] {
        &self.bindings
    }

    pub fn route_for_port(&self, port: u16) -> Option<&PortRouteBinding> {
        self.bindings.iter().find(|b| b.port == port && b.enabled)
    }

    pub fn target_ipc_endpoint(&self, port: u16) -> Option<String> {
        self.route_for_port(port)
            .map(|b| b.service_role.ipc_endpoint())
    }
}

pub async fn bind_single_owner_listeners(
    bindings: &[PortRouteBinding],
    bind_host: &str,
) -> Result<Vec<(PortRouteBinding, TcpListener)>, String> {
    let mut seen_ports = HashSet::new();
    let mut bound = Vec::new();
    for binding in bindings.iter().filter(|b| b.enabled) {
        if binding.port != 0 && !seen_ports.insert(binding.port) {
            return Err(format!(
                "Duplicate port {} in NetworkManager bind list (prevents os error 10048)",
                binding.port
            ));
        }
        let addr = format!("{bind_host}:{}", binding.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            format!(
                "NetworkManager failed binding '{}' for role '{}': {e}",
                addr,
                binding.service_role.as_str()
            )
        })?;
        let actual_port = listener
            .local_addr()
            .map(|a| a.port())
            .unwrap_or(binding.port);
        let mut resolved = binding.clone();
        resolved.port = actual_port;
        bound.push((resolved, listener));
    }
    Ok(bound)
}

pub async fn forward_payload_over_ipc(
    role: WorkerRole,
    endpoint: &str,
    method: &str,
    path: &str,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<serde_json::Value, String> {
    let body_text = String::from_utf8_lossy(&body).to_string();
    let payload = serde_json::json!({
        "role": role.as_str(),
        "method": method,
        "path": path,
        "headers": headers,
        "body": body_text,
    });
    let frame = IpcFrame::json(IpcOpcode::NetworkForward, 1, &payload)?;
    let reply = send_frame_with_timeout(endpoint, &frame, Duration::from_secs(5)).await?;
    reply.payload_json()
}

pub struct NetworkManagerHandle {
    pub bound_ports: Vec<(WorkerRole, u16)>,
    shutdown_tx: watch::Sender<bool>,
}

impl NetworkManagerHandle {
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

async fn handle_http_connection(
    mut stream: TcpStream,
    binding: PortRouteBinding,
    endpoint: String,
) {
    let Ok(Some(req)) = read_http_request(&mut stream).await else {
        return;
    };
    let path_only = req
        .raw_uri
        .split('?')
        .next()
        .unwrap_or(req.raw_uri.as_str())
        .to_string();

    if req.method.eq_ignore_ascii_case("OPTIONS") {
        let _ = write_http_response(&mut stream, 200, "OK", &[], b"").await;
        return;
    }

    if path_only == "/livez" || path_only == "/healthz" || path_only == "/readyz" {
        let body = serde_json::json!({
            "status": "ok",
            "manager": "network-manager",
            "service_role": binding.service_role.as_str(),
            "port": binding.port,
            "ipc_endpoint": endpoint,
        })
        .to_string();
        let headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let _ = write_http_response(&mut stream, 200, "OK", &headers, body.as_bytes()).await;
        return;
    }

    let (status, resp_body) = match forward_payload_over_ipc(
        binding.service_role,
        &endpoint,
        &req.method,
        &req.raw_uri,
        req.headers,
        req.body,
    )
    .await
    {
        Ok(val) => {
            let code = val
                .get("status_code")
                .and_then(|v| v.as_u64())
                .and_then(|n| u16::try_from(n).ok())
                .unwrap_or(200);
            let body_str = if let Some(s) = val.get("body").and_then(|v| v.as_str()) {
                s.to_string()
            } else {
                val.to_string()
            };
            (code, body_str)
        }
        Err(err) => {
            let err_json = serde_json::json!({
                "error": "worker_ipc_unavailable",
                "service_role": binding.service_role.as_str(),
                "ipc_endpoint": endpoint,
                "detail": err,
            })
            .to_string();
            (503, err_json)
        }
    };

    let headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    let _ = write_http_response(
        &mut stream,
        status,
        status_reason(status),
        &headers,
        resp_body.as_bytes(),
    )
    .await;
}

pub async fn spawn_network_manager_listeners(
    bindings: Vec<PortRouteBinding>,
    bind_host: &str,
) -> Result<NetworkManagerHandle, String> {
    let bound = bind_single_owner_listeners(&bindings, bind_host).await?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut bound_ports = Vec::with_capacity(bound.len());

    for (binding, listener) in bound {
        bound_ports.push((binding.service_role, binding.port));
        let mut rx = shutdown_rx.clone();
        let endpoint = binding.service_role.ipc_endpoint();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    changed = rx.changed() => {
                        if changed.is_err() || *rx.borrow() {
                            break;
                        }
                    }
                    accepted = listener.accept() => {
                        match accepted {
                            Ok((stream, _)) => {
                                let b = binding.clone();
                                let ep = endpoint.clone();
                                tokio::spawn(async move {
                                    handle_http_connection(stream, b, ep).await;
                                });
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });
    }

    Ok(NetworkManagerHandle {
        bound_ports,
        shutdown_tx,
    })
}

pub async fn run_network_manager(
    plan: DaemonServicePlan,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), String> {
    let ports = crate::config::resolved_ports();
    let table = NetworkServiceRouteTable::from_daemon_plan(&plan, &ports);
    let _watchdog = crate::gateway::reconnect::start_global_watchdog(Duration::from_secs(10));
    let handle = spawn_network_manager_listeners(table.bindings().to_vec(), "127.0.0.1").await?;

    loop {
        if *shutdown.borrow() {
            break;
        }
        if shutdown.changed().await.is_err() || *shutdown.borrow() {
            break;
        }
    }
    handle.shutdown();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_table_prevents_duplicate_port_ownership() {
        let mut table = NetworkServiceRouteTable::new();
        assert!(table
            .claim_port(PortRouteBinding {
                port: 8787,
                service_role: WorkerRole::Port8787,
                protocol: "http".to_string(),
                enabled: true,
            })
            .is_ok());

        let dup = table.claim_port(PortRouteBinding {
            port: 8787,
            service_role: WorkerRole::Port9099,
            protocol: "http".to_string(),
            enabled: true,
        });
        assert!(dup.is_err());
        let err = dup.err().unwrap_or_default();
        assert!(err.contains("already owned"));
        assert!(err.contains("10048"));
    }

    #[tokio::test]
    async fn test_single_owner_listeners_bind_ephemeral_ports() {
        let bindings = vec![
            PortRouteBinding {
                port: 0,
                service_role: WorkerRole::Port8787,
                protocol: "http".to_string(),
                enabled: true,
            },
            PortRouteBinding {
                port: 0,
                service_role: WorkerRole::Port9098,
                protocol: "mcp".to_string(),
                enabled: true,
            },
        ];
        let handle = spawn_network_manager_listeners(bindings, "127.0.0.1")
            .await
            .expect("bind ephemeral ports");
        assert_eq!(handle.bound_ports.len(), 2);
        assert_ne!(handle.bound_ports[0].1, 0);
        assert_ne!(handle.bound_ports[1].1, 0);
        handle.shutdown();
    }
}

