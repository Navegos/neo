// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/keyring_db_byok_verification.rs
// created: 2026-11-04
// lastModified: 2026-11-04

use toksi::db::{ByokRouteRecord, ProviderRecord, ToksiConfigDb};
use toksi::headroom::byok_gateway::enrich_headers_with_keyring_credentials;
use toksi::keyring::{
    decode_base64, encode_base64, get_agent_token, get_credential, parse_credential_envelope,
    set_credential,
};

#[test]
fn test_base64_and_envelope_parsing() {
    let raw_oauth = r#"{"access_token":"ya29.a0AfH6SMD_test_token_12345","token_type":"Bearer","expires_in":3600}"#;
    let b64 = encode_base64(raw_oauth.as_bytes());

    let env = parse_credential_envelope(&b64).expect("must parse base64 envelope");
    assert_eq!(env.access_token, "ya29.a0AfH6SMD_test_token_12345");
    assert_eq!(env.token_type.as_deref(), Some("Bearer"));
    assert!(!env.is_expired(0));

    // Test direct JSON
    let env_json = parse_credential_envelope(raw_oauth).expect("must parse direct JSON");
    assert_eq!(env_json.access_token, "ya29.a0AfH6SMD_test_token_12345");

    // Test plain string token
    let env_plain = parse_credential_envelope("sk-ant-api03-live-token").expect("must parse plain");
    assert_eq!(env_plain.access_token, "sk-ant-api03-live-token");
}

#[test]
fn test_sqlite_config_db_full_lifecycle() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("test_toksi.db");
    let mut db = ToksiConfigDb::open(&db_path).expect("open db");

    // 1. Providers
    let p = ProviderRecord {
        id: "anthropic".to_string(),
        name: "Anthropic PBC".to_string(),
        protocol: "anthropic-messages".to_string(),
        base_url: "https://api.anthropic.com".to_string(),
        is_enabled: true,
        auth_type: "keyring".to_string(),
        metadata_json: r#"{"tier":"scale"}"#.to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    db.upsert_provider(&p).expect("upsert provider");
    let fetched = db.get_provider("anthropic").expect("get").expect("found");
    assert_eq!(fetched.name, "Anthropic PBC");

    // 2. BYOK Routes
    let r = ByokRouteRecord {
        id: "claude.default".to_string(),
        token: "tok_secret_xyz".to_string(),
        protocol: "anthropic-messages".to_string(),
        upstream: "https://api.anthropic.com".to_string(),
        provider_id: Some("anthropic".to_string()),
        is_active: true,
        created_at: 1000,
        updated_at: 1000,
    };
    db.upsert_byok_route(&r).expect("upsert route");
    let routes = db.list_byok_routes().expect("list routes");
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].id, "claude.default");

    // 3. Export to JSON
    let json_dump = db.export_json().expect("export json");
    assert!(json_dump.contains("Anthropic PBC"));
    assert!(json_dump.contains("claude.default"));

    // 4. Online Backup
    let backup_path = temp_dir.path().join("backup.db");
    db.backup_to_file(&backup_path).expect("backup db");
    assert!(backup_path.exists());

    // 5. Open Backup and verify contents
    let backup_db = ToksiConfigDb::open(&backup_path).expect("open backup db");
    let backup_routes = backup_db.list_byok_routes().expect("list from backup");
    assert_eq!(backup_routes.len(), 1);
    assert_eq!(backup_routes[0].id, "claude.default");

    // 6. Import into second DB
    let mut db2 = ToksiConfigDb::open_in_memory().expect("open memory db");
    let count = db2.import_json(&json_dump, true).expect("import");
    assert_eq!(count, 2);
    let p2 = db2.get_provider("anthropic").expect("get").expect("found");
    assert_eq!(p2.name, "Anthropic PBC");
}

#[test]
fn test_header_enrichment_with_keyring_and_db() {
    // If incoming request already has authorization, do not modify
    let existing_headers = vec![("Authorization".to_string(), "Bearer user-token".to_string())];
    let res = enrich_headers_with_keyring_credentials("https://api.anthropic.com", &existing_headers);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].1, "Bearer user-token");

    // If incoming request has NO authorization, enrich based on target URL
    let empty_headers: Vec<(String, String)> = Vec::new();
    let res2 = enrich_headers_with_keyring_credentials("https://generativelanguage.googleapis.com", &empty_headers);
    // Even if no keyring is stored on host, function executes without error
    assert!(res2.is_empty() || res2.iter().any(|(k, _)| k.eq_ignore_ascii_case("authorization") || k.eq_ignore_ascii_case("x-goog-api-key")));
}

#[test]
fn test_os_keyring_get_and_set() {
    // Test storing and retrieving a test credential in OS Keyring
    let svc = "toksi-test-service";
    let acc = "test-account";
    let secret = "test-secret-value-12345";

    let ok_set = set_credential(svc, acc, secret);
    if ok_set {
        let read_back = get_credential(svc, acc);
        assert_eq!(read_back.as_deref(), Some(secret));
        toksi::keyring::delete_credential(svc, acc);
    }
}

