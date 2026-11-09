// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tools/multi_protocol.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolKind {
    Mcp,
    A2a,
    Acp,
    Stdio,
    Sse,
    Http,
}

impl ProtocolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mcp => "mcp",
            Self::A2a => "a2a",
            Self::Acp => "acp",
            Self::Stdio => "stdio",
            Self::Sse => "sse",
            Self::Http => "http",
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
            "stdio" => Ok(Self::Stdio),
            "sse" => Ok(Self::Sse),
            "http" | "rest" => Ok(Self::Http),
            other => Err(ProtocolError::UnsupportedProtocol(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolTarget {
    Rtk,
    Codegraph,
    Context,
    Projectmem,
    Icm,
    Aimemory,
}

impl ToolTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rtk => "rtk",
            Self::Codegraph => "codegraph",
            Self::Context => "context",
            Self::Projectmem => "projectmem",
            Self::Icm => "icm",
            Self::Aimemory => "aimemory",
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
        match s.to_ascii_lowercase().as_str() {
            "rtk" => Ok(Self::Rtk),
            "codegraph" => Ok(Self::Codegraph),
            "context" | "contextmode" | "context-mode" => Ok(Self::Context),
            "projectmem" => Ok(Self::Projectmem),
            "icm" => Ok(Self::Icm),
            "aimemory" | "ai-memory" => Ok(Self::Aimemory),
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
                    "sender": format!("toksi-{}", self.tool),
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
        }
    }
}

pub struct MultiProtocolDispatcher {
    workspace_root: Option<PathBuf>,
}

impl MultiProtocolDispatcher {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.map(|p| PathBuf::from(normalize_path(&p.to_string_lossy()))),
        }
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

        match req.tool {
            ToolTarget::Rtk => self.dispatch_rtk(req),
            ToolTarget::Codegraph => self.dispatch_codegraph(req, &root),
            ToolTarget::Context => self.dispatch_context(req, &root),
            ToolTarget::Projectmem => self.dispatch_projectmem(req, &root),
            ToolTarget::Icm => self.dispatch_icm(req),
            ToolTarget::Aimemory => self.dispatch_aimemory(req),
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
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("RTK action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_codegraph(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
        let db_path = crate::codegraph::resolve_db_path(root);
        let cg = match crate::codegraph::CodeGraph::open(&db_path) {
            Ok(cg) => cg,
            Err(e) => return UnifiedResponse::error(req.protocol, req.tool, format!("Failed to open codegraph db: {e}")),
        };

        match req.action.as_str() {
            "explore" | "codegraph_explore" => {
                let res = crate::codegraph::mcp::handle_explore(&cg, &req.payload);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "impact" | "codegraph_impact" => {
                let res = crate::codegraph::mcp::handle_impact(&cg, &req.payload);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "callers" | "codegraph_callers" => {
                let res = crate::codegraph::mcp::handle_callers(&cg, &req.payload);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "callees" | "codegraph_callees" => {
                let res = crate::codegraph::mcp::handle_callees(&cg, &req.payload);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "search" | "codegraph_search" => {
                let res = crate::codegraph::mcp::handle_search(&cg, &req.payload);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "node" | "codegraph_node" => {
                let res = crate::codegraph::mcp::handle_node(&cg, &req.payload);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "status" | "codegraph_status" => {
                let res = crate::codegraph::mcp::handle_status(&cg);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            "files" | "codegraph_files" => {
                let res = crate::codegraph::mcp::handle_files(&cg);
                UnifiedResponse::success(req.protocol, req.tool, serde_json::json!({ "result": res }), Some(res))
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("CodeGraph action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_context(&self, req: &UnifiedRequest, root: &Path) -> UnifiedResponse {
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
        if payload.get("project").is_none() {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("project".to_string(), Value::from(root.to_string_lossy().to_string()));
            }
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
        let store = match crate::icm::store::IcmStore::open_default() {
            Ok(s) => s,
            Err(e) => return UnifiedResponse::error(req.protocol, req.tool, format!("Failed to open ICM store: {e}")),
        };

        match req.action.as_str() {
            "search_memory" | "search" => {
                let query = req.payload.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = req.payload.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                match store.search_bm25(query, limit) {
                    Ok(hits) => {
                        let text = serde_json::to_string_pretty(&hits).unwrap_or_default();
                        UnifiedResponse::success(
                            req.protocol,
                            req.tool,
                            serde_json::to_value(&hits).unwrap_or(Value::Null),
                            Some(text),
                        )
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("ICM search error: {e}")),
                }
            }
            "recent" | "get_recent" => {
                let limit = req.payload.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                match store.get_recent(limit) {
                    Ok(hits) => {
                        let text = serde_json::to_string_pretty(&hits).unwrap_or_default();
                        UnifiedResponse::success(
                            req.protocol,
                            req.tool,
                            serde_json::to_value(&hits).unwrap_or(Value::Null),
                            Some(text),
                        )
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("ICM get_recent error: {e}")),
                }
            }
            "list_topics" => {
                match store.list_topics() {
                    Ok(topics) => {
                        let text = serde_json::to_string_pretty(&topics).unwrap_or_default();
                        UnifiedResponse::success(
                            req.protocol,
                            req.tool,
                            serde_json::to_value(&topics).unwrap_or(Value::Null),
                            Some(text),
                        )
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("ICM list_topics error: {e}")),
                }
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("ICM action '{}' not implemented", other),
            ),
        }
    }

    fn dispatch_aimemory(&self, req: &UnifiedRequest) -> UnifiedResponse {
        let store = crate::aimemory::wiki::WikiStore::open_default();

        match req.action.as_str() {
            "list" | "list_pages" => {
                match store.list_pages() {
                    Ok(pages) => {
                        let text = serde_json::to_string_pretty(&pages).unwrap_or_default();
                        UnifiedResponse::success(
                            req.protocol,
                            req.tool,
                            serde_json::to_value(&pages).unwrap_or(Value::Null),
                            Some(text),
                        )
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("ai-memory list_pages error: {e}")),
                }
            }
            "read" | "read_page" => {
                let slug = req.payload.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                match store.read_page(slug) {
                    Ok(Some(page)) => {
                        let text = serde_json::to_string_pretty(&page).unwrap_or_default();
                        UnifiedResponse::success(
                            req.protocol,
                            req.tool,
                            serde_json::to_value(&page).unwrap_or(Value::Null),
                            Some(text),
                        )
                    }
                    Ok(None) => UnifiedResponse::error(req.protocol, req.tool, format!("Page '{slug}' not found")),
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("ai-memory read_page error: {e}")),
                }
            }
            "tiered" | "to_tiered_entries" => {
                match store.to_tiered_entries() {
                    Ok(entries) => {
                        let text = serde_json::to_string_pretty(&entries).unwrap_or_default();
                        UnifiedResponse::success(
                            req.protocol,
                            req.tool,
                            serde_json::to_value(&entries).unwrap_or(Value::Null),
                            Some(text),
                        )
                    }
                    Err(e) => UnifiedResponse::error(req.protocol, req.tool, format!("ai-memory to_tiered_entries error: {e}")),
                }
            }
            other => UnifiedResponse::error(
                req.protocol,
                req.tool,
                format!("ai-memory action '{}' not implemented", other),
            ),
        }
    }
}

pub fn unified_dispatcher() -> Arc<MultiProtocolDispatcher> {
    Arc::new(MultiProtocolDispatcher::new(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_parsing() {
        assert_eq!("mcp".parse::<ProtocolKind>().unwrap(), ProtocolKind::Mcp);
        assert_eq!("a2a".parse::<ProtocolKind>().unwrap(), ProtocolKind::A2a);
        assert_eq!("acp".parse::<ProtocolKind>().unwrap(), ProtocolKind::Acp);
        assert_eq!("stdio".parse::<ProtocolKind>().unwrap(), ProtocolKind::Stdio);
        assert_eq!("sse".parse::<ProtocolKind>().unwrap(), ProtocolKind::Sse);
        assert_eq!("http".parse::<ProtocolKind>().unwrap(), ProtocolKind::Http);
        assert!("unsupported".parse::<ProtocolKind>().is_err());
    }

    #[test]
    fn test_tool_parsing() {
        assert_eq!("rtk".parse::<ToolTarget>().unwrap(), ToolTarget::Rtk);
        assert_eq!("codegraph".parse::<ToolTarget>().unwrap(), ToolTarget::Codegraph);
        assert_eq!("context".parse::<ToolTarget>().unwrap(), ToolTarget::Context);
        assert_eq!("projectmem".parse::<ToolTarget>().unwrap(), ToolTarget::Projectmem);
        assert_eq!("icm".parse::<ToolTarget>().unwrap(), ToolTarget::Icm);
        assert_eq!("aimemory".parse::<ToolTarget>().unwrap(), ToolTarget::Aimemory);
        assert!("unknown".parse::<ToolTarget>().is_err());
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
        assert!(frame_a2a.contains("\"sender\":\"toksi-codegraph\""));
    }
}
