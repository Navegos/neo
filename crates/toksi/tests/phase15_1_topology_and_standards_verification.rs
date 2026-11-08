// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase15_1_topology_and_standards_verification.rs
// created: 2026-11-08

use std::fs;
use toksi::agents::hooks::*;
use toksi::agents::mcp_adapters::*;
use toksi::commands::doctor::*;
use toksi::db::*;
use toksi::util::env_path::*;
use toksi::util::paths::*;

#[test]
fn test_phase15_1_forward_slash_normalization() {
    assert_eq!(
        normalize_path(r"C:\Users\developer\.toksi\toksi.db"),
        "C:/Users/developer/.toksi/toksi.db"
    );
    assert_eq!(
        normalize_path(r"d:\AI\neo\crates\toksi"),
        "D:/AI/neo/crates/toksi"
    );
    assert_eq!(
        normalize_path(r"\\?\C:\Projects\app"),
        "C:/Projects/app"
    );
    assert_eq!(
        normalize_path(r"\\server\share\file.txt"),
        "//server/share/file.txt"
    );
    assert_eq!(
        normalize_path("/home/user/.toksi/toksi.db"),
        "/home/user/.toksi/toksi.db"
    );

    let cmd_escaped = escape_path_for_cmd(r"C:\Programs\neo\bin\toksi.exe");
    assert_eq!(cmd_escaped, r"C:\\Programs\\neo\\bin\\toksi.exe");
}

#[test]
fn test_phase15_1_path_validation_mandate() {
    // Missing binary returns exact error format
    let missing = check_executable_in_path("non_existent_binary_phase15_test");
    assert!(missing.is_err());
    let err = missing.err().unwrap_or_default();
    assert!(err.contains("[ERROR] Required executable 'non_existent_binary_phase15_test' not found in system PATH."));
    assert!(err.contains("Please add 'non_existent_binary_phase15_test' to your PATH environment variable and retry."));

    // Cargo is guaranteed to exist when running cargo tests
    let cargo_check = check_executable_in_path("cargo");
    assert!(cargo_check.is_ok());
}

#[test]
fn test_phase15_1_default_sqlite_db_location() {
    let db_path = default_db_path();
    let db_str = normalize_path(&db_path.to_string_lossy());
    assert!(db_str.ends_with("/.toksi/toksi.db"));
    assert!(!db_str.contains("AppData/Local/toksi"));

    let data_dir = toksi_data_dir();
    let norm_dir = normalize_path(&data_dir);
    assert!(norm_dir.ends_with("/.toksi"));
}

#[test]
fn test_phase15_1_workspace_root_consolidation() {
    let temp_root = std::env::temp_dir().join(format!(
        "toksi-workspace-topo-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::create_dir_all(&temp_root);

    // 1. Create subfolder with nested .toksi containing toksi.db
    let sub_crate = temp_root.join("crates").join("toksi");
    let nested_toksi = sub_crate.join(".toksi");
    let _ = fs::create_dir_all(&nested_toksi);

    let sub_db = nested_toksi.join("toksi.db");
    {
        let mut db = ToksiConfigDb::open(&sub_db).expect("create sub db");
        let p = ProviderRecord {
            id: "sub-gemini".to_string(),
            name: "Sub Gemini".to_string(),
            protocol: "google-gemini".to_string(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            is_enabled: true,
            auth_type: "keyring".to_string(),
            metadata_json: "{}".to_string(),
            created_at: 100,
            updated_at: 100,
        };
        db.upsert_provider(&p).expect("upsert provider");
    }

    // 2. Create loose legacy folder .codegraph in workspace
    let loose_cg = temp_root.join(".codegraph");
    let _ = fs::create_dir_all(&loose_cg);
    let cg_file = loose_cg.join("meta.json");
    let _ = fs::write(&cg_file, "{\"index\":true}");

    // 3. Create nested agent dir in subfolder (violating root-only rule)
    let nested_agent = sub_crate.join(".claude");
    let _ = fs::create_dir_all(&nested_agent);
    let agent_cfg = nested_agent.join("config.json");
    let _ = fs::write(&agent_cfg, "{\"model\":\"claude-3\"}");

    // Audit finds nested agent dir
    let warnings = audit_nested_agent_dirs(&temp_root);
    assert!(!warnings.is_empty());

    // Execute consolidation
    let consolidated_toksi = consolidate_nested_toksi_dirs(&temp_root).expect("consolidate toksi");
    assert!(consolidated_toksi >= 1);

    let consolidated_agents = consolidate_nested_agent_dirs(&temp_root).expect("consolidate agents");
    assert!(consolidated_agents >= 1);

    // Verify nested .toksi is deleted
    assert!(!nested_toksi.exists());

    // Verify loose .codegraph migrated to .toksi/codegraph
    assert!(!loose_cg.exists());
    let root_cg = temp_root.join(".toksi").join("codegraph").join("meta.json");
    assert!(root_cg.exists());

    // Verify root SQLite database received rows from subfolder .toksi
    let root_db_path = temp_root.join(".toksi").join("toksi.db");
    assert!(root_db_path.exists());
    let root_db = ToksiConfigDb::open(&root_db_path).expect("open root db");
    let rec = root_db.get_provider("sub-gemini").expect("get provider").expect("found");
    assert_eq!(rec.name, "Sub Gemini");

    // Verify nested .claude moved to root .claude
    assert!(!nested_agent.exists());
    let root_agent_cfg = temp_root.join(".claude").join("config.json");
    assert!(root_agent_cfg.exists());

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_phase15_1_standard_tool_hooks_and_mcp_naming() {
    // 1. Uniform hook commands: <tool>-hook <agent>
    assert_eq!(standard_hook_command("codegraph", "agy"), vec!["codegraph-hook", "agy"]);
    assert_eq!(standard_hook_command("projectmem", "copilot"), vec!["projectmem-hook", "copilot"]);
    assert_eq!(standard_hook_command("context-mode", "cursor"), vec!["context-hook", "cursor"]);
    assert_eq!(standard_hook_command("rtk", "claude"), vec!["rtk-hook", "claude"]);

    // 2. Normalized MCP tool names
    assert_eq!(normalize_mcp_tool_id("toksi-projectmem"), "projectmem");
    assert_eq!(normalize_mcp_tool_id("toksi-context-mode"), "context");
    assert_eq!(normalize_mcp_tool_id("context-mode"), "context");
    assert_eq!(normalize_mcp_tool_id("toksi-codegraph-index"), "codegraph");

    // 3. MCP settings to skills mapping
    assert_eq!(agent_mcp_to_skills_path(".gemini", "codegraph"), ".gemini/skills/codegraph");
    assert_eq!(agent_mcp_to_skills_path(".copilot", "context"), ".copilot/skills/context");
    assert_eq!(agent_mcp_to_skills_path(".claude", "projectmem"), ".claude/skills/projectmem");

    // 4. Agent taxonomy: CLI vs Editor/IDE
    assert!(is_agent_cli("antigravity-cli"));
    assert!(is_agent_cli("agy"));
    assert!(is_agent_cli("copilot-cli"));
    assert!(is_agent_cli("claude-cli"));
    assert!(is_agent_cli("cursor-cli"));
    assert!(is_agent_cli("grok-cli"));

    assert!(is_agent_ide("antigravity-ide"));
    assert!(is_agent_ide("copilot-vscode"));
    assert!(is_agent_ide("claude-desktop"));
    assert!(is_agent_ide("cursor-ide"));
    assert!(is_agent_ide("grok-ide"));
}
