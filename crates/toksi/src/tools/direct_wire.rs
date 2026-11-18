// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tools/direct_wire.rs
// created: 2026-11-18
// lastModified: 2026-11-18

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use super::multi_protocol::{MultiProtocolDispatcher, ProtocolKind, ToolTarget, UnifiedRequest, UnifiedResponse};
use crate::util::paths::normalize_path;

pub const DIRECT_WIRE_BUS_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WireRegistrationMode {
    DirectWired,
    ConfigWired,
}

impl WireRegistrationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectWired => "direct-wired",
            Self::ConfigWired => "config-wired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectWireAgent {
    pub agent_id: String,
    pub mode: WireRegistrationMode,
    pub capabilities: Vec<String>,
    pub registered_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectWireEvent {
    pub event_type: String,
    pub source_agent: String,
    pub target: String,
    pub action: String,
    pub payload: Value,
    pub timestamp_ms: i64,
}

/// Config-less in-process DirectWireBus (`direct-wired` alongside `config-wired`)
/// enabling hooked/configured agents to invoke `toksi` tools and agent-to-agent
/// commands directly in-process without config-file entries.
pub struct DirectWireBus {
    sender: broadcast::Sender<DirectWireEvent>,
    agents: RwLock<HashMap<String, DirectWireAgent>>,
    dispatcher: Arc<MultiProtocolDispatcher>,
}

impl Default for DirectWireBus {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DirectWireBus {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        let (sender, _) = broadcast::channel(DIRECT_WIRE_BUS_CAPACITY);
        let norm_root = workspace_root.map(|p| PathBuf::from(normalize_path(&p.to_string_lossy())));
        let dispatcher = Arc::new(MultiProtocolDispatcher::new(norm_root));
        Self {
            sender,
            agents: RwLock::new(HashMap::new()),
            dispatcher,
        }
    }

    pub fn with_dispatcher(dispatcher: Arc<MultiProtocolDispatcher>) -> Self {
        let (sender, _) = broadcast::channel(DIRECT_WIRE_BUS_CAPACITY);
        Self {
            sender,
            agents: RwLock::new(HashMap::new()),
            dispatcher,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DirectWireEvent> {
        self.sender.subscribe()
    }

    pub fn register_agent(
        &self,
        agent_id: impl Into<String>,
        mode: WireRegistrationMode,
        capabilities: Vec<String>,
    ) -> DirectWireAgent {
        let id = agent_id.into().to_ascii_lowercase();
        let entry = DirectWireAgent {
            agent_id: id.clone(),
            mode,
            capabilities,
            registered_at: chrono::Utc::now().timestamp_millis(),
        };
        if let Ok(mut guard) = self.agents.write() {
            guard.insert(id, entry.clone());
        }
        let _ = self.sender.send(DirectWireEvent {
            event_type: "agent_registered".to_string(),
            source_agent: entry.agent_id.clone(),
            target: "direct-wire".to_string(),
            action: mode.as_str().to_string(),
            payload: serde_json::json!({ "capabilities": entry.capabilities }),
            timestamp_ms: entry.registered_at,
        });
        entry
    }

    pub fn unregister_agent(&self, agent_id: &str) -> bool {
        let id = agent_id.to_ascii_lowercase();
        if let Ok(mut guard) = self.agents.write() {
            return guard.remove(&id).is_some();
        }
        false
    }

    pub fn list_agents(&self) -> Vec<DirectWireAgent> {
        let mut out = match self.agents.read() {
            Ok(guard) => guard.values().cloned().collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        };
        out.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
        out
    }

    pub fn invoke_tool(
        &self,
        source_agent: &str,
        target: ToolTarget,
        action: &str,
        payload: Value,
        workspace_root: Option<&PathBuf>,
    ) -> UnifiedResponse {
        let mut req = UnifiedRequest::new(ProtocolKind::DirectWire, target, action, payload.clone());
        if let Some(root) = workspace_root {
            req = req.with_workspace(root);
        }
        let resp = self.dispatcher.dispatch(&req);

        let _ = self.sender.send(DirectWireEvent {
            event_type: "tool_invoked".to_string(),
            source_agent: source_agent.to_string(),
            target: target.as_str().to_string(),
            action: action.to_string(),
            payload,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        });

        resp
    }

    pub fn send_agent_message(
        &self,
        from_agent: &str,
        to_agent: &str,
        action: &str,
        payload: Value,
    ) -> Value {
        let ts = chrono::Utc::now().timestamp_millis();
        let receivers = self
            .sender
            .send(DirectWireEvent {
                event_type: "agent_to_agent".to_string(),
                source_agent: from_agent.to_string(),
                target: to_agent.to_string(),
                action: action.to_string(),
                payload: payload.clone(),
                timestamp_ms: ts,
            })
            .unwrap_or(0);

        serde_json::json!({
            "status": "ok",
            "mode": "direct-wired",
            "from": from_agent,
            "to": to_agent,
            "action": action,
            "delivered_subscribers": receivers,
            "timestamp_ms": ts,
        })
    }

    pub fn status(&self) -> Value {
        let agents = self.list_agents();
        let direct_count = agents
            .iter()
            .filter(|a| a.mode == WireRegistrationMode::DirectWired)
            .count();
        let config_count = agents
            .iter()
            .filter(|a| a.mode == WireRegistrationMode::ConfigWired)
            .count();
        serde_json::json!({
            "bus": "direct-wired",
            "status": "active",
            "direct_wired_agents": direct_count,
            "config_wired_agents": config_count,
            "agents": agents,
        })
    }
}

static GLOBAL_DIRECT_WIRE_BUS: OnceLock<Arc<DirectWireBus>> = OnceLock::new();

pub fn global_direct_wire_bus() -> Arc<DirectWireBus> {
    Arc::clone(GLOBAL_DIRECT_WIRE_BUS.get_or_init(|| Arc::new(DirectWireBus::default())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_wire_bus_agent_and_tool_dispatch() {
        let bus = DirectWireBus::new(None);
        let mut rx = bus.subscribe();

        let agent = bus.register_agent(
            "antigravity-cli",
            WireRegistrationMode::DirectWired,
            vec!["rtk".to_string(), "codegraph".to_string()],
        );
        assert_eq!(agent.mode, WireRegistrationMode::DirectWired);

        let resp = bus.invoke_tool(
            "antigravity-cli",
            ToolTarget::Rtk,
            "filter",
            serde_json::json!({
                "command": "cargo test",
                "output": "test result: ok. 3 passed"
            }),
            None,
        );
        assert_eq!(resp.status, "ok");

        let a2a = bus.send_agent_message(
            "antigravity-cli",
            "copilot-cli",
            "handoff",
            serde_json::json!({ "task": "verify" }),
        );
        assert_eq!(a2a.get("mode").and_then(Value::as_str), Some("direct-wired"));

        let ev1 = rx.try_recv().expect("register event");
        assert_eq!(ev1.event_type, "agent_registered");
        let ev2 = rx.try_recv().expect("tool event");
        assert_eq!(ev2.event_type, "tool_invoked");
        let ev3 = rx.try_recv().expect("a2a event");
        assert_eq!(ev3.event_type, "agent_to_agent");
    }
}
