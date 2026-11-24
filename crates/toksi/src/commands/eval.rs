// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/eval.rs
// created: 2026-11-24

//! CLI dispatcher for `toksi eval <list|score> [harness] [model] [passed] [total]`.

#![deny(unsafe_code)]

use crate::telemetry::evals::{
    compute_eval_summary, AutoFaFailureClusterer, DeepEvalMetric, DeepEvalMetricKind,
    EvalHarnessKind, RagasScorecard, SUPPORTED_EVAL_HARNESSES,
};

pub fn run_eval_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => {
            for h in SUPPORTED_EVAL_HARNESSES {
                println!("  * {h}");
            }
            0
        }
        "score" | "lighteval" => {
            let harness = if sub == "lighteval" {
                "lighteval"
            } else {
                args.get(1).map(String::as_str).unwrap_or("lm-evaluation-harness")
            };
            let canonical = EvalHarnessKind::from_str_loose(harness)
                .map(|k| k.as_str())
                .unwrap_or(harness);
            let model = args.get(2).map(String::as_str).unwrap_or("local-model");
            let passed = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let total = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let summary = compute_eval_summary(canonical, model, total, passed);
            println!("{}", serde_json::to_string_pretty(&summary).unwrap_or_default());
            0
        }
        "deepeval" => {
            let score = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.9);
            let threshold = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.7);
            let m = DeepEvalMetric::evaluate(DeepEvalMetricKind::Faithfulness, score, threshold);
            println!("{}", serde_json::to_string_pretty(&m).unwrap_or_default());
            if m.passed { 0 } else { 1 }
        }
        "ragas" => {
            let f = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.85);
            let card = RagasScorecard {
                faithfulness: f,
                answer_relevancy: f,
                context_precision: f,
                context_recall: f,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "scorecard": card,
                    "composite_score": card.composite_score(),
                }))
                .unwrap_or_default()
            );
            0
        }
        "autofa" => {
            let msg = args.get(1).map(String::as_str).unwrap_or("rate limit 429");
            let clusters = AutoFaFailureClusterer::cluster_failures(&[("trace-1", msg)]);
            println!("{}", serde_json::to_string_pretty(&clusters).unwrap_or_default());
            0
        }
        _ => {
            eprintln!("Usage: toksi eval <list|score|lighteval|deepeval|ragas|autofa> [args...]");
            1
        }
    }
}
