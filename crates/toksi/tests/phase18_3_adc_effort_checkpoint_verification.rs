// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase18_3_adc_effort_checkpoint_verification.rs
// created: 2026-11-21

use std::fs;
use std::time::Instant;

use serde_json::json;
use tempfile::tempdir;

use toksi::agent_runtime::checkpoint::SafePlanCheckpointer;
use toksi::agent_runtime::slash::global_slash_engine;
use toksi::db::{
    list_adc_byok_profiles, resolve_active_adc_profile, upsert_adc_byok_profile, ToksiConfigDb,
};
use toksi::gateway::effort_router::{
    parse_effort_cli_flag, route_and_translate, EffortContextStrategy,
};
use toksi::gateway::vertex_adc::VertexAdcEndpoint;
use toksi::headroom::effort::{
    apply_effort_wire_translation, detect_effort_level, global_effort_metrics, EffortLevel,
    HeadroomEffortCompressor,
};
use toksi::keyring::adc::{
    clear_adc_token_cache, format_missing_adc_guidance, get_cached_adc_token,
    parse_adc_credentials_json, put_cached_adc_token, CachedAdcToken, DEFAULT_GCP_LOCATION,
};
use toksi::projectmem::plan_checkpoint::{
    load_plan_checkpoint, CheckpointStepStatus, PlanStepRecord, YieldReason,
};

#[test]
fn test_phase18_3_google_adc_db_cache_and_vertex_v1beta1_signing() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("toksi.db");
    let mut db = ToksiConfigDb::open(&db_path).expect("open db");

    let rec = upsert_adc_byok_profile(
        &mut db,
        "adc-prod",
        "gemini-2.5-pro",
        "neo-prod-123",
        "",
        "high",
    )
    .expect("upsert adc profile");
    assert_eq!(rec.auth_mode, "adc");
    assert_eq!(rec.gcp_project, "neo-prod-123");
    assert_eq!(rec.gcp_location, DEFAULT_GCP_LOCATION);

    let list = list_adc_byok_profiles(&db).expect("list adc profiles");
    assert_eq!(list.len(), 1);
    assert!(resolve_active_adc_profile(&db).is_some());

    // Parse authorized_user and service_account JSON formats
    let auth_user = parse_adc_credentials_json(
        r#"{
            "type": "authorized_user",
            "client_id": "cid",
            "client_secret": "csec",
            "refresh_token": "1//tok",
            "quota_project_id": "neo-prod-123"
        }"#,
    )
    .expect("parse authorized_user");
    assert_eq!(auth_user.kind_label(), "authorized_user");

    let svc_acct = parse_adc_credentials_json(
        r#"{
            "type": "service_account",
            "client_email": "toksi@neo-prod-123.iam.gserviceaccount.com",
            "private_key": "-----BEGIN PRIVATE KEY-----\nMII=\n-----END PRIVATE KEY-----",
            "project_id": "neo-prod-123"
        }"#,
    )
    .expect("parse service_account");
    assert_eq!(svc_acct.kind_label(), "service_account");

    // 60s expiry skew cache check
    clear_adc_token_cache();
    let now = 1_750_000_000;
    put_cached_adc_token(
        "adc-test",
        CachedAdcToken {
            access_token: "ya29.cached".to_string(),
            token_type: "Bearer".to_string(),
            expires_at_unix: now + 180,
            source_kind: "authorized_user".to_string(),
        },
    );
    assert!(get_cached_adc_token("adc-test", now + 119).is_some());
    assert!(get_cached_adc_token("adc-test", now + 120).is_none());
    clear_adc_token_cache();

    // Regional Vertex AI v1beta1 URL & header signing
    let ep = VertexAdcEndpoint::from_env_or_config("neo-prod-123", "us-central1", "gemini-2.5-pro");
    let stream_url = ep.stream_generate_content_url().expect("stream url");
    assert_eq!(
        stream_url,
        "https://us-central1-aiplatform.googleapis.com/v1beta1/projects/neo-prod-123/locations/us-central1/publishers/google/models/gemini-2.5-pro:streamGenerateContent"
    );
    let headers = ep.sign_vertex_headers("ya29.cached");
    assert!(headers
        .iter()
        .any(|(k, v)| k == "Authorization" && v == "Bearer ya29.cached"));
    assert!(headers
        .iter()
        .any(|(k, v)| k == "x-goog-user-project" && v == "neo-prod-123"));

    // Actionable missing-ADC guidance
    let guidance = format_missing_adc_guidance(Some("neo-prod-123"), "/home/u/.config/gcloud/adc.json");
    assert!(guidance.contains("gcloud auth application-default login --project neo-prod-123"));
    assert!(guidance.contains("toksi auth adc --project neo-prod-123 --location us-central1"));
}

#[test]
fn test_phase18_3_effort_router_translation_and_slash_latency() {
    let args = vec!["--effort".to_string(), "high".to_string()];
    assert_eq!(parse_effort_cli_flag(&args), Some(EffortLevel::High));
    let eq_args = vec!["--effort=low".to_string()];
    assert_eq!(parse_effort_cli_flag(&eq_args), Some(EffortLevel::Low));

    // OpenAI / DeepSeek / Grok
    let mut oai = json!({
        "model": "o3-mini",
        "messages": [{"role": "user", "content": "Analyze this"}]
    });
    let dec = route_and_translate("/v1/chat/completions", &[], &mut oai, Some(EffortLevel::High));
    assert_eq!(dec.effort, EffortLevel::High);
    assert!(dec.wire_translated);
    assert_eq!(oai["reasoning_effort"], "high");
    assert!(dec.context_strategy.scip_multi_hop);
    assert!(dec.context_strategy.lossless_constraints_and_traces);

    // Anthropic thinking + non-thinking omission
    let mut anth_think = json!({
        "model": "claude-3-7-sonnet",
        "messages": [{"role": "user", "content": "Plan"}]
    });
    assert!(apply_effort_wire_translation(
        "/v1/messages",
        &mut anth_think,
        EffortLevel::Medium,
        true
    ));
    assert_eq!(anth_think["thinking"]["budget_tokens"], 8192);

    let mut anth_haiku = json!({
        "model": "claude-3-5-haiku",
        "thinking": {"type": "enabled", "budget_tokens": 1024},
        "messages": [{"role": "user", "content": "Hi"}]
    });
    assert!(!apply_effort_wire_translation(
        "/v1/messages",
        &mut anth_haiku,
        EffortLevel::Low,
        true
    ));
    assert!(anth_haiku.get("thinking").is_none());

    // Gemini / Vertex thinking + non-thinking strip
    let mut gem = json!({
        "model": "gemini-2.5-pro",
        "contents": [{"role": "user", "parts": [{"text": "Solve"}]}]
    });
    assert!(apply_effort_wire_translation(
        "streamGenerateContent",
        &mut gem,
        EffortLevel::High,
        true
    ));
    assert_eq!(
        gem["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        24576
    );

    let mut gem_old = json!({
        "model": "gemini-1.5-flash",
        "generationConfig": {"thinkingConfig": {"thinkingBudget": 1024}},
        "contents": [{"role": "user", "parts": [{"text": "Hi"}]}]
    });
    assert!(!apply_effort_wire_translation(
        "generateContent",
        &mut gem_old,
        EffortLevel::High,
        true
    ));
    assert!(gem_old["generationConfig"].get("thinkingConfig").is_none());

    // Detect effort from JSON body fields when header absent
    assert_eq!(
        detect_effort_level(&[], Some(&json!({"reasoning_effort": "low"}))),
        EffortLevel::Low
    );
    assert_eq!(
        EffortContextStrategy::for_effort(EffortLevel::Low).rtk_max_compaction,
        true
    );

    // Slash /effort < 5ms switching & live telemetry registry
    let started = Instant::now();
    let res = global_slash_engine().execute("/effort high", "toksi-test");
    assert!(res.handled);
    assert!(started.elapsed().as_millis() < 5);
    assert_eq!(global_effort_metrics().active_effort(), EffortLevel::High);

    let mut payload = json!({
        "model": "o3-mini",
        "messages": [{"role": "user", "content": "Hello"}]
    });
    let _ = HeadroomEffortCompressor::gateway_default().compress_json_payload(
        "/v1/chat/completions",
        &[("x-toksi-effort".to_string(), "low".to_string())],
        &mut payload,
    );
    let snap = global_effort_metrics().snapshot();
    assert!(snap.low_requests >= 1);
}

#[test]
fn test_phase18_3_safe_plan_checkpoint_and_crash_resume() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).expect("mkdir .git");
    let mem_dir = root.join(".toksi").join("projectmem");
    fs::create_dir_all(&mem_dir).expect("mkdir projectmem");
    fs::write(
        mem_dir.join("plan.md"),
        "# Plan\n- [ ] Step One\n- [ ] Step Two\n",
    )
    .expect("write plan.md");

    let touched = root.join("src_sample.rs");
    fs::write(&touched, "pub fn alpha() -> i32 { 1 }\n").expect("write src_sample.rs");

    let mut cp_mgr = SafePlanCheckpointer::new(root.to_path_buf(), "plan-18-3", "Phase 18.3 Plan");
    cp_mgr.checkpoint.total_steps = 2;
    cp_mgr.checkpoint.current_step = 1;
    cp_mgr.checkpoint.active_effort = "High".to_string();
    cp_mgr.checkpoint.active_model = "gemini-2.5-pro".to_string();
    cp_mgr.checkpoint.completed_steps = vec!["Step One".to_string()];
    cp_mgr.checkpoint.pending_steps = vec!["Step Two".to_string()];
    cp_mgr.checkpoint.steps = vec![
        PlanStepRecord {
            id: 1,
            step_key: "step-1".to_string(),
            title: "Step One".to_string(),
            status: CheckpointStepStatus::Completed,
            files_touched: vec![],
            test_baseline_delta: Some("+1".to_string()),
            verified_by: Some("unit".to_string()),
            completed_at: Some("2026-11-21T07:00:00Z".to_string()),
        },
        PlanStepRecord {
            id: 2,
            step_key: "step-2".to_string(),
            title: "Step Two".to_string(),
            status: CheckpointStepStatus::Paused,
            files_touched: vec![],
            test_baseline_delta: None,
            verified_by: None,
            completed_at: None,
        },
    ];
    cp_mgr.snapshot_runtime_state(&[std::path::Path::new("src_sample.rs")]);
    let saved_path = cp_mgr
        .yield_and_checkpoint(
            YieldReason::RateLimit429,
            Some("Paused before Step Two on HTTP 429"),
            Some("429 Too Many Requests"),
        )
        .expect("yield_and_checkpoint");
    assert!(saved_path.is_file());

    // Verify plan.md checkbox synced & events.jsonl written
    let plan_md = fs::read_to_string(mem_dir.join("plan.md")).expect("read plan.md");
    assert!(plan_md.contains("- [x] Step One"));
    assert!(plan_md.contains("- [ ] Step Two"));
    assert!(mem_dir.join("events.jsonl").is_file());

    // Clean preflight resume check
    let (loaded, preflight) =
        SafePlanCheckpointer::prepare_resume(root, "plan-18-3").expect("prepare_resume");
    assert_eq!(loaded.plan_id, "plan-18-3");
    assert_eq!(preflight.completed_count, 1);
    assert_eq!(preflight.next_step_index, 1);
    assert_eq!(preflight.next_step_title.as_deref(), Some("Step Two"));
    assert!(preflight.dirty_mismatches.is_empty());

    // /resume slash command hydrates exact incomplete step
    global_slash_engine().set_workspace_root(root.to_path_buf());
    let resume_res = global_slash_engine().execute("/resume plan-18-3", "toksi-test");
    assert!(resume_res.handled);
    assert!(resume_res.output.contains("next_step=1 ('Step Two')"));

    // Modifying dirty file triggers SHA-256 mismatch detection on resume
    fs::write(&touched, "pub fn alpha() -> i32 { 999 }\n").expect("mutate src_sample.rs");
    let (_, tampered) =
        SafePlanCheckpointer::prepare_resume(root, "plan-18-3").expect("prepare_resume tampered");
    assert_eq!(tampered.dirty_mismatches.len(), 1);
    let _ = load_plan_checkpoint(Some(root), "plan-18-3").expect("load_plan_checkpoint");
}
