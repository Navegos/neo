// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/plan_checkpoint.rs
// created: 2026-11-18

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::error::{ProjectMemError, Result};
use super::redaction::redact;
use super::schema::{utc_now_iso, Event};
use super::store::{
    append_event, atomic_write_file, find_repo_root, require_mem_dir, TOKSI_MEM_SUBDIR,
};
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointStepStatus {
    Pending,
    Running,
    Completed,
    Paused,
    Failed,
}

impl Default for CheckpointStepStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDigestRecord {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanStepRecord {
    pub id: usize,
    #[serde(default)]
    pub step_key: String,
    pub title: String,
    #[serde(default)]
    pub status: CheckpointStepStatus,
    #[serde(default)]
    pub files_touched: Vec<FileDigestRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_baseline_delta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YieldReason {
    Milestone,
    RateLimit429,
    NetworkDrop,
    Sigint,
    Crash,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResumePreflightReport {
    pub plan_id: String,
    pub next_step_index: usize,
    pub next_step_title: Option<String>,
    pub completed_count: usize,
    pub total_steps: usize,
    pub dirty_mismatches: Vec<String>,
    pub missing_plugins: Vec<String>,
    pub verified_plugins: Vec<String>,
}

fn default_schema_version() -> u32 {
    1
}

/// Atomic durable plan checkpoint persisted at
/// `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json`
/// for Temporal/Restate/LangGraph-compatible pause & `/resume`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCheckpoint {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub plan_id: String,
    #[serde(default)]
    pub repo_root: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: CheckpointStatus,
    #[serde(default)]
    pub current_step: usize,
    #[serde(default)]
    pub total_steps: usize,
    #[serde(default)]
    pub active_effort: String,
    #[serde(default)]
    pub active_model: String,
    #[serde(default)]
    pub active_phase: String,
    #[serde(default)]
    pub active_byok_profile: String,
    #[serde(default)]
    pub completed_steps: Vec<String>,
    #[serde(default)]
    pub pending_steps: Vec<String>,
    #[serde(default)]
    pub steps: Vec<PlanStepRecord>,
    #[serde(default)]
    pub active_plugins: Vec<String>,
    #[serde(default)]
    pub dirty_files_sha256: BTreeMap<String, String>,
    #[serde(default)]
    pub dirty_files_blake3: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_backend: Option<String>,
    #[serde(default)]
    pub durable_journal_seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_snapshot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yield_reason: Option<YieldReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subproject_id: Option<String>,
    #[serde(default)]
    pub updated_at: String,
}

impl PlanCheckpoint {
    pub fn new(plan_id: &str, title: &str) -> Self {
        Self {
            schema_version: 1,
            plan_id: sanitize_plan_id(plan_id),
            repo_root: String::new(),
            title: title.trim().to_string(),
            status: CheckpointStatus::Paused,
            current_step: 0,
            total_steps: 0,
            active_effort: "Medium".to_string(),
            active_model: String::new(),
            active_phase: String::new(),
            active_byok_profile: String::new(),
            completed_steps: Vec::new(),
            pending_steps: Vec::new(),
            steps: Vec::new(),
            active_plugins: Vec::new(),
            dirty_files_sha256: BTreeMap::new(),
            dirty_files_blake3: BTreeMap::new(),
            sandbox_backend: None,
            durable_journal_seq: 0,
            context_snapshot: None,
            handoff_summary: None,
            last_error: None,
            last_error_reason: None,
            yield_reason: None,
            subproject_id: None,
            updated_at: utc_now_iso(),
        }
    }

    /// Returns the index and title of the first incomplete step (`Pending | Running | Paused | Failed`).
    pub fn first_incomplete_step(&self) -> (usize, Option<String>) {
        if !self.steps.is_empty() {
            for (idx, st) in self.steps.iter().enumerate() {
                if st.status != CheckpointStepStatus::Completed {
                    return (idx, Some(st.title.clone()));
                }
            }
            return (self.steps.len(), None);
        }
        (self.completed_steps.len(), self.pending_steps.first().cloned())
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

pub fn compute_file_sha256(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(hex::encode(hasher.finalize()))
}

/// Synchronizes completed step titles into `<repo>/.toksi/projectmem/plan.md` checkboxes (`[ ]` -> `[x]`).
pub fn sync_plan_md_checkboxes(root: Option<&Path>, checkpoint: &PlanCheckpoint) -> Result<usize> {
    let repo_root = find_repo_root(root);
    let plan_md = repo_root.join(TOKSI_MEM_SUBDIR).join("plan.md");
    if !plan_md.is_file() {
        return Ok(0);
    }
    let raw = fs::read_to_string(&plan_md)?;
    let mut completed_titles: Vec<String> = checkpoint
        .steps
        .iter()
        .filter(|s| s.status == CheckpointStepStatus::Completed)
        .map(|s| s.title.trim().to_ascii_lowercase())
        .collect();
    for t in &checkpoint.completed_steps {
        let low = t.trim().to_ascii_lowercase();
        if !low.is_empty() && !completed_titles.contains(&low) {
            completed_titles.push(low);
        }
    }
    if completed_titles.is_empty() {
        return Ok(0);
    }

    let mut updated = 0usize;
    let mut out_lines = Vec::new();
    for line in raw.lines() {
        let low = line.to_ascii_lowercase();
        if line.contains("- [ ]") && completed_titles.iter().any(|t| low.contains(t)) {
            out_lines.push(line.replacen("- [ ]", "- [x]", 1));
            updated += 1;
        } else {
            out_lines.push(line.to_string());
        }
    }
    if updated > 0 {
        let mut rendered = out_lines.join("\n");
        if raw.ends_with('\n') {
            rendered.push('\n');
        }
        atomic_write_file(&plan_md, &rendered)?;
    }
    Ok(updated)
}

/// Appends a checkpoint decision/note event to `<repo>/.toksi/projectmem/events.jsonl`.
pub fn append_checkpoint_event(
    root: Option<&Path>,
    checkpoint: &PlanCheckpoint,
    reason: &str,
) -> Result<()> {
    let repo_root = find_repo_root(root);
    let mem_dir = match require_mem_dir(Some(&repo_root)) {
        Ok(d) => d,
        Err(_) => {
            let d = repo_root.join(TOKSI_MEM_SUBDIR);
            fs::create_dir_all(&d)?;
            d
        }
    };
    let events_file = mem_dir.join("events.jsonl");
    if !events_file.exists() {
        let _ = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&events_file)?;
    }
    let (summary, _) = redact(&format!(
        "PlanCheckpoint '{}' ({:?}): step {}/{} — {}",
        checkpoint.plan_id,
        checkpoint.status,
        checkpoint.current_step,
        checkpoint.total_steps,
        reason
    ));
    let mut event = Event::new("decision", &summary);
    append_event(&mut event, Some(&repo_root))?;
    Ok(())
}

/// Validates dirty working-tree SHA-256 digests and active plugins under `<enforce>`
/// before resuming from the exact incomplete step.
pub fn validate_dirty_tree_and_plugins(
    root: Option<&Path>,
    checkpoint: &PlanCheckpoint,
) -> ResumePreflightReport {
    let repo_root = find_repo_root(root);
    let mut dirty_mismatches = Vec::new();
    for (rel, expected_sha) in &checkpoint.dirty_files_sha256 {
        let full = repo_root.join(rel);
        match compute_file_sha256(&full) {
            Some(actual) if actual == *expected_sha => {}
            Some(actual) => dirty_mismatches.push(format!(
                "{} (expected={}, actual={})",
                normalize_path(rel),
                &expected_sha[..expected_sha.len().min(8)],
                &actual[..actual.len().min(8)]
            )),
            None => dirty_mismatches.push(format!("{} (missing on disk)", normalize_path(rel))),
        }
    }

    let registry = crate::plugins::global_plugin_registry();
    let loaded = registry.list();
    let mut verified_plugins = Vec::new();
    let mut missing_plugins = Vec::new();
    for pid in &checkpoint.active_plugins {
        let base_id = pid.split('@').next().unwrap_or(pid);
        if loaded.iter().any(|p| p.manifest.id == base_id && p.is_enabled) {
            verified_plugins.push(pid.clone());
        } else {
            missing_plugins.push(pid.clone());
        }
    }

    let (next_step_index, next_step_title) = checkpoint.first_incomplete_step();
    let completed_count = if !checkpoint.steps.is_empty() {
        checkpoint
            .steps
            .iter()
            .filter(|s| s.status == CheckpointStepStatus::Completed)
            .count()
    } else {
        checkpoint.completed_steps.len()
    };
    let total_steps = checkpoint
        .total_steps
        .max(checkpoint.steps.len())
        .max(checkpoint.completed_steps.len() + checkpoint.pending_steps.len());

    ResumePreflightReport {
        plan_id: checkpoint.plan_id.clone(),
        next_step_index,
        next_step_title,
        completed_count,
        total_steps,
        dirty_mismatches,
        missing_plugins,
        verified_plugins,
    }
}
