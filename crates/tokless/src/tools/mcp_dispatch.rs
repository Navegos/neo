// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/mcp_dispatch.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::sync::Arc;

use crate::agents::{mcp_adapter_for, mcp_command};
use crate::core::{with_any_shape, with_command, McpAdapter, McpOption};

pub fn mcp_adapter(agent_id: &str) -> Option<Arc<dyn McpAdapter>> {
    mcp_adapter_for(agent_id)
}

pub fn mcp_configure(agent_id: &str, tool_id: &str) -> bool {
    let Some(a) = mcp_adapter(agent_id) else {
        return true;
    };
    let reply = a.configure_mcp(tool_id, &[with_command(mcp_command(agent_id, tool_id))]);
    reply.err.is_none() && (reply.changed || a.has_mcp(tool_id, &[]))
}

pub fn mcp_remove(agent_id: &str, tool_id: &str) -> bool {
    let Some(a) = mcp_adapter(agent_id) else {
        return true;
    };
    let reply = a.remove_mcp(tool_id, &[with_command(mcp_command(agent_id, tool_id))]);
    reply.err.is_none()
}

pub fn mcp_has(agent_id: &str, tool_id: &str, any_shape: bool) -> bool {
    let Some(a) = mcp_adapter(agent_id) else {
        return false;
    };
    let mut opts: Vec<McpOption> = vec![with_command(mcp_command(agent_id, tool_id))];
    if any_shape {
        opts.push(with_any_shape());
    }
    a.has_mcp(tool_id, &opts)
}

