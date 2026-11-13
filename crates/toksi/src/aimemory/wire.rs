// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/wire.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

pub const DIRECT_WIRE_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub topic: String,
    pub source: String,
    pub action: String,
    pub payload: Value,
    pub timestamp: i64,
}

/// Zero-hop in-process broadcast hook bus for DirectWire ai-memory communications.
pub struct DirectWireBus {
    sender: broadcast::Sender<WireMessage>,
}

impl Default for DirectWireBus {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectWireBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(DIRECT_WIRE_CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WireMessage> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, msg: WireMessage) -> Result<usize, broadcast::error::SendError<WireMessage>> {
        self.sender.send(msg)
    }

    /// Dispatch direct in-process tool call without IPC or network overhead.
    pub fn dispatch(&self, action: &str, payload: &Value) -> Value {
        let (res_str, is_err) = super::mcp::handle_aimemory_tool_call(action, payload);
        let result = if is_err {
            serde_json::json!({ "status": "error", "error": res_str })
        } else {
            serde_json::json!({ "status": "ok", "result": res_str })
        };

        let _ = self.sender.send(WireMessage {
            topic: "aimemory".to_string(),
            source: "direct-wire".to_string(),
            action: action.to_string(),
            payload: payload.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        });

        result
    }
}
