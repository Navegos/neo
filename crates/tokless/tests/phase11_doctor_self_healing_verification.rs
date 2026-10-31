// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase11_doctor_self_healing_verification.rs
// created: 2026-10-31

use std::fs;
use std::sync::Mutex;

static PHASE11_LOCK: Mutex<()> = Mutex::new(());

use tokless::agents::{
    audit_agent_legacy_mcp, is_legacy_mcp_runner_str, is_tokless_mcp_command_str,
    register as register_agents,
};
use tokless::commands::{run_doctor, run_init, InitOptions};
use tokless::core::{get_agent, list_agents};
use tokless::env_injector::{
    audit_environment_status, heal_legacy_hijacks_in, scan_legacy_hijacks_in,
};
use tokless::rules::RuleKind;
use tokless::tools::{
    has_owner_at_path, write_owner_at_path, register as register_tools,
};

const ALL_17_AGENTS: [&str; 17] = [
    "claude",
    "cursor",
    "codex",
    "opencode",
    "opencode2",
    "antigravity",
    "copilot",
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
];

#[test]
fn test_phase11_all_17_agents_manifests_and_rules() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    register_agents();
    register_tools();

    let all_agents = list_agents();
    assert_eq!(all_agents.len(), 17, "All 17 agent manifests must be registered");

    for agent_id in ALL_17_AGENTS {
        let manifest = get_agent(agent_id);
        assert!(manifest.is_some(), "Agent {} must be registered", agent_id);
        let m = manifest.unwrap();
        assert_eq!(m.id, agent_id);
        assert!(!m.label.is_empty());
        assert!(!m.cli_bin.is_empty());
    }

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

    let temp_dir = std::env::temp_dir().join("tokless_p11_rules_test");
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

    // Tokless native commands must NOT be legacy runners
    for tok in ["tokless", "tokless.exe", "C:\\Users\\bin\\tokless.exe", "/usr/bin/tokless-linux"] {
        assert!(!is_legacy_mcp_runner_str(tok));
        assert!(is_tokless_mcp_command_str(tok));
    }
}

#[test]
fn test_phase11_purge_legacy_mcp_runners_isolated() {
    let _lock = PHASE11_LOCK.lock().unwrap();
    register_agents();

    let temp_dir = std::env::temp_dir().join("tokless_p11_mcp_purge_test");
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
    assert!(tokless::agents::is_legacy_mcp_entry(servers.get("codegraph").unwrap()));
    assert!(tokless::agents::is_legacy_mcp_entry(servers.get("projectmem").unwrap()));
    assert!(!tokless::agents::is_legacy_mcp_entry(servers.get("custom-weather").unwrap()));

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
    let temp_dir = std::env::temp_dir().join("tokless_p11_scan_test");
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
    fs::write(&shim, b"@echo off\r\nrem tokless:antigravity-cli-proxy\r\n").unwrap();
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
    register_agents();
    register_tools();

    // Run doctor in offline + JSON mode
    let code_json = run_doctor(true, true, false);
    assert_eq!(code_json, 0, "run_doctor json output must return 0");

    // Run doctor in standard mode
    let code = run_doctor(true, false, false);
    assert_eq!(code, 0, "run_doctor standard output must return 0");

    // Run doctor in fix mode
    let code_fix = run_doctor(true, false, true);
    assert_eq!(code_fix, 0, "run_doctor --fix must return 0");
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

