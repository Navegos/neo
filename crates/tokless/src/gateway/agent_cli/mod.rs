// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/agent_cli/mod.rs

pub mod claude_code;
pub mod copilot;
pub mod cursor;
pub mod gemini_agy;
pub mod grok_cli;

pub use claude_code::read_claude_code_session;
pub use copilot::read_copilot_session;
pub use cursor::read_cursor_session;
pub use gemini_agy::read_gemini_adc_session;
pub use grok_cli::read_grok_cli_session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAgentSession {
    pub agent: String,
    pub token: String,
    pub source: String,
}

pub fn detect_local_agent_session(agent: &str) -> Option<LocalAgentSession> {
    match agent.to_lowercase().as_str() {
        "claude" | "claude_code" => read_claude_code_session().map(|token| LocalAgentSession {
            agent: "claude".to_string(),
            token,
            source: "~/.claude.json".to_string(),
        }),
        "copilot" | "githubcopilot" => read_copilot_session().map(|token| LocalAgentSession {
            agent: "copilot".to_string(),
            token,
            source: "gh auth token".to_string(),
        }),
        "gemini" | "antigravity" => read_gemini_adc_session().map(|token| LocalAgentSession {
            agent: "gemini".to_string(),
            token,
            source: "Google ADC".to_string(),
        }),
        "grok" => read_grok_cli_session().map(|token| LocalAgentSession {
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

