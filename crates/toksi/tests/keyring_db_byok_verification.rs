// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/keyring_db_byok_verification.rs
// created: 2026-11-04
// lastModified: 2026-11-18

use toksi::db::{
    AdrAuditEventRecord, AdvisoryEmbargoRecord, BlindSignatureRecord, ByokProfileRecord,
    ByokRouteRecord, PluginRegistryRecord, ProviderModelRecord, ProviderRecord, ToksiConfigDb,
};
use toksi::headroom::byok_gateway::enrich_headers_with_keyring_credentials;
use toksi::keyring::{
    delete_spire_svid, encode_base64, get_agent_token, get_credential, inspect_adc_status,
    parse_credential_envelope, read_spire_svid, set_credential, store_spire_svid,
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

#[test]
fn test_wave18_db_dump_and_blind_signature_guard() {
    let mut db = ToksiConfigDb::open_in_memory().expect("open memory db");

    db.upsert_provider(&ProviderRecord {
        id: "vertex".to_string(),
        name: "Google Cloud Vertex AI".to_string(),
        protocol: "google-gemini".to_string(),
        base_url: "https://us-central1-aiplatform.googleapis.com".to_string(),
        is_enabled: true,
        auth_type: "adc".to_string(),
        metadata_json: "{}".to_string(),
        created_at: 1000,
        updated_at: 1000,
    })
    .expect("upsert provider");

    db.upsert_provider_model(&ProviderModelRecord {
        id: "gemini-2.5-pro".to_string(),
        provider_id: "vertex".to_string(),
        display_name: "Gemini 2.5 Pro".to_string(),
        context_window: 1_000_000,
        max_output_tokens: 65536,
        supports_streaming: true,
        supports_tools: true,
        is_default: true,
        created_at: 1000,
    })
    .expect("upsert model");

    db.upsert_byok_profile(&ByokProfileRecord {
        id: "vertex.adc.high".to_string(),
        provider_id: "vertex".to_string(),
        instance_label: "adc".to_string(),
        model_id: "gemini-2.5-pro".to_string(),
        thinking_effort: "high".to_string(),
        base_url: "https://us-central1-aiplatform.googleapis.com".to_string(),
        secret_ref: "adc://gcloud".to_string(),
        auth_mode: "adc".to_string(),
        gcp_project: "my-gcp-project".to_string(),
        gcp_location: "us-central1".to_string(),
        priority: 20,
        weight: 1,
        headroom_enabled: true,
        is_active: true,
        created_at: 1000,
        updated_at: 1000,
    })
    .expect("upsert profile");

    db.upsert_plugin(&PluginRegistryRecord {
        id: "toksi.security.scanner".to_string(),
        name: "Security Scanner".to_string(),
        version: "1.0.0".to_string(),
        manifest_json: "{}".to_string(),
        blake3_hash: "0123456789abcdef".to_string(),
        capabilities_json: r#"["tool"]"#.to_string(),
        protocols_json: r#"["MCP"]"#.to_string(),
        security_scan_status: "passed".to_string(),
        is_enabled: true,
        installed_at: 1000,
        updated_at: 1000,
    })
    .expect("upsert plugin");

    // Reject raw exploit PoC / regex in blind_signatures
    let raw_poc = BlindSignatureRecord {
        id: "bad-sig".to_string(),
        signature_kind: "blake3_ngram".to_string(),
        signature_payload: "(eval|exec)\\(.*\\)".to_string(),
        severity: "high".to_string(),
        language: "python".to_string(),
        created_at: 1000,
    };
    assert!(db.upsert_blind_signature(&raw_poc).is_err());

    // Accept valid blind blake3 hex signature
    db.upsert_blind_signature(&BlindSignatureRecord {
        id: "good-sig".to_string(),
        signature_kind: "blake3_ngram".to_string(),
        signature_payload: "deadbeef0123456789abcdef".to_string(),
        severity: "high".to_string(),
        language: "python".to_string(),
        created_at: 1000,
    })
    .expect("upsert blind sig");

    db.upsert_advisory_embargo(&AdvisoryEmbargoRecord {
        advisory_id: "GHSA-0001".to_string(),
        disclosure_state: "embargoed".to_string(),
        embargo_until_epoch: 2_000_000_000,
        upstream_repo: "Navegos/advisory".to_string(),
        pr_url: None,
        updated_at: 1000,
    })
    .expect("upsert embargo");

    db.insert_adr_audit_event(&AdrAuditEventRecord {
        id: "adr-1".to_string(),
        agent_id: "antigravity".to_string(),
        action: "shell_exec".to_string(),
        decision: "allowed".to_string(),
        reason: "policy_ok".to_string(),
        created_at: 1000,
    })
    .expect("insert adr event");

    let dump = db.export_dump().expect("export dump");
    assert_eq!(dump.provider_models.len(), 1);
    assert_eq!(dump.byok_profiles.len(), 1);
    assert_eq!(dump.byok_profiles[0].auth_mode, "adc");
    assert_eq!(dump.byok_profiles[0].gcp_project, "my-gcp-project");
    assert_eq!(dump.plugins_registry.len(), 1);
    assert_eq!(dump.blind_signatures.len(), 1);
    assert_eq!(dump.advisory_embargoes.len(), 1);
    assert_eq!(dump.adr_audit_events.len(), 1);

    let mut db2 = ToksiConfigDb::open_in_memory().expect("open memory db 2");
    let imported = db2.import_dump(&dump, true).expect("import overwrite true");
    assert_eq!(imported, 7);

    // Import with overwrite=false should insert 0 new rows when already present
    let imported_no_overwrite = db2.import_dump(&dump, false).expect("import overwrite false");
    assert_eq!(imported_no_overwrite, 0);
}

#[test]
fn test_adc_and_spire_svid_and_alias() {
    let adc = inspect_adc_status();
    assert!(!adc.gcp_location.is_empty());

    let spiffe_id = "spiffe://toksi.local/agent/test";
    if store_spire_svid(spiffe_id, "svid-token-test-12345") {
        let env = read_spire_svid(spiffe_id);
        assert_eq!(env.as_ref().map(|e| e.access_token.as_str()), Some("svid-token-test-12345"));
        assert!(delete_spire_svid(spiffe_id));
    }

    let _ = get_agent_token("gemini:antigravity");
}

#[test]
fn test_multi_instance_byok_crud_and_overwrite_reset() {
    let mut db = ToksiConfigDb::open_in_memory().expect("open memory db");
    let now = 1000;

    // Seed a stale profile that should be wiped when import_dump(..., overwrite = true) runs
    db.upsert_byok_profile(&ByokProfileRecord {
        id: "stale.profile".to_string(),
        provider_id: "stale".to_string(),
        instance_label: "old".to_string(),
        model_id: "old-model".to_string(),
        thinking_effort: "low".to_string(),
        base_url: "https://stale.example.com".to_string(),
        secret_ref: String::new(),
        auth_mode: "api_key".to_string(),
        gcp_project: String::new(),
        gcp_location: "us-central1".to_string(),
        priority: 1,
        weight: 1,
        headroom_enabled: true,
        is_active: true,
        created_at: now,
        updated_at: now,
    })
    .expect("seed stale profile");

    let mut clean_db = ToksiConfigDb::open_in_memory().expect("open clean db");
    let flash_low = ByokProfileRecord {
        id: "google:flash-low:gemini-2.5-flash:low".to_string(),
        provider_id: "google".to_string(),
        instance_label: "flash-low".to_string(),
        model_id: "gemini-2.5-flash".to_string(),
        thinking_effort: "low".to_string(),
        base_url: "https://generativelanguage.googleapis.com".to_string(),
        secret_ref: "keyring://gemini/antigravity".to_string(),
        auth_mode: "oauth".to_string(),
        gcp_project: String::new(),
        gcp_location: "us-central1".to_string(),
        priority: 100,
        weight: 100,
        headroom_enabled: true,
        is_active: true,
        created_at: now,
        updated_at: now,
    };
    let pro_high = ByokProfileRecord {
        id: "google:pro-high:gemini-2.5-pro:high".to_string(),
        provider_id: "google".to_string(),
        instance_label: "pro-high".to_string(),
        model_id: "gemini-2.5-pro".to_string(),
        thinking_effort: "high".to_string(),
        base_url: "https://generativelanguage.googleapis.com".to_string(),
        secret_ref: "keyring://gemini/antigravity".to_string(),
        auth_mode: "oauth".to_string(),
        gcp_project: String::new(),
        gcp_location: "us-central1".to_string(),
        priority: 90,
        weight: 50,
        headroom_enabled: true,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    clean_db.upsert_byok_profile(&flash_low).expect("upsert flash-low");
    clean_db.upsert_byok_profile(&pro_high).expect("upsert pro-high");
    assert_eq!(clean_db.list_byok_profiles().expect("list").len(), 2);

    let dump = clean_db.export_dump().expect("export dump");
    db.import_dump(&dump, true).expect("import with overwrite");

    // Stale profile must be gone, leaving only the 2 clean profiles
    let after = db.list_byok_profiles().expect("list after overwrite");
    assert_eq!(after.len(), 2);
    assert!(db.get_byok_profile("stale.profile").expect("get stale").is_none());

    // CRUD updates
    assert!(db
        .update_byok_profile_url(&flash_low.id, "https://custom.example.com")
        .expect("update url"));
    assert!(db
        .update_byok_profile_model(&flash_low.id, "gemini-2.5-flash-lite", Some("medium"))
        .expect("update model"));
    let updated = db
        .get_byok_profile(&flash_low.id)
        .expect("get")
        .expect("present");
    assert_eq!(updated.base_url, "https://custom.example.com");
    assert_eq!(updated.model_id, "gemini-2.5-flash-lite");
    assert_eq!(updated.thinking_effort, "medium");
    assert!(updated.headroom_enabled);

    assert!(db.delete_byok_profile(&pro_high.id).expect("delete"));
    assert_eq!(db.list_byok_profiles().expect("list").len(), 1);
}

#[test]
fn test_ccr_mcp_tools_compress_retrieve_stats_read() {
    use toksi::headroom::ccr::{handle_mcp_ccr_tool, CcrStore, InMemoryCcrStore};

    let store = InMemoryCcrStore::default();
    let hash = "test_hash_1234";
    store.put(hash, "alpha\nbeta\ngamma\ndelta");

    let ret = handle_mcp_ccr_tool("headroom_retrieve", &serde_json::json!({ "hash": hash }), &store)
        .expect("retrieve");
    assert_eq!(ret["found"], true);
    assert_eq!(ret["content"], "alpha\nbeta\ngamma\ndelta");

    let read_slice = handle_mcp_ccr_tool(
        "headroom_read",
        &serde_json::json!({ "hash": hash, "offset": 6, "limit": 4 }),
        &store,
    )
    .expect("read");
    assert_eq!(read_slice["content"], "beta");

    let stats = handle_mcp_ccr_tool("headroom_stats", &serde_json::json!({}), &store).expect("stats");
    assert!(stats.get("raw_bytes_total").is_some());

    let comp = handle_mcp_ccr_tool(
        "headroom_compress",
        &serde_json::json!({ "content": "{\"a\": 1, \"b\": 2}", "hint": "json" }),
        &store,
    )
    .expect("compress");
    assert!(comp.get("compressed").is_some());
}

