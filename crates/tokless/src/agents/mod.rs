// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/mod.rs
// created: 2026-10-05
// lastModified: 2026-10-14

pub mod amp;
pub mod antigravity;
pub mod byok_providers;
pub mod claude;
pub mod cline;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod droid;
pub mod dsh;
pub mod grok;
pub mod hooks;
pub mod kilo;
pub mod kimi;
pub mod mcp_adapters;
pub mod omp;
pub mod opencode;
pub mod pi;
pub mod proxy_agent;
pub mod proxy_detection;
pub mod proxy_protocol;
pub mod proxy_provider;
pub mod vscode;
pub mod zcode;

pub use amp::*;
pub use antigravity::*;
pub use byok_providers::*;
pub use claude::*;
pub use cline::*;
pub use codex::*;
pub use copilot::*;
pub use cursor::*;
pub use droid::*;
pub use dsh::*;
pub use grok::*;
pub use hooks::CursorHookSnapshot;
pub use kilo::*;
pub use kimi::*;
pub use mcp_adapters::*;
pub use omp::*;
pub use opencode::*;
pub use pi::*;
pub use proxy_agent::*;
pub use proxy_detection::*;
pub use proxy_protocol::*;
pub use proxy_provider::*;
pub use vscode::*;
pub use zcode::*;

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
    register_agent(opencode2_manifest());
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
    register_agent(kimi_manifest());
    register_agent(zcode_manifest());
    register_agent(dsh_manifest());
    register_agent(amp_manifest());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::get_agent;

    #[test]
    fn test_phase1_agents_registered() {
        register();

        for id in ["amp", "dsh", "kimi", "zcode", "opencode", "opencode2"] {
            let agent = get_agent(id);
            assert!(agent.is_some(), "Agent {} must be registered", id);
            let a = agent.unwrap();
            assert_eq!(a.id, id);
            assert!(!a.label.is_empty());
            assert!(!a.cli_bin.is_empty());
            assert!(a.config_dir.is_some());
        }

        for id in ["amp", "dsh", "kimi", "zcode"] {
            assert!(proxy_spec_for(id).is_some(), "Proxy spec for {} must exist", id);
        }

        let rules = amp_rtk_rules(&["git".to_string(), "cargo".to_string()]);
        assert_eq!(rules, vec!["rtk git *", "rtk cargo *"]);

        assert_eq!(kimi_provider_protocol("openai"), "openai-completions");
        assert_eq!(kimi_provider_protocol("openai-responses"), "openai-responses");
        assert_eq!(zcode_provider_protocol("openai-chat-completions"), "openai-completions");
        assert_eq!(zcode_provider_protocol("anthropic-messages"), "anthropic-messages");
        assert_eq!(dsh_provider_protocol("openai"), "openai-completions");
    }
}
