// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/wire.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use crate::icm::store::{resolve_icm_db_path_from, IcmStore};

pub const DIRECT_WIRE_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub topic: String,
    pub source: String,
    pub action: String,
    pub payload: Value,
    pub timestamp: i64,
}

/// Zero-hop in-process broadcast hook bus for DirectWire ICM communications.
pub struct DirectWireBus {
    sender: broadcast::Sender<WireMessage>,
    store: Arc<Mutex<Option<IcmStore>>>,
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
            store: Arc::new(Mutex::new(None)),
            project_root,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WireMessage> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, msg: WireMessage) -> Result<usize, broadcast::error::SendError<WireMessage>> {
        self.sender.send(msg)
    }

    /// Dispatch direct in-process tool call without IPC or network overhead.
    pub fn dispatch(&self, action: &str, payload: &Value) -> Value {
        let mut store_guard = self.store.lock().unwrap_or_else(|p| p.into_inner());
        if store_guard.is_none() {
            if let Ok(path) = resolve_icm_db_path_from(self.project_root.as_deref()) {
                if let Ok(store) = IcmStore::open(path) {
                    *store_guard = Some(store);
                }
            }
        }

        let result = if let Some(ref mut store) = *store_guard {
            let (res_str, is_err) = super::mcp::handle_tool_call(store, action, payload);
            if is_err {
                serde_json::json!({ "status": "error", "error": res_str })
            } else {
                serde_json::json!({ "status": "ok", "result": res_str })
            }
        } else {
            serde_json::json!({ "status": "error", "error": "Failed to open ICM store" })
        };

        let _ = self.sender.send(WireMessage {
            topic: "icm".to_string(),
            source: "direct-wire".to_string(),
            action: action.to_string(),
            payload: payload.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        });

        result
    }
}
