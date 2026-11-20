// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/crew.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! CrewAI-inspired orchestration primitives: `TaskAnalyzer`, `ReflectionAgent` (max 3 rounds),
//! `ProcessMode`, `ReasoningPlan`, `InterceptionPoint`/`HookAborted`, and causal `EventBus`.

#![deny(unsafe_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::mofa::SubtaskDAG;

pub const MAX_REFLECTION_ROUNDS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessMode {
    Sequential,
    Hierarchical,
    Parallel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_id: String,
    pub description: String,
    pub rationale: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPlan {
    pub goal: String,
    pub mode: ProcessMode,
    pub steps: Vec<ReasoningStep>,
}

/// Decomposes goals into `ReasoningPlan` and `SubtaskDAG`.
#[derive(Debug, Default)]
pub struct TaskAnalyzer;

impl TaskAnalyzer {
    pub fn analyze(&self, goal: &str, mode: ProcessMode) -> (ReasoningPlan, SubtaskDAG) {
        let mut dag = SubtaskDAG::new();
        let steps = vec![
            ReasoningStep {
                step_id: "step-1-inspect".to_string(),
                description: format!("Inspect workspace and gather context for: {}", goal),
                rationale: "Ground execution in repository state before changes".to_string(),
                completed: false,
            },
            ReasoningStep {
                step_id: "step-2-execute".to_string(),
                description: format!("Execute primary objective: {}", goal),
                rationale: "Apply minimal, verified changes or analysis".to_string(),
                completed: false,
            },
            ReasoningStep {
                step_id: "step-3-verify".to_string(),
                description: "Verify results against constraints and reflection criteria".to_string(),
                rationale: "Ensure correctness before finalizing".to_string(),
                completed: false,
            },
        ];

        dag.add_task("step-1-inspect", &steps[0].description, "researcher");
        dag.add_task("step-2-execute", &steps[1].description, "builder");
        dag.add_task("step-3-verify", &steps[2].description, "reviewer");
        let _ = dag.add_dependency("step-1-inspect", "step-2-execute");
        let _ = dag.add_dependency("step-2-execute", "step-3-verify");

        (
            ReasoningPlan {
                goal: goal.to_string(),
                mode,
                steps,
            },
            dag,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionOutcome {
    pub rounds_used: usize,
    pub approved: bool,
    pub critique: String,
    pub final_output: String,
}

/// Iterative reflection agent capped at `MAX_REFLECTION_ROUNDS` (3).
#[derive(Debug)]
pub struct ReflectionAgent {
    pub max_rounds: usize,
}

impl Default for ReflectionAgent {
    fn default() -> Self {
        Self {
            max_rounds: MAX_REFLECTION_ROUNDS,
        }
    }
}

impl ReflectionAgent {
    pub fn reflect<F>(&self, initial_output: &str, mut refine: F) -> ReflectionOutcome
    where
        F: FnMut(usize, &str, &str) -> (bool, String, String),
    {
        let cap = self.max_rounds.min(MAX_REFLECTION_ROUNDS).max(1);
        let mut current = initial_output.to_string();
        let mut last_critique = String::new();

        for round in 1..=cap {
            let (approved, critique, revised) = refine(round, &current, &last_critique);
            last_critique = critique;
            current = revised;
            if approved {
                return ReflectionOutcome {
                    rounds_used: round,
                    approved: true,
                    critique: last_critique,
                    final_output: current,
                };
            }
        }

        ReflectionOutcome {
            rounds_used: cap,
            approved: false,
            critique: last_critique,
            final_output: current,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterceptionPoint {
    BeforeTask,
    AfterTask,
    BeforeTool,
    AfterTool,
    OnError,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Hook aborted at {point:?}: {reason}")]
pub struct HookAborted {
    pub point: InterceptionPoint,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvent {
    pub seq: u64,
    pub parent_seq: Option<u64>,
    pub topic: String,
    pub payload: Value,
}

/// Causal event bus tracking parent-child event lineage.
#[derive(Debug, Default)]
pub struct EventBus {
    seq: AtomicU64,
    events: RwLock<Vec<CausalEvent>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&self, parent_seq: Option<u64>, topic: impl Into<String>, payload: Value) -> u64 {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        let ev = CausalEvent {
            seq,
            parent_seq,
            topic: topic.into(),
            payload,
        };
        if let Ok(mut guard) = self.events.write() {
            guard.push(ev);
        }
        seq
    }

    pub fn snapshot(&self) -> Vec<CausalEvent> {
        self.events
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}
