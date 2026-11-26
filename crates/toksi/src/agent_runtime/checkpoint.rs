// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/checkpoint.rs
// created: 2026-11-21

//! Unified Safe Plan Checkpointer combining Temporal event-history replay,
//! Restate journaled awakeables, LangGraph cyclic super-steps, and atomic
//! `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` persistence.

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};

use crate::projectmem::plan_checkpoint::{
    append_checkpoint_event, compute_file_sha256, load_plan_checkpoint, save_plan_checkpoint,
    sync_plan_md_checkboxes, validate_dirty_tree_and_plugins, CheckpointStatus, PlanCheckpoint,
    ResumePreflightReport, YieldReason,
};
use crate::util::paths::normalize_path;

use super::durable_journal::DurableJournal;
use super::graph_checkpoint::GraphCheckpoint;
use super::workflow_state::{DurableStepStatus, DurableWorkflowState};

pub struct SafePlanCheckpointer {
    pub repo_root: PathBuf,
    pub checkpoint: PlanCheckpoint,
    pub workflow: DurableWorkflowState,
    pub journal: DurableJournal,
    pub graph: GraphCheckpoint,
}

impl SafePlanCheckpointer {
    pub fn new(repo_root: PathBuf, plan_id: &str, title: &str) -> Self {
        let mut cp = PlanCheckpoint::new(plan_id, title);
        cp.repo_root = normalize_path(&repo_root.to_string_lossy());
        Self {
            repo_root,
            checkpoint: cp,
            workflow: DurableWorkflowState::new(plan_id, "temporal"),
            journal: DurableJournal::new(plan_id),
            graph: GraphCheckpoint::new(plan_id, 16),
        }
    }

    /// Captures active enabled plugins from `ToksiConfigDb` (`plugins_registry`)
    /// and `global_plugin_registry()`, plus SHA-256 digests of touched files.
    pub fn snapshot_runtime_state(&mut self, touched_files: &[&Path]) {
        let mut plugins: Vec<String> = crate::plugins::global_plugin_registry()
            .list()
            .into_iter()
            .filter(|p| p.is_enabled)
            .map(|p| format!("{}@{}", p.manifest.id, p.manifest.version))
            .collect();

        if let Ok(db) = crate::db::open_default_db()
            && let Ok(rows) = db.list_plugins() {
                for r in rows {
                    if r.is_enabled {
                        let tag = format!("{}@{}", r.id, r.version);
                        if !plugins.contains(&tag) {
                            plugins.push(tag);
                        }
                    }
                }
            }
        self.checkpoint.active_plugins = plugins;

        for rel_or_abs in touched_files {
            let full = if rel_or_abs.is_absolute() {
                rel_or_abs.to_path_buf()
            } else {
                self.repo_root.join(rel_or_abs)
            };
            if let Some(sha) = compute_file_sha256(&full) {
                let key = normalize_path(&rel_or_abs.to_string_lossy());
                self.checkpoint.dirty_files_sha256.insert(key, sha);
            }
        }
    }

    /// Atomically persists checkpoint + workflow + journal + graph + `events.jsonl` + `plan.md`
    /// before yielding on milestones, `429` rate-limit exhaustion, network drop, `SIGINT`, or crash.
    pub fn yield_and_checkpoint(
        &mut self,
        reason: YieldReason,
        handoff_summary: Option<&str>,
        last_error: Option<&str>,
    ) -> Result<PathBuf, String> {
        self.checkpoint.status = CheckpointStatus::Paused;
        self.checkpoint.yield_reason = Some(reason.clone());
        if let Some(h) = handoff_summary {
            self.checkpoint.handoff_summary = Some(h.to_string());
        }
        if let Some(err) = last_error {
            self.checkpoint.last_error = Some(err.to_string());
        }

        let step_id = format!("step-{}", self.checkpoint.current_step);
        self.workflow.append_entry(
            &step_id,
            DurableStepStatus::PausedRecoverable,
            format!("{}-{}", self.checkpoint.plan_id, self.checkpoint.current_step),
            handoff_summary.unwrap_or("yielded"),
        );
        self.checkpoint.durable_journal_seq = self.journal.record_activity(
            &step_id,
            format!("yield-{}-{}", self.checkpoint.plan_id, self.checkpoint.current_step),
            handoff_summary.unwrap_or("yielded"),
            false,
        );
        let _ = self.graph.advance_super_step(
            &step_id,
            vec![format!("step-{}", self.checkpoint.current_step + 1)],
            handoff_summary.unwrap_or("yielded"),
        );

        self.workflow.save_to_repo(&self.repo_root)?;
        self.journal.save_to_repo(&self.repo_root)?;
        self.graph.save_to_repo(&self.repo_root)?;

        let path = save_plan_checkpoint(Some(&self.repo_root), &mut self.checkpoint)
            .map_err(|e| e.to_string())?;
        let _ = sync_plan_md_checkboxes(Some(&self.repo_root), &self.checkpoint);
        let _ = append_checkpoint_event(
            Some(&self.repo_root),
            &self.checkpoint,
            &format!("yielded ({:?})", reason),
        );
        Ok(path)
    }

    /// Loads a checkpoint and runs preflight validation (dirty tree SHA-256 + `<enforce>` plugins).
    pub fn prepare_resume(repo_root: &Path, plan_id: &str) -> Result<(PlanCheckpoint, ResumePreflightReport), String> {
        let cp = load_plan_checkpoint(Some(repo_root), plan_id).map_err(|e| e.to_string())?;
        let report = validate_dirty_tree_and_plugins(Some(repo_root), &cp);
        Ok((cp, report))
    }
}
