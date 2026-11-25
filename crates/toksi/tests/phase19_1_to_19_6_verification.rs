// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase19_1_to_19_6_verification.rs
// created: 2026-11-24

#![deny(unsafe_code)]

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toksi::agent_runtime::openspec::{SpecContract, SpecLifecycle};
use toksi::agent_runtime::workforce::{WorkTaskStatus, WorkforceCoordinator};
use toksi::aimemory::{rank_recall, FunesMemoryEntry};
use toksi::codegraph::pmat::score_source_pmat;
use toksi::commands::test_runner::{
    blake3_hex, build_docker_test_command_args, live_safe_test_env,
};
use toksi::gateway::local_engines::lookup_local_engine;
use toksi::headroom::effort::EffortLevel;
use toksi::ipc::{IpcFrame, IpcOpcode, WorkerRole};
use toksi::rules::skills_repo::{audit_neo_skills_hygiene, CANONICAL_NEO_SKILLS};
use toksi::security::{
    verify_model_spec_artifact, verify_safetensors_header, AtrEngine, AtrVerdict,
    LlguidanceJsonMask, ModelArtifactChecksum, ModelSpecManifest,
};
use toksi::telemetry::evals::{compute_eval_summary, EvalHarnessKind};
use toksi::tools::multi_protocol::{ProtocolKind, ToolTarget, CONSOLIDATED_VAULT_COUNT};
use toksi::training::{
    build_serving_profile, build_training_recipe, plan_training_job, quant::build_quant_plan,
    QuantFormat, TrainingBackend, TrainingMethod,
};

#[test]
fn test_wave19_1_to_19_3_renames_ipc_and_isolation() {
    // 19.1 ctxmode & aimemory canonical naming + backward-compatible parsing
    assert_eq!(ToolTarget::Ctxmode.as_str(), "ctxmode");
    assert_eq!(ToolTarget::Aimemory.as_str(), "aimemory");
    assert_eq!("context-mode".parse::<ToolTarget>().ok(), Some(ToolTarget::Ctxmode));
    assert_eq!("ai-memory".parse::<ToolTarget>().ok(), Some(ToolTarget::Aimemory));

    // 19.2 multi-process worker roles + TKSI IPC frame codec
    assert_eq!(WorkerRole::ALL.len(), 17);
    assert_eq!(
        "network-manager".parse::<WorkerRole>().ok(),
        Some(WorkerRole::NetworkManager)
    );
    assert_eq!("siagent".parse::<WorkerRole>().ok(), Some(WorkerRole::Siagent));
    let frame = IpcFrame::json(
        IpcOpcode::Ping,
        77,
        &serde_json::json!({ "role": "siagent" }),
    )
    .expect("encode json ipc frame");
    let wire = frame.encode().expect("encode bytes");
    let decoded = IpcFrame::decode(&wire).expect("decode bytes");
    assert_eq!(decoded.opcode, IpcOpcode::Ping);
    assert_eq!(decoded.correlation_id, 77);

    // 19.3 Dockerfile.test + live-safe env isolation
    let dockerfile = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docker/Dockerfile.test");
    assert!(dockerfile.is_file(), "docker/Dockerfile.test missing");
    let df_body = fs::read_to_string(&dockerfile).expect("read Dockerfile.test");
    assert!(df_body.contains("TOKSI_TEST=1"));

    let tmp = TempDir::new().expect("tempdir");
    let envs = live_safe_test_env(tmp.path(), "wave19-e2e");
    assert!(envs.iter().any(|(k, v)| k == "TOKSI_TEST" && v == "1"));
    let docker_args = build_docker_test_command_args(
        tmp.path(),
        "toksi-test:local",
        &["test".to_string(), "-p".to_string(), "toksi".to_string()],
    );
    assert!(docker_args.windows(2).any(|w| w == ["--network", "none"]));
    assert!(docker_args.iter().all(|a| !a.contains("~/.toksi")));
}

#[test]
fn test_wave19_4_to_19_6_hygiene_si_stack_and_training_plane() {
    // 19.4 skills hygiene audit in tempdir
    assert!(!CANONICAL_NEO_SKILLS.is_empty());
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join(".agents/skills/foreign-temp")).expect("mkdir");
    fs::write(root.join(".agents/skills/foreign-temp/run.sh"), "#!/bin/sh\n").expect("write");
    let clean = audit_neo_skills_hygiene(root, true, true).expect("hygiene audit");
    assert_eq!(clean.pruned_skills, vec!["foreign-temp"]);

    // 19.5 493 vaults + ANP/QUIC + Workforce + OpenSpec + ATR + ModelSpec + PMAT + FunesRecall
    assert_eq!(CONSOLIDATED_VAULT_COUNT, 493);
    assert_eq!("anp".parse::<ProtocolKind>().ok(), Some(ProtocolKind::Anp));
    assert_eq!("quic".parse::<ProtocolKind>().ok(), Some(ProtocolKind::Quic));

    let mut wf = WorkforceCoordinator::new();
    wf.register_worker("worker-1", "builder", "codex");
    wf.submit_task("task-1", "builder", "Verify module", 1);
    let claimed = wf.claim_next_for_worker("worker-1").expect("claim");
    assert_eq!(claimed.task_id, "task-1");
    wf.complete_task("task-1", "verified").expect("complete");
    assert_eq!(wf.tasks()[0].status, WorkTaskStatus::Completed);

    let mut spec = SpecContract::new(
        "spec-19",
        "Validate OpenSpec",
        vec!["passes assertions".to_string()],
        vec!["crates/toksi/src/lib.rs".to_string()],
    );
    spec.transition_to(SpecLifecycle::Reviewed).expect("reviewed");
    spec.transition_to(SpecLifecycle::Approved).expect("approved");
    spec.transition_to(SpecLifecycle::Executing).expect("executing");
    spec.transition_to(SpecLifecycle::Verified).expect("verified");

    let atr = AtrEngine::with_default_rules();
    assert_eq!(
        atr.evaluate_payload("tool", "inspect git status").verdict,
        AtrVerdict::Allow
    );

    let blob = b"safe-weights";
    let manifest = ModelSpecManifest {
        model_id: "qwen".to_string(),
        revision: "r1".to_string(),
        license: "Apache-2.0".to_string(),
        artifacts: vec![ModelArtifactChecksum {
            path: "w.safetensors".to_string(),
            sha256_hex: "bad".repeat(16),
            blake3_hex: blake3_hex(blob),
        }],
    };
    assert!(verify_model_spec_artifact(&manifest, "w.safetensors", blob).is_err());

    let pmat = score_source_pmat("fn helper(a: i32) -> i32 { a + 1 }");
    assert_eq!(pmat.function_count, 1);

    let recalled = rank_recall(
        "direct wire",
        &[FunesMemoryEntry {
            id: "m1".to_string(),
            content: "direct wire ipc transport".to_string(),
            salience: 0.9,
            age_hours: 1.0,
        }],
        24.0,
        1,
    );
    assert_eq!(recalled.len(), 1);

    // 19.6 training recipes, quant/safetensors guard, 9 serving engines, evals
    let recipe = build_training_recipe(
        TrainingBackend::Unsloth,
        TrainingMethod::Qlora,
        "Qwen/Qwen2.5-Coder-7B",
        "data/train.jsonl",
        EffortLevel::Medium,
    )
    .expect("recipe");
    let job = plan_training_job(&recipe, true);
    assert!(job.dry_run);
    let qp = build_quant_plan(QuantFormat::GgufQ4KM, "Qwen", "out/q4.gguf");
    assert!(qp.estimated_compression_ratio > 3.0);
    assert!(verify_safetensors_header(b"\x80\x04pickle").is_err());

    for id in [
        "oxillama",
        "xinfer",
        "crane",
        "paddler",
        "llama-swap",
        "whisper.cpp",
        "stable-diffusion.cpp",
        "triton-server",
        "deepspeed-mii",
    ] {
        assert!(lookup_local_engine(id).is_some(), "missing engine {id}");
    }
    let srv = build_serving_profile("oxillama", "qwen2.5", EffortLevel::Medium).expect("serving");
    assert_eq!(srv.engine, "oxillama");

    let mask = LlguidanceJsonMask {
        required_fields: vec!["ok".to_string()],
        allow_additional_properties: false,
    };
    assert!(mask.validate_json_output(r#"{"ok":true}"#).is_ok());

    assert_eq!(
        EvalHarnessKind::from_str_loose("lighteval"),
        Some(EvalHarnessKind::Lighteval)
    );
    let es = compute_eval_summary("lighteval", "qwen", 4, 4);
    assert!((es.score - 1.0).abs() < 1e-6);
}
