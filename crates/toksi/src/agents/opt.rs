// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/opt.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentOptError {
    #[error("Token budget exceeded: limit {budget}, consumed {consumed}")]
    TokenBudgetExceeded { budget: u64, consumed: u64 },
    #[error("Turn timeout exceeded: limit {limit_ms}ms, elapsed {elapsed_ms}ms")]
    TurnTimeoutExceeded { limit_ms: u64, elapsed_ms: u64 },
    #[error("Execution step limit reached: maximum {max_steps} steps allowed")]
    StepLimitReached { max_steps: usize },
}

/// A recorded tool execution step in an agent conversational trajectory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStep {
    pub step_index: usize,
    pub tool_name: String,
    pub arguments_hash: String,
    pub outcome_summary: String,
    pub tokens_used: u64,
    pub duration_ms: u64,
    pub is_redundant: bool,
}

/// Budget and latency constraints for an agent turn or session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBudgetConstraints {
    pub max_tokens: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub max_steps: usize,
    pub prune_redundant_reads: bool,
}

impl Default for AgentBudgetConstraints {
    fn default() -> Self {
        Self {
            max_tokens: Some(50000),
            max_duration_ms: Some(120000),
            max_steps: 30,
            prune_redundant_reads: true,
        }
    }
}

/// Summary verdict of agent optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub total_steps_analyzed: usize,
    pub pruned_steps: usize,
    pub tokens_saved: u64,
    pub total_tokens_used: u64,
    pub total_duration_ms: u64,
    pub loop_detected: bool,
}

/// Agent step and tool execution optimizer modeled after agent-opt.
pub struct AgentOptimizer {
    constraints: AgentBudgetConstraints,
    executed_steps: Vec<AgentStep>,
    seen_calls: HashSet<(String, String)>, // (tool_name, arguments_hash)
    total_tokens: u64,
    total_duration_ms: u64,
}

impl Default for AgentOptimizer {
    fn default() -> Self {
        Self::new(AgentBudgetConstraints::default())
    }
}

impl AgentOptimizer {
    pub fn new(constraints: AgentBudgetConstraints) -> Self {
        Self {
            constraints,
            executed_steps: Vec::new(),
            seen_calls: HashSet::new(),
            total_tokens: 0,
            total_duration_ms: 0,
        }
    }

    /// Evaluates a candidate tool execution call before dispatch.
    /// Returns true if the step is permitted; false if pruned as redundant.
    pub fn evaluate_candidate_step(
        &mut self,
        tool_name: &str,
        arguments_repr: &str,
    ) -> Result<bool, AgentOptError> {
        if self.executed_steps.len() >= self.constraints.max_steps {
            return Err(AgentOptError::StepLimitReached {
                max_steps: self.constraints.max_steps,
            });
        }

        if let Some(budget) = self.constraints.max_tokens {
            if self.total_tokens >= budget {
                return Err(AgentOptError::TokenBudgetExceeded {
                    budget,
                    consumed: self.total_tokens,
                });
            }
        }

        if let Some(max_time) = self.constraints.max_duration_ms {
            if self.total_duration_ms >= max_time {
                return Err(AgentOptError::TurnTimeoutExceeded {
                    limit_ms: max_time,
                    elapsed_ms: self.total_duration_ms,
                });
            }
        }

        let arg_hash = format!("{:x}", simple_hash(arguments_repr));
        let call_key = (tool_name.to_string(), arg_hash);

        // Read-only tools like file reads or searches are pruned if identical arguments already executed
        let is_read_only = tool_name.contains("read") || tool_name.contains("search") || tool_name.contains("get_");
        if self.constraints.prune_redundant_reads && is_read_only && self.seen_calls.contains(&call_key) {
            return Ok(false); // Pruned!
        }

        self.seen_calls.insert(call_key);
        Ok(true)
    }

    /// Records completed tool step execution.
    pub fn record_completed_step(
        &mut self,
        tool_name: &str,
        arguments_repr: &str,
        outcome_summary: &str,
        tokens_used: u64,
        duration_ms: u64,
    ) {
        let arg_hash = format!("{:x}", simple_hash(arguments_repr));
        self.total_tokens = self.total_tokens.saturating_add(tokens_used);
        self.total_duration_ms = self.total_duration_ms.saturating_add(duration_ms);

        self.executed_steps.push(AgentStep {
            step_index: self.executed_steps.len() + 1,
            tool_name: tool_name.to_string(),
            arguments_hash: arg_hash,
            outcome_summary: outcome_summary.to_string(),
            tokens_used,
            duration_ms,
            is_redundant: false,
        });
    }

    /// Prunes and compresses intermediate trajectories for prompt injection.
    pub fn generate_optimized_trajectory(&self) -> (Vec<AgentStep>, OptimizationReport) {
        let mut pruned = Vec::new();
        let mut pruned_count = 0usize;
        let mut saved_tokens = 0u64;
        let mut loop_detected = false;

        let mut prev_tool = "";
        let mut repeat_count = 0usize;

        for step in &self.executed_steps {
            if step.tool_name == prev_tool {
                repeat_count += 1;
                if repeat_count >= 3 {
                    loop_detected = true;
                    pruned_count += 1;
                    saved_tokens = saved_tokens.saturating_add(step.tokens_used);
                    continue;
                }
            } else {
                repeat_count = 0;
                prev_tool = &step.tool_name;
            }

            pruned.push(step.clone());
        }

        let report = OptimizationReport {
            total_steps_analyzed: self.executed_steps.len(),
            pruned_steps: pruned_count,
            tokens_saved: saved_tokens,
            total_tokens_used: self.total_tokens,
            total_duration_ms: self.total_duration_ms,
            loop_detected,
        };

        (pruned, report)
    }
}

fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_optimizer_redundant_read_pruning() {
        let mut opt = AgentOptimizer::default();
        let t1 = opt.evaluate_candidate_step("read_file", "path: /src/main.rs").expect("eval");
        assert!(t1); // first read allowed

        opt.record_completed_step("read_file", "path: /src/main.rs", "fn main()", 300, 20);

        // Repeated identical read should be pruned
        let t2 = opt.evaluate_candidate_step("read_file", "path: /src/main.rs").expect("eval 2");
        assert!(!t2); // duplicate read pruned!
    }

    #[test]
    fn test_budget_exceeded_enforcement() {
        let constraints = AgentBudgetConstraints {
            max_tokens: Some(500),
            max_duration_ms: Some(1000),
            max_steps: 10,
            prune_redundant_reads: true,
        };
        let mut opt = AgentOptimizer::new(constraints);
        opt.record_completed_step("cmd", "cargo check", "ok", 600, 50);

        let res = opt.evaluate_candidate_step("cmd", "cargo test");
        assert!(matches!(res, Err(AgentOptError::TokenBudgetExceeded { .. })));
    }
}
