// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-09

pub mod antigravity;
pub mod claude;
pub mod claude_code;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod gemini_agy;
pub mod grok;
pub mod grok_cli;
pub mod tier_check;

pub use antigravity::{launch_antigravity_bridge, read_antigravity_session, verify_antigravity_health};
pub use claude::{launch_claude_bridge, read_claude_session, verify_claude_health};
pub use claude_code::read_claude_code_session;
pub use codex::{launch_codex_bridge, read_codex_session, verify_codex_health};
pub use copilot::{launch_copilot_bridge, read_copilot_session, verify_copilot_health};
pub use cursor::{launch_cursor_bridge, read_cursor_session, verify_cursor_health};
pub use gemini_agy::read_gemini_adc_session;
pub use grok::{launch_grok_bridge, read_grok_session, verify_grok_health};
pub use grok_cli::read_grok_cli_session;
pub use tier_check::{
    active_paid_provider_count, inspect_provider_tier, is_gateway_routing_active,
    set_active_paid_provider_count, validate_and_update_key_tier, AccountTier, TierCheckError,
};

use std::process::Child;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAgentSession {
    pub agent: String,
    pub token: String,
    pub source: String,
}

/// Enumeration of the 6 Local Agent CLIs registered as First-Class BYOK Providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalAgentCli {
    Antigravity,
    Copilot,
    Claude,
    Codex,
    Cursor,
    Grok,
}

impl LocalAgentCli {
    pub const ALL_6: [LocalAgentCli; 6] = [
        LocalAgentCli::Antigravity,
        LocalAgentCli::Copilot,
        LocalAgentCli::Claude,
        LocalAgentCli::Codex,
        LocalAgentCli::Cursor,
        LocalAgentCli::Grok,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Antigravity => "AntigravityCli",
            Self::Copilot => "CopilotCli",
            Self::Claude => "ClaudeCli",
            Self::Codex => "CodexCli",
            Self::Cursor => "CursorBridge",
            Self::Grok => "GrokCli",
        }
    }

    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::Antigravity => if cfg!(windows) { "agy.exe" } else { "agy" },
            Self::Copilot => if cfg!(windows) { "copilot.exe" } else { "copilot" },
            Self::Claude => if cfg!(windows) { "claude.exe" } else { "claude" },
            Self::Codex => if cfg!(windows) { "codex.exe" } else { "codex" },
            Self::Cursor => if cfg!(windows) { "cursor.exe" } else { "cursor" },
            Self::Grok => if cfg!(windows) { "grok.exe" } else { "grok" },
        }
    }

    /// Read authenticated session token from OS Keyring or local agent configs.
    pub fn read_session(&self) -> Option<String> {
        match self {
            Self::Antigravity => read_antigravity_session(),
            Self::Copilot => read_copilot_session(),
            Self::Claude => read_claude_session(),
            Self::Codex => read_codex_session(),
            Self::Cursor => read_cursor_session(),
            Self::Grok => read_grok_session(),
        }
    }

    /// Launch background bridge process detached.
    pub fn launch_bridge(&self, port: u16) -> Result<Child, String> {
        match self {
            Self::Antigravity => launch_antigravity_bridge(port),
            Self::Copilot => launch_copilot_bridge(port),
            Self::Claude => launch_claude_bridge(port),
            Self::Codex => launch_codex_bridge(port),
            Self::Cursor => launch_cursor_bridge(port),
            Self::Grok => launch_grok_bridge(port),
        }
    }

    /// Probes reachability of the bridge process endpoint.
    pub fn verify_health(&self, port: u16, timeout: Duration) -> bool {
        match self {
            Self::Antigravity => verify_antigravity_health(port, timeout),
            Self::Copilot => verify_copilot_health(port, timeout),
            Self::Claude => verify_claude_health(port, timeout),
            Self::Codex => verify_codex_health(port, timeout),
            Self::Cursor => verify_cursor_health(port, timeout),
            Self::Grok => verify_grok_health(port, timeout),
        }
    }
}

pub fn detect_local_agent_session(agent: &str) -> Option<LocalAgentSession> {
    let lower = agent.to_lowercase();
    // 1. Try native OS Keyring first
    if let Some(env) = crate::keyring::get_agent_token(&lower) {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(LocalAgentSession {
                agent: lower,
                token: env.access_token,
                source: crate::keyring::current_backend_name().to_string(),
            });
        }
    }

    // 2. Fallbacks by agent
    match lower.as_str() {
        "claude" | "claude_code" => read_claude_session().map(|token| LocalAgentSession {
            agent: "claude".to_string(),
            token,
            source: "~/.claude.json".to_string(),
        }),
        "copilot" | "githubcopilot" => read_copilot_session().map(|token| LocalAgentSession {
            agent: "copilot".to_string(),
            token,
            source: "gh auth token".to_string(),
        }),
        "gemini" | "antigravity" | "agy" => read_antigravity_session().map(|token| LocalAgentSession {
            agent: "gemini".to_string(),
            token,
            source: "Google ADC / Antigravity".to_string(),
        }),
        "codex" | "openai" => read_codex_session().map(|token| LocalAgentSession {
            agent: "codex".to_string(),
            token,
            source: "~/.codex/auth.json".to_string(),
        }),
        "grok" => read_grok_session().map(|token| LocalAgentSession {
            agent: "grok".to_string(),
            token,
            source: "~/.grok/config.json".to_string(),
        }),
        "cursor" => read_cursor_session().map(|token| LocalAgentSession {
            agent: "cursor".to_string(),
            token,
            source: "Cursor state.vscdb".to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_agent_cli_registry() {
        assert_eq!(LocalAgentCli::ALL_6.len(), 6);
        let names: Vec<&str> = LocalAgentCli::ALL_6.iter().map(|c| c.name()).collect();
        assert!(names.contains(&"AntigravityCli"));
        assert!(names.contains(&"CopilotCli"));
        assert!(names.contains(&"ClaudeCli"));
        assert!(names.contains(&"CodexCli"));
        assert!(names.contains(&"CursorBridge"));
        assert!(names.contains(&"GrokCli"));
    }
}
