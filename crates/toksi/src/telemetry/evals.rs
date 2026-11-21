// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/evals.rs
// created: 2026-11-21

//! Standardized Evaluation Harness registry (`lm-evaluation-harness`, `sgl-eval`,
//! `genai-bench`, `guidellm`, `perf-eval`, `Evaluator`, `nemo-compass`, `eval-author`,
//! `Gym`, `nemo-helix`, `UserSim`, `DataDesigner*`, `deepseek-harness`,
//! `optimum-benchmark`, `torchmetrics`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const SUPPORTED_EVAL_HARNESSES: &[&str] = &[
    "lm-evaluation-harness",
    "sgl-eval",
    "genai-bench",
    "guidellm",
    "perf-eval",
    "Evaluator",
    "nemo-compass",
    "eval-author",
    "Gym",
    "nemo-helix",
    "UserSim",
    "DataDesigner",
    "DataDesignerPlugins",
    "deepseek-harness",
    "optimum-benchmark",
    "torchmetrics",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalRunSummary {
    pub harness: String,
    pub model_id: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub score: f64,
}

pub fn compute_eval_summary(
    harness: &str,
    model_id: &str,
    total_cases: usize,
    passed_cases: usize,
) -> EvalRunSummary {
    let clamped_passed = passed_cases.min(total_cases);
    let score = if total_cases == 0 {
        0.0
    } else {
        clamped_passed as f64 / total_cases as f64
    };
    EvalRunSummary {
        harness: harness.to_string(),
        model_id: model_id.to_string(),
        total_cases,
        passed_cases: clamped_passed,
        score,
    }
}
