// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/config.rs
// created: 2026-11-20
// lastModified: 2026-11-20

#![deny(unsafe_code)]

use std::process::Command;

use crate::config::{
    ensure_default_config_toml, load_toksi_toml_config_checked, save_toksi_toml_config,
    toksi_config_toml_path, ByokTableEntry, ToksiTomlConfig,
};
use crate::util::colors::*;
use crate::util::env_path::check_executable_in_path;
use crate::util::logger::L;
use crate::util::paths::normalize_path;

fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(format!("invalid boolean '{}': expected true|false", other)),
    }
}

fn parse_port(raw: &str) -> Result<u16, String> {
    let p = raw
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("invalid port '{}': expected 1..65535", raw))?;
    if p == 0 {
        return Err("port must be >= 1".to_string());
    }
    Ok(p)
}

fn parse_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn apply_config_set(cfg: &mut ToksiTomlConfig, key: &str, value: &str) -> Result<(), String> {
    let k = key.trim();
    match k {
        "general.default_agents" | "default_agents" => {
            cfg.general.default_agents = parse_list(value);
        }
        "general.direct_wire" | "direct_wire" => {
            cfg.general.direct_wire = parse_bool(value)?;
        }
        "general.auto_install_user_hooks" => {
            cfg.general.auto_install_user_hooks = parse_bool(value)?;
        }
        "general.auto_install_repo_hooks" => {
            cfg.general.auto_install_repo_hooks = parse_bool(value)?;
        }
        "ports.headroom" | "ports.headroom_proxy" => {
            cfg.ports.headroom_proxy = parse_port(value)?;
        }
        "ports.grok_oauth" | "ports.grok_oauth_proxy" => {
            cfg.ports.grok_oauth_proxy = parse_port(value)?;
        }
        "ports.copilot_vscode" | "ports.copilot_vscode_proxy" => {
            cfg.ports.copilot_vscode_proxy = parse_port(value)?;
        }
        "ports.copilot_cli" | "ports.copilot_cli_proxy" => {
            cfg.ports.copilot_cli_proxy = parse_port(value)?;
        }
        "ports.protocol_bus" | "ports.protocol" => {
            cfg.ports.protocol_bus = parse_port(value)?;
        }
        "ports.byok_gateway" | "ports.byok" => {
            cfg.ports.byok_gateway = parse_port(value)?;
        }
        "ports.gateway" | "ports.ai_gateway" => {
            cfg.ports.ai_gateway = parse_port(value)?;
        }
        "ports.dashboard" | "ports.cockpit" => {
            cfg.ports.dashboard = parse_port(value)?;
        }
        "test.incremental" => {
            cfg.test.incremental = parse_bool(value)?;
        }
        "test.baseline_path" => {
            cfg.test.baseline_path = normalize_path(value.trim());
        }
        "auth.gcp_adc.agy_adc_auth" => {
            cfg.auth.gcp_adc.agy_adc_auth = parse_bool(value)?;
        }
        "auth.gcp_adc.gcp_project" => {
            cfg.auth.gcp_adc.gcp_project = value.trim().to_string();
        }
        "auth.gcp_adc.gcp_location" => {
            cfg.auth.gcp_adc.gcp_location = value.trim().to_string();
        }
        _ if k.starts_with("base_urls.") => {
            let provider = k.trim_start_matches("base_urls.").trim().to_ascii_lowercase();
            if provider.is_empty() {
                return Err("missing provider name in base_urls.<provider>".to_string());
            }
            let val = value.trim().trim_end_matches('/').to_string();
            if val.is_empty() {
                cfg.base_urls.remove(&provider);
            } else {
                cfg.base_urls.insert(provider, val);
            }
        }
        _ if k.starts_with("byok.") => {
            let rest = k.trim_start_matches("byok.");
            let Some((id, field)) = rest.split_once('.') else {
                return Err("expected key format byok.<id>.<base_url|model|effort|enabled|provider_id>".to_string());
            };
            let id = id.trim();
            if id.is_empty() {
                return Err("byok profile id cannot be empty".to_string());
            }
            let entry = cfg.byok.entry(id.to_string()).or_insert_with(ByokTableEntry::default);
            match field.trim() {
                "base_url" => entry.base_url = value.trim().trim_end_matches('/').to_string(),
                "model" | "model_id" => entry.model = value.trim().to_string(),
                "effort" | "thinking_effort" => entry.effort = value.trim().to_ascii_lowercase(),
                "enabled" => entry.enabled = parse_bool(value)?,
                "provider_id" => entry.provider_id = value.trim().to_ascii_lowercase(),
                other => return Err(format!("unknown byok field '{}'", other)),
            }
        }
        _ => return Err(format!("unknown config key '{}'", k)),
    }
    cfg.ports.validate().map_err(|e| e.to_string())?;
    Ok(())
}

fn print_config_help() {
    println!("Usage: toksi config <subcommand>");
    println!("  show [--json]        Show ~/.toksi/config.toml (creates commented default if absent)");
    println!("  path                 Print normalized ~/.toksi/config.toml path");
    println!("  set <key> <value>    Update a key (e.g. ports.dashboard, general.direct_wire, byok.openai.model)");
    println!("  edit                 Open ~/.toksi/config.toml in $VISUAL/$EDITOR");
    println!("  reload               Reload and validate ~/.toksi/config.toml");
}

pub fn run_config_cli(argv: &[String]) -> i32 {
    let sub = argv.first().map(|s| s.as_str()).unwrap_or("show");
    match sub {
        "--help" | "-h" | "help" => {
            print_config_help();
            0
        }
        "path" => {
            println!("{}", normalize_path(&toksi_config_toml_path().to_string_lossy()));
            0
        }
        "show" => {
            if let Err(e) = ensure_default_config_toml() {
                L.err(&e.to_string());
                return 1;
            }
            let json_mode = argv.iter().any(|a| a == "--json");
            let cfg = match load_toksi_toml_config_checked() {
                Ok(c) => c,
                Err(e) => {
                    L.err(&e.to_string());
                    return 1;
                }
            };
            if json_mode {
                match serde_json::to_string_pretty(&cfg) {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        L.err(&format!("Failed to serialize config as JSON: {}", e));
                        return 1;
                    }
                }
            } else {
                match toml::to_string_pretty(&cfg) {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        L.err(&format!("Failed to serialize config as TOML: {}", e));
                        return 1;
                    }
                }
            }
            0
        }
        "reload" => {
            if let Err(e) = ensure_default_config_toml() {
                L.err(&e.to_string());
                return 1;
            }
            match load_toksi_toml_config_checked() {
                Ok(cfg) => {
                    if let Err(e) = cfg.ports.validate() {
                        L.err(&e.to_string());
                        return 1;
                    }
                    println!(
                        "  {} Reloaded {} (direct_wire={}, default_agents={}, byok={})",
                        C.green(Sym::CHECK),
                        normalize_path(&toksi_config_toml_path().to_string_lossy()),
                        cfg.general.direct_wire,
                        cfg.general.default_agents.len(),
                        cfg.byok.len() + cfg.byok_profiles.len()
                    );
                    0
                }
                Err(e) => {
                    L.err(&e.to_string());
                    1
                }
            }
        }
        "set" => {
            if argv.len() < 3 {
                L.err("Usage: toksi config set <key> <value>");
                return 2;
            }
            if let Err(e) = ensure_default_config_toml() {
                L.err(&e.to_string());
                return 1;
            }
            let mut cfg = match load_toksi_toml_config_checked() {
                Ok(c) => c,
                Err(e) => {
                    L.err(&e.to_string());
                    return 1;
                }
            };
            if let Err(e) = apply_config_set(&mut cfg, &argv[1], &argv[2]) {
                L.err(&e);
                return 2;
            }
            if let Err(e) = save_toksi_toml_config(&cfg) {
                L.err(&e.to_string());
                return 1;
            }
            println!("  {} Updated {} = {}", C.green(Sym::CHECK), argv[1], argv[2]);
            0
        }
        "edit" => {
            let path = match ensure_default_config_toml() {
                Ok(p) => p,
                Err(e) => {
                    L.err(&e.to_string());
                    return 1;
                }
            };
            let norm = normalize_path(&path.to_string_lossy());
            let editor = std::env::var("VISUAL")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| std::env::var("EDITOR").ok().filter(|s| !s.trim().is_empty()))
                .unwrap_or_else(|| {
                    if cfg!(windows) {
                        "notepad".to_string()
                    } else {
                        "vi".to_string()
                    }
                });
            let editor_bin = editor.split_whitespace().next().unwrap_or(editor.as_str());
            if let Err(msg) = check_executable_in_path(editor_bin) {
                L.err(&msg);
                return 1;
            }
            match Command::new(editor_bin).arg(&norm).status() {
                Ok(st) if st.success() => {
                    if let Err(e) = load_toksi_toml_config_checked().and_then(|c| {
                        c.ports.validate()?;
                        Ok(())
                    }) {
                        L.warn(&format!("Config saved with validation warning: {}", e));
                        return 1;
                    }
                    println!("  {} Edited and validated {}", C.green(Sym::CHECK), norm);
                    0
                }
                Ok(st) => {
                    L.err(&format!("Editor exited with status {}", st));
                    1
                }
                Err(e) => {
                    L.err(&format!("Failed to launch editor '{}': {}", editor_bin, e));
                    1
                }
            }
        }
        other => {
            L.err(&format!("Unknown config subcommand: {}", other));
            print_config_help();
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_config_set_ports_agents_and_byok() {
        let mut cfg = ToksiTomlConfig::default();
        assert!(apply_config_set(&mut cfg, "general.default_agents", "claude, codex").is_ok());
        assert_eq!(cfg.general.default_agents, vec!["claude", "codex"]);

        assert!(apply_config_set(&mut cfg, "general.direct_wire", "false").is_ok());
        assert!(!cfg.general.direct_wire);

        assert!(apply_config_set(&mut cfg, "ports.dashboard", "19100").is_ok());
        assert_eq!(cfg.ports.dashboard, 19100);

        assert!(apply_config_set(&mut cfg, "test.incremental", "false").is_ok());
        assert!(!cfg.test.incremental);

        assert!(apply_config_set(&mut cfg, "byok.local.base_url", "http://127.0.0.1:11434/v1/").is_ok());
        assert!(apply_config_set(&mut cfg, "byok.local.model", "qwen2.5-coder").is_ok());
        assert!(apply_config_set(&mut cfg, "byok.local.effort", "high").is_ok());
        assert!(apply_config_set(&mut cfg, "byok.local.enabled", "true").is_ok());

        let entry = cfg.byok.get("local");
        assert!(entry.is_some());
        if let Some(e) = entry {
            assert_eq!(e.base_url, "http://127.0.0.1:11434/v1");
            assert_eq!(e.model, "qwen2.5-coder");
            assert_eq!(e.effort, "high");
            assert!(e.enabled);
        }
        assert!(cfg.has_enabled_byok_profiles());
    }

    #[test]
    fn test_apply_config_set_rejects_port_collision() {
        let mut cfg = ToksiTomlConfig::default();
        let res = apply_config_set(&mut cfg, "ports.dashboard", "9099");
        assert!(res.is_err());
    }
}
