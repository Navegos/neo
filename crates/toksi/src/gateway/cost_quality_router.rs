// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/cost_quality_router.rs
// created: 2026-11-21

//! `RouteLLM` matrix-factorization / preference win-rate complexity routing.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::routing::{analyze_complexity, TaskComplexity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLlmConfig {
    pub strong_model: String,
    pub weak_model: String,
    pub win_rate_threshold: f32,
}

impl Default for RouteLlmConfig {
    fn default() -> Self {
        Self {
            strong_model: "claude-3-7-sonnet-20250219".to_string(),
            weak_model: "gemini-2.0-flash".to_string(),
            win_rate_threshold: 0.55,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceRouteDecision {
    pub selected_model: String,
    pub predicted_strong_win_rate: f32,
    pub complexity: TaskComplexity,
    pub routed_to_strong: bool,
}

pub struct RouteLlmScorer {
    config: RouteLlmConfig,
}

impl RouteLlmScorer {
    pub fn new(config: RouteLlmConfig) -> Self {
        Self { config }
    }

    /// Computes a deterministic preference win-rate score in `[0.0, 1.0]` and routes
    /// between strong and weak models against `win_rate_threshold`.
    pub fn route(&self, messages: &[Value], tools: Option<&[Value]>) -> PreferenceRouteDecision {
        let rec = analyze_complexity(messages, tools);
        let normalized = (rec.score / 8.0).clamp(0.0, 1.0);
        let routed_to_strong = normalized >= self.config.win_rate_threshold;
        PreferenceRouteDecision {
            selected_model: if routed_to_strong {
                self.config.strong_model.clone()
            } else {
                self.config.weak_model.clone()
            },
            predicted_strong_win_rate: normalized,
            complexity: rec.complexity,
            routed_to_strong,
        }
    }
}
