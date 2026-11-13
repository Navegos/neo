// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/wire.rs
// created: 2026-11-13
// lastModified: 2026-11-13

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use super::mcp::handle_tool_call;
use super::CodeGraph;

pub const DIRECT_WIRE_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub topic: String,
    pub source: String,
    pub action: String,
    pub payload: Value,
    pub timestamp: i64,
}

/// Zero-hop in-process broadcast hook bus for DirectWire CodeGraph communications.
pub struct DirectWireBus {
    sender: broadcast::Sender<WireMessage>,
    graph: Arc<Mutex<Option<CodeGraph>>>,
    project_root: Option<PathBuf>,
}

impl Default for DirectWireBus {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DirectWireBus {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let (sender, _) = broadcast::channel(DIRECT_WIRE_CHANNEL_CAPACITY);
        Self {
            sender,
            graph: Arc::new(Mutex::new(None)),
            project_root,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WireMessage> {
        self.sender.subscribe()
    }

    pub fn broadcast(
        &self,
        msg: WireMessage,
    ) -> Result<usize, broadcast::error::SendError<WireMessage>> {
        self.sender.send(msg)
    }

    /// Dispatch a CodeGraph tool call in-process without IPC or network overhead.
    pub fn dispatch(&self, action: &str, payload: &Value) -> Value {
        let mut guard = self.graph.lock().unwrap_or_else(|p| p.into_inner());
        if guard.is_none() {
            let root = self
                .project_root
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            if let Ok(mut cg) = CodeGraph::open(&root) {
                let _ = cg.ensure_indexed();
                *guard = Some(cg);
            }
        }

        let result = if let Some(ref cg) = *guard {
            let (text, is_err) = handle_tool_call(cg, action, payload);
            if is_err {
                serde_json::json!({ "status": "error", "error": text })
            } else {
                serde_json::json!({ "status": "ok", "result": text })
            }
        } else {
            serde_json::json!({
                "status": "error",
                "error": "Failed to open repository CodeGraph database"
            })
        };

        let _ = self.sender.send(WireMessage {
            topic: "codegraph".to_string(),
            source: "direct-wire".to_string(),
            action: action.to_string(),
            payload: payload.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        });

        result
    }
}
