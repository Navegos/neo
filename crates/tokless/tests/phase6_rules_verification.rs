// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase6_rules_verification.rs

use std::fs;
use tempfile::tempdir;
use tokless::rules::{
    content_has_rule, has_rule_at_path, inject_rule_at_path, inject_rule_into_content,
    inspect_agent_rules, patch_agents_md, patch_all_agent_configs, patch_claude_md,
    patch_cursor_rules, remove_rule_from_content, remove_rule_from_path,
    render_rules_markdown, run_rules_cli, RuleKind,
};

#[test]
fn test_phase6_rule_kinds_completeness() {
    let all = RuleKind::all();
    assert_eq!(all.len(), 3);
    assert!(all.contains(&RuleKind::Principles));
    assert!(all.contains(&RuleKind::Caveman));
    assert!(all.contains(&RuleKind::Ponytail));

    // String mappings
    assert_eq!(RuleKind::from_str("principles"), Some(RuleKind::Principles));
    assert_eq!(RuleKind::from_str("karpathy"), Some(RuleKind::Principles));
    assert_eq!(RuleKind::from_str("karpathy-guidelines"), Some(RuleKind::Principles));

    assert_eq!(RuleKind::from_str("caveman"), Some(RuleKind::Caveman));
    assert_eq!(RuleKind::from_str("response-style"), Some(RuleKind::Caveman));

    assert_eq!(RuleKind::from_str("ponytail"), Some(RuleKind::Ponytail));
    assert_eq!(RuleKind::from_str("build-discipline"), Some(RuleKind::Ponytail));

    assert_eq!(RuleKind::from_str("unknown-rule"), None);
}

#[test]
fn test_phase6_embedded_assets_compiled_via_include_str() {
    for rule in RuleKind::all() {
        let md = rule.markdown();
        let cursor = rule.cursor_rule();
        let body = rule.section_body();

        // Guaranteed non-empty embedded strings
        assert!(!md.is_empty(), "Rule {} markdown must not be empty", rule);
        assert!(!cursor.is_empty(), "Rule {} cursor rule must not be empty", rule);
        assert!(!body.is_empty(), "Rule {} body must not be empty", rule);

        // Cursor rules have valid YAML frontmatter
        assert!(cursor.starts_with("---\n"), "Rule {} cursor rule must start with frontmatter", rule);
        assert!(cursor.contains("alwaysApply: true"));
        assert!(cursor.contains("description:"));

        // No external repository clone links in the rules
        assert!(!md.contains("git clone"), "Rule {} markdown must not contain git clone", rule);
        assert!(!cursor.contains("git clone"), "Rule {} cursor rule must not contain git clone", rule);
    }

    // Specific content checks
    let principles = RuleKind::Principles;
    assert!(principles.markdown().contains("Think Before Coding"));
    assert!(principles.markdown().contains("Simplicity First"));
    assert!(principles.markdown().contains("Surgical Changes"));
    assert!(principles.markdown().contains("Goal-Driven Execution"));

    let caveman = RuleKind::Caveman;
    assert!(caveman.markdown().contains("ASD-STE100"));
    assert!(caveman.markdown().contains("Respond terse like smart caveman"));

    let ponytail = RuleKind::Ponytail;
    assert!(ponytail.markdown().contains("The Ladder"));
    assert!(ponytail.markdown().contains("Does this need to exist at all?"));
}

#[test]
fn test_phase6_content_injection_and_removal_cycle() {
    let original = "# My Custom Project\n\nCustom prompt instructions for the team.\n";

    // 1. Inject Principles
    let with_principles = inject_rule_into_content(original, RuleKind::Principles);
    assert!(content_has_rule(&with_principles, RuleKind::Principles));
    assert!(with_principles.contains("# My Custom Project"));
    assert!(with_principles.contains("Custom prompt instructions for the team."));

    // 2. Inject Ponytail
    let with_ponytail = inject_rule_into_content(&with_principles, RuleKind::Ponytail);
    assert!(content_has_rule(&with_ponytail, RuleKind::Principles));
    assert!(content_has_rule(&with_ponytail, RuleKind::Ponytail));

    // 3. Idempotency: injecting Ponytail again changes nothing
    let idempotency = inject_rule_into_content(&with_ponytail, RuleKind::Ponytail);
    assert_eq!(idempotency, with_ponytail);

    // 4. Remove Principles
    let without_principles = remove_rule_from_content(&with_ponytail, RuleKind::Principles);
    assert!(!content_has_rule(&without_principles, RuleKind::Principles));
    assert!(content_has_rule(&without_principles, RuleKind::Ponytail));
    assert!(without_principles.contains("# My Custom Project"));

    // 5. Remove Ponytail
    let empty_rules = remove_rule_from_content(&without_principles, RuleKind::Ponytail);
    assert!(!content_has_rule(&empty_rules, RuleKind::Ponytail));
    assert!(empty_rules.contains("# My Custom Project"));
}

#[test]
fn test_phase6_claude_md_patching_and_preservation() {
    let tmp = tempdir().unwrap();
    let claude_file = tmp.path().join("CLAUDE.md");

    // Initial user content
    fs::write(&claude_file, "# Repository Overview\n\nImportant architecture notes.\n").unwrap();

    // Patch with Principles & Caveman
    let modified = patch_claude_md(&claude_file, &[RuleKind::Principles, RuleKind::Caveman]).unwrap();
    assert!(modified);

    let content = fs::read_to_string(&claude_file).unwrap();
    assert!(content.contains("# Repository Overview"));
    assert!(content.contains("Important architecture notes."));
    assert!(content.contains("## Principles"));
    assert!(content.contains("## Response Style (caveman)"));
    assert!(!content.contains("## Build Discipline (ponytail)"));

    // Idempotency: re-running returns false
    let modified_again = patch_claude_md(&claude_file, &[RuleKind::Principles, RuleKind::Caveman]).unwrap();
    assert!(!modified_again);

    // Add Ponytail
    let modified_add = patch_claude_md(&claude_file, &[RuleKind::Ponytail]).unwrap();
    assert!(modified_add);
    let content_all = fs::read_to_string(&claude_file).unwrap();
    assert!(content_all.contains("## Build Discipline (ponytail)"));
}

#[test]
fn test_phase6_agents_md_patching() {
    let tmp = tempdir().unwrap();
    let agents_file = tmp.path().join("AGENTS.md");

    let modified = patch_agents_md(&agents_file, &[RuleKind::Ponytail]).unwrap();
    assert!(modified);
    assert!(agents_file.exists());

    let content = fs::read_to_string(&agents_file).unwrap();
    assert!(content.contains("# Agent Instructions"));
    assert!(content.contains("## Build Discipline (ponytail)"));

    // Idempotency check
    let modified_again = patch_agents_md(&agents_file, &[RuleKind::Ponytail]).unwrap();
    assert!(!modified_again);
}

#[test]
fn test_phase6_cursor_rules_patching() {
    let tmp = tempdir().unwrap();
    let written = patch_cursor_rules(tmp.path(), RuleKind::all()).unwrap();
    assert_eq!(written.len(), 3);

    let rules_dir = tmp.path().join(".cursor").join("rules");
    assert!(rules_dir.is_dir());

    for rule in RuleKind::all() {
        let expected_path = rules_dir.join(rule.cursor_filename());
        assert!(expected_path.exists());
        let rule_content = fs::read_to_string(&expected_path).unwrap();
        assert!(rule_content.starts_with("---\n"));
        assert!(rule_content.contains("alwaysApply: true"));
        assert!(rule_content.contains(rule.title()));
    }
}

#[test]
fn test_phase6_patch_all_agent_configs_and_inspect() {
    let tmp = tempdir().unwrap();
    let report = patch_all_agent_configs(tmp.path(), RuleKind::all()).unwrap();

    assert!(report.claude_patched);
    assert!(report.agents_patched);
    assert_eq!(report.cursor_rules_written.len(), 3);

    assert!(tmp.path().join("CLAUDE.md").exists());
    assert!(tmp.path().join("AGENTS.md").exists());

    // Inspect rules
    let statuses = inspect_agent_rules(tmp.path());
    assert_eq!(statuses.len(), 3);

    for st in &statuses {
        assert_eq!(st.present_rules.len(), 3);
    }
}

#[test]
fn test_phase6_single_file_injection_and_removal() {
    let tmp = tempdir().unwrap();
    let custom_file = tmp.path().join("CUSTOM_GUIDE.md");

    assert!(!has_rule_at_path(&custom_file, RuleKind::Caveman));

    let injected = inject_rule_at_path(&custom_file, RuleKind::Caveman).unwrap();
    assert!(injected);
    assert!(has_rule_at_path(&custom_file, RuleKind::Caveman));

    // Injected again is a no-op
    let injected_again = inject_rule_at_path(&custom_file, RuleKind::Caveman).unwrap();
    assert!(!injected_again);

    // Remove
    let removed = remove_rule_from_path(&custom_file, RuleKind::Caveman).unwrap();
    assert!(removed);
    assert!(!has_rule_at_path(&custom_file, RuleKind::Caveman));
}

#[test]
fn test_phase6_cli_commands() {
    // 1. List
    let code_list = run_rules_cli(&["list".to_string()]);
    assert_eq!(code_list, 0);

    // 2. Show principles markdown
    let code_show = run_rules_cli(&["show".to_string(), "principles".to_string()]);
    assert_eq!(code_show, 0);

    // 3. Show caveman cursor rule
    let code_show_cursor = run_rules_cli(&["show".to_string(), "caveman".to_string(), "--cursor".to_string()]);
    assert_eq!(code_show_cursor, 0);

    // 4. Show ponytail section body
    let code_show_body = run_rules_cli(&["show".to_string(), "ponytail".to_string(), "--body".to_string()]);
    assert_eq!(code_show_body, 0);

    // 5. Patch temporary directory
    let tmp = tempdir().unwrap();
    let tmp_str = tmp.path().to_string_lossy().to_string();
    let code_patch = run_rules_cli(&["patch".to_string(), tmp_str.clone(), "--rules".to_string(), "principles,ponytail".to_string()]);
    assert_eq!(code_patch, 0);

    // 6. Status of patched directory
    let code_status = run_rules_cli(&["status".to_string(), tmp_str]);
    assert_eq!(code_status, 0);
}

