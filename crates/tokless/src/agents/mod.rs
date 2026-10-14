// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/mod.rs
// created: 2026-10-05
// lastModified: 2026-10-14

pub mod antigravity;
pub mod claude;
pub mod cline;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod droid;
pub mod grok;
pub mod hooks;
pub mod kilo;
pub mod omp;
pub mod opencode;
pub mod pi;
pub mod proxy_agent;
pub mod proxy_detection;
pub mod proxy_protocol;
pub mod proxy_provider;
pub mod vscode;

pub use antigravity::*;
pub use claude::*;
pub use cline::*;
pub use codex::*;
pub use copilot::*;
pub use cursor::*;
pub use droid::*;
pub use grok::*;
pub use hooks::CursorHookSnapshot;
pub use kilo::*;
pub use omp::*;
pub use opencode::*;
pub use pi::*;
pub use proxy_agent::*;
pub use proxy_detection::*;
pub use proxy_protocol::*;
pub use proxy_provider::*;
pub use vscode::*;

use std::sync::Arc;
use crate::core::{register_agent, AgentManifest};
use crate::util::paths;

pub fn claude_manifest() -> AgentManifest {
    AgentManifest {
        id: "claude".to_string(),
        label: "Claude Code".to_string(),
        homepage: "https://github.com/anthropics/claude-code".to_string(),
        cli_bin: "claude".to_string(),
        config_dir: Some(Arc::new(|| paths::claude_code_paths().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with(
                "claude",
                &paths::claude_code_paths().dir,
                &claude_known_bin_dirs(),
                &claude_desktop_paths(),
            )
        })),
    }
}

pub fn codex_manifest() -> AgentManifest {
    AgentManifest {
        id: "codex".to_string(),
        label: "Codex".to_string(),
        homepage: "https://github.com/openai/codex".to_string(),
        cli_bin: "codex".to_string(),
        config_dir: Some(Arc::new(|| paths::codex_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_vscode_agent(
                "codex",
                &paths::codex_paths_resolved().dir,
                &codex_known_bin_dirs(),
                "openai.chatgpt",
            )
        })),
    }
}

pub fn cursor_manifest() -> AgentManifest {
    AgentManifest {
        id: "cursor".to_string(),
        label: "Cursor".to_string(),
        homepage: "https://www.cursor.com".to_string(),
        cli_bin: "agent".to_string(),
        config_dir: Some(Arc::new(cursor_config_dir)),
        detect: Some(Arc::new(detect_cursor)),
    }
}

pub fn droid_manifest() -> AgentManifest {
    AgentManifest {
        id: "droid".to_string(),
        label: "Factory Droid".to_string(),
        homepage: "https://factory.ai".to_string(),
        cli_bin: "droid".to_string(),
        config_dir: Some(Arc::new(droid_dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with(
                "droid",
                &droid_dir(),
                &droid_known_bin_dirs(),
                &droid_desktop_paths(),
            )
        })),
    }
}

/// Register all built-in agents in the global agent registry.
pub fn register() {
    register_agent(claude_manifest());
    register_agent(opencode_manifest());
    register_agent(codex_manifest());
    register_agent(cursor_manifest());
    register_agent(antigravity_manifest());
    register_agent(copilot_manifest());
    register_agent(droid_manifest());
    register_agent(grok_manifest());
    register_agent(kilo_manifest());
    register_agent(omp_manifest());
    register_agent(pi_manifest());
    register_agent(cline_manifest());
}
