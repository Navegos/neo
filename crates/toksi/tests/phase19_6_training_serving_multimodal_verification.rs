// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase19_6_training_serving_multimodal_verification.rs

#![deny(unsafe_code)]

use toksi::gateway::local_engines::lookup_local_engine;
use toksi::headroom::effort::EffortLevel;
use toksi::security::{verify_safetensors_header, LlguidanceJsonMask};
use toksi::telemetry::evals::{compute_eval_summary, EvalHarnessKind, SUPPORTED_EVAL_HARNESSES};
use toksi::training::{
    build_serving_profile, build_training_recipe, plan_training_job, quant::build_quant_plan,
    render_recipe_yaml, QuantFormat, TrainingBackend, TrainingMethod,
};

#[test]
fn test_phase19_6_training_recipes_quant_and_safetensors_security() {
    let recipe = build_training_recipe(
        TrainingBackend::Unsloth,
        TrainingMethod::Qlora,
        "Qwen/Qwen2.5-Coder-7B",
        "data\\train.jsonl",
        EffortLevel::High,
    )
    .expect("recipe");
    assert_eq!(recipe.dataset_path, "data/train.jsonl");
    let yaml = render_recipe_yaml(&recipe);
    assert!(yaml.contains("backend: unsloth"));
    assert!(yaml.contains("method: qlora"));

    let plan = plan_training_job(&recipe, true);
    assert!(plan.dry_run);
    assert_eq!(plan.backend, TrainingBackend::Unsloth);

    let qplan = build_quant_plan(QuantFormat::GgufQ4KM, "Qwen/Qwen2.5", "out/q4.gguf");
    assert!(qplan.estimated_compression_ratio > 3.0);

    // Valid safetensors payload
    let header_json = br#"{"w1":{"dtype":"BF16","shape":[4,4],"data_offsets":[0,32]}}"#;
    let mut valid_bytes = (header_json.len() as u64).to_le_bytes().to_vec();
    valid_bytes.extend_from_slice(header_json);
    valid_bytes.extend_from_slice(&[0u8; 32]);
    let info = verify_safetensors_header(&valid_bytes).expect("valid safetensors");
    assert_eq!(info.tensor_count, 1);
    assert_eq!(info.tensor_names, vec!["w1"]);

    // Pickle payload rejected fail-closed
    let pickle_blob = b"\x80\x04\x95fake_pickle_payload";
    assert!(verify_safetensors_header(pickle_blob).is_err());

    // Empty-tensor header rejected
    let empty_json = br#"{"__metadata__":{"format":"pt"}}"#;
    let mut empty_bytes = (empty_json.len() as u64).to_le_bytes().to_vec();
    empty_bytes.extend_from_slice(empty_json);
    assert!(verify_safetensors_header(&empty_bytes).is_err());
}

#[test]
fn test_phase19_6_serving_engines_llguidance_and_eval_harnesses() {
    for engine in [
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
        assert!(
            lookup_local_engine(engine).is_some(),
            "Missing Wave 19 local serving engine: {engine}"
        );
    }

    let whisper = build_serving_profile("whisper.cpp", "whisper-large-v3", EffortLevel::Low)
        .expect("whisper profile");
    assert_eq!(whisper.modality, "audio-asr");
    let sd = build_serving_profile("stable-diffusion.cpp", "flux-dev", EffortLevel::Medium)
        .expect("sd profile");
    assert_eq!(sd.modality, "image-diffusion");

    let mask = LlguidanceJsonMask {
        required_fields: vec!["status".into(), "score".into()],
        allow_additional_properties: false,
    };
    assert!(mask.validate_json_output(r#"{"status":"ok","score":1}"#).is_ok());
    assert!(mask
        .validate_json_output(r#"{"status":"ok","score":1,"extra":2}"#)
        .is_err());

    assert!(SUPPORTED_EVAL_HARNESSES.contains(&"deepeval"));
    assert!(SUPPORTED_EVAL_HARNESSES.contains(&"ragas"));
    assert_eq!(
        EvalHarnessKind::from_str_loose("inspect-ai"),
        Some(EvalHarnessKind::InspectAi)
    );
    let summary = compute_eval_summary("lighteval", "qwen-coder", 10, 9);
    assert!((summary.score - 0.9).abs() < 1e-6);
}
