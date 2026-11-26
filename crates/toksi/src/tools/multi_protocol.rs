// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tools/multi_protocol.rs
// created: 2026-11-09
// lastModified: 2026-11-18

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::util::paths::normalize_path;

pub const DEFAULT_PROTOCOL_BUS_PORT: u16 = crate::config::DEFAULT_PROTOCOL_BUS_PORT;
pub const MAX_A2A_PAYLOAD_BYTES: usize = 1024 * 1024; // 1 MiB cap
pub const CONSOLIDATED_VAULT_COUNT: usize = 495;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolKind {
    Mcp,
    A2a,
    Acp,
    Anp,
    Quic,
    Stdio,
    Sse,
    Http,
    DirectWire,
}

impl ProtocolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mcp => "mcp",
            Self::A2a => "a2a",
            Self::Acp => "acp",
            Self::Anp => "anp",
            Self::Quic => "quic",
            Self::Stdio => "stdio",
            Self::Sse => "sse",
            Self::Http => "http",
            Self::DirectWire => "directwire",
        }
    }
}

impl fmt::Display for ProtocolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ProtocolKind {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mcp" => Ok(Self::Mcp),
            "a2a" | "agent2agent" => Ok(Self::A2a),
            "acp" => Ok(Self::Acp),
            "anp" | "agent-network-protocol" => Ok(Self::Anp),
            "quic" | "http3" => Ok(Self::Quic),
            "stdio" => Ok(Self::Stdio),
            "sse" => Ok(Self::Sse),
            "http" | "rest" => Ok(Self::Http),
            "directwire" | "direct-wire" | "direct_wire" | "wire" => Ok(Self::DirectWire),
            other => Err(ProtocolError::UnsupportedProtocol(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolTarget {
    Rtk,
    Codegraph,
    Ctxmode,
    Projectmem,
    Icm,
    Aimemory,
    Headroom,
    Security,
    Advisory,
    Sandbox,
    Plugins,
    Vault,
}

impl ToolTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rtk => "rtk",
            Self::Codegraph => "codegraph",
            Self::Ctxmode => "ctxmode",
            Self::Projectmem => "projectmem",
            Self::Icm => "icm",
            Self::Aimemory => "aimemory",
            Self::Headroom => "headroom",
            Self::Security => "security",
            Self::Advisory => "advisory",
            Self::Sandbox => "sandbox",
            Self::Plugins => "plugins",
            Self::Vault => "vault",
        }
    }
}

impl fmt::Display for ToolTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ToolTarget {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let low = s.to_ascii_lowercase();
        match low.as_str() {
            "rtk" => Ok(Self::Rtk),
            "codegraph" => Ok(Self::Codegraph),
            "ctxmode" | "ctx" | "context" | "contextmode" | "context-mode" => Ok(Self::Ctxmode),
            "projectmem" => Ok(Self::Projectmem),
            "icm" => Ok(Self::Icm),
            "aimemory" | "ai-memory" => Ok(Self::Aimemory),
            "headroom" => Ok(Self::Headroom),
            "security" => Ok(Self::Security),
            "advisory" => Ok(Self::Advisory),
            "sandbox" => Ok(Self::Sandbox),
            "plugins" | "plugin" => Ok(Self::Plugins),
            "vault" | "inspect-vault" => Ok(Self::Vault),
            _ if low.starts_with("vault:") || low.starts_with("inspect-vault/") => Ok(Self::Vault),
            other => Err(ProtocolError::UnknownTool(other.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("Unknown tool target: {0}")]
    UnknownTool(String),
    #[error("Invalid request payload: {0}")]
    InvalidPayload(String),
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub protocol: ProtocolKind,
    pub tool: ToolTarget,
    pub action: String,
    pub payload: Value,
    pub session_id: Option<String>,
    pub workspace_root: Option<String>,
}

impl UnifiedRequest {
    pub fn new(protocol: ProtocolKind, tool: ToolTarget, action: impl Into<String>, payload: Value) -> Self {
        Self {
            protocol,
            tool,
            action: action.into(),
            payload,
            session_id: None,
            workspace_root: None,
        }
    }

    pub fn with_workspace(mut self, root: &Path) -> Self {
        self.workspace_root = Some(normalize_path(&root.to_string_lossy()));
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub protocol: ProtocolKind,
    pub tool: ToolTarget,
    pub status: String,
    pub result: Value,
    pub raw_text: Option<String>,
}

impl UnifiedResponse {
    pub fn success(protocol: ProtocolKind, tool: ToolTarget, result: Value, raw_text: Option<String>) -> Self {
        Self {
            protocol,
            tool,
            status: "ok".to_string(),
            result,
            raw_text,
        }
    }

    pub fn error(protocol: ProtocolKind, tool: ToolTarget, error_msg: impl Into<String>) -> Self {
        let msg = error_msg.into();
        Self {
            protocol,
            tool,
            status: "error".to_string(),
            result: serde_json::json!({ "error": msg }),
            raw_text: Some(msg),
        }
    }

    /// Format response according to the requested protocol
    pub fn format_protocol_frame(&self) -> String {
        match self.protocol {
            ProtocolKind::Mcp => {
                let id = self.result.get("id").cloned().unwrap_or(Value::from(1));
                if self.status == "ok" {
                    serde_json::to_string(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": self.raw_text.as_deref().unwrap_or("")
                                }
                            ]
                        }
                    }))
                    .unwrap_or_else(|_| "{}".to_string())
                } else {
                    serde_json::to_string(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32603,
                            "message": self.raw_text.as_deref().unwrap_or("Internal error")
                        }
                    }))
                    .unwrap_or_else(|_| "{}".to_string())
                }
            }
            ProtocolKind::A2a => {
                serde_json::to_string(&serde_json::json!({
                    "version": "1.0",
                    "protocol": "a2a",
                    "sender": self.tool.to_string(),
                    "status": self.status,
                    "body": self.result,
                    "text": self.raw_text
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
            ProtocolKind::Acp => {
                serde_json::to_string(&serde_json::json!({
                    "acp_version": "2.0",
                    "status": self.status,
                    "target": self.tool.to_string(),
                    "data": self.result,
                    "message": self.raw_text
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
            ProtocolKind::Stdio => {
                self.raw_text
                    .clone()
                    .unwrap_or_else(|| self.result.to_string())
            }
            ProtocolKind::Sse => {
                let json_data = serde_json::to_string(&self.result).unwrap_or_else(|_| "{}".to_string());
                format!("event: message\ndata: {}\n\n", json_data)
            }
            ProtocolKind::Http => {
                serde_json::to_string(&serde_json::json!({
                    "status": self.status,
                    "tool": self.tool.to_string(),
                    "data": self.result,
                    "text": self.raw_text
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
            ProtocolKind::Anp => {
                serde_json::to_string(&serde_json::json!({
                    "protocol": "anp",
                    "version": "1.0",
                    "status": self.status,
                    "tool": self.tool.to_string(),
                    "data": self.result,
                    "text": self.raw_text
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
            ProtocolKind::Quic => {
                serde_json::to_string(&serde_json::json!({
                    "protocol": "quic",
                    "transport": "datagram-stream",
                    "status": self.status,
                    "tool": self.tool.to_string(),
                    "data": self.result,
                    "text": self.raw_text
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
            ProtocolKind::DirectWire => {
                serde_json::to_string(&serde_json::json!({
                    "protocol": "directwire",
                    "status": self.status,
                    "tool": self.tool.to_string(),
                    "data": self.result,
                    "text": self.raw_text
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum A2aTaskState {
    Submitted,
    Working,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aTask {
    pub id: String,
    pub peer: String,
    pub message_id: String,
    pub state: A2aTaskState,
    pub tool: String,
    pub action: String,
    pub result: Value,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Constant-time 32-byte digest comparison (no early exit).
fn ct_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// A2A v1.0 JSON-RPC 2.0 Task Store supporting `SendMessage`/`GetTask`
/// and `"message/send"`/`"tasks/get"` aliases, SHA-256 constant-time bearer auth,
/// 1 MiB payload cap, and `(peer, messageId)` deduplication.
pub struct A2aTaskStore {
    tasks: Mutex<HashMap<String, A2aTask>>,
    dedupe: Mutex<HashMap<(String, String), String>>,
    expected_token_sha256: Mutex<Option<[u8; 32]>>,
}

impl Default for A2aTaskStore {
    fn default() -> Self {
        Self::new(None)
    }
}

impl A2aTaskStore {
    pub fn new(bearer_token: Option<&str>) -> Self {
        let digest = bearer_token.map(Self::sha256_bytes);
        Self {
            tasks: Mutex::new(HashMap::new()),
            dedupe: Mutex::new(HashMap::new()),
            expected_token_sha256: Mutex::new(digest),
        }
    }

    pub fn sha256_bytes(token: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let res = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&res);
        out
    }

    pub fn set_bearer_token(&self, token: Option<&str>) {
        if let Ok(mut guard) = self.expected_token_sha256.lock() {
            *guard = token.map(Self::sha256_bytes);
        }
    }

    pub fn verify_bearer(&self, auth_header: Option<&str>) -> bool {
        let expected = match self.expected_token_sha256.lock() {
            Ok(g) => *g,
            Err(_) => return false,
        };
        let Some(expected_digest) = expected else {
            return true;
        };
        let Some(header) = auth_header else {
            return false;
        };
        let token = header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
            .unwrap_or("")
            .trim();
        if token.is_empty() {
            return false;
        }
        let actual_digest = Self::sha256_bytes(token);
        ct_eq_32(&expected_digest, &actual_digest)
    }

    pub fn get_task(&self, task_id: &str) -> Option<A2aTask> {
        self.tasks
            .lock()
            .ok()
            .and_then(|g| g.get(task_id).cloned())
    }

    pub fn handle_jsonrpc(
        &self,
        raw_bytes: &[u8],
        auth_header: Option<&str>,
        dispatcher: &MultiProtocolDispatcher,
    ) -> (u16, Value) {
        if raw_bytes.len() > MAX_A2A_PAYLOAD_BYTES {
            return (
                413,
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32600, "message": "Payload exceeds 1 MiB A2A cap" }
                }),
            );
        }
        if !self.verify_bearer(auth_header) {
            return (
                401,
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32001, "message": "Unauthorized A2A bearer token" }
                }),
            );
        }

        let val: Value = match serde_json::from_slice(raw_bytes) {
            Ok(v) => v,
            Err(e) => {
                return (
                    400,
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("Parse error: {e}") }
                    }),
                );
            }
        };

        let id = val.get("id").cloned().unwrap_or(Value::from(1));
        let method = val.get("method").and_then(Value::as_str).unwrap_or("");
        let params = val.get("params").cloned().unwrap_or_else(|| Value::Object(Default::default()));

        match method {
            "SendMessage" | "message/send" => {
                let peer = params
                    .get("peer")
                    .or_else(|| params.get("sender"))
                    .and_then(Value::as_str)
                    .unwrap_or("anonymous-peer")
                    .to_string();
                let message_id = params
                    .get("messageId")
                    .or_else(|| params.get("message_id"))
                    .or_else(|| params.get("message").and_then(|m| m.get("messageId")))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();

                if !message_id.is_empty()
                    && let Ok(dedupe_guard) = self.dedupe.lock()
                    && let Some(existing_id) = dedupe_guard.get(&(peer.clone(), message_id.clone()))
                    && let Some(existing_task) = self.get_task(existing_id)
                {
                    return (
                        200,
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "task": existing_task, "deduplicated": true }
                        }),
                    );
                }

                let tool = params
                    .get("tool")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<ToolTarget>().ok())
                    .unwrap_or(ToolTarget::Headroom);
                let action = params
                    .get("action")
                    .and_then(Value::as_str)
                    .unwrap_or("status")
                    .to_string();
                let payload = params.get("payload").cloned().unwrap_or_else(|| params.clone());

                let unified = UnifiedRequest::new(ProtocolKind::A2a, tool, action.clone(), payload);
                let resp = dispatcher.dispatch(&unified);
                let now = chrono::Utc::now().timestamp_millis();
                let seq = self.tasks.lock().map(|g| g.len() + 1).unwrap_or(1);
                let task_id = format!("a2a-task-{now}-{seq}");
                let state = if resp.status == "ok" {
                    A2aTaskState::Completed
                } else {
                    A2aTaskState::Failed
                };
                let task = A2aTask {
                    id: task_id.clone(),
                    peer: peer.clone(),
                    message_id: message_id.clone(),
                    state,
                    tool: tool.as_str().to_string(),
                    action,
                    result: resp.result,
                    created_at_ms: now,
                    updated_at_ms: now,
                };

                if let Ok(mut tasks_guard) = self.tasks.lock() {
                    tasks_guard.insert(task_id.clone(), task.clone());
                }
                if !message_id.is_empty()
                    && let Ok(mut dedupe_guard) = self.dedupe.lock()
                {
                    dedupe_guard.insert((peer, message_id), task_id);
                }

                (
                    200,
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "task": task, "deduplicated": false }
                    }),
                )
            }
            "GetTask" | "tasks/get" => {
                let task_id = params
                    .get("id")
                    .or_else(|| params.get("taskId"))
                    .or_else(|| params.get("task_id"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                match self.get_task(task_id) {
                    Some(task) => (
                        200,
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "task": task }
                        }),
                    ),
                    None => (
                        200,
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32004, "message": format!("Task not found: {task_id}") }
                        }),
                    ),
                }
            }
            "CancelTask" | "tasks/cancel" => {
                let task_id = params
                    .get("id")
                    .or_else(|| params.get("taskId"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if let Ok(mut guard) = self.tasks.lock()
                    && let Some(task) = guard.get_mut(task_id)
                {
                    task.state = A2aTaskState::Canceled;
                    task.updated_at_ms = chrono::Utc::now().timestamp_millis();
                    return (
                        200,
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "task": task.clone() }
                        }),
                    );
                }
                (
                    200,
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32004, "message": format!("Task not found: {task_id}") }
                    }),
                )
            }
            other => (
                200,
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("Method not found: {other}") }
                }),
            ),
        }
    }
}

/// ACP v2.0 per-session serialized actor queue.
/// Guarantees FIFO serialization within each `session_id` while allowing
/// independent sessions to execute concurrently.
#[derive(Default)]
pub struct SessionActorQueue {
    session_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl SessionActorQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn session_lock(&self, session_id: &str) -> Arc<Mutex<()>> {
        let key = if session_id.trim().is_empty() {
            "default".to_string()
        } else {
            session_id.to_string()
        };
        if let Ok(mut guard) = self.session_locks.lock() {
            return Arc::clone(
                guard
                    .entry(key)
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            );
        }
        Arc::new(Mutex::new(()))
    }

    pub fn dispatch_serialized<F, R>(&self, session_id: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let lock = self.session_lock(session_id);
        let _guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        f()
    }
}

pub struct MultiProtocolDispatcher {
    workspace_root: Option<PathBuf>,
    codegraph_wire: Arc<crate::codegraph::wire::DirectWireBus>,
    a2a_store: Arc<A2aTaskStore>,
    acp_queue: Arc<SessionActorQueue>,
}

impl MultiProtocolDispatcher {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        let norm = workspace_root.map(|p| PathBuf::from(normalize_path(&p.to_string_lossy())));
        let codegraph_wire = Arc::new(crate::codegraph::wire::DirectWireBus::new(norm.clone()));
        Self {
            workspace_root: norm,
            codegraph_wire,
            a2a_store: Arc::new(A2aTaskStore::default()),
            acp_queue: Arc::new(SessionActorQueue::new()),
        }
    }

    pub fn codegraph_wire(&self) -> Arc<crate::codegraph::wire::DirectWireBus> {
        Arc::clone(&self.codegraph_wire)
    }

    pub fn a2a_store(&self) -> Arc<A2aTaskStore> {
        Arc::clone(&self.a2a_store)
    }

    pub fn acp_queue(&self) -> Arc<SessionActorQueue> {
        Arc::clone(&self.acp_queue)
    }

    pub fn dispatch(&self, req: &UnifiedRequest) -> UnifiedResponse {
        let root = req
            .workspace_root
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| self.workspace_root.clone())
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            });

        let exec = || {
            let mut resp = match req.tool {
                ToolTarget::Rtk => self.dispatch_rtk(req),
                ToolTarget::Codegraph => self.dispatch_codegraph(req, &root),
                ToolTarget::Ctxmode => self.dispatch_context(req, &root),
                ToolTarget::Projectmem => self.dispatch_projectmem(req, &root),
                ToolTarget::Icm => self.dispatch_icm(req),
                ToolTarget::Aimemory => self.dispatch_aimemory(req),
                ToolTarget::Headroom => self.dispatch_headroom(req),
                ToolTarget::Security => self.dispatch_security(req, &root),
                ToolTarget::Advisory => self.dispatch_advisory(req, &root),
                ToolTarget::Sandbox => self.dispatch_sandbox(req, &root),
                ToolTarget::Plugins => self.dispatch_plugins(req, &root),
                ToolTarget::Vault => self.dispatch_vault(req, &root),
            };
            Self::compress_large_response(req, &mut resp);
            resp
        };

        if req.protocol == ProtocolKind::Acp {
            let sid = req.session_id.as_deref().unwrap_or("default");
            self.acp_queue.dispatch_serialized(sid, exec)
        } else {
            exec()
        }
    }

    fn compress_large_response(req: &UnifiedRequest, resp: &mut UnifiedResponse) {
        let force = req
            .payload
            .get("compress_response")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let raw_len = resp.raw_text.as_ref().map_or(0, String::len);
        if !force && raw_len <= 2048 {
            return;
        }
        if let Some(ref raw) = resp.raw_text {
            let compressed = crate::headroom::compression::compress_content(raw, None);
            if compressed.len() < raw.len() {
                resp.raw_text = Some(compressed);
            }
        }
    }

    fn dispatch_rtk(&self, req: &UnifiedRequest) -> UnifiedResponse {
        match req.action.as_str() {
            "filter" | "filter_sandbox_output" => {
                let cmd = req.payload.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let output = req.payload.get("output").and_then(|v| v.as_str()).unwrap_or("");
                let filtered = crate::rtk::filter_sandbox_output(cmd, output);
                UnifiedResponse::success(
                    req.protocol,
                    req.tool,
                    serde_json::json!({ "command": cmd, "filtered": filtered }),
                    Some(filtered),
                )
            }
            "compress" | "compress_tool_output" => {
                let cmd = req.payload.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let output = req.payload.get("output").and_then(|v| v.as_str()).unwrap_or("");
                let compressed = crate::rtk::compress_tool_output(cmd, output);
                UnifiedResponse::success(
                    req.protocol,
                    req.tool,
                    serde_json::json!({ "command": cmd, "compressed": compressed }),
                    Some(compressed),
                )
            }
            "root_cause" => {
                let cmd = req.payload.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let output = req.payload.get("output").and_then(|v| v.as_str()).unwrap_or("");
                let cause = crate::rtk::extract_root_cause(cmd, output);
                UnifiedResponse::success(
                    req.protocol,
                    req.tool,
                    serde_json::json!({ "command": cmd, "root_cause": cause }),
                    cause,
                )
            }
            "gain" | "rtk_gain" => {
                let days = req.payload.get("days").and_then(|v| v.as_u64()).map(|d| d as u32);
                match crate::rtk::Tracker::new().and_then(|t| t.get_summary(days)) {
                    Ok(summary) => {
                        let json = serde_json::to_value(&summary).unwrap_or_default();
                        let text = format!("Total saved: {} tokens ({:.1}%)", summary.total_saved, summary.avg_savings_pct);
                        UnifiedResponse::success(req.protocol, req.tool, json, Some(text))
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("Tracker error: {e}")),
                }
            }
            "discover" | "rtk_discover" => {
                let days = req.payload.get("since").and_then(|v| v.as_u64()).map(|d| d as u32).unwrap_or(30);
                match crate::rtk::scan_missed_savings(None, days) {
                    Ok(rep) => {
                        let json = serde_json::to_value(&rep).unwrap_or_default();
                        let text = format!("Missed savings: {} tokens across {} commands", rep.estimated_missed_tokens, rep.filterable_uncovered_count);
                        UnifiedResponse::success(req.protocol, req.tool, json, Some(text))
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("Discover error: {e}")),
                }
            }
            "learn" | "rtk_learn" => {
                let days = req.payload.get("since").and_then(|v| v.as_u64()).map(|d| d as u32).unwrap_or(30);
                match crate::rtk::detect_corrections(None, days) {
                    Ok(rep) => {
                        let json = serde_json::to_value(&rep).unwrap_or_default();
                        let text = format!("Detected {} CLI corrections", rep.corrections_detected);
                        UnifiedResponse::success(req.protocol, req.tool, json, Some(text))
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("Learn error: {e}")),
                }
            }
            "recall" | "rtk_recall" => {
                let hash = req.payload.get("hash").and_then(|v| v.as_str()).unwrap_or("");
                match crate::rtk::recall_output(hash) {
                    Ok(raw) => UnifiedResponse::success(
                        req.protocol,
                        req.tool,
                        serde_json::json!({ "hash": hash, "output": raw }),
                        Some(raw),
                    ),
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("Recall error: {e}")),
                }
            }
            "slash" | "status" => {
                let cmd = req.payload.get("command").and_then(|v| v.as_str()).unwrap_or("/rtk status");
                let text = crate::rtk::handle_slash_command(cmd);
                UnifiedResponse::success(
                    req.protocol,
                    req.tool,
                    serde_json::json!({ "command": cmd, "response": text }),
                    Some(text),
                )
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("RTK action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_codegraph(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        let use_shared = req.workspace_root.is_none()
            || self
                .workspace_root
                .as_ref()
                .is_some_and(|w| w == root);
        let bus_owned;
        let bus = if use_shared {
            self.codegraph_wire.as_ref()
        } else {
            bus_owned = crate::codegraph::wire::DirectWireBus::new(Some(root.to_path_buf()));
            &bus_owned
        };
        let wire_res = bus.dispatch(&req.action, &req.payload);
        if wire_res.get("status").and_then(|v| v.as_str()) == Some("ok") {
            let text = wire_res
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            UnifiedResponse::success(
                req.protocol,
                req.tool,
                serde_json::json!({ "result": text }),
                Some(text),
            )
        } else {
            let err = wire_res
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("CodeGraph error")
                .to_string();
            UnifiedResponse::error(req.protocol, req.tool, err)
        }
    }

    fn dispatch_context(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        if req.action == "status" {
            let norm = normalize_path(&root.to_string_lossy());
            let db_exists = root.join(".toksi").join("ctxmode").join("ctxmode.db").exists()
                || root.join(".toksi").join("context").join("context.db").exists();
            let res = serde_json::json!({
                "status": "ok",
                "tool": "ctxmode",
                "workspace": norm,
                "db_initialized": db_exists,
            });
            let text = serde_json::to_string(&res).ok();
            return UnifiedResponse::success(req.protocol, req.tool, res, text);
        }
        let server = crate::contextmode::mcp::ContextMcpServer::new(Some(root.to_path_buf()));
        let res = server.execute_tool(&req.action, &req.payload);
        UnifiedResponse::success(
            req.protocol,
            req.tool,
            serde_json::json!({ "result": res }),
            Some(res),
        )
    }

    fn dispatch_projectmem(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        let mut payload = req.payload.clone();
        if payload.get("project").is_none()
            && let Some(obj) = payload.as_object_mut()
        {
            obj.insert("project".to_string(), Value::from(root.to_string_lossy().to_string()));
        }
        let res = crate::projectmem::mcp::execute_mcp_tool(&req.action, &payload);
        UnifiedResponse::success(
            req.protocol,
            req.tool,
            serde_json::json!({ "result": res }),
            Some(res),
        )
    }

    fn dispatch_icm(&self, req: &UnifiedRequest) -> UnifiedResponse {
        let mut store = match crate::icm::store::IcmStore::open_default() {
            Ok(s) => s,
            Err(e) => return UnifiedResponse::error(req.protocol, req.tool, format!("Failed to open ICM store: {e}")),
        };

        let tool_name = if req.action.starts_with("icm_") {
            req.action.as_str()
        } else {
            match req.action.as_str() {
                "search_memory" | "search" => "icm_memory_search",
                "recent" | "get_recent" => "icm_memory_recent",
                "list_topics" => "icm_topic_list",
                "store" | "save" => "icm_memory_store",
                "recall" => "icm_memory_recall",
                "pin" => "icm_pin",
                "forget" => "icm_forget",
                "stats" => "icm_stats",
                _ => req.action.as_str(),
            }
        };

        let (text, is_err) = crate::icm::mcp::handle_tool_call(&mut store, tool_name, &req.payload);
        if is_err {
            UnifiedResponse::error(req.protocol, req.tool, text)
        } else {
            let val = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text.clone()));
            UnifiedResponse::success(req.protocol, req.tool, val, Some(text))
        }
    }

    fn dispatch_aimemory(&self, req: &UnifiedRequest) -> UnifiedResponse {
        if req.action == "help" {
            let text = crate::aimemory::slash::handle_slash_command("/aimemory help", None);
            return UnifiedResponse::success(
                req.protocol,
                req.tool,
                serde_json::json!({ "help": text }),
                Some(text),
            );
        }
        let tool_name = if req.action.starts_with("memory_") {
            req.action.as_str()
        } else {
            match req.action.as_str() {
                "list" | "list_pages" => "memory_query",
                "read" | "read_page" => "memory_read_page",
                "write" | "write_page" => "memory_write_page",
                "patch" | "patch_page" => "memory_patch_page",
                "delete" | "delete_page" => "memory_delete_page",
                "handoff_offer" | "handoff_begin" => "memory_handoff_offer",
                "handoff_list" => "memory_handoff_list",
                "handoff_claim" | "handoff_accept" => "memory_handoff_claim",
                "handoff_resolve" => "memory_handoff_resolve",
                "handoff_cancel" => "memory_handoff_cancel",
                "message_send" => "memory_message_send",
                "message_list" => "memory_message_list",
                "message_pop" => "memory_message_pop",
                "message_cancel" => "memory_message_cancel",
                "status" => "memory_status",
                "briefing" | "brief" => "memory_briefing",
                _ => req.action.as_str(),
            }
        };

        let (text, is_err) = crate::aimemory::mcp::handle_aimemory_tool_call(tool_name, &req.payload);
        if is_err {
            UnifiedResponse::error(req.protocol, req.tool, text)
        } else {
            let val = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text.clone()));
            UnifiedResponse::success(req.protocol, req.tool, val, Some(text))
        }
    }

    fn dispatch_headroom(&self, req: &UnifiedRequest) -> UnifiedResponse {
        match req.action.as_str() {
            "headroom_retrieve" | "retrieve" => {
                let store = crate::headroom::ccr::global_ccr_store();
                match crate::headroom::ccr::handle_mcp_ccr_tool(
                    "headroom_retrieve",
                    &req.payload,
                    store.as_ref(),
                ) {
                    Ok(val) => {
                        let text = val.get("content").and_then(Value::as_str).map(|s| s.to_string());
                        UnifiedResponse::success(req.protocol, req.tool, val, text)
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, e.to_string()),
                }
            }
            "headroom_search" | "search" => {
                let store = crate::headroom::ccr::global_ccr_store();
                match crate::headroom::ccr::handle_mcp_ccr_tool(
                    "headroom_search",
                    &req.payload,
                    store.as_ref(),
                ) {
                    Ok(val) => {
                        let text = serde_json::to_string(&val).ok();
                        UnifiedResponse::success(req.protocol, req.tool, val, text)
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, e.to_string()),
                }
            }
            "compress" | "headroom_compress" | "effort_compress" => {
                let effort_str = req
                    .payload
                    .get("effort")
                    .and_then(Value::as_str)
                    .unwrap_or("medium");
                let headers = vec![("x-toksi-effort".to_string(), effort_str.to_string())];
                let path = req
                    .payload
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or("/v1/chat/completions");

                if let Some(payload_obj) = req.payload.get("payload").cloned() {
                    let mut mutable_payload = payload_obj;
                    let stats = crate::headroom::effort::HeadroomEffortCompressor::default()
                        .compress_json_payload(path, &headers, &mut mutable_payload);
                    let res = serde_json::json!({
                        "payload": mutable_payload,
                        "stats": stats,
                    });
                    let text = serde_json::to_string(&res).ok();
                    UnifiedResponse::success(req.protocol, req.tool, res, text)
                } else {
                    let content = req
                        .payload
                        .get("content")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let hint = req.payload.get("hint").and_then(Value::as_str);
                    let effort = crate::headroom::effort::EffortLevel::from_str_loose(effort_str)
                        .unwrap_or_default();
                    let compressed_content = crate::headroom::compression::compress_content(content, hint);
                    let econ_opts = crate::headroom::compression::economizer::EconomizerOptions {
                        deduplicate_instructions: effort != crate::headroom::effort::EffortLevel::High,
                        minify_whitespace: effort == crate::headroom::effort::EffortLevel::Low,
                        strip_redundant_comments: effort != crate::headroom::effort::EffortLevel::High,
                        prune_schema_verbose_descriptions: effort != crate::headroom::effort::EffortLevel::High,
                        loop_guard_threshold: effort.loop_guard_threshold(),
                    };
                    let mut econ = crate::headroom::compression::economizer::TokenEconomizer::new(econ_opts);
                    let (final_text, report) = econ.economize_prompt(&compressed_content);
                    let chosen = if final_text.len() < compressed_content.len() {
                        final_text
                    } else {
                        compressed_content
                    };
                    let bytes_saved = content.len().saturating_sub(chosen.len());
                    let res = serde_json::json!({
                        "compressed": chosen,
                        "effort": effort.as_str(),
                        "original_bytes": content.len(),
                        "compressed_bytes": chosen.len(),
                        "bytes_saved": bytes_saved,
                        "estimated_tokens_saved": bytes_saved / 4,
                        "economizer_report": report,
                    });
                    UnifiedResponse::success(req.protocol, req.tool, res, Some(chosen))
                }
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("Headroom action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_security(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        match req.action.as_str() {
            "check_path" | "check_path_deny" => {
                let raw_path = req
                    .payload
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let norm = normalize_path(raw_path);
                let denied = crate::contextmode::security::check_path_deny(Path::new(&norm));
                let res = serde_json::json!({
                    "path": norm,
                    "allowed": denied.is_none(),
                    "reason": denied,
                });
                let text = serde_json::to_string(&res).ok();
                UnifiedResponse::success(req.protocol, req.tool, res, text)
            }
            "check_command" | "check_command_deny" => {
                let cmd = req
                    .payload
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let denied = crate::contextmode::security::check_command_deny(cmd);
                let res = serde_json::json!({
                    "command": cmd,
                    "allowed": denied.is_none(),
                    "reason": denied,
                });
                let text = serde_json::to_string(&res).ok();
                UnifiedResponse::success(req.protocol, req.tool, res, text)
            }
            "redact" | "redact_secrets" => {
                let input = req
                    .payload
                    .get("content")
                    .or_else(|| req.payload.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let (redacted, count) = crate::codegraph::repomix::redact_secrets(input);
                let res = serde_json::json!({ "redacted": redacted, "secrets_redacted": count });
                UnifiedResponse::success(req.protocol, req.tool, res, Some(redacted))
            }
            "status" | "scan" => {
                let res = serde_json::json!({
                    "status": "ok",
                    "workspace": normalize_path(&root.to_string_lossy()),
                    "enforce_tag": true,
                    "guards": ["path_deny", "command_deny", "secret_redaction"],
                });
                let text = serde_json::to_string(&res).ok();
                UnifiedResponse::success(req.protocol, req.tool, res, text)
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("Security action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_advisory(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        match req.action.as_str() {
            "status" | "info" | "disclosure_check" => {
                let res = serde_json::json!({
                    "status": "ok",
                    "workspace": normalize_path(&root.to_string_lossy()),
                    "advisory_repo": "Navegos/advisory",
                    "disclosure_gate": "active",
                    "embargo_days": 90,
                });
                let text = serde_json::to_string(&res).ok();
                UnifiedResponse::success(req.protocol, req.tool, res, text)
            }
            "precheck" => {
                let diff_res = crate::projectmem::precheck::run_precheck(None, true, None, true, Some(root));
                match diff_res {
                    Ok(report) => {
                        let json = serde_json::to_value(&report).unwrap_or_default();
                        let text = serde_json::to_string(&json).ok();
                        UnifiedResponse::success(req.protocol, req.tool, json, text)
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("Advisory precheck error: {e}")),
                }
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("Advisory action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_sandbox(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        let server = crate::contextmode::mcp::ContextMcpServer::new(Some(root.to_path_buf()));
        match req.action.as_str() {
            "execute" | "ctx_execute" => {
                let res = server.execute_tool("ctx_execute", &req.payload);
                UnifiedResponse::success(
                    req.protocol,
                    req.tool,
                    serde_json::json!({ "result": res }),
                    Some(res),
                )
            }
            "execute_file" | "ctx_execute_file" => {
                let res = server.execute_tool("ctx_execute_file", &req.payload);
                UnifiedResponse::success(
                    req.protocol,
                    req.tool,
                    serde_json::json!({ "result": res }),
                    Some(res),
                )
            }
            "status" => {
                let mgr = crate::agent_runtime::sandbox::global_sandbox_manager();
                let res = serde_json::json!({
                    "status": "ok",
                    "workspace": normalize_path(&root.to_string_lossy()),
                    "backends": ["native", "sandbox-runtime", "openshell", "e2b", "sbx"],
                    "instances": mgr.list().len(),
                });
                let text = serde_json::to_string(&res).ok();
                UnifiedResponse::success(req.protocol, req.tool, res, text)
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("Sandbox action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_plugins(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        let reg = crate::plugins::global_plugin_registry();
        match req.action.as_str() {
            "status" | "list" => {
                let wire_status = super::direct_wire::global_direct_wire_bus().status();
                let plugins = reg.list();
                let res = serde_json::json!({
                    "status": "ok",
                    "workspace": normalize_path(&root.to_string_lossy()),
                    "direct_wire": wire_status,
                    "plugins_count": plugins.len(),
                    "plugins": plugins,
                    "wiring_modes": ["direct-wired", "config-wired"],
                    "protocols": ["mcp", "a2a", "acp", "stdio", "sse", "http", "directwire"],
                });
                let text = serde_json::to_string(&res).ok();
                UnifiedResponse::success(req.protocol, req.tool, res, text)
            }
            "load" | "enable" => {
                let id = req.payload.get("id").and_then(Value::as_str).unwrap_or("");
                match reg.load(id) {
                    Ok(p) => UnifiedResponse::success(req.protocol, req.tool, serde_json::json!(p), None),
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, e),
                }
            }
            "unload" | "disable" => {
                let id = req.payload.get("id").and_then(Value::as_str).unwrap_or("");
                let res = if req.action == "disable" { reg.disable(id) } else { reg.unload(id) };
                match res {
                    Ok(p) => UnifiedResponse::success(req.protocol, req.tool, serde_json::json!(p), None),
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, e),
                }
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("Plugins action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_vault(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        let engine = req
            .payload
            .get("engine")
            .and_then(Value::as_str)
            .unwrap_or(req.action.as_str())
            .to_ascii_lowercase();
        let sub_action = req
            .payload
            .get("tool_action")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                if req.action.eq_ignore_ascii_case(&engine) {
                    "status"
                } else {
                    req.action.as_str()
                }
            });
        let sub_req = UnifiedRequest {
            protocol: req.protocol,
            tool: req.tool,
            action: sub_action.to_string(),
            payload: req.payload.clone(),
            session_id: req.session_id.clone(),
            workspace_root: req.workspace_root.clone(),
        };

        // Route consolidated inspect-vault/* engines to their native in-tree subsystems
        if engine.contains("rtk") {
            return self.dispatch_rtk(&sub_req);
        }
        if engine.contains("codegraph")
            || engine.contains("scip")
            || engine.contains("ast-grep")
            || engine.contains("jcodemunch")
            || engine.contains("repomix")
            || engine.contains("openwiki")
        {
            return self.dispatch_codegraph(&sub_req, root);
        }
        if engine.contains("context") {
            return self.dispatch_context(&sub_req, root);
        }
        if engine.contains("projectmem") {
            return self.dispatch_projectmem(&sub_req, root);
        }
        if engine.contains("icm") {
            return self.dispatch_icm(&sub_req);
        }
        if engine.contains("aimemory")
            || engine.contains("ai-memory")
            || engine.contains("mem0")
            || engine.contains("cognee")
            || engine.contains("graphiti")
            || engine.contains("letta")
        {
            return self.dispatch_aimemory(&sub_req);
        }
        if engine.contains("headroom") || engine.contains("economizer") || engine.contains("agent-opt") {
            return self.dispatch_headroom(&sub_req);
        }

        let res = serde_json::json!({
            "status": "ok",
            "consolidated_vaults": CONSOLIDATED_VAULT_COUNT,
            "engine": engine,
            "workspace": normalize_path(&root.to_string_lossy()),
            "bus": "direct-wired",
        });
        let text = serde_json::to_string(&res).ok();
        UnifiedResponse::success(req.protocol, req.tool, res, text)
    }
}

pub fn unified_dispatcher() -> Arc<MultiProtocolDispatcher> {
    Arc::new(MultiProtocolDispatcher::new(None))
}

pub fn protocol_bus_port() -> u16 {
    crate::config::resolved_ports().protocol_bus
}

pub fn protocol_bus_live_z(port: u16, timeout: Duration) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    use std::io::{Read, Write};
    let req = format!(
        "GET /livez HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 512];
    let Ok(n) = stream.read(&mut buf) else {
        return false;
    };
    if n == 0 {
        return false;
    }
    let s = String::from_utf8_lossy(&buf[..n]);
    (s.starts_with("HTTP/1.1 200") || s.starts_with("HTTP/1.0 200"))
        && (s.contains("toksi-protocol-bus") || s.contains("\"status\":\"ok\""))
}

pub fn is_protocol_bus_running() -> bool {
    protocol_bus_live_z(protocol_bus_port(), Duration::from_millis(500))
}

/// Runs the Unified Protocol Bus listener on Port `9098` (MCP, A2A v1.0, ACP, stdio, SSE, HTTP).
pub async fn run_protocol_bus_server(
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::TcpListener;

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    let dispatcher = unified_dispatcher();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let disp = Arc::clone(&dispatcher);
                tokio::spawn(async move {
                    let _ = serve_protocol_bus_connection(stream, port, disp).await;
                });
            }
            Err(_) => break,
        }
    }
    Ok(())
}

async fn serve_protocol_bus_connection(
    mut stream: tokio::net::TcpStream,
    port: u16,
    dispatcher: Arc<MultiProtocolDispatcher>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncWriteExt;

    loop {
        let Some(req) = crate::headroom::byok_gateway::read_http_request(&mut stream).await? else {
            return Ok(());
        };
        let is_close_requested = req.headers.iter().any(|(k, v)| {
            k.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close")
        });
        let (path, _, _) = crate::headroom::byok_gateway::split_uri_path_and_query(&req.raw_uri);

        if path == "/sse" || path == "/events" {
            let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\nevent: ready\ndata: {\"service\":\"toksi-protocol-bus\",\"status\":\"ok\"}\n\n";
            stream.write_all(head.as_bytes()).await?;
            stream.flush().await?;

            let mut rx = dispatcher.codegraph_wire().subscribe();
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(15)) => {
                        if stream.write_all(b": keepalive\n\n").await.is_err() {
                            break;
                        }
                        let _ = stream.flush().await;
                    }
                    msg = rx.recv() => {
                        match msg {
                            Ok(evt) => {
                                let json = serde_json::to_string(&evt).unwrap_or_else(|_| "{}".to_string());
                                let frame = format!("event: wire\ndata: {}\n\n", json);
                                if stream.write_all(frame.as_bytes()).await.is_err() {
                                    break;
                                }
                                let _ = stream.flush().await;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
            return Ok(());
        }

        let (status_code, mut headers, bytes) = dispatch_protocol_bus_http_request(
            port,
            &dispatcher,
            &req.method,
            &req.raw_uri,
            &req.headers,
            &req.body,
        )
        .await;
        if is_close_requested && !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("connection")) {
            headers.push(("Connection".to_string(), "close".to_string()));
        }
        crate::headroom::byok_gateway::write_http_response(
            &mut stream,
            status_code,
            crate::headroom::byok_gateway::status_reason(status_code),
            &headers,
            &bytes,
        )
        .await?;

        if is_close_requested {
            let _ = stream.shutdown().await;
            break;
        }
    }
    Ok(())
}

pub async fn dispatch_protocol_bus_http_request(
    port: u16,
    dispatcher: &Arc<MultiProtocolDispatcher>,
    method: &str,
    raw_uri: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let (path, _, _) = crate::headroom::byok_gateway::split_uri_path_and_query(raw_uri);

    if method.eq_ignore_ascii_case("OPTIONS") {
        return (
            204,
            vec![
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                (
                    "Access-Control-Allow-Methods".to_string(),
                    "GET, POST, OPTIONS".to_string(),
                ),
                ("Access-Control-Allow-Headers".to_string(), "*".to_string()),
            ],
            Vec::new(),
        );
    }

    if path == "/livez" || path == "/healthz" || path == "/readyz" {
        let val = serde_json::json!({
            "service": "toksi-protocol-bus",
            "port": port,
            "status": "ok",
            "protocols": ["mcp", "a2a", "acp", "stdio", "sse", "http", "directwire"]
        });
        return (
            200,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            serde_json::to_vec(&val).unwrap_or_default(),
        );
    }

    if path == "/metrics" {
        let metrics = crate::telemetry::render_metrics();
        return (
            200,
            vec![(
                "Content-Type".to_string(),
                "text/plain; version=0.0.4; charset=utf-8".to_string(),
            )],
            metrics.into_bytes(),
        );
    }

    if path == "/sse" || path == "/events" {
        return (
            200,
            vec![
                ("Content-Type".to_string(), "text/event-stream".to_string()),
                ("Cache-Control".to_string(), "no-cache".to_string()),
            ],
            b"event: ready\ndata: {\"service\":\"toksi-protocol-bus\",\"status\":\"ok\"}\n\n"
                .to_vec(),
        );
    }

    if let Some((status_code, resp_val)) =
        crate::tools::ide_plugin_host::handle_ide_http_route(&path, method, body)
    {
        return (
            status_code,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            serde_json::to_vec(&resp_val).unwrap_or_default(),
        );
    }

    let default_proto = match path.as_str() {
        "/mcp" => ProtocolKind::Mcp,
        "/a2a" => ProtocolKind::A2a,
        "/acp" => ProtocolKind::Acp,
        "/anp" => ProtocolKind::Anp,
        "/quic" => ProtocolKind::Quic,
        "/stdio" => ProtocolKind::Stdio,
        _ => ProtocolKind::Http,
    };

    if default_proto == ProtocolKind::A2a {
        let auth_header = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.as_str())
            .map(str::to_string);
        let body_bytes = body.to_vec();
        let disp_clone = Arc::clone(dispatcher);
        let (status_code, json_resp) = tokio::task::spawn_blocking(move || {
            disp_clone
                .a2a_store()
                .handle_jsonrpc(&body_bytes, auth_header.as_deref(), &disp_clone)
        })
        .await
        .unwrap_or_else(|e| {
            (
                500,
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32603, "message": format!("A2A task failed: {e}") }
                }),
            )
        });
        return (
            status_code,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            serde_json::to_vec(&json_resp).unwrap_or_default(),
        );
    }

    if default_proto == ProtocolKind::Acp
        && let Ok(raw_val) = serde_json::from_slice::<Value>(body)
    {
        let acp_method = raw_val.get("method").and_then(Value::as_str).unwrap_or("");
        if matches!(
            acp_method,
            "initialize" | "session/new" | "session/prompt" | "session/cancel"
        ) {
            let effort = crate::headroom::effort::detect_effort_level(headers, Some(&raw_val));
            let resp = crate::agent_runtime::acp::handle_acp_jsonrpc(&raw_val, effort);
            return (
                200,
                vec![("Content-Type".to_string(), "application/json".to_string())],
                serde_json::to_vec(&resp).unwrap_or_default(),
            );
        }
    }

    let mut body_val: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            let err = serde_json::json!({ "error": format!("Invalid JSON payload: {}", e) });
            return (
                400,
                vec![("Content-Type".to_string(), "application/json".to_string())],
                serde_json::to_vec(&err).unwrap_or_default(),
            );
        }
    };

    let _stats = crate::headroom::effort::HeadroomEffortCompressor::default()
        .compress_json_payload(&path, headers, &mut body_val);

    let unified_req = if let Ok(r) = serde_json::from_value::<UnifiedRequest>(body_val.clone()) {
        r
    } else {
        let proto = body_val
            .get("protocol")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<ProtocolKind>().ok())
            .unwrap_or(default_proto);
        let tool = body_val
            .get("tool")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<ToolTarget>().ok())
            .unwrap_or(ToolTarget::Headroom);
        let action = body_val
            .get("action")
            .or_else(|| body_val.get("method"))
            .and_then(Value::as_str)
            .unwrap_or("status")
            .to_string();
        let payload = body_val
            .get("payload")
            .or_else(|| body_val.get("params"))
            .cloned()
            .unwrap_or(body_val);
        UnifiedRequest::new(proto, tool, action, payload)
    };

    let disp_clone = Arc::clone(dispatcher);
    let resp = tokio::task::spawn_blocking(move || disp_clone.dispatch(&unified_req))
        .await
        .unwrap_or_else(|e| {
            UnifiedResponse::error(
                default_proto,
                ToolTarget::Headroom,
                format!("Protocol bus task failed: {}", e),
            )
        });

    let ctype = if resp.protocol == ProtocolKind::Sse {
        "text/event-stream"
    } else {
        "application/json"
    };
    let frame = resp.format_protocol_frame();
    (
        200,
        vec![("Content-Type".to_string(), ctype.to_string())],
        frame.into_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_parsing() {
        assert_eq!("mcp".parse::<ProtocolKind>().unwrap(), ProtocolKind::Mcp);
        assert_eq!("a2a".parse::<ProtocolKind>().unwrap(), ProtocolKind::A2a);
        assert_eq!("acp".parse::<ProtocolKind>().unwrap(), ProtocolKind::Acp);
        assert_eq!("anp".parse::<ProtocolKind>().unwrap(), ProtocolKind::Anp);
        assert_eq!("quic".parse::<ProtocolKind>().unwrap(), ProtocolKind::Quic);
        assert_eq!("stdio".parse::<ProtocolKind>().unwrap(), ProtocolKind::Stdio);
        assert_eq!("sse".parse::<ProtocolKind>().unwrap(), ProtocolKind::Sse);
        assert_eq!("http".parse::<ProtocolKind>().unwrap(), ProtocolKind::Http);
        assert_eq!("directwire".parse::<ProtocolKind>().unwrap(), ProtocolKind::DirectWire);
        assert!("unsupported".parse::<ProtocolKind>().is_err());
    }

    #[test]
    fn test_tool_parsing() {
        assert_eq!("rtk".parse::<ToolTarget>().unwrap(), ToolTarget::Rtk);
        assert_eq!("codegraph".parse::<ToolTarget>().unwrap(), ToolTarget::Codegraph);
        assert_eq!("ctxmode".parse::<ToolTarget>().unwrap(), ToolTarget::Ctxmode);
        assert_eq!("context".parse::<ToolTarget>().unwrap(), ToolTarget::Ctxmode);
        assert_eq!("projectmem".parse::<ToolTarget>().unwrap(), ToolTarget::Projectmem);
        assert_eq!("icm".parse::<ToolTarget>().unwrap(), ToolTarget::Icm);
        assert_eq!("aimemory".parse::<ToolTarget>().unwrap(), ToolTarget::Aimemory);
        assert_eq!("headroom".parse::<ToolTarget>().unwrap(), ToolTarget::Headroom);
        assert_eq!("security".parse::<ToolTarget>().unwrap(), ToolTarget::Security);
        assert_eq!("advisory".parse::<ToolTarget>().unwrap(), ToolTarget::Advisory);
        assert_eq!("sandbox".parse::<ToolTarget>().unwrap(), ToolTarget::Sandbox);
        assert_eq!("plugins".parse::<ToolTarget>().unwrap(), ToolTarget::Plugins);
        assert_eq!("vault".parse::<ToolTarget>().unwrap(), ToolTarget::Vault);
        assert_eq!("inspect-vault/cognee".parse::<ToolTarget>().unwrap(), ToolTarget::Vault);
        assert!("unknown".parse::<ToolTarget>().is_err());
    }

    #[test]
    fn test_a2a_task_store_jsonrpc_auth_cap_and_dedupe() {
        let dispatcher = MultiProtocolDispatcher::new(None);
        let store = A2aTaskStore::new(Some("secret-a2a-token"));

        // Unauthorized rejected
        let req_bytes = serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "message/send",
            "params": {
                "peer": "agent-alpha",
                "messageId": "msg-001",
                "tool": "rtk",
                "action": "filter",
                "payload": { "command": "cargo test", "output": "test result: ok. 2 passed" }
            }
        }))
        .unwrap();
        let (code_unauth, _) = store.handle_jsonrpc(&req_bytes, None, &dispatcher);
        assert_eq!(code_unauth, 401);

        // Authorized succeeds
        let (code_ok, resp1) = store.handle_jsonrpc(
            &req_bytes,
            Some("Bearer secret-a2a-token"),
            &dispatcher,
        );
        assert_eq!(code_ok, 200);
        assert_eq!(resp1["result"]["deduplicated"], false);
        let task_id = resp1["result"]["task"]["id"].as_str().unwrap().to_string();

        // Duplicate (peer, messageId) deduplicated
        let (code_dup, resp2) = store.handle_jsonrpc(
            &req_bytes,
            Some("Bearer secret-a2a-token"),
            &dispatcher,
        );
        assert_eq!(code_dup, 200);
        assert_eq!(resp2["result"]["deduplicated"], true);
        assert_eq!(resp2["result"]["task"]["id"], task_id);

        // GetTask / tasks/get retrieves same task
        let get_bytes = serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "GetTask",
            "params": { "id": task_id }
        }))
        .unwrap();
        let (code_get, resp_get) = store.handle_jsonrpc(
            &get_bytes,
            Some("Bearer secret-a2a-token"),
            &dispatcher,
        );
        assert_eq!(code_get, 200);
        assert_eq!(resp_get["result"]["task"]["id"], task_id);

        // 1 MiB cap enforced
        let oversized = vec![b'a'; MAX_A2A_PAYLOAD_BYTES + 1];
        let (code_big, _) = store.handle_jsonrpc(
            &oversized,
            Some("Bearer secret-a2a-token"),
            &dispatcher,
        );
        assert_eq!(code_big, 413);
    }

    #[test]
    fn test_acp_session_actor_queue_and_expanded_targets() {
        let dispatcher = MultiProtocolDispatcher::new(None);
        for (target, action) in [
            (ToolTarget::Security, "status"),
            (ToolTarget::Advisory, "status"),
            (ToolTarget::Sandbox, "status"),
            (ToolTarget::Plugins, "status"),
            (ToolTarget::Vault, "cognee"),
        ] {
            let req = UnifiedRequest::new(ProtocolKind::Acp, target, action, serde_json::json!({}))
                .with_session("acp-sess-1");
            let resp = dispatcher.dispatch(&req);
            assert_eq!(resp.status, "ok", "target {target:?} failed: {:?}", resp.raw_text);
        }
    }

    #[test]
    fn test_rtk_dispatch() {
        let dispatcher = MultiProtocolDispatcher::new(None);
        let req = UnifiedRequest::new(
            ProtocolKind::Http,
            ToolTarget::Rtk,
            "filter",
            serde_json::json!({
                "command": "cargo test",
                "output": "test result: ok. 10 passed"
            }),
        );
        let resp = dispatcher.dispatch(&req);
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.protocol, ProtocolKind::Http);
        assert_eq!(resp.tool, ToolTarget::Rtk);
        assert!(resp.raw_text.unwrap().contains("passed"));
    }

    #[test]
    fn test_protocol_formatting() {
        let resp = UnifiedResponse::success(
            ProtocolKind::Mcp,
            ToolTarget::Rtk,
            serde_json::json!({ "id": 42 }),
            Some("cleaned output".to_string()),
        );
        let frame = resp.format_protocol_frame();
        assert!(frame.contains("\"jsonrpc\":\"2.0\""));
        assert!(frame.contains("cleaned output"));

        let resp_sse = UnifiedResponse::success(
            ProtocolKind::Sse,
            ToolTarget::Rtk,
            serde_json::json!({ "msg": "hello" }),
            None,
        );
        let frame_sse = resp_sse.format_protocol_frame();
        assert!(frame_sse.starts_with("event: message\ndata: {"));

        let resp_a2a = UnifiedResponse::success(
            ProtocolKind::A2a,
            ToolTarget::Codegraph,
            serde_json::json!({ "nodes": 5 }),
            None,
        );
        let frame_a2a = resp_a2a.format_protocol_frame();
        assert!(frame_a2a.contains("\"protocol\":\"a2a\""));
        assert!(frame_a2a.contains("\"sender\":\"codegraph\""));
    }

    #[test]
    fn test_icm_and_aimemory_dispatch() {
        let dispatcher = MultiProtocolDispatcher::new(None);

        let req_icm = UnifiedRequest::new(
            ProtocolKind::Mcp,
            ToolTarget::Icm,
            "icm_stats",
            serde_json::json!({}),
        );
        let resp_icm = dispatcher.dispatch(&req_icm);
        assert_eq!(resp_icm.status, "ok", "icm dispatch failed: {:?}", resp_icm.raw_text);

        let req_aimem = UnifiedRequest::new(
            ProtocolKind::Stdio,
            ToolTarget::Aimemory,
            "memory_status",
            serde_json::json!({}),
        );
        let resp_aimem = dispatcher.dispatch(&req_aimem);
        assert_eq!(resp_aimem.status, "ok");
    }

    #[test]
    fn test_codegraph_all_protocols_and_direct_wire() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            "pub fn entry_point() { helper_fn(); }\npub fn helper_fn() {}\n",
        )
        .unwrap();

        let dispatcher = MultiProtocolDispatcher::new(Some(root.to_path_buf()));
        let mut rx = dispatcher.codegraph_wire().subscribe();

        for (proto, action, payload) in [
            (ProtocolKind::Mcp, "codegraph_explore", serde_json::json!({ "query": "entry_point" })),
            (ProtocolKind::A2a, "codegraph_impact", serde_json::json!({ "symbol": "helper_fn" })),
            (ProtocolKind::Acp, "codegraph_symbol", serde_json::json!({ "symbol": "helper_fn" })),
            (ProtocolKind::Stdio, "codegraph_explore", serde_json::json!({ "query": "helper_fn" })),
            (ProtocolKind::Sse, "codegraph_impact", serde_json::json!({ "symbol": "helper_fn" })),
            (ProtocolKind::Http, "codegraph_symbol", serde_json::json!({ "symbol": "entry_point" })),
            (ProtocolKind::DirectWire, "codegraph_explore", serde_json::json!({ "query": "helper_fn" })),
        ] {
            let req = UnifiedRequest::new(proto, ToolTarget::Codegraph, action, payload);
            let resp = dispatcher.dispatch(&req);
            assert_eq!(resp.status, "ok", "protocol {proto:?} action {action} failed: {:?}", resp.raw_text);
            let frame = resp.format_protocol_frame();
            assert!(!frame.is_empty());
            if proto == ProtocolKind::Acp {
                assert!(frame.contains("\"acp_version\":\"2.0\""));
            } else if proto == ProtocolKind::DirectWire {
                assert!(frame.contains("\"protocol\":\"directwire\""));
            }
        }

        let msg = rx.try_recv().expect("DirectWireBus should broadcast events");
        assert_eq!(msg.topic, "codegraph");
        assert_eq!(msg.source, "direct-wire");
    }
}
