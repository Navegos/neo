// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/train.rs
// created: 2026-11-24

//! CLI dispatcher for `toksi train <recipe|plan|quant-plan|verify-safetensors>`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::headroom::effort::EffortLevel;
use crate::training::{
    build_training_recipe, plan_training_job, quant::build_quant_plan, render_recipe_yaml,
    verify_safetensors_header, QuantFormat, TrainingBackend, TrainingMethod,
};

pub fn run_train_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("plan");
    match sub {
        "recipe" | "plan" => {
            let backend = args
                .get(1)
                .and_then(|s| TrainingBackend::from_str_loose(s))
                .unwrap_or(TrainingBackend::Unsloth);
            let method = args
                .get(2)
                .and_then(|s| TrainingMethod::from_str_loose(s))
                .unwrap_or(TrainingMethod::Lora);
            let model = args.get(3).map(String::as_str).unwrap_or("Qwen/Qwen2.5-Coder-7B");
            let dataset = args.get(4).map(String::as_str).unwrap_or("data/train.jsonl");
            let effort = args
                .get(5)
                .and_then(|s| EffortLevel::from_str_loose(s))
                .unwrap_or_default();
            match build_training_recipe(backend, method, model, dataset, effort) {
                Ok(r) => {
                    if sub == "recipe" {
                        println!("{}", render_recipe_yaml(&r));
                    } else {
                        let plan = plan_training_job(&r, true);
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&plan).unwrap_or_default()
                        );
                    }
                    0
                }
                Err(e) => {
                    eprintln!("train error: {e}");
                    1
                }
            }
        }
        "quant-plan" => {
            let fmt = args
                .get(1)
                .and_then(|s| QuantFormat::from_str_loose(s))
                .unwrap_or(QuantFormat::GgufQ4KM);
            let model = args.get(2).map(String::as_str).unwrap_or("local/model");
            let out = args.get(3).map(String::as_str).unwrap_or("out/model.gguf");
            let plan = build_quant_plan(fmt, model, out);
            println!("{}", serde_json::to_string_pretty(&plan).unwrap_or_default());
            0
        }
        "verify-safetensors" => {
            let Some(file) = args.get(1) else {
                eprintln!("Usage: toksi train verify-safetensors <file.safetensors>");
                return 1;
            };
            let Ok(bytes) = std::fs::read(Path::new(file)) else {
                eprintln!("Cannot read '{file}'");
                return 1;
            };
            match verify_safetensors_header(&bytes) {
                Ok(info) => {
                    println!("{}", serde_json::to_string_pretty(&info).unwrap_or_default());
                    0
                }
                Err(e) => {
                    eprintln!("safetensors verification failed: {e}");
                    1
                }
            }
        }
        _ => {
            eprintln!("Usage: toksi train <recipe|plan|quant-plan|verify-safetensors>");
            1
        }
    }
}
