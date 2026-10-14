// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/core/mod.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Default)]
pub struct RunOpts {
    pub dry_run: bool,
    pub upgrade: bool,
    pub agent: String,
    pub report: Option<Arc<dyn Fn(&str, f64) + Send + Sync>>,
}

impl RunOpts {
    pub fn report_f(&self, phase: &str, frac: f64) {
        if let Some(ref r) = self.report {
            r(phase, frac);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Npm,
    GitHub,
    Cargo,
    UV,
    Binary,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Npm => write!(f, "npm"),
            Channel::GitHub => write!(f, "github"),
            Channel::Cargo => write!(f, "cargo"),
            Channel::UV => write!(f, "uv"),
            Channel::Binary => write!(f, "binary"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Detection {
    pub installed: bool,
    pub source: String, // "cli" | "desktop" | "ide" | "cli+desktop" | "cli+ide" | "config" | ""
}

pub type DetectFn = Arc<dyn Fn() -> Detection + Send + Sync>;
pub type ConfigDirFn = Arc<dyn Fn() -> String + Send + Sync>;
pub type AgentFn = Arc<dyn Fn(&RunOpts) -> Result<bool, DynError> + Send + Sync>;
pub type VerifyFn = Arc<dyn Fn() -> Option<bool> + Send + Sync>;
pub type InstallFn = Arc<dyn Fn(&RunOpts) -> Result<bool, DynError> + Send + Sync>;
pub type IndexProjectFn = Arc<dyn Fn(&str, &RunOpts) -> Result<bool, DynError> + Send + Sync>;
pub type IndexReadyFn = Arc<dyn Fn() -> bool + Send + Sync>;

#[derive(Clone)]
pub struct AgentManifest {
    pub id: String,
    pub label: String,
    pub homepage: String,
    pub cli_bin: String,
    pub config_dir: Option<ConfigDirFn>,
    pub detect: Option<DetectFn>,
}

impl AgentManifest {
    pub fn detect(&self) -> Detection {
        if let Some(ref d) = self.detect {
            d()
        } else {
            Detection::default()
        }
    }
}

#[derive(Clone)]
pub struct ToolManifest {
    pub id: String,
    pub label: String,
    pub description: String,
    pub homepage: String,
    pub install_hint: String,
    pub channel: Channel,
    pub not_trackable: bool,
    pub instruction_only: bool,
    pub needs_node: bool,
    pub needs_git: bool,
    pub min_node_major: i32,
    pub install: Option<InstallFn>,
    pub wire_for: HashMap<String, AgentFn>,
    pub unwire_for: HashMap<String, AgentFn>,
    pub verify_for: HashMap<String, VerifyFn>,
    pub wired_any_for: HashMap<String, VerifyFn>,
    pub index_project: Option<IndexProjectFn>,
    pub index_ready: Option<IndexReadyFn>,
}

impl ToolManifest {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: String::new(),
            homepage: String::new(),
            install_hint: String::new(),
            channel: Channel::Npm,
            not_trackable: false,
            instruction_only: false,
            needs_node: false,
            needs_git: false,
            min_node_major: 0,
            install: None,
            wire_for: HashMap::new(),
            unwire_for: HashMap::new(),
            verify_for: HashMap::new(),
            wired_any_for: HashMap::new(),
            index_project: None,
            index_ready: None,
        }
    }
}

struct RegistryState {
    agent_list: Vec<Arc<AgentManifest>>,
    tool_list: Vec<Arc<ToolManifest>>,
    agent_by_id: HashMap<String, Arc<AgentManifest>>,
    tool_by_id: HashMap<String, Arc<ToolManifest>>,
}

static REGISTRY: RwLock<Option<RegistryState>> = RwLock::new(None);

fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut RegistryState) -> R,
{
    let mut guard = REGISTRY.write().expect("registry write lock");
    if guard.is_none() {
        *guard = Some(RegistryState {
            agent_list: Vec::new(),
            tool_list: Vec::new(),
            agent_by_id: HashMap::new(),
            tool_by_id: HashMap::new(),
        });
    }
    f(guard.as_mut().unwrap())
}

fn read_registry<F, R>(f: F) -> R
where
    F: FnOnce(&RegistryState) -> R,
{
    let guard = REGISTRY.read().expect("registry read lock");
    if let Some(ref state) = *guard {
        f(state)
    } else {
        let empty = RegistryState {
            agent_list: Vec::new(),
            tool_list: Vec::new(),
            agent_by_id: HashMap::new(),
            tool_by_id: HashMap::new(),
        };
        f(&empty)
    }
}

pub fn register_agent(a: AgentManifest) {
    with_registry(|state| {
        let arc = Arc::new(a);
        if !state.agent_by_id.contains_key(&arc.id) {
            state.agent_list.push(arc.clone());
        } else {
            if let Some(pos) = state.agent_list.iter().position(|x| x.id == arc.id) {
                state.agent_list[pos] = arc.clone();
            }
        }
        state.agent_by_id.insert(arc.id.clone(), arc);
    });
}

pub fn register_tool(t: ToolManifest) {
    with_registry(|state| {
        let arc = Arc::new(t);
        if !state.tool_by_id.contains_key(&arc.id) {
            state.tool_list.push(arc.clone());
        } else {
            if let Some(pos) = state.tool_list.iter().position(|x| x.id == arc.id) {
                state.tool_list[pos] = arc.clone();
            }
        }
        state.tool_by_id.insert(arc.id.clone(), arc);
    });
}

pub fn list_agents() -> Vec<Arc<AgentManifest>> {
    read_registry(|s| s.agent_list.clone())
}

pub fn list_tools() -> Vec<Arc<ToolManifest>> {
    read_registry(|s| s.tool_list.clone())
}

pub fn get_agent(id: &str) -> Option<Arc<AgentManifest>> {
    read_registry(|s| s.agent_by_id.get(id).cloned())
}

pub fn get_tool(id: &str) -> Option<Arc<ToolManifest>> {
    read_registry(|s| s.tool_by_id.get(id).cloned())
}

pub fn agent_ids() -> Vec<String> {
    read_registry(|s| s.agent_list.iter().map(|a| a.id.clone()).collect())
}

pub fn tool_ids() -> Vec<String> {
    read_registry(|s| s.tool_list.iter().map(|t| t.id.clone()).collect())
}

pub fn bool_ptr(b: bool) -> Option<bool> {
    Some(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_list() {
        let a1 = AgentManifest {
            id: "test_agent_1".into(),
            label: "Test Agent 1".into(),
            homepage: "".into(),
            cli_bin: "".into(),
            config_dir: None,
            detect: Some(Arc::new(|| Detection {
                installed: true,
                source: "config".into(),
            })),
        };
        let a2 = AgentManifest {
            id: "test_agent_2".into(),
            label: "Test Agent 2".into(),
            homepage: "".into(),
            cli_bin: "".into(),
            config_dir: None,
            detect: Some(Arc::new(|| Detection {
                installed: false,
                source: "".into(),
            })),
        };

        let t1 = ToolManifest::new("test_tool_1", "Test Tool 1");
        let t2 = ToolManifest::new("test_tool_2", "Test Tool 2");

        register_agent(a1);
        register_agent(a2);
        register_tool(t1);
        register_tool(t2);

        assert!(get_agent("test_agent_1").is_some());
        assert!(get_agent("unknown_agent").is_none());
        assert!(get_tool("test_tool_1").is_some());
        assert!(get_tool("unknown_tool").is_none());

        let a_ids = agent_ids();
        let t_ids = tool_ids();

        let idx1 = a_ids.iter().position(|x| x == "test_agent_1").unwrap();
        let idx2 = a_ids.iter().position(|x| x == "test_agent_2").unwrap();
        assert!(idx1 < idx2);

        let tidx1 = t_ids.iter().position(|x| x == "test_tool_1").unwrap();
        let tidx2 = t_ids.iter().position(|x| x == "test_tool_2").unwrap();
        assert!(tidx1 < tidx2);
    }
}
