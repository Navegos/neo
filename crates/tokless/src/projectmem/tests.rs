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
    assert_eq!(location_to_file(Some("src/core.rs:120 (OrderService::process)")), Some("src/core.rs".into()));
    assert_eq!(location_to_file(Some("src/core.rs:120:15")), Some("src/core.rs".into()));
    assert_eq!(location_to_file(Some("C:\\project\\src\\main.py:45")), Some("C:\\project\\src\\main.py".into()));
    assert_eq!(location_to_file(Some("C:\\project\\src\\main.py:45 (charge)")), Some("C:\\project\\src\\main.py".into()));
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

    // Record attempt with command and notes
    let att_msg = tool_record_attempt(
        root,
        "MCP test attempt",
        Some("failed"),
        Some("src/test.rs:1"),
        None,
        Some("cargo check"),
        Some("trace: test failed"),
    )
    .unwrap();
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

#[test]
fn test_typed_models_and_constructors() {
    let issue_item = Issue {
        summary: "Null dereference in parser".into(),
        location: Some("src/parser.rs:42".into()),
    };
    let ev_issue: Event = issue_item.into();
    assert_eq!(ev_issue.event_type, "issue");
    assert_eq!(ev_issue.summary, "Null dereference in parser");
    assert_eq!(ev_issue.location, Some("src/parser.rs:42".into()));
    assert!(ev_issue.validate().is_ok());

    let attempt_item = Attempt {
        summary: "Wrapped in Option check".into(),
        outcome: "failed".into(),
        issue_id: Some("0001".into()),
        location: Some("src/parser.rs:42".into()),
        command: Some("cargo test".into()),
    };
    let ev_attempt: Event = attempt_item.into();
    assert_eq!(ev_attempt.event_type, "attempt");
    assert_eq!(ev_attempt.outcome, Some("failed".into()));
    assert_eq!(ev_attempt.command, Some("cargo test".into()));

    let fix_item = Fix {
        summary: "Used match guard".into(),
        issue_id: Some("0001".into()),
        location: Some("src/parser.rs:42".into()),
    };
    let ev_fix: Event = fix_item.into();
    assert_eq!(ev_fix.event_type, "fix");

    let decision_item = Decision {
        summary: "Strict zero-allocation parser design".into(),
        supersedes: Some("evt_1234".into()),
        location: None,
    };
    let ev_decision: Event = decision_item.into();
    assert_eq!(ev_decision.event_type, "decision");
    assert_eq!(ev_decision.supersedes, Some("evt_1234".into()));

    let note_item = Note {
        summary: "gotcha: Windows CRLF line endings in parser".into(),
        location: Some("src/parser.rs".into()),
    };
    let ev_note: Event = note_item.into();
    assert_eq!(ev_note.event_type, "note");

    let convenience_issue = Event::issue("Convenience issue", Some("src/lib.rs:10"));
    assert_eq!(convenience_issue.event_type, "issue");

    let convenience_attempt = Event::attempt("Convenience attempt", "worked", Some("0001"), None, Some("cargo build"));
    assert_eq!(convenience_attempt.outcome, Some("worked".into()));

    let convenience_fix = Event::fix("Convenience fix", Some("0001"), None);
    assert_eq!(convenience_fix.event_type, "fix");

    let convenience_decision = Event::decision("Convenience decision", None, None);
    assert_eq!(convenience_decision.event_type, "decision");

    let convenience_note = Event::note("Convenience note", None);
    assert_eq!(convenience_note.event_type, "note");
}

#[test]
fn test_zero_allocation_redaction() {
    let clean = "This is a clean debugging note without any tokens or keys.";
    let (cow, fired) = super::redaction::redact_cow(clean);
    match cow {
        std::borrow::Cow::Borrowed(b) => assert_eq!(b, clean),
        std::borrow::Cow::Owned(_) => panic!("Expected Borrowed Cow for clean text!"),
    }
    assert!(fired.is_empty());

    let dirty = "Key is sk-1234567890123456789012345678901234567890 here";
    let (cow_dirty, fired_dirty) = super::redaction::redact_cow(dirty);
    match cow_dirty {
        std::borrow::Cow::Borrowed(_) => panic!("Expected Owned Cow for redacted text!"),
        std::borrow::Cow::Owned(o) => assert!(o.contains("[REDACTED:openai_key]")),
    }
    assert_eq!(fired_dirty, vec!["openai_key"]);
}

#[test]
fn test_extract_project_purpose_from_readme() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let readme_content = "# Test Project\n\nHigh-performance streaming engine for real-time analytics.\n\n## Installation\n";
    std::fs::write(root.join("README.md"), readme_content).unwrap();

    let purpose = extract_project_purpose_from_readme(Some(root));
    assert_eq!(
        purpose,
        Some("High-performance streaming engine for real-time analytics.".to_string())
    );
}

#[test]
fn test_rtk_auto_populate_failure_trace() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    initialize(Some(root), Some("RTK Auto-Trace Test")).unwrap();

    let cargo_error_output = "\
   Compiling my-crate v0.1.0\n\
error[E0433]: failed to resolve: use of undeclared crate or module `foo`\n\
 --> src/main.rs:2:5\n\
  |\n\
2 | use foo::bar;\n\
  |     ^^^ use of undeclared crate or module `foo`\n\
error: could not compile `my-crate`";

    // Auto-creates parent issue and extracts root cause into summary while saving full trace in notes
    let msg = tool_record_attempt(
        root,
        cargo_error_output,
        Some("failed"),
        Some("src/main.rs:2"),
        None,
        Some("cargo build"),
        None,
    )
    .unwrap();

    assert!(msg.contains("Recorded failed attempt on #0001"));
    assert!(msg.contains("error[E0433]: failed to resolve"));

    let events = read_events(Some(root)).unwrap();
    assert_eq!(events.len(), 2); // Auto-created issue + attempt
    let attempt_ev = &events[1];
    assert_eq!(attempt_ev.event_type, "attempt");
    assert!(attempt_ev.summary.contains("error[E0433]"));
    assert_eq!(attempt_ev.command, Some("cargo build".to_string()));
    assert!(attempt_ev.notes.as_ref().unwrap().contains("error: could not compile"));
}

#[test]
fn test_cli_all_subcommands_end_to_end() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Use PROJECTMEM_ROOT env so CLI routes directly to tmp dir
    unsafe {
        std::env::set_var("PROJECTMEM_ROOT", root.to_str().unwrap());
    }

    // 1. init
    assert_eq!(run_mem_cli(&["init".into(), "--purpose".into(), "CLI E2E Test".into()]), 0);

    // 2. log
    assert_eq!(run_mem_cli(&["log".into(), "CLI test issue".into(), "--at".into(), "src/main.rs:1".into()]), 0);

    // 3. attempt (with RTK root cause extraction)
    let multiline_err = "running 1 test\ntest test_foo ... FAILED\nthread 'test_foo' panicked at 'assertion failed: left == right'\n";
    assert_eq!(
        run_mem_cli(&[
            "attempt".into(),
            multiline_err.into(),
            "--failed".into(),
            "--command".into(),
            "cargo test".into(),
            "--at".into(),
            "src/main.rs:1".into(),
        ]),
        0
    );

    // 4. fix
    assert_eq!(run_mem_cli(&["fix".into(), "Fixed assertion in test_foo".into()]), 0);

    // 5. decision
    assert_eq!(run_mem_cli(&["decision".into(), "Use assertion macros consistently".into()]), 0);

    // 6. note
    assert_eq!(run_mem_cli(&["note".into(), "gotcha: assertion orders".into()]), 0);

    // 7. summary / show
    assert_eq!(run_mem_cli(&["summary".into()]), 0);
    assert_eq!(run_mem_cli(&["show".into()]), 0);

    // 8. precheck
    assert_eq!(run_mem_cli(&["precheck".into(), "--files".into(), "src/main.rs".into()]), 0);

    // 9. stats / score / heatmap
    assert_eq!(run_mem_cli(&["stats".into()]), 0);
    assert_eq!(run_mem_cli(&["score".into()]), 0);
    assert_eq!(run_mem_cli(&["heatmap".into()]), 0);

    // 10. map / plan / instructions / search / regenerate
    assert_eq!(run_mem_cli(&["map".into()]), 0);
    assert_eq!(run_mem_cli(&["plan".into()]), 0);
    assert_eq!(run_mem_cli(&["instructions".into()]), 0);
    assert_eq!(run_mem_cli(&["search".into(), "assertion".into()]), 0);
    assert_eq!(run_mem_cli(&["regenerate".into()]), 0);

    unsafe {
        std::env::remove_var("PROJECTMEM_ROOT");
    }
}
