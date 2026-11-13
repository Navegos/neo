// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/wire.rs
// created: 2026-11-13

use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use super::mcp::ContextMcpServer;

pub const DIRECT_WIRE_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub topic: String,
    pub source: String,
    pub action: String,
    pub payload: Value,
    pub timestamp: i64,
}

/// Zero-hop in-process broadcast hook bus for DirectWire communications.
pub struct DirectWireBus {
    sender: broadcast::Sender<WireMessage>,
    server: Arc<ContextMcpServer>,
}

impl Default for DirectWireBus {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DirectWireBus {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let (sender, _) = broadcast::channel(DIRECT_WIRE_CHANNEL_CAPACITY);
        let server = Arc::new(ContextMcpServer::new(project_root));
        Self { sender, server }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WireMessage> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, msg: WireMessage) -> Result<usize, broadcast::error::SendError<WireMessage>> {
        self.sender.send(msg)
    }

    /// Direct execution without intermediate network or IPC hops.
    pub fn dispatch(&self, action: &str, payload: &Value) -> Value {
        let result_str = self.server.execute_tool(action, payload);
        let _ = self.sender.send(WireMessage {
            topic: "context".to_string(),
            source: "direct-wire".to_string(),
            action: action.to_string(),
            payload: payload.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        });

        serde_json::json!({
            "status": "ok",
            "action": action,
            "result": result_str
        })
    }
}
