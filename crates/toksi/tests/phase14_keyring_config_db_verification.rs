// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase14_keyring_config_db_verification.rs

#![deny(unsafe_code)]

use tempfile::tempdir;
use toksi::db::schema::{
    AdrAuditEventRecord, AdvisoryEmbargoRecord, BlindSignatureRecord, ByokProfileRecord,
    PluginRegistryRecord,
};
use toksi::db::ToksiConfigDb;
use toksi::keyring::{audit_keyring_statuses, current_backend_name};

#[test]
fn test_phase14_keyring_normalization_and_sqlite_tables() {
    assert!(!current_backend_name().is_empty());
    let statuses = audit_keyring_statuses();
    for st in &statuses {
        assert_ne!(st.service, "antigravity");
        assert!(!st.service.contains("<AGENT>_API_KEY"));
    }

    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("toksi.db");
    let mut db = ToksiConfigDb::open(&db_path).expect("open db");

    db.upsert_byok_profile(&ByokProfileRecord {
        id: "openai:work".into(),
        provider_id: "openai".into(),
        instance_label: "work".into(),
        model_id: "gpt-4o".into(),
        thinking_effort: "high".into(),
        base_url: "https://api.openai.com/v1".into(),
        secret_ref: "kr:openai".into(),
        auth_mode: "api_key".into(),
        gcp_project: String::new(),
        gcp_location: "us-central1".into(),
        priority: 1,
        weight: 100,
        headroom_enabled: true,
        is_active: true,
        created_at: 1,
        updated_at: 1,
    })
    .expect("upsert profile");

    db.upsert_plugin(&PluginRegistryRecord {
        id: "sample-plugin".into(),
        name: "Sample Plugin".into(),
        version: "1.0.0".into(),
        manifest_json: "{}".into(),
        blake3_hash: "abc".into(),
        capabilities_json: "[\"tool\"]".into(),
        protocols_json: "[\"mcp\"]".into(),
        security_scan_status: "passed".into(),
        is_enabled: true,
        installed_at: 1,
        updated_at: 1,
    })
    .expect("upsert plugin");

    db.upsert_blind_signature(&BlindSignatureRecord {
        id: "sig-1".into(),
        signature_kind: "blake3_ngram".into(),
        signature_payload: "deadbeef".into(),
        severity: "high".into(),
        language: "rust".into(),
        created_at: 1,
    })
    .expect("upsert blind sig");

    db.upsert_advisory_embargo(&AdvisoryEmbargoRecord {
        advisory_id: "GHSA-0001".into(),
        disclosure_state: "embargoed".into(),
        embargo_until_epoch: 2000000000,
        upstream_repo: "Navegos/advisory".into(),
        pr_url: None,
        updated_at: 1,
    })
    .expect("upsert embargo");

    db.insert_adr_audit_event(&AdrAuditEventRecord {
        id: "adr-1".into(),
        agent_id: "claude".into(),
        action: "command".into(),
        decision: "sandboxed".into(),
        reason: "reports/scan.sarif".into(),
        created_at: 1,
    })
    .expect("insert adr event");

    let exported = db.export_json().expect("export json");
    assert!(exported.contains("openai:work"));
    assert!(exported.contains("sample-plugin"));
    assert!(exported.contains("GHSA-0001"));

    let db2_path = dir.path().join("toksi2.db");
    let mut db2 = ToksiConfigDb::open(&db2_path).expect("open db2");
    let count = db2.import_json(&exported, true).expect("import overwrite");
    assert!(count >= 4);
    assert_eq!(db2.list_byok_profiles().expect("profiles").len(), 1);
    assert_eq!(db2.list_plugins().expect("plugins").len(), 1);
    assert_eq!(db2.list_blind_signatures().expect("sigs").len(), 1);
    assert_eq!(db2.list_advisory_embargoes().expect("embargoes").len(), 1);
    assert_eq!(db2.list_adr_audit_events().expect("adr").len(), 1);
}
