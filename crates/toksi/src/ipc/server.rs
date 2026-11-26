// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/ipc/server.rs
// created: 2026-11-24

//! Cross-platform IPC server listener for worker subprocesses (`toksi _worker <role>`).

#![deny(unsafe_code)]

use std::sync::{Arc, OnceLock};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::watch;

use super::frame::{IpcFrame, IpcOpcode};
use super::transport::WorkerRole;

pub trait IpcRequestHandler: Send + Sync {
    fn handle(&self, frame: IpcFrame) -> IpcFrame;

    fn handle_async<'a>(
        &'a self,
        frame: IpcFrame,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = IpcFrame> + Send + 'a>> {
        Box::pin(std::future::ready(self.handle(frame)))
    }
}

pub struct DefaultWorkerIpcHandler {
    role: WorkerRole,
    kill_tx: watch::Sender<bool>,
    gateway_orch: OnceLock<Arc<crate::gateway::proxy::GatewayProxyOrchestrator>>,
    protocol_bus: OnceLock<Arc<crate::tools::multi_protocol::MultiProtocolDispatcher>>,
}

impl DefaultWorkerIpcHandler {
    pub fn new(role: WorkerRole, kill_tx: watch::Sender<bool>) -> Self {
        Self {
            role,
            kill_tx,
            gateway_orch: OnceLock::new(),
            protocol_bus: OnceLock::new(),
        }
    }
}

impl IpcRequestHandler for DefaultWorkerIpcHandler {
    fn handle(&self, frame: IpcFrame) -> IpcFrame {
        let corr = frame.correlation_id;
        match frame.opcode {
            IpcOpcode::Ping => IpcFrame::json(
                IpcOpcode::Pong,
                corr,
                &serde_json::json!({
                    "role": self.role.as_str(),
                    "status": "ok",
                }),
            )
            .unwrap_or_else(|_| IpcFrame::new(IpcOpcode::Pong, corr, Vec::new())),
            IpcOpcode::Heartbeat | IpcOpcode::StatusQuery => IpcFrame::json(
                IpcOpcode::StatusReply,
                corr,
                &serde_json::json!({
                    "role": self.role.as_str(),
                    "status": "running",
                    "port": self.role.default_port(),
                }),
            )
            .unwrap_or_else(|_| IpcFrame::new(IpcOpcode::StatusReply, corr, Vec::new())),
            IpcOpcode::KillSwitchTriggered | IpcOpcode::GracefulDrain => {
                let _ = self.kill_tx.send(true);
                IpcFrame::json(
                    IpcOpcode::StatusReply,
                    corr,
                    &serde_json::json!({
                        "role": self.role.as_str(),
                        "status": "draining",
                    }),
                )
                .unwrap_or_else(|_| IpcFrame::new(IpcOpcode::StatusReply, corr, Vec::new()))
            }
            IpcOpcode::ToolDispatch => {
                let req_json = frame.payload_json().unwrap_or_else(|_| serde_json::json!({}));
                let tool = req_json
                    .get("tool")
                    .and_then(|v| v.as_str())
                    .unwrap_or(self.role.as_str());
                let action = req_json
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("status");
                let payload = req_json
                    .get("payload")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let target = tool.parse().unwrap_or(crate::tools::multi_protocol::ToolTarget::Ctxmode);
                let resp = crate::tools::direct_wire::global_direct_wire_bus().invoke_tool(
                    "ipc-worker",
                    target,
                    action,
                    payload,
                    None,
                );
                let body = serde_json::to_vec(&resp).unwrap_or_default();
                IpcFrame::new(IpcOpcode::ToolReply, corr, body)
            }
            IpcOpcode::SiagentPrompt => {
                let req_json = frame.payload_json().unwrap_or_else(|_| serde_json::json!({}));
                let fallback_msg_prompt = req_json
                    .get("messages")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.iter().rev().find_map(|m| m.get("content").and_then(|c| c.as_str())))
                    .unwrap_or("");
                let prompt = req_json
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(fallback_msg_prompt);
                let mode = req_json
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("code");
                let engine = crate::agent_runtime::AgentRuntimeEngine::new(
                    std::path::PathBuf::from("."),
                    "siagent-worker",
                );
                let resp = match engine.dispatch_remote_siagent_prompt(prompt, mode) {
                    Ok(reply) => serde_json::json!({
                        "status": "ok",
                        "mode": mode,
                        "output": reply,
                    }),
                    Err(e) => serde_json::json!({
                        "status": "gateway_unavailable",
                        "mode": mode,
                        "error": e,
                    }),
                };
                IpcFrame::json(IpcOpcode::SiagentReply, corr, &resp)
                    .unwrap_or_else(|_| IpcFrame::new(IpcOpcode::SiagentReply, corr, Vec::new()))
            }
            IpcOpcode::NetworkForward => {
                let req_json = frame.payload_json().unwrap_or_else(|_| serde_json::json!({}));
                let method = req_json
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET");
                let path = req_json
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("/");
                let path_only = path.split('?').next().unwrap_or(path);
                let body_str = req_json.get("body").and_then(|v| v.as_str()).unwrap_or("");
                let reply = if path_only == "/livez" || path_only == "/healthz" || path_only == "/readyz" {
                    serde_json::json!({
                        "status_code": 200,
                        "role": self.role.as_str(),
                        "status": "ok",
                        "method": method,
                        "path": path_only,
                    })
                } else if let Some((status_code, ide_body)) =
                    crate::tools::ide_plugin_host::handle_ide_http_route(
                        path_only,
                        method,
                        body_str.as_bytes(),
                    )
                {
                    serde_json::json!({
                        "status_code": status_code,
                        "role": self.role.as_str(),
                        "body": ide_body.to_string(),
                    })
                } else {
                    let target = match self.role {
                        WorkerRole::Port8787 => crate::tools::multi_protocol::ToolTarget::Headroom,
                        WorkerRole::Codegraph => crate::tools::multi_protocol::ToolTarget::Codegraph,
                        WorkerRole::Ctxmode => crate::tools::multi_protocol::ToolTarget::Ctxmode,
                        WorkerRole::Projectmem => crate::tools::multi_protocol::ToolTarget::Projectmem,
                        WorkerRole::Icm => crate::tools::multi_protocol::ToolTarget::Icm,
                        WorkerRole::Aimemory => crate::tools::multi_protocol::ToolTarget::Aimemory,
                        WorkerRole::Rtk => crate::tools::multi_protocol::ToolTarget::Rtk,
                        WorkerRole::Security => crate::tools::multi_protocol::ToolTarget::Security,
                        _ => crate::tools::multi_protocol::ToolTarget::Headroom,
                    };
                    let body_val = serde_json::from_str::<serde_json::Value>(body_str)
                        .unwrap_or_else(|_| serde_json::json!({ "raw_body": body_str, "path": path, "method": method }));
                    let resp = crate::tools::direct_wire::global_direct_wire_bus().invoke_tool(
                        "network-manager",
                        target,
                        path_only.trim_start_matches('/'),
                        body_val,
                        None,
                    );
                    serde_json::json!({
                        "status_code": 200,
                        "role": self.role.as_str(),
                        "dispatched": resp,
                    })
                };
                IpcFrame::json(IpcOpcode::NetworkReply, corr, &reply)
                    .unwrap_or_else(|_| IpcFrame::new(IpcOpcode::NetworkReply, corr, Vec::new()))
            }
            _ => IpcFrame::new(IpcOpcode::Pong, corr, Vec::new()),
        }
    }

    fn handle_async<'a>(
        &'a self,
        frame: IpcFrame,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = IpcFrame> + Send + 'a>> {
        Box::pin(async move {
            if frame.opcode != IpcOpcode::NetworkForward {
                return self.handle(frame);
            }
            let corr = frame.correlation_id;
            let req_json = frame.payload_json().unwrap_or_else(|_| serde_json::json!({}));
            let method = req_json
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET")
                .to_string();
            let raw_uri = req_json
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();
            let headers: Vec<(String, String)> = req_json
                .get("headers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let pair = item.as_array()?;
                            let k = pair.first()?.as_str()?.to_string();
                            let v = pair.get(1)?.as_str()?.to_string();
                            Some((k, v))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let body_bytes: Vec<u8> = if let Some(arr) = req_json.get("body_bytes").and_then(|v| v.as_array()) {
                arr.iter()
                    .filter_map(|b| b.as_u64().and_then(|n| u8::try_from(n).ok()))
                    .collect()
            } else {
                req_json
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .as_bytes()
                    .to_vec()
            };

            type HttpForwardResp = (u16, Vec<(String, String)>, Vec<u8>);
            let port = self.role.default_port().unwrap_or(0);
            let http_resp: Option<HttpForwardResp> = match self.role {
                WorkerRole::Port9099 => {
                    let orch = Arc::clone(self.gateway_orch.get_or_init(|| {
                        Arc::new(crate::gateway::proxy::GatewayProxyOrchestrator::new())
                    }));
                    Some(
                        crate::gateway::server::dispatch_gateway_http_request(
                            if port == 0 { 9099 } else { port },
                            &orch,
                            &method,
                            &raw_uri,
                            &headers,
                            &body_bytes,
                        )
                        .await,
                    )
                }
                WorkerRole::Port9098 => {
                    let bus = Arc::clone(self.protocol_bus.get_or_init(|| {
                        Arc::new(crate::tools::multi_protocol::MultiProtocolDispatcher::new(None))
                    }));
                    Some(
                        crate::tools::multi_protocol::dispatch_protocol_bus_http_request(
                            if port == 0 { 9098 } else { port },
                            &bus,
                            &method,
                            &raw_uri,
                            &headers,
                            &body_bytes,
                        )
                        .await,
                    )
                }
                WorkerRole::Port18787 => Some(
                    crate::headroom::byok_gateway::handle_byok_gateway_request(
                        &method,
                        &raw_uri,
                        &headers,
                        &body_bytes,
                        None,
                    )
                    .await,
                ),
                WorkerRole::Port8787 => Some(
                    crate::headroom::proxy::dispatch_headroom_proxy_http_request(
                        &method,
                        &raw_uri,
                        &headers,
                        &body_bytes,
                    )
                    .await,
                ),
                WorkerRole::Port8788 => Some(
                    crate::headroom::grok_daemon::dispatch_grok_proxy_http_request(
                        if port == 0 { 8788 } else { port },
                        &method,
                        &raw_uri,
                        &headers,
                        &body_bytes,
                    )
                    .await,
                ),
                WorkerRole::Port8789 => Some(
                    crate::headroom::copilot_daemon::dispatch_copilot_proxy_http_request(
                        if port == 0 { 8789 } else { port },
                        "copilot-vscode",
                        false,
                        &method,
                        &raw_uri,
                        &headers,
                        &body_bytes,
                    )
                    .await,
                ),
                WorkerRole::Port8790 => Some(
                    crate::headroom::copilot_daemon::dispatch_copilot_proxy_http_request(
                        if port == 0 { 8790 } else { port },
                        "copilot-cli",
                        true,
                        &method,
                        &raw_uri,
                        &headers,
                        &body_bytes,
                    )
                    .await,
                ),
                _ => None,
            };

            let Some((status_code, resp_headers, resp_body)) = http_resp else {
                return self.handle(frame);
            };

            let body_utf8 = std::str::from_utf8(&resp_body).ok().map(str::to_string);
            let reply = if let Some(text) = body_utf8 {
                serde_json::json!({
                    "status_code": status_code,
                    "role": self.role.as_str(),
                    "headers": resp_headers,
                    "body": text,
                })
            } else {
                let lossy = String::from_utf8_lossy(&resp_body).to_string();
                serde_json::json!({
                    "status_code": status_code,
                    "role": self.role.as_str(),
                    "headers": resp_headers,
                    "body": lossy,
                    "body_bytes": resp_body,
                })
            };
            IpcFrame::json(IpcOpcode::NetworkReply, corr, &reply)
                .unwrap_or_else(|_| IpcFrame::new(IpcOpcode::NetworkReply, corr, Vec::new()))
        })
    }
}

async fn handle_stream<S, H>(mut stream: S, handler: Arc<H>)
where
    S: AsyncRead + AsyncWrite + Unpin,
    H: IpcRequestHandler + ?Sized,
{
    if let Ok(req) = IpcFrame::read_from(&mut stream).await {
        let reply = handler.handle_async(req).await;
        let _ = reply.write_to(&mut stream).await;
    }
}

pub async fn serve_worker_ipc<H: IpcRequestHandler + 'static>(
    endpoint: &str,
    handler: Arc<H>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use tokio::net::windows::named_pipe::ServerOptions;
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(endpoint)
            .map_err(|e| format!("Failed creating named pipe '{endpoint}': {e}"))?;

        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                res = server.connect() => {
                    match res {
                        Ok(()) => {
                            let conn = server;
                            let h = Arc::clone(&handler);
                            tokio::spawn(async move {
                                handle_stream(conn, h).await;
                            });
                            server = ServerOptions::new()
                                .create(endpoint)
                                .map_err(|e| format!("Failed recreating named pipe '{endpoint}': {e}"))?;
                        }
                        Err(e) => {
                            return Err(format!("Named pipe connect error on '{endpoint}': {e}"));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        use std::path::Path;
        use tokio::net::UnixListener;

        let path = Path::new(endpoint);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        let listener = UnixListener::bind(path)
            .map_err(|e| format!("Failed binding UDS '{endpoint}': {e}"))?;

        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                accepted = listener.accept() => {
                    match accepted {
                        Ok((stream, _)) => {
                            let h = Arc::clone(&handler);
                            tokio::spawn(async move {
                                handle_stream(stream, h).await;
                            });
                        }
                        Err(e) => {
                            return Err(format!("UDS accept error on '{endpoint}': {e}"));
                        }
                    }
                }
            }
        }
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
