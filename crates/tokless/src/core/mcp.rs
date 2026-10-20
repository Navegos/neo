// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/core/mcp.rs
// created: 2026-10-20
// lastModified: 2026-10-20

#[derive(Debug, Clone, Default)]
pub struct McpRequest {
    pub tool_id: String,
    pub command: Vec<String>,
    pub ide: bool,
    pub any_shape: bool,
}

#[derive(Debug, Clone, Default)]
pub struct McpReply {
    pub changed: bool,
    pub file: String,
    pub err: Option<String>,
}

pub type McpOption = Box<dyn Fn(&mut McpRequest) + Send + Sync>;

pub fn with_command(cmd: Vec<String>) -> McpOption {
    Box::new(move |r: &mut McpRequest| r.command = cmd.clone())
}

pub fn with_ide(ide: bool) -> McpOption {
    Box::new(move |r: &mut McpRequest| r.ide = ide)
}

pub fn with_any_shape() -> McpOption {
    Box::new(|r: &mut McpRequest| r.any_shape = true)
}

pub trait McpAdapter: Send + Sync {
    fn configure_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply;
    fn remove_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply;
    fn has_mcp(&self, tool_id: &str, opts: &[McpOption]) -> bool;
}

