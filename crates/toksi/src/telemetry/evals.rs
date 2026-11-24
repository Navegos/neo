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
    "lighteval",
    "inspect_ai",
    "promptfoo",
    "deepeval",
    "ragas",
    "autofa",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalHarnessKind {
    LmEvalHarness,
    Lighteval,
    InspectAi,
    Promptfoo,
    DeepEval,
    Ragas,
    AutoFa,
}

impl EvalHarnessKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LmEvalHarness => "lm-evaluation-harness",
            Self::Lighteval => "lighteval",
            Self::InspectAi => "inspect_ai",
            Self::Promptfoo => "promptfoo",
            Self::DeepEval => "deepeval",
            Self::Ragas => "ragas",
            Self::AutoFa => "autofa",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "lm-evaluation-harness" | "lm-eval" => Some(Self::LmEvalHarness),
            "lighteval" => Some(Self::Lighteval),
            "inspect_ai" | "inspect-ai" => Some(Self::InspectAi),
            "promptfoo" => Some(Self::Promptfoo),
            "deepeval" => Some(Self::DeepEval),
            "ragas" => Some(Self::Ragas),
            "autofa" => Some(Self::AutoFa),
            _ => None,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeepEvalMetricKind {
    Faithfulness,
    AnswerRelevancy,
    ContextualPrecision,
    Hallucination,
    Toxicity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeepEvalMetric {
    pub kind: DeepEvalMetricKind,
    pub score: f64,
    pub threshold: f64,
    pub passed: bool,
}

impl DeepEvalMetric {
    pub fn evaluate(kind: DeepEvalMetricKind, score: f64, threshold: f64) -> Self {
        let clamped_score = score.clamp(0.0, 1.0);
        let clamped_threshold = threshold.clamp(0.0, 1.0);
        let passed = match kind {
            DeepEvalMetricKind::Hallucination | DeepEvalMetricKind::Toxicity => {
                clamped_score <= clamped_threshold
            }
            _ => clamped_score >= clamped_threshold,
        };
        Self {
            kind,
            score: clamped_score,
            threshold: clamped_threshold,
            passed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RagasScorecard {
    pub faithfulness: f64,
    pub answer_relevancy: f64,
    pub context_precision: f64,
    pub context_recall: f64,
}

impl RagasScorecard {
    pub fn composite_score(&self) -> f64 {
        let sum = self.faithfulness.clamp(0.0, 1.0)
            + self.answer_relevancy.clamp(0.0, 1.0)
            + self.context_precision.clamp(0.0, 1.0)
            + self.context_recall.clamp(0.0, 1.0);
        (sum / 4.0 * 1000.0).round() / 1000.0
    }

    pub fn passed(&self, min_score: f64) -> bool {
        self.composite_score() >= min_score
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFaFailureCluster {
    pub category: String,
    pub count: usize,
    pub sample_traces: Vec<String>,
}

pub struct AutoFaFailureClusterer;

impl AutoFaFailureClusterer {
    /// Deterministically clusters failed agent traces into root-cause buckets (`AutoFA`).
    pub fn cluster_failures(traces: &[(&str, &str)]) -> Vec<AutoFaFailureCluster> {
        let mut buckets: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for (trace_id, message) in traces {
            let low = message.to_ascii_lowercase();
            let category = if low.contains("429") || low.contains("rate limit") {
                "rate_limit"
            } else if low.contains("timeout") || low.contains("timed out") {
                "timeout"
            } else if low.contains("schema") || low.contains("json") || low.contains("parse") {
                "schema_parse"
            } else if low.contains("auth") || low.contains("401") || low.contains("403") {
                "auth_permission"
            } else {
                "execution_error"
            };
            buckets
                .entry(category.to_string())
                .or_default()
                .push((*trace_id).to_string());
        }
        buckets
            .into_iter()
            .map(|(category, sample_traces)| AutoFaFailureCluster {
                category,
                count: sample_traces.len(),
                sample_traces,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepeval_ragas_and_autofa_clustering() {
        let m = DeepEvalMetric::evaluate(DeepEvalMetricKind::Faithfulness, 0.92, 0.7);
        assert!(m.passed);
        let tox = DeepEvalMetric::evaluate(DeepEvalMetricKind::Toxicity, 0.05, 0.1);
        assert!(tox.passed);

        let ragas = RagasScorecard {
            faithfulness: 0.9,
            answer_relevancy: 0.8,
            context_precision: 0.85,
            context_recall: 0.85,
        };
        assert!(ragas.passed(0.8));

        let clusters = AutoFaFailureClusterer::cluster_failures(&[
            ("t1", "HTTP 429 rate limit exceeded"),
            ("t2", "JSON parse failure on tool call"),
            ("t3", "upstream 429 throttle"),
        ]);
        assert_eq!(clusters.len(), 2);
    }
}
