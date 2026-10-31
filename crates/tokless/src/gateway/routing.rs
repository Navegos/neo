// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/routing.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,
    Medium,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRecommendation {
    pub complexity: TaskComplexity,
    pub recommended_model: &'static str,
    pub reasoning_effort: Option<&'static str>,
    pub score: f32,
}

/// Analyzes incoming request payload to classify task complexity
pub fn analyze_complexity(messages: &[Value], tools: Option<&[Value]>) -> ComplexityRecommendation {
    let mut total_chars = 0;
    let mut code_blocks = 0;
    let mut step_markers = 0;

    const KEYWORDS: &[&str] = &[
        "step-by-step", "prove", "architect", "refactor", "debug", "verify",
        "solve", "analyze", "explain why", "root cause",
    ];

    for m in messages {
        let content = m.get("content").and_then(Value::as_str).unwrap_or("");
        total_chars += content.chars().count();

        // Check for code blocks
        if content.contains("```") {
            code_blocks += 1;
        }

        // Check for multi-step reasoning keywords
        let lower = content.to_lowercase();
        for kw in KEYWORDS {
            if lower.contains(kw) {
                step_markers += 1;
            }
        }
    }

    let tool_count = tools.map(|t| t.len()).unwrap_or(0);

    // Score calculation
    let mut score = 0.0;
    score += (total_chars as f32 / 1000.0).min(5.0);
    score += (code_blocks as f32 * 1.5).min(3.0);
    score += (step_markers as f32 * 1.0).min(4.0);
    score += (tool_count as f32 * 0.5).min(2.0);

    if score >= 4.0 {
        ComplexityRecommendation {
            complexity: TaskComplexity::Complex,
            recommended_model: "claude-3-7-sonnet-20250219",
            reasoning_effort: Some("high"),
            score,
        }
    } else if score >= 1.5 {
        ComplexityRecommendation {
            complexity: TaskComplexity::Medium,
            recommended_model: "gpt-4o",
            reasoning_effort: None,
            score,
        }
    } else {
        ComplexityRecommendation {
            complexity: TaskComplexity::Simple,
            recommended_model: "gemini-2.0-flash",
            reasoning_effort: None,
            score,
        }
    }
}
