// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/plan_checkpoint.rs
// created: 2026-11-18

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use super::error::{ProjectMemError, Result};
use super::redaction::redact;
use super::schema::utc_now_iso;
use super::store::{atomic_write_file, find_repo_root, require_mem_dir, TOKSI_MEM_SUBDIR};
use crate::util::paths::normalize_path;

pub const PLAN_CHECKPOINTS_DIR: &str = "plan_checkpoints";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

impl Default for CheckpointStatus {
    fn default() -> Self {
        Self::Paused
    }
}

/// Atomic durable plan checkpoint persisted at
/// `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json`
/// for Temporal/Restate/LangGraph-compatible pause & `/resume`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCheckpoint {
    pub plan_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: CheckpointStatus,
    #[serde(default)]
    pub current_step: usize,
    #[serde(default)]
    pub total_steps: usize,
    #[serde(default)]
    pub completed_steps: Vec<String>,
    #[serde(default)]
    pub pending_steps: Vec<String>,
    #[serde(default)]
    pub active_plugins: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_backend: Option<String>,
    #[serde(default)]
    pub durable_journal_seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_snapshot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subproject_id: Option<String>,
    #[serde(default)]
    pub updated_at: String,
}

impl PlanCheckpoint {
    pub fn new(plan_id: &str, title: &str) -> Self {
        Self {
            plan_id: sanitize_plan_id(plan_id),
            title: title.trim().to_string(),
            status: CheckpointStatus::Paused,
            current_step: 0,
            total_steps: 0,
            completed_steps: Vec::new(),
            pending_steps: Vec::new(),
            active_plugins: Vec::new(),
            sandbox_backend: None,
            durable_journal_seq: 0,
            context_snapshot: None,
            subproject_id: None,
            updated_at: utc_now_iso(),
        }
    }
}

pub fn sanitize_plan_id(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "default_plan".to_string()
    } else {
        cleaned
    }
}

pub fn checkpoints_dir(root: Option<&Path>) -> Result<PathBuf> {
    let mem_dir = match require_mem_dir(root) {
        Ok(d) => d,
        Err(_) => {
            let repo_root = find_repo_root(root);
            repo_root.join(TOKSI_MEM_SUBDIR)
        }
    };
    Ok(mem_dir.join(PLAN_CHECKPOINTS_DIR))
}

pub fn checkpoint_path(root: Option<&Path>, plan_id: &str) -> Result<PathBuf> {
    let safe_id = sanitize_plan_id(plan_id);
    Ok(checkpoints_dir(root)?.join(format!("{}.json", safe_id)))
}

pub fn save_plan_checkpoint(root: Option<&Path>, checkpoint: &mut PlanCheckpoint) -> Result<PathBuf> {
    if checkpoint.plan_id.trim().is_empty() {
        return Err(ProjectMemError::Validation("plan_id cannot be empty".to_string()));
    }
    checkpoint.plan_id = sanitize_plan_id(&checkpoint.plan_id);
    checkpoint.updated_at = utc_now_iso();

    let (title_redacted, _) = redact(&checkpoint.title);
    checkpoint.title = title_redacted;
    if let Some(ref ctx) = checkpoint.context_snapshot {
        let (ctx_redacted, _) = redact(ctx);
        checkpoint.context_snapshot = Some(ctx_redacted);
    }
    checkpoint.completed_steps = checkpoint
        .completed_steps
        .iter()
        .map(|s| redact(s).0)
        .collect();
    checkpoint.pending_steps = checkpoint
        .pending_steps
        .iter()
        .map(|s| redact(s).0)
        .collect();

    let path = checkpoint_path(root, &checkpoint.plan_id)?;
    let json = serde_json::to_string_pretty(checkpoint)?;
    atomic_write_file(&path, &json)?;
    Ok(path)
}

pub fn load_plan_checkpoint(root: Option<&Path>, plan_id: &str) -> Result<PlanCheckpoint> {
    let path = checkpoint_path(root, plan_id)?;
    if !path.exists() {
        return Err(ProjectMemError::NotFound(format!(
            "Plan checkpoint '{}' not found at {}",
            plan_id,
            normalize_path(&path.to_string_lossy())
        )));
    }
    let raw = fs::read_to_string(&path)?;
    let cp: PlanCheckpoint = serde_json::from_str(&raw)?;
    Ok(cp)
}

pub fn list_plan_checkpoints(root: Option<&Path>) -> Result<Vec<PlanCheckpoint>> {
    let dir = checkpoints_dir(root)?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut list = Vec::new();
    for entry in fs::read_dir(&dir)?.flatten() {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(cp) = serde_json::from_str::<PlanCheckpoint>(&raw) {
                    list.push(cp);
                }
            }
        }
    }
    list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(list)
}

pub fn latest_resumable_checkpoint(root: Option<&Path>) -> Result<Option<PlanCheckpoint>> {
    let all = list_plan_checkpoints(root)?;
    Ok(all
        .into_iter()
        .find(|cp| matches!(cp.status, CheckpointStatus::Paused | CheckpointStatus::Active)))
}

pub fn delete_plan_checkpoint(root: Option<&Path>, plan_id: &str) -> Result<bool> {
    let path = checkpoint_path(root, plan_id)?;
    if path.exists() {
        fs::remove_file(&path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
