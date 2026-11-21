// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/workflow_state.rs
// created: 2026-11-21

//! Durable workflow state & replay journal (`temporal`, `restate`, `langgraph`, `deepagents`)
//! persisted inside `<repo>/.toksi/projectmem/plan_checkpoints/`.

#![deny(unsafe_code)]

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurableStepStatus {
    Pending,
    Running,
    Completed,
    PausedRecoverable,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableJournalEntry {
    pub seq: u64,
    pub step_id: String,
    pub status: DurableStepStatus,
    pub idempotency_key: String,
    pub output_digest: String,
    pub timestamp_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableWorkflowState {
    pub workflow_id: String,
    pub engine: String, // temporal | restate | langgraph
    pub current_step: usize,
    pub journal: Vec<DurableJournalEntry>,
}

impl DurableWorkflowState {
    pub fn new(workflow_id: impl Into<String>, engine: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            engine: engine.into(),
            current_step: 0,
            journal: Vec::new(),
        }
    }

    pub fn append_entry(
        &mut self,
        step_id: impl Into<String>,
        status: DurableStepStatus,
        idempotency_key: impl Into<String>,
        output_digest: impl Into<String>,
    ) {
        let seq = self.journal.len() as u64 + 1;
        if status == DurableStepStatus::Completed {
            self.current_step += 1;
        }
        self.journal.push(DurableJournalEntry {
            seq,
            step_id: step_id.into(),
            status,
            idempotency_key: idempotency_key.into(),
            output_digest: output_digest.into(),
            timestamp_unix: chrono::Utc::now().timestamp(),
        });
    }

    pub fn save_to_repo(&self, repo_root: &Path) -> Result<(), String> {
        let dir = repo_root
            .join(".toksi")
            .join("projectmem")
            .join("plan_checkpoints");
        fs::create_dir_all(&dir).map_err(|e| format!("workflow mkdir failed: {e}"))?;
        let path = dir.join(format!("wf_{}.json", self.workflow_id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("workflow serialize failed: {e}"))?;
        fs::write(path, json).map_err(|e| format!("workflow write failed: {e}"))
    }

    pub fn load_from_repo(repo_root: &Path, workflow_id: &str) -> Result<Self, String> {
        let path = repo_root
            .join(".toksi")
            .join("projectmem")
            .join("plan_checkpoints")
            .join(format!("wf_{workflow_id}.json"));
        let raw = fs::read_to_string(path).map_err(|e| format!("workflow read failed: {e}"))?;
        serde_json::from_str(&raw).map_err(|e| format!("workflow parse failed: {e}"))
    }
}
