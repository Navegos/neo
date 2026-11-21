// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase17_7_ui_parity_and_test_baseline_verification.rs

#![deny(unsafe_code)]

use std::path::Path;
use toksi::commands::test_runner::{blake3_hex, compute_tree_blake3, TestBaselineManifest};
use toksi::dashboard::api::{
    handle_db_table_browse, handle_plugins_get, handle_security_advisory,
    handle_security_audit_evidence, handle_security_sandbox, handle_security_scan,
    handle_skills_status,
};
use toksi::dashboard::assets::INDEX_HTML;

#[test]
fn test_phase17_7_dashboard_wave18_endpoints_and_ui() {
    assert!(INDEX_HTML.contains("advisory-search-input"));
    assert!(INDEX_HTML.contains("sandbox-adr-events"));
    assert!(INDEX_HTML.contains("plugins-manager-tbody"));
    assert!(INDEX_HTML.contains("db-table-select"));

    let scan = handle_security_scan(b"{}");
    assert_eq!(scan["status"], "clean");
    let adv = handle_security_advisory(Some("CVE-2026"));
    assert!(adv.get("publish_pr_status").is_some());
    let sb = handle_security_sandbox();
    assert!(sb.get("active_backend").is_some());
    let ev = handle_security_audit_evidence();
    assert!(ev.get("events").is_some());
    let sk = handle_skills_status();
    assert_eq!(sk["zero_binary_bloat"], true);
    let pl = handle_plugins_get();
    assert_eq!(pl["enforce_security_scan"], true);
    let tbl = handle_db_table_browse("byok_profiles");
    assert_eq!(tbl["table"], "byok_profiles");
}

#[test]
fn test_phase17_7_blake3_and_baseline_manifest() {
    let empty_hash = blake3_hex(b"");
    assert_eq!(
        empty_hash,
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
    );

    let root = Path::new("../..");
    let baseline_path = root.join(".toksi/test-baseline.json");
    assert!(baseline_path.exists(), ".toksi/test-baseline.json must exist");
    let raw = std::fs::read_to_string(&baseline_path).expect("read baseline");
    let manifest: TestBaselineManifest = serde_json::from_str(&raw).expect("parse baseline");
    assert!(manifest.suites.contains_key("phase17_7_ui_parity_and_test_baseline_verification"));
    assert!(manifest.suites.contains_key("phase18_1_to_18_6_siagent_and_vaults_verification"));

    let h = compute_tree_blake3(root, &["crates/toksi/Cargo.toml".to_string()]);
    assert_eq!(h.len(), 64);
}
