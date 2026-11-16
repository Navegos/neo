// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/ports.rs
// created: 2026-11-13
// lastModified: 2026-11-16

//! Unified 8-Port Topology re-exported from `crate::config`.

#![deny(unsafe_code)]

pub use crate::config::{
    load_toksi_toml_config, resolve_custom_base_url, resolved_ports, toksi_config_toml_path,
    PortsConfig, ToksiTomlConfig, DEFAULT_AI_GATEWAY_PORT, DEFAULT_BYOK_GATEWAY_PORT,
    DEFAULT_COPILOT_CLI_PORT, DEFAULT_COPILOT_VSCODE_PORT, DEFAULT_DASHBOARD_PORT,
    DEFAULT_GROK_OAUTH_PROXY_PORT, DEFAULT_HEADROOM_PROXY_PORT, DEFAULT_PROTOCOL_BUS_PORT,
};
