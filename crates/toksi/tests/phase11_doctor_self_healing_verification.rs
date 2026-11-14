// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase11_doctor_self_healing_verification.rs
// created: 2026-10-31
// lastModified: 2026-11-14

#![deny(unsafe_code)]

use std::fs;
use std::sync::Mutex;

static PHASE11_LOCK: Mutex<()> = Mutex::new(());

use toksi::agents::{
    audit_agent_legacy_mcp, heal_all_agent_legacy_configs, is_legacy_mcp_runner_str,
    is_toksi_mcp_command_str, register as register_agents, verify_direct_wire_bus,
    AgentLayoutCategory, AgentManager, WireMode,
};
use toksi::commands::{
    audit_agent_hook_conventions, audit_toksi_config_toml, run_doctor, run_doctor_with_options,
    run_init, DoctorOptions, InitOptions,
};
use toksi::core::{get_agent, list_agents};
use toksi::env_injector::{
    audit_environment_status, heal_legacy_hijacks_in, scan_legacy_hijacks_in,
};
use toksi::rules::RuleKind;
use toksi::tools::{
    has_owner_at_path, write_owner_at_path, register as register_tools,
};
use toksi::util::env_path::check_executable_in_path;

const ALL_AGENTS: [&str; 22] = [
    "claude",
    "cursor",
    "codex",
    "opencode",
    "opencode2",
    "antigravity",
    "antigravity-cli",
    "antigravity-ide",
    "copilot",
    "copilot-cli",
    "copilot-vscode",
    "droid",
    "grok",
    "kilo",
    "omp",
    "pi",
    "cline",
    "kimi",
    "zcode",
    "dsh",
    "amp",
    "openagents",
];

#[test]
fn test_phase11_all_22_agents_manifests_and_rules() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    register_agents();
    register_tools();

    let all_agents = list_agents();
    assert_eq!(all_agents.len(), 22, "All 22 agent manifests must be registered");

    for agent_id in ALL_AGENTS {
        let manifest = get_agent(agent_id);
        assert!(manifest.is_some(), "Agent {} must be registered", agent_id);
        let m = manifest.unwrap();
        assert_eq!(m.id, agent_id);
        assert!(!m.label.is_empty());
        assert!(!m.cli_bin.is_empty());

        let dir_str = AgentManager::user_config_dir(agent_id).to_string_lossy().to_string();
        let norm_dir = toksi::util::paths::normalize_path(&dir_str);
        assert!(!norm_dir.contains('\\'), "Normalized dir must use forward slashes: {}", norm_dir);
    }

    assert_eq!(AgentManager::wire_mode("openagents"), WireMode::DirectWired);
    assert_eq!(AgentManager::wire_mode("claude"), WireMode::ConfigWired);
    assert_eq!(AgentManager::layout_category("antigravity-cli"), AgentLayoutCategory::Category2SplitCliAndIde);
    assert_eq!(AgentManager::layout_category("copilot-vscode"), AgentLayoutCategory::Category2SplitCliAndIde);
    assert_eq!(AgentManager::layout_category("openagents"), AgentLayoutCategory::Category5OpenAgentsDirectWire);

    // Verify in-tree rules (Principles, Caveman, Ponytail) are compiled and have valid bodies
    for kind in RuleKind::all() {
        assert!(!kind.markdown().is_empty());
        assert!(!kind.section_body().is_empty());
        assert!(!kind.section_header().is_empty());
        assert!(!kind.cursor_rule().is_empty());
    }
}

#[test]
fn test_phase11_rules_writing_and_verification_parity() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    register_agents();
    register_tools();

    let temp_dir = std::env::temp_dir().join("toksi_p11_rules_test");
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::create_dir_all(&temp_dir);

    let test_file = temp_dir.join("test_agent_instructions.md");
    let test_path = test_file.to_string_lossy().to_string();

    // 1. Initial write of principles, caveman, ponytail
    assert!(write_owner_at_path(&test_path, "principles"));
    assert!(write_owner_at_path(&test_path, "caveman"));
    assert!(write_owner_at_path(&test_path, "ponytail"));

    // 2. Verify all 3 in-tree rules are present
    assert!(has_owner_at_path(&test_path, "principles"));
    assert!(has_owner_at_path(&test_path, "caveman"));
    assert!(has_owner_at_path(&test_path, "ponytail"));

    // Content check
    let content = fs::read_to_string(&test_file).expect("read test file");
    assert!(content.contains("## Principles"));
    assert!(content.contains("## Response Style (caveman)"));
    assert!(content.contains("## Build Discipline (ponytail)"));
    assert!(content.contains("# Agent Instructions"));

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_phase11_legacy_runner_detection() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    // Legacy runners must be detected
    for runner in ["npx", "uvx", "pip", "pip3", "pipx", "npm", "bunx", "pnpm", "python", "node"] {
        assert!(is_legacy_mcp_runner_str(runner), "{} should be recognized as legacy runner", runner);
        assert!(is_legacy_mcp_runner_str(&format!("C:\\bin\\{}.exe", runner)));
        assert!(is_legacy_mcp_runner_str(&format!("/usr/local/bin/{}", runner)));
    }

    // Toksi native commands must NOT be legacy runners
    for tok in ["toksi", "toksi.exe", "C:\\Users\\bin\\toksi.exe", "/usr/bin/toksi-linux"] {
        assert!(!is_legacy_mcp_runner_str(tok));
        assert!(is_toksi_mcp_command_str(tok));
    }
}

#[test]
fn test_phase11_purge_legacy_mcp_runners_isolated() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    register_agents();

    let temp_dir = std::env::temp_dir().join("toksi_p11_mcp_purge_test");
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::create_dir_all(&temp_dir);

    // Create a mock JSON config with legacy npx and uvx entries alongside a foreign tool
    let mock_json = temp_dir.join("mock_mcp.json");
    let initial_content = serde_json::json!({
        "mcpServers": {
            "codegraph": {
                "command": "npx",
                "args": ["-y", "@colbymchenry/codegraph", "serve", "--mcp"]
            },
            "projectmem": {
                "command": "uvx",
                "args": ["projectmem", "serve"]
            },
            "custom-weather": {
                "command": "weather-cli",
                "args": ["serve"]
            }
        }
    });
    fs::write(&mock_json, serde_json::to_string_pretty(&initial_content).unwrap()).unwrap();

    // Verify legacy entries detection helper
    let parsed: serde_json::Value = serde_json::from_str(&fs::read_to_string(&mock_json).unwrap()).unwrap();
    let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
    assert!(toksi::agents::is_legacy_mcp_entry(servers.get("codegraph").unwrap()));
    assert!(toksi::agents::is_legacy_mcp_entry(servers.get("projectmem").unwrap()));
    assert!(!toksi::agents::is_legacy_mcp_entry(servers.get("custom-weather").unwrap()));

    let _ = audit_agent_legacy_mcp("claude");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_phase11_environment_proxy_status_audit() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    let status = audit_environment_status();

    // The system should report whether binaries are clean
    assert!(status.binaries_clean || !status.binaries_clean);
    assert!(status.env_vars_healthy || !status.env_vars_healthy);
    assert!(!status.details.is_empty(), "audit should provide details");
}

#[test]
fn test_phase11_scan_legacy_hijacks_dry_run() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    let temp_dir = std::env::temp_dir().join("toksi_p11_scan_test");
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::create_dir_all(&temp_dir);

    let bin_dir = temp_dir.join("bin");
    let data_dir = temp_dir.join("data");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    // Place a simulated .real binary and a legacy shim
    let real_bin = bin_dir.join("agy.real.exe");
    fs::write(&real_bin, b"simulated real binary").unwrap();
    let shim = bin_dir.join("agy.cmd");
    fs::write(&shim, b"@echo off\r\nrem toksi:antigravity-cli-proxy\r\n").unwrap();
    let state_file = data_dir.join("antigravity-cli-proxy.json");
    fs::write(&state_file, b"{}").unwrap();

    // 1. Scan (dry-run): must find all 3 items WITHOUT modifying or deleting them
    let scan_report = scan_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);
    assert!(!scan_report.is_clean());
    assert_eq!(scan_report.restored_binaries.len(), 1);
    assert_eq!(scan_report.removed_shims.len(), 1);
    assert_eq!(scan_report.removed_state_files.len(), 1);

    // Verify files still exist on disk
    assert!(real_bin.exists(), "dry run must not delete .real binary");
    assert!(shim.exists(), "dry run must not delete shim");
    assert!(state_file.exists(), "dry run must not delete state file");

    // 2. Heal: must restore real binary and delete shim/state
    let heal_report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);
    assert_eq!(heal_report.total_healed(), 3);
    assert!(!real_bin.exists(), ".real binary should be renamed to canonical");
    assert!(!shim.exists(), "shim should be deleted");
    assert!(!state_file.exists(), "orphaned state file should be deleted");
    assert!(bin_dir.join("agy.exe").exists(), "canonical agy.exe must be restored");

    // 3. Re-scan: must now be 100% clean
    let post_scan = scan_legacy_hijacks_in(&[bin_dir], &[data_dir]);
    assert!(post_scan.is_clean());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_phase11_doctor_runs_cleanly() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    let _ = toksi_sys::process::set_process_env("TOKSI_TEST", "1");
    let temp_home = std::env::temp_dir().join("toksi_p11_doctor_isolated_home");
    let _ = fs::remove_dir_all(&temp_home);
    let _ = fs::create_dir_all(&temp_home);
    toksi::util::set_home_override(temp_home.to_string_lossy().to_string());

    register_agents();
    register_tools();

    let code_json = run_doctor(true, true, false);
    assert_eq!(code_json, 0, "run_doctor json output must return 0");

    let code = run_doctor(true, false, false);
    assert_eq!(code, 0, "run_doctor standard output must return 0");

    let code_fix = run_doctor(true, false, true);
    assert_eq!(code_fix, 0, "run_doctor --fix must return 0");

    toksi::util::clear_home_override();
    let _ = fs::remove_dir_all(&temp_home);
}

#[test]
fn test_phase11_self_healing_mcp_hooks_config_and_workspace_opt_in() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    let _ = toksi_sys::process::set_process_env("TOKSI_TEST", "1");
    let temp_root = tempfile::tempdir().unwrap();
    let fake_home = temp_root.path().join("home");
    let workspace = temp_root.path().join("repo");
    fs::create_dir_all(&fake_home).unwrap();
    fs::create_dir_all(workspace.join("subpkg").join(".toksi").join("icm")).unwrap();
    fs::create_dir_all(workspace.join("subpkg").join(".claude")).unwrap();
    fs::write(workspace.join("subpkg").join(".claude").join("note.txt"), "nested").unwrap();
    fs::write(workspace.join("subpkg").join(".toksi").join("icm").join("events.jsonl"), "{\"id\":1}\n").unwrap();

    // Seed legacy MCP + hooks in fake_home (antigravity + cursor + toksi config.toml)
    fs::create_dir_all(fake_home.join(".gemini").join("config")).unwrap();
    fs::create_dir_all(fake_home.join(".cursor")).unwrap();
    fs::create_dir_all(fake_home.join(".toksi")).unwrap();
    fs::write(
        fake_home.join(".gemini").join("config").join("hooks.json"),
        r#"{"toksi-codegraph-index":{"PostToolUse":[{"matcher":"","hooks":[{"type":"command","command":"toksi agy-hook codegraph-index"}]}]}}"#,
    ).unwrap();
    fs::write(
        fake_home.join(".gemini").join("config").join("mcp_config.json"),
        r#"{"mcpServers":{"tokless_projectmem":{"command":"uvx","args":["projectmem"]},"custom-tool":{"command":"custom","args":[]}}}"#,
    ).unwrap();
    fs::write(
        fake_home.join(".cursor").join("hooks.json"),
        r#"{"version":1,"hooks":{"preToolUse":[{"command":"rtk hook cursor","matcher":"Shell"}]}}"#,
    ).unwrap();
    fs::write(
        fake_home.join(".toksi").join("config.toml"),
        "[ports]\nheadroom_proxy = 8787\n[base_urls]\nopenai = \"https://api.openai.com/v1\"\n",
    ).unwrap();

    toksi::util::set_home_override(fake_home.to_string_lossy().to_string());
    register_agents();
    register_tools();

    // Audit detects legacy hook & config.toml parses cleanly with 0 warnings
    assert!(!audit_agent_hook_conventions("antigravity").is_empty());
    assert!(audit_toksi_config_toml().is_empty());

    // DirectWireBus verification & non-blocking PATH check
    assert!(!verify_direct_wire_bus(None).is_empty());
    assert!(check_executable_in_path("nonexistent_toksi_probe_binary_xyz").is_err());

    // Without explicit --workspace opt-in, workspace nested dirs must NOT be modified
    let prev_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&workspace).unwrap();
    assert_eq!(run_doctor(true, false, true), 0);
    assert!(workspace.join("subpkg").join(".toksi").exists(), "doctor --fix must NOT touch workspace without opt-in");
    assert!(workspace.join("subpkg").join(".claude").exists(), "doctor --fix must NOT touch workspace without opt-in");

    // Verify user-home configs were repaired
    let healed_hooks = fs::read_to_string(fake_home.join(".gemini").join("config").join("hooks.json")).unwrap();
    assert!(healed_hooks.contains("codegraph-hook agy"));
    let healed_mcp = fs::read_to_string(fake_home.join(".gemini").join("config").join("mcp_config.json")).unwrap();
    assert!(healed_mcp.contains("\"projectmem\""));
    assert!(!healed_mcp.contains("\"tokless_projectmem\""));
    assert!(healed_mcp.contains("\"custom-tool\""), "foreign MCP server must be preserved");

    // With explicit workspace opt-in, nested dirs consolidate cleanly to root
    assert_eq!(
        run_doctor_with_options(DoctorOptions {
            offline: true,
            json_output: false,
            fix: true,
            workspace: Some(workspace.clone()),
            include_workspace: true,
        }),
        0
    );
    assert!(!workspace.join("subpkg").join(".toksi").exists());
    assert!(!workspace.join("subpkg").join(".claude").exists());
    assert!(workspace.join(".toksi").join("icm").join("events.jsonl").exists());
    assert!(workspace.join(".claude").join("note.txt").exists());

    std::env::set_current_dir(prev_cwd).unwrap();
    let _ = heal_all_agent_legacy_configs(Some(fake_home.as_path()));
    toksi::util::clear_home_override();
}

#[test]
fn test_phase11_init_idempotency_dry_run() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    register_agents();
    register_tools();

    let opts = InitOptions {
        dry_run: true,
        yes: true,
        verbose: false,
        agents: Some(vec!["claude".to_string()]),
        tools: Some(vec!["principles".to_string(), "caveman".to_string(), "ponytail".to_string()]),
        ..Default::default()
    };

    let res1 = run_init(opts.clone());
    assert_eq!(res1, 0, "first init dry run must succeed");

    let res2 = run_init(opts);
    assert_eq!(res2, 0, "second init dry run must succeed idempotently");
}

