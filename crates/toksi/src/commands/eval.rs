// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/eval.rs
// created: 2026-11-24

//! CLI dispatcher for `toksi eval <list|score> [harness] [model] [passed] [total]`.

#![deny(unsafe_code)]

use crate::telemetry::evals::{compute_eval_summary, EvalHarnessKind, SUPPORTED_EVAL_HARNESSES};

pub fn run_eval_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => {
            for h in SUPPORTED_EVAL_HARNESSES {
                println!("  * {h}");
            }
            0
        }
        "score" => {
            let harness = args.get(1).map(String::as_str).unwrap_or("lm-evaluation-harness");
            let canonical = EvalHarnessKind::from_str_loose(harness)
                .map(|k| k.as_str())
                .unwrap_or(harness);
            let model = args.get(2).map(String::as_str).unwrap_or("local-model");
            let passed = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let total = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let summary = compute_eval_summary(canonical, model, total, passed);
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).unwrap_or_default()
            );
            0
        }
        _ => {
            eprintln!("Usage: toksi eval <list|score [harness] [model] [passed] [total]>");
            1
        }
    }
}
