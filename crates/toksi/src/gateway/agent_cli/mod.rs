// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-18

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
    discover_models_zero_cost, extract_readonly_plan_label, inspect_provider_tier,
    is_gateway_routing_active, parse_models_response, parse_ratelimit_headers,
    record_ratelimit_headers, select_next_enabled_profile, sync_discovered_models,
    validate_and_update_key_tier, AccountTier, QuotaWindowKind, TierCheckError,
};

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::config::ToksiTomlConfig;
use crate::db::operations::ToksiConfigDb;
use crate::util::env_path::check_executable_in_path;
use crate::util::logger::L;

/// Spawns a local CLI bridge process in detached background mode with safe Rust (`process_group(0)` on Unix,
/// `CREATE_NO_WINDOW | DETACHED_PROCESS` on Windows) after verifying the binary exists in `PATH`.
pub(crate) fn launch_detached_bridge(
    bin_name: &str,
    args: &[&str],
    label: &str,
) -> Result<Child, String> {
    check_executable_in_path(bin_name)?;

    let mut cmd = Command::new(bin_name);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to spawn {} CLI bridge: {}", label, e))
}

/// Probes a local bridge `/livez` HTTP endpoint on `127.0.0.1:port`.
pub(crate) fn probe_local_livez(port: u16, timeout: Duration) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let req = format!(
        "GET /livez HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut resp = [0u8; 256];
    let Ok(n) = stream.read(&mut resp) else {
        return false;
    };
    let s = String::from_utf8_lossy(&resp[..n]);
    s.starts_with("HTTP/1.1 200") || s.starts_with("HTTP/1.0 200")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAgentSession {
    pub agent: String,
    pub token: String,
    pub source: String,
}

/// Enumeration of the Local Agent CLIs registered as First-Class BYOK Providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalAgentCli {
    Antigravity,
    Copilot,
    Claude,
    Codex,
    Cursor,
    Grok,
    Gemini,
    Goose,
    Crush,
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

    pub const ALL: [LocalAgentCli; 9] = [
        LocalAgentCli::Antigravity,
        LocalAgentCli::Copilot,
        LocalAgentCli::Claude,
        LocalAgentCli::Codex,
        LocalAgentCli::Cursor,
        LocalAgentCli::Grok,
        LocalAgentCli::Gemini,
        LocalAgentCli::Goose,
        LocalAgentCli::Crush,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Antigravity => "AntigravityCli",
            Self::Copilot => "CopilotCli",
            Self::Claude => "ClaudeCli",
            Self::Codex => "CodexCli",
            Self::Cursor => "CursorBridge",
            Self::Grok => "GrokCli",
            Self::Gemini => "GeminiCli",
            Self::Goose => "GooseCli",
            Self::Crush => "CrushCli",
        }
    }

    pub fn config_id(&self) -> &'static str {
        match self {
            Self::Antigravity => "antigravity",
            Self::Copilot => "copilot",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Grok => "grok",
            Self::Gemini => "gemini",
            Self::Goose => "goose",
            Self::Crush => "crush",
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
            Self::Gemini => if cfg!(windows) { "gemini.exe" } else { "gemini" },
            Self::Goose => if cfg!(windows) { "goose.exe" } else { "goose" },
            Self::Crush => if cfg!(windows) { "crush.exe" } else { "crush" },
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
            Self::Gemini => read_gemini_adc_session(),
            Self::Goose => read_env_or_keyring_session("goose", &["GOOSE_PROVIDER_API_KEY", "OPENAI_API_KEY"]),
            Self::Crush => read_env_or_keyring_session("crush", &["CRUSH_API_KEY", "OPENAI_API_KEY"]),
        }
    }

    /// Launch background bridge process detached.
    pub fn launch_bridge(&self, port: u16) -> Result<Child, String> {
        let port_str = port.to_string();
        match self {
            Self::Antigravity => launch_antigravity_bridge(port),
            Self::Copilot => launch_copilot_bridge(port),
            Self::Claude => launch_claude_bridge(port),
            Self::Codex => launch_codex_bridge(port),
            Self::Cursor => launch_cursor_bridge(port),
            Self::Grok => launch_grok_bridge(port),
            Self::Gemini => launch_detached_bridge(self.binary_name(), &["serve", "--port", &port_str], "Gemini"),
            Self::Goose => launch_detached_bridge(self.binary_name(), &["serve", "--port", &port_str], "Goose"),
            Self::Crush => launch_detached_bridge(self.binary_name(), &["serve", "--port", &port_str], "Crush"),
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
            Self::Gemini | Self::Goose | Self::Crush => probe_local_livez(port, timeout),
        }
    }
}

fn read_env_or_keyring_session(agent: &str, env_vars: &[&str]) -> Option<String> {
    if let Some(mut env) = crate::keyring::get_agent_token(agent)
        && !env.is_expired(30)
        && !env.access_token.is_empty()
    {
        return Some(env.take_access_token());
    }
    for var in env_vars {
        if let Ok(val) = std::env::var(var)
            && !val.trim().is_empty()
        {
            return Some(val.trim().to_string());
        }
    }
    None
}

/// Configured-Only Activation Guard: returns `true` only if `cli` is explicitly
/// configured/enabled in `~/.toksi/config.toml` or SQLite `toksi.db`.
pub fn is_cli_configured(
    cli: LocalAgentCli,
    config: &ToksiTomlConfig,
    db: Option<&ToksiConfigDb>,
) -> bool {
    let cid = cli.config_id();
    let cname = cli.name();
    let matches_id = |s: &str| {
        let lower = s.trim().to_ascii_lowercase();
        lower == cid
            || lower == cname.to_ascii_lowercase()
            || (cli == LocalAgentCli::Antigravity && (lower == "agy" || lower == "antigravity-cli" || lower == "gemini:antigravity"))
            || (cli == LocalAgentCli::Copilot && (lower == "copilot-cli" || lower == "githubcopilot"))
            || (cli == LocalAgentCli::Claude && (lower == "claude-cli" || lower == "claude-code" || lower == "claude_code"))
            || (cli == LocalAgentCli::Cursor && lower == "cursor-cli")
            || (cli == LocalAgentCli::Grok && lower == "grok-cli")
    };

    if config.default_agents.iter().any(|a| matches_id(a))
        || config.general.default_agents.iter().any(|a| matches_id(a))
        || config.byok_profiles.iter().any(|p| p.enabled && matches_id(&p.provider_id))
    {
        return true;
    }

    if let Some(db_ref) = db {
        if let Ok(profiles) = db_ref.list_byok_profiles()
            && profiles.iter().any(|p| p.is_active && matches_id(&p.provider_id))
        {
            return true;
        }
        if let Ok(agents) = db_ref.list_agent_configs()
            && agents.iter().any(|a| matches_id(&a.agent_id))
        {
            return true;
        }
    }

    false
}

/// Smart background launcher respecting Configured-Only Activation, safe `PATH` validation,
/// OS Keyring (`gemini:antigravity` alias normalized), and SQLite `byok_profiles`.
/// Never panics or fails on missing/unconfigured binaries — skips cleanly with structured logs.
pub fn smart_launch_configured_cli(
    cli: LocalAgentCli,
    port: u16,
    config: &ToksiTomlConfig,
    db: Option<&ToksiConfigDb>,
) -> Result<Option<Child>, String> {
    if !is_cli_configured(cli, config, db) {
        L.debug(&format!("Skipping unconfigured local agent CLI: {}", cli.name()));
        return Ok(None);
    }

    let bin = cli.binary_name();
    if let Err(err) = check_executable_in_path(bin) {
        L.warn(&format!(
            "Configured CLI '{}' ({}) missing from PATH; skipping: {}",
            cli.name(),
            bin,
            err
        ));
        return Ok(None);
    }

    let has_db_profile = db
        .and_then(|d| d.list_byok_profiles().ok())
        .map(|ps| ps.iter().any(|p| p.is_active && p.provider_id.eq_ignore_ascii_case(cli.config_id())))
        .unwrap_or(false);

    if cli.read_session().is_none() && !has_db_profile {
        L.debug(&format!(
            "No active keyring session or BYOK profile found for {}; skipping bridge launch",
            cli.name()
        ));
        return Ok(None);
    }

    cli.launch_bridge(port).map(Some)
}

pub fn detect_local_agent_session(agent: &str) -> Option<LocalAgentSession> {
    let lower = agent.to_lowercase();
    // 1. Try native OS Keyring first
    if let Some(mut env) = crate::keyring::get_agent_token(&lower)
        && !env.is_expired(30)
        && !env.access_token.is_empty()
    {
        return Some(LocalAgentSession {
            agent: lower,
            token: env.take_access_token(),
            source: crate::keyring::current_backend_name().to_string(),
        });
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
        "goose" => read_env_or_keyring_session("goose", &["GOOSE_PROVIDER_API_KEY", "OPENAI_API_KEY"])
            .map(|token| LocalAgentSession {
                agent: "goose".to_string(),
                token,
                source: "Goose Keyring / Env".to_string(),
            }),
        "crush" => read_env_or_keyring_session("crush", &["CRUSH_API_KEY", "OPENAI_API_KEY"])
            .map(|token| LocalAgentSession {
                agent: "crush".to_string(),
                token,
                source: "Crush Keyring / Env".to_string(),
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
        assert_eq!(LocalAgentCli::ALL.len(), 9);
        let names: Vec<&str> = LocalAgentCli::ALL.iter().map(|c| c.name()).collect();
        assert!(names.contains(&"AntigravityCli"));
        assert!(names.contains(&"CopilotCli"));
        assert!(names.contains(&"ClaudeCli"));
        assert!(names.contains(&"CodexCli"));
        assert!(names.contains(&"CursorBridge"));
        assert!(names.contains(&"GrokCli"));
        assert!(names.contains(&"GeminiCli"));
        assert!(names.contains(&"GooseCli"));
        assert!(names.contains(&"CrushCli"));
    }

    #[test]
    fn test_configured_only_activation_guard_skips_unconfigured() {
        let cfg = ToksiTomlConfig::default();
        assert!(!is_cli_configured(LocalAgentCli::Copilot, &cfg, None));
        let res = smart_launch_configured_cli(LocalAgentCli::Copilot, 18999, &cfg, None);
        assert!(res.is_ok());
        assert!(res.ok().flatten().is_none());
    }
}
