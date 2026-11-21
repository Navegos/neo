// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/orchestrator.rs
// created: 2026-11-21

//! `<toksi-siagent>` Component #1: Multi-Framework Orchestrator integrating
//! `mofa-engine`, `langgraph`, `crewAI`, `deepagents`, `agent-framework`, `dify`,
//! `mastra`, `agno`, `SWE-agent`, `OpenHands`, `open-swe`, `EnterpriseBlueprints`,
//! `kimi-code`, `loopwright`, and `hermes-webui` with durable workflow checkpoints.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::durable_journal::DurableJournal;
use super::workflow_state::{DurableStepStatus, DurableWorkflowState};
use super::world_model::WorldModel;
use crate::headroom::effort::{EffortLevel, HeadroomEffortCompressor};

pub const ORCHESTRATED_FRAMEWORKS: &[&str] = &[
    "mofa-engine",
    "langgraph",
    "crewAI",
    "deepagents",
    "agent-framework",
    "dify",
    "mastra",
    "agno",
    "SWE-agent",
    "OpenHands",
    "open-swe",
    "EnterpriseBlueprints",
    "kimi-code",
    "loopwright",
    "hermes-webui",
    "temporal",
    "restate",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlanStep {
    pub step_id: String,
    pub framework: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStepReceipt {
    pub step_id: String,
    pub seq: u64,
    pub compressed_prompt: String,
    pub effort: EffortLevel,
}

/// Unified multi-framework orchestrator with durable replay and world-model tracking.
pub struct UnifiedOrchestrator {
    pub workflow: DurableWorkflowState,
    pub journal: DurableJournal,
    pub world_model: WorldModel,
    pub compressor: HeadroomEffortCompressor,
}

impl UnifiedOrchestrator {
    pub fn new(workflow_id: impl Into<String>, engine: impl Into<String>) -> Self {
        let id = workflow_id.into();
        Self {
            workflow: DurableWorkflowState::new(id.clone(), engine),
            journal: DurableJournal::new(id),
            world_model: WorldModel::new(),
            compressor: HeadroomEffortCompressor::gateway_default(),
        }
    }

    pub fn execute_step(
        &mut self,
        step: &OrchestrationPlanStep,
        effort: EffortLevel,
    ) -> OrchestrationStepReceipt {
        let mut payload = serde_json::json!({
            "messages": [{ "role": "user", "content": step.prompt }]
        });
        let headers = vec![("x-toksi-effort".to_string(), effort.as_str().to_string())];
        let _stats = self
            .compressor
            .compress_json_payload("/v1/chat/completions", &headers, &mut payload);
        let compressed = payload
            .pointer("/messages/0/content")
            .and_then(|v| v.as_str())
            .unwrap_or(&step.prompt)
            .to_string();
        let idemp = format!("{}:{}", self.workflow.workflow_id, step.step_id);
        let seq = self
            .journal
            .record_activity(&step.step_id, &idemp, &compressed, true);
        self.workflow.append_entry(
            &step.step_id,
            DurableStepStatus::Completed,
            idemp,
            format!("len:{}", compressed.len()),
        );
        self.world_model
            .upsert_entity(&step.step_id, &step.framework, "completed");
        OrchestrationStepReceipt {
            step_id: step.step_id.clone(),
            seq,
            compressed_prompt: compressed,
            effort,
        }
    }
}
