// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase2_projectmem_verification.rs
// created: 2026-10-30
// lastModified: 2026-11-05

use tempfile::tempdir;
use toksi::projectmem::*;

#[test]
fn test_phase2_schema_and_redaction_verification() {
    let raw_text = "API key sk-abcdef1234567890abcdef1234567890abcdef12 and secret";
    let (redacted, fired) = redact(raw_text);
    assert!(redacted.contains("[REDACTED:openai_key]"));
    assert_eq!(fired, vec!["openai_key"]);

    let mut ev = Event::new("issue", "Bug when calling endpoint");
    assert!(ev.id.starts_with("evt_"));
    assert_eq!(ev.event_type, "issue");
    assert!(ev.validate().is_ok());

    ev.outcome = Some("worked".to_string());
    assert!(ev.validate().is_ok());
    ev.outcome = Some("invalid_outcome".to_string());
    assert!(ev.validate().is_err());
}

#[test]
fn test_phase2_store_summary_and_precheck_verification() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // 1. Initialize store
    let mem_dir = initialize(Some(root), Some("Phase 2 Verification Engine")).unwrap();
    assert!(mem_dir.exists());

    // 2. Log issue
    let mut ev_issue = Event::new("issue", "Database connection timeout in pool");
    ev_issue.issue_id = Some("0001".to_string());
    ev_issue.location = Some("src/db.rs:42".to_string());
    append_event(&mut ev_issue, Some(root)).unwrap();

    // 3. Record attempt
    let mut ev_attempt = Event::new("attempt", "Increased pool timeout from 5s to 30s");
    ev_attempt.issue_id = Some("0001".to_string());
    ev_attempt.outcome = Some("failed".to_string());
    ev_attempt.location = Some("src/db.rs:42".to_string());
    append_event(&mut ev_attempt, Some(root)).unwrap();

    // 4. Record fix
    let mut ev_fix = Event::new("fix", "Reset idle connections before checkout");
    ev_fix.issue_id = Some("0001".to_string());
    ev_fix.location = Some("src/db.rs:42".to_string());
    append_event(&mut ev_fix, Some(root)).unwrap();

    // 5. Add decision with supersedes
    let mut ev_dec1 = Event::new("decision", "Use deadpool for postgres pooling");
    ev_dec1.id = "evt_dec00000000000000001".to_string();
    append_event(&mut ev_dec1, Some(root)).unwrap();

    let mut ev_dec2 = Event::new("decision", "Switch to sqlx built-in pool");
    ev_dec2.id = "evt_dec00000000000000002".to_string();
    ev_dec2.supersedes = Some("evt_dec00000000000000001".to_string());
    append_event(&mut ev_dec2, Some(root)).unwrap();

    // 6. Add gotcha note
    let mut ev_note = Event::new("note", "gotcha: postgres SSL mode require fails without root cert");
    ev_note.location = Some("src/db.rs:10".to_string());
    append_event(&mut ev_note, Some(root)).unwrap();

    // 7. Regenerate deterministic summary
    let sum_path = regenerate_summary(Some(root)).unwrap();
    assert!(sum_path.exists());
    let summary_md = std::fs::read_to_string(&sum_path).unwrap();
    assert!(summary_md.contains("Phase 2 Verification Engine"));
    assert!(summary_md.contains("- [DONE] #0001 Database connection timeout in pool [src/db.rs:42] (fixed)"));
    assert!(summary_md.contains("Switch to sqlx built-in pool"));
    // Superseded decision must drop out of summary
    assert!(!summary_md.contains("Use deadpool for postgres pooling"));

    // 8. Precheck
    let warnings = run_precheck(Some(&["src/db.rs".to_string()]), false, None, true, Some(root)).unwrap();
    assert!(!warnings.is_empty());
    assert!(warnings.iter().any(|w| w.file == "src/db.rs"));

    // 9. Heatmap and ROI stats
    let events = read_events(Some(root)).unwrap();
    let score = calculate_score(&events, None);
    assert_eq!(score.components.failed_approaches, 1);
    assert_eq!(score.components.fixes_with_context, 1);
    assert!(score.value.tokens_saved > 0);
    assert!(score.value.usd_saved > 0.0);
}

#[test]
fn test_phase2_rtk_integration_and_mcp_verification() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), Some("Phase 2 RTK + MCP Verification")).unwrap();

    // 1. In-process MCP tool: log_issue
    let msg1 = mcp::tool_log_issue(root, "Build failure on missing trait", Some("src/lib.rs:5")).unwrap();
    assert!(msg1.contains("Logged issue #0001"));

    // 2. In-process MCP tool: record_attempt connected to RTK
    let compiler_trace = "\
   Compiling demo v0.1.0\n\
error[E0277]: the trait bound `MyType: Clone` is not satisfied\n\
 --> src/lib.rs:5:10\n\
  |\n\
5 | #[derive(Clone)]\n\
  |          ^^^^^ the trait `Clone` is not implemented for `Inner`\n\
error: could not compile `demo`";

    let msg2 = mcp::tool_record_attempt(
        root,
        compiler_trace,
        Some("failed"),
        Some("src/lib.rs:5"),
        Some("0001"),
        Some("cargo check"),
        None,
    )
    .unwrap();

    assert!(msg2.contains("Recorded failed attempt on #0001"));
    assert!(msg2.contains("error[E0277]: the trait bound"));

    // Verify RTK extracted root cause into attempt summary while preserving full notes
    let events = read_events(Some(root)).unwrap();
    let attempt_ev = events.iter().find(|e| e.event_type == "attempt").unwrap();
    assert!(attempt_ev.summary.contains("error[E0277]"));
    assert!(attempt_ev.notes.as_ref().unwrap().contains("could not compile `demo`"));

    // 3. In-process MCP tool: record_fix
    let msg3 = mcp::tool_record_fix(root, "Implemented Clone for Inner", Some("src/lib.rs:5"), Some("0001")).unwrap();
    assert!(msg3.contains("Fixed issue #0001"));

    // 4. In-process MCP tool: add_decision
    let msg4 = mcp::tool_add_decision(root, "Use derive macros where possible", Some("src/lib.rs:1"), None).unwrap();
    assert!(msg4.contains("Recorded decision"));

    // 5. In-process MCP tool: add_note
    let msg5 = mcp::tool_add_note(root, "gotcha: derive(Clone) requires all fields to implement Clone", Some("src/lib.rs:1")).unwrap();
    assert!(msg5.contains("Recorded note"));

    // 6. JSON-RPC MCP message handler
    let rpc_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "get_summary",
            "arguments": {
                "project": root.to_str().unwrap()
            }
        }
    });

    let resp = mcp::handle_mcp_message(&rpc_req).unwrap();
    assert_eq!(resp["id"], 1);
    let content = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(content.contains("- [DONE] #0001 Build failure on missing trait"));
}

#[test]
fn test_phase2_global_cross_project_memory() {
    let tmp = tempdir().unwrap();
    let global_home = tmp.path().join("global_home");
    std::fs::create_dir_all(&global_home).unwrap();

    global::set_global_dir_override(Some(global_home.clone()));

    // 1. Detect project stack from manifest
    let proj = tmp.path().join("rust_web_project");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(
        proj.join("Cargo.toml"),
        "[package]\nname = \"my_web\"\n[dependencies]\naxum = \"0.7\"\ntokio = \"1.0\"\n",
    )
    .unwrap();

    let stack = global::detect_stack(&proj);
    assert!(stack.tags.contains(&"rust".to_string()));
    assert!(stack.tags.contains(&"axum".to_string()));
    assert!(stack.libraries.contains(&"axum".to_string()));
    assert!(stack.libraries.contains(&"tokio".to_string()));

    // 2. Auto-promote cross-project gotcha
    let promoted = global::auto_promote_event(
        "gotcha: axum extractors order matters — body extractor must be last",
        "decision",
        "rust_web_project",
        None,
        Some(&stack.libraries),
        None,
    );
    assert!(promoted.is_some());
    let gotcha = promoted.unwrap();
    assert_eq!(gotcha.library, "axum");

    // 3. Query global gotchas via MCP
    let res = mcp::tool_get_global_gotchas(Some("axum"));
    assert!(res.contains("Global gotchas (1):"));
    assert!(res.contains("body extractor must be last"));

    global::set_global_dir_override(None);
}

#[test]
fn test_phase2_rtk_failure_trace_autopopulate() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), Some("Phase 2 RTK Auto-Populate Test")).unwrap();

    // 1. Manually write a LastFailureTrace as if RTK captured a failed test
    let trace = LastFailureTrace {
        command: "cargo test --test integration".to_string(),
        root_cause: "error[E0433]: failed to resolve: use of undeclared crate `tokio`".to_string(),
        raw_output: "   Compiling toksi v0.1.0\nerror[E0433]: failed to resolve: use of undeclared crate `tokio`\n --> tests/int.rs:2:5".to_string(),
        timestamp: utc_now_iso(),
    };
    write_last_failure_trace(&trace, Some(root)).unwrap();

    // Verify it is readable
    let loaded = read_last_failure_trace(Some(root)).unwrap();
    assert_eq!(loaded.command, "cargo test --test integration");
    assert!(loaded.root_cause.contains("error[E0433]"));

    // 2. Call tool_record_attempt with empty summary to test auto-population
    let msg = mcp::tool_record_attempt(
        root,
        "",
        Some("failed"),
        Some("tests/int.rs:2"),
        None,
        None,
        None,
    )
    .unwrap();

    assert!(msg.contains("Recorded failed attempt"));
    assert!(msg.contains("error[E0433]"));

    let events = read_events(Some(root)).unwrap();
    let att = events.iter().find(|e| e.event_type == "attempt").unwrap();
    assert!(att.summary.contains("error[E0433]"));
    assert_eq!(att.command.as_deref(), Some("cargo test --test integration"));
    assert!(att.notes.as_ref().unwrap().contains("Compiling toksi v0.1.0"));
}

#[test]
fn test_phase2_python_v035_parity_edge_cases() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), Some("Phase 2 Parity Check")).unwrap();

    // 1. A blank location on an issue falls back to fix's location in summary
    let mut ev_issue = Event::new("issue", "blank location fallback check");
    ev_issue.issue_id = Some("0001".to_string());
    ev_issue.location = Some("   ".to_string());
    append_event(&mut ev_issue, Some(root)).unwrap();

    let mut ev_fix = Event::new("fix", "patched properly");
    ev_fix.issue_id = Some("0001".to_string());
    ev_fix.location = Some("src/fixed.rs".to_string());
    append_event(&mut ev_fix, Some(root)).unwrap();

    let sum_path = regenerate_summary(Some(root)).unwrap();
    let summary_md = std::fs::read_to_string(&sum_path).unwrap();
    assert!(summary_md.contains("- [DONE] #0001 blank location fallback check [src/fixed.rs] (fixed)"));
    assert!(!summary_md.contains("[]"));

    // 2. A blank path is not a file and yields no warnings in precheck
    for blank in ["", "   ", "\n", "\t"] {
        let resp = mcp::tool_precheck_file(root, blank);
        assert!(resp.contains("no warnings. Safe to modify."));
    }

    // 3. MCP tool descriptions must state length limits
    let rpc_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let resp = mcp::handle_mcp_message(&rpc_req).unwrap();
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool_name in ["log_issue", "record_attempt", "record_fix", "add_decision", "add_note"] {
        let tool = tools.iter().find(|t| t["name"] == tool_name).expect("Tool must exist");
        let desc = tool["inputSchema"]["properties"]["summary"]["description"].as_str().unwrap();
        assert!(
            desc.contains("under 140 characters") || desc.contains("under 300 characters"),
            "Tool {} summary description must state character limits: {}",
            tool_name,
            desc
        );
    }
}
