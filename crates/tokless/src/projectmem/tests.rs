// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/tests.rs
// created: 2026-10-27

use chrono::Utc;
use tempfile::tempdir;

use super::cli::run_mem_cli;
use super::heatmap::{calculate_score, generate_heatmap};
use super::mcp::*;
use super::precheck::{
    active_snooze, analyze_files, clear_snooze, parse_snooze_duration, set_snooze,
};
use super::redaction::{redact, redact_event_fields};
use super::schema::*;
use super::store::*;
use super::summary::*;

#[test]
fn test_schema_models_and_resolution() {
    let mut ev = Event::new("issue", "Cannot connect to database");
    assert!(ev.id.starts_with("evt_"));
    assert_eq!(ev.id.len(), 24);
    assert_eq!(ev.event_type, "issue");
    assert_eq!(ev.summary, "Cannot connect to database");
    assert!(ev.validate().is_ok());

    ev.outcome = Some("invalid_outcome".into());
    assert!(ev.validate().is_err());
    ev.outcome = Some("failed".into());
    assert!(ev.validate().is_ok());

    let mut e1 = Event::new("decision", "Use PostgreSQL");
    e1.id = "evt_11111111111111111111".into();
    let mut e2 = Event::new("decision", "Use SQLite");
    e2.id = "evt_22222222222222222222".into();
    e2.supersedes = Some(e1.id.clone());

    let events = vec![e1.clone(), e2.clone()];
    let superseded = superseded_ids(&events);
    assert!(superseded.contains(&e1.id));
    assert_eq!(superseded.len(), 1);

    // Reference resolution
    assert_eq!(resolve_event_ref(&events, &e1.id).unwrap().summary, "Use PostgreSQL");
    assert_eq!(resolve_event_ref(&events, "11111").unwrap().summary, "Use PostgreSQL");
    assert_eq!(resolve_event_ref(&events, "22222").unwrap().summary, "Use SQLite");
    assert!(resolve_event_ref(&events, "nonexistent").is_err());
}

#[test]
fn test_location_to_file_parsing() {
    assert_eq!(location_to_file(Some("src/core.rs")), Some("src/core.rs".into()));
    assert_eq!(location_to_file(Some("src/core.rs:120")), Some("src/core.rs".into()));
    assert_eq!(location_to_file(Some("src/core.rs:120:15")), Some("src/core.rs".into()));
    assert_eq!(location_to_file(Some("C:\\project\\src\\main.py:45")), Some("C:\\project\\src\\main.py".into()));
    assert_eq!(location_to_file(Some("D:/AI/neo/crates/tokless/Cargo.toml")), Some("D:/AI/neo/crates/tokless/Cargo.toml".into()));
    assert_eq!(location_to_file(Some("main.rs")), Some("main.rs".into()));
    assert_eq!(location_to_file(Some("app.py")), Some("app.py".into()));
    assert_eq!(location_to_file(Some("ClassName.method_name")), None);
    assert_eq!(location_to_file(Some("")), None);
}

#[test]
fn test_timestamp_canonicalization() {
    let raw = "2026-05-12 21:07:46 -0600";
    let norm = normalize_timestamp(Some(raw));
    assert!(norm.ends_with('Z'));
    assert_eq!(norm, "2026-05-13T03:07:46Z");

    let iso = "2026-10-27T12:00:00Z";
    assert_eq!(normalize_timestamp(Some(iso)), iso);
}

#[test]
fn test_secret_redaction() {
    let test_str = "Keys: sk-abcdefghijklmnopqrstuvwxyz1234567890abcdef and AKIA1234567890123456 and ghp_123456789012345678901234567890123456 and Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let (redacted, fired) = redact(test_str);
    assert!(redacted.contains("[REDACTED:openai_key]"));
    assert!(redacted.contains("[REDACTED:aws_access_key]"));
    assert!(redacted.contains("[REDACTED:github_token]"));
    assert!(!fired.is_empty());

    let mut ev = Event::new("note", "secret openai key is sk-1234567890123456789012345678901234567890");
    let event_fired = redact_event_fields(&mut ev);
    assert_eq!(event_fired, vec!["openai_key"]);
    assert_eq!(ev.summary, "secret openai key is [REDACTED:openai_key]");
}

#[test]
fn test_store_initialization_and_events() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let mem = initialize(Some(root), Some("Tokless ProjectMem Rust Port")).unwrap();
    assert!(mem.exists());
    assert!(mem.join(CONFIG_FILE).exists());
    assert!(mem.join(EVENTS_FILE).exists());
    assert!(mem.join(SUMMARY_FILE).exists());
    assert!(mem.join(PROJECT_MAP_FILE).exists());
    assert!(mem.join(PLAN_FILE).exists());
    assert!(mem.join(AI_INSTRUCTIONS_FILE).exists());

    // Write events and verify atomic appends
    let mut e1 = Event::new("issue", "First Issue");
    e1.issue_id = Some("0001".into());
    append_event(&mut e1, Some(root)).unwrap();

    let mut e2 = Event::new("attempt", "Failed first try");
    e2.issue_id = Some("0001".into());
    e2.outcome = Some("failed".into());
    append_event(&mut e2, Some(root)).unwrap();

    let mut e3 = Event::new("fix", "Working fix confirmed");
    e3.issue_id = Some("0001".into());
    append_event(&mut e3, Some(root)).unwrap();

    let events = read_events(Some(root)).unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(next_issue_id(&events), "0002");
    assert!(current_issue_id(&events).is_none());

    // Active issue marker
    write_current_issue("0002", Some(root));
    assert_eq!(read_current_issue(Some(root)), Some("0002".into()));
    clear_current_issue(Some(root));
    assert_eq!(read_current_issue(Some(root)), None);
}

#[test]
fn test_summary_regeneration_and_limits() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), Some("High Performance Core")).unwrap();

    let mut events = Vec::new();
    for i in 1..=25 {
        let iid = format!("{:04}", i);
        let mut iss = Event::new("issue", &format!("Issue {}", i));
        iss.issue_id = Some(iid.clone());
        events.push(iss);

        if i <= 15 {
            let mut fx = Event::new("fix", &format!("Fixed {}", i));
            fx.issue_id = Some(iid);
            events.push(fx);
        } else {
            let mut att = Event::new("attempt", &format!("Attempt {}", i));
            att.issue_id = Some(iid);
            att.outcome = Some("failed".into());
            events.push(att);
        }
    }

    for i in 1..=20 {
        events.push(Event::new("decision", &format!("Decision {}", i)));
    }

    let summary = build_summary(&events, "tokless", Some("High Performance Core"));
    assert!(summary.contains("# projectmem - tokless"));
    assert!(summary.contains("High Performance Core"));
    assert!(summary.contains("## Recent issues"));
    assert!(summary.contains("## Decisions"));
    assert!(summary.contains("earlier decision"));
    assert!(summary.contains("older fixed issue"));
}

#[test]
fn test_precheck_warnings_and_snooze() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), None).unwrap();

    let mut e1 = Event::new("issue", "Parser panic");
    e1.issue_id = Some("0001".into());
    e1.location = Some("src/parser.rs:50".into());

    let mut e2 = Event::new("attempt", "Approach 1 failed");
    e2.issue_id = Some("0001".into());
    e2.outcome = Some("failed".into());
    e2.location = Some("src/parser.rs:50".into());

    let mut e3 = Event::new("attempt", "Approach 2 failed");
    e3.issue_id = Some("0001".into());
    e3.outcome = Some("failed".into());
    e3.location = Some("src/parser.rs:50".into());

    let mut e4 = Event::new("attempt", "Approach 3 failed");
    e4.issue_id = Some("0001".into());
    e4.outcome = Some("failed".into());
    e4.location = Some("src/parser.rs:50".into());

    let events = vec![e1, e2, e3, e4];
    let warnings = analyze_files(&["src/parser.rs".to_string()], &events, Some(root));
    assert!(!warnings.is_empty());
    let failed_w = warnings.iter().find(|w| w.warning_type == "failed_attempts").unwrap();
    assert_eq!(failed_w.severity, "block");
    assert_eq!(failed_w.details.len(), 3);

    // Snooze
    assert!(parse_snooze_duration("30m").is_ok());
    assert!(parse_snooze_duration("2h").is_ok());
    let expiry = set_snooze("1h", Some(root)).unwrap();
    assert!(expiry > Utc::now());
    assert!(active_snooze(Some(root)).is_some());
    assert!(clear_snooze(Some(root)));
    assert!(active_snooze(Some(root)).is_none());
}

#[test]
fn test_heatmap_and_score_metrics() {
    let mut e1 = Event::new("issue", "Memory leak");
    e1.issue_id = Some("0001".into());
    e1.location = Some("src/alloc.rs:20".into());

    let mut e2 = Event::new("attempt", "Free called too late");
    e2.issue_id = Some("0001".into());
    e2.outcome = Some("failed".into());
    e2.location = Some("src/alloc.rs:20".into());

    let mut e3 = Event::new("fix", "Used Box::leak appropriately");
    e3.issue_id = Some("0001".into());
    e3.location = Some("src/alloc.rs:20".into());

    let mut e4 = Event::new("decision", "Custom memory pool architecture");
    e4.location = Some("src/alloc.rs:20".into());

    let events = vec![e1, e2, e3, e4];
    let score = calculate_score(&events, None);

    assert_eq!(score.components.failed_approaches, 1);
    assert_eq!(score.components.fixes_with_context, 1);
    assert_eq!(score.components.decisions_documented, 1);
    assert!(score.value.tokens_saved >= 5500);
    assert!(score.value.debugging_hours_saved >= 0.9);
    assert!(score.value.usd_saved >= 0.05);

    let heatmap = generate_heatmap(&events);
    assert_eq!(heatmap.top_hotspots.len(), 1);
    assert_eq!(heatmap.top_hotspots[0].file, "src/alloc.rs");
    assert_eq!(heatmap.top_hotspots[0].failed_attempts, 1);
    assert_eq!(heatmap.directory_clusters.get("src"), Some(&4));
}

#[test]
fn test_mcp_all_tools() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), Some("MCP Parity Suite")).unwrap();

    assert!(tool_get_instructions(root).contains("projectmem AI Instructions"));
    assert!(tool_get_summary(root).contains("MCP Parity Suite"));
    assert!(tool_get_project_map(root).contains("Project Map"));
    assert!(tool_get_plan(root).contains("plan"));

    // Log issue
    let log_msg = tool_log_issue(root, "MCP test issue", Some("src/test.rs:1")).unwrap();
    assert!(log_msg.contains("Logged issue #0001"));

    // Record attempt
    let att_msg = tool_record_attempt(root, "MCP test attempt", Some("failed"), Some("src/test.rs:1"), None).unwrap();
    assert!(att_msg.contains("Recorded failed attempt on #0001"));

    // Precheck file
    let precheck_msg = tool_precheck_file(root, "src/test.rs");
    assert!(precheck_msg.contains("MCP test attempt"));

    // Search
    let search_msg = tool_search_events(root, "test issue", 10);
    assert!(search_msg.contains("MCP test issue"));

    // Get issue
    let issue_msg = tool_get_issue(root, "0001");
    assert!(issue_msg.contains("MCP test issue"));

    // Record fix
    let fix_msg = tool_record_fix(root, "MCP test fix confirmed", Some("src/test.rs:1"), None).unwrap();
    assert!(fix_msg.contains("Fixed issue #0001"));

    // Add decision
    let dec_msg = tool_add_decision(root, "MCP decision choice", None, None).unwrap();
    assert!(dec_msg.contains("Recorded decision"));

    // Add note
    let note_msg = tool_add_note(root, "gotcha: MCP stdio buffering", None).unwrap();
    assert!(note_msg.contains("Recorded note"));

    // Score & Context
    let score_msg = tool_get_score(root);
    assert!(score_msg.contains("Prevention Score"));
    let ctx_msg = tool_get_context(root, 2000, None);
    assert!(ctx_msg.contains("projectmem context"));

    // Tools list
    let list = mcp_tools_list();
    assert_eq!(list.as_array().unwrap().len(), 17);
}

#[test]
fn test_cli_execution_suite() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    assert_eq!(run_mem_cli(&["init".into(), "--purpose".into(), "CLI test".into()]), 0);
    assert_eq!(run_mem_cli(&["help".into()]), 0);
    assert!(root.exists());
}
