// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/workforce.rs
// created: 2026-11-24

//! Pure-Rust multi-worker `WorkforceCoordinator` with role queues, task handoffs,
//! failure-aware retry recovery, and `agent_id` attribution.

#![deny(unsafe_code)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkTaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSlot {
    pub worker_id: String,
    pub role: String,
    pub agent_id: String,
    pub busy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkforceTask {
    pub task_id: String,
    pub role: String,
    pub description: String,
    pub status: WorkTaskStatus,
    pub assigned_worker: Option<String>,
    pub assigned_agent_id: Option<String>,
    pub attempts: u32,
    pub max_retries: u32,
    pub result: Option<String>,
    pub last_error: Option<String>,
}

/// Multi-worker coordinator with role queues and failure-aware retry handoffs.
#[derive(Debug, Clone, Default)]
pub struct WorkforceCoordinator {
    workers: Vec<WorkerSlot>,
    tasks: Vec<WorkforceTask>,
    queue: VecDeque<usize>,
}

impl WorkforceCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_worker(&mut self, worker_id: &str, role: &str, agent_id: &str) {
        if let Some(existing) = self.workers.iter_mut().find(|w| w.worker_id == worker_id) {
            existing.role = role.to_string();
            existing.agent_id = agent_id.to_string();
            return;
        }
        self.workers.push(WorkerSlot {
            worker_id: worker_id.to_string(),
            role: role.to_string(),
            agent_id: agent_id.to_string(),
            busy: false,
        });
    }

    pub fn submit_task(&mut self, task_id: &str, role: &str, description: &str, max_retries: u32) {
        let idx = self.tasks.len();
        self.tasks.push(WorkforceTask {
            task_id: task_id.to_string(),
            role: role.to_string(),
            description: description.to_string(),
            status: WorkTaskStatus::Queued,
            assigned_worker: None,
            assigned_agent_id: None,
            attempts: 0,
            max_retries,
            result: None,
            last_error: None,
        });
        self.queue.push_back(idx);
    }

    /// Claims the next queued task matching the idle worker's role (or general queue if matching).
    pub fn claim_next_for_worker(&mut self, worker_id: &str) -> Option<WorkforceTask> {
        let worker_idx = self.workers.iter().position(|w| w.worker_id == worker_id)?;
        if self.workers[worker_idx].busy {
            return None;
        }
        let worker_role = self.workers[worker_idx].role.clone();
        let worker_agent = self.workers[worker_idx].agent_id.clone();

        let queue_pos = self.queue.iter().position(|&task_idx| {
            let role = &self.tasks[task_idx].role;
            role.eq_ignore_ascii_case(&worker_role) || role == "*"
        })?;

        let task_idx = self.queue.remove(queue_pos)?;
        self.workers[worker_idx].busy = true;
        let task = &mut self.tasks[task_idx];
        task.status = WorkTaskStatus::Running;
        task.attempts += 1;
        task.assigned_worker = Some(worker_id.to_string());
        task.assigned_agent_id = Some(worker_agent);
        Some(task.clone())
    }

    pub fn complete_task(&mut self, task_id: &str, output: &str) -> Result<(), String> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| format!("Unknown task '{task_id}'"))?;
        if let Some(wid) = &task.assigned_worker
            && let Some(w) = self.workers.iter_mut().find(|w| &w.worker_id == wid)
        {
            w.busy = false;
        }
        task.status = WorkTaskStatus::Completed;
        task.result = Some(output.to_string());
        Ok(())
    }

    /// Records failure and either re-queues for handoff retry or marks terminal `Failed`.
    pub fn fail_task(&mut self, task_id: &str, error: &str) -> Result<WorkTaskStatus, String> {
        let task_idx = self
            .tasks
            .iter()
            .position(|t| t.task_id == task_id)
            .ok_or_else(|| format!("Unknown task '{task_id}'"))?;
        if let Some(wid) = self.tasks[task_idx].assigned_worker.clone()
            && let Some(w) = self.workers.iter_mut().find(|w| w.worker_id == wid)
        {
            w.busy = false;
        }
        let task = &mut self.tasks[task_idx];
        task.last_error = Some(error.to_string());
        if task.attempts <= task.max_retries {
            task.status = WorkTaskStatus::Queued;
            task.assigned_worker = None;
            self.queue.push_back(task_idx);
            Ok(WorkTaskStatus::Queued)
        } else {
            task.status = WorkTaskStatus::Failed;
            Ok(WorkTaskStatus::Failed)
        }
    }

    pub fn tasks(&self) -> &[WorkforceTask] {
        &self.tasks
    }
}
