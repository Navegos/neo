// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/rl.rs
// created: 2026-11-21

//! Reinforcement Learning & agentic trajectory contracts (`trl`, `torch-rl`, `Gym`, `RL`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RlAlgorithm {
    Ppo,
    Grpo,
    Dpo,
    Sft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlTrajectoryStep {
    pub step_idx: usize,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RlEpisodeSummary {
    pub algorithm: RlAlgorithm,
    pub steps: usize,
    pub mean_reward: f64,
    pub total_tokens: usize,
}

/// Summarizes an RL rollout episode for `trl`/`torch-rl`/`Gym`/`RL` telemetry.
pub fn summarize_rl_episode(algorithm: RlAlgorithm, steps: &[RlTrajectoryStep]) -> RlEpisodeSummary {
    let count = steps.len();
    let total_reward: f64 = steps.iter().map(|s| s.reward).sum();
    let total_tokens: usize = steps
        .iter()
        .map(|s| s.prompt_tokens + s.completion_tokens)
        .sum();
    RlEpisodeSummary {
        algorithm,
        steps: count,
        mean_reward: if count == 0 {
            0.0
        } else {
            total_reward / count as f64
        },
        total_tokens,
    }
}
