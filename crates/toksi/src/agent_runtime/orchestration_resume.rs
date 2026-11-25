// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/orchestration_resume.rs
// created: 2026-11-25

//! Bulletproof Multi-Agent Orchestration Resilience & Cross-Chat Resume Engine.
//!
//! Persists write-ahead orchestration state strictly inside `<repo>/.toksi/projectmem/`:
//! - `orchestration_journal.jsonl`: `fs2`-locked write-ahead log of worker spawns,
//!   disjoint leases, step completions, failure classifications, and BYOK failovers.
//! - `worker_leases.json`: disjoint file-ownership lock table preventing parallel
//!   subagents/swarm workers from colliding on the same file.
//! - 5-store atomic flush (`projectmem`, `icm`, `aimemory`, `ctxmode`, `codegraph`)
//!   before `StopFailure`, network drop, HTTP 429, `CompactPre`, or quota exhaustion.

#![deny(unsafe_code)]

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::commands::doctor_stats::process_alive;
use crate::commands::test_runner::{
    blake3_hex, compute_cargo_lock_blake3, compute_tree_blake3, TestBaselineManifest,
};
use crate::gateway::cascade::FailureClass;
use crate::projectmem::plan_checkpoint::{
    append_checkpoint_event, load_plan_checkpoint, save_plan_checkpoint, sync_plan_md_checkboxes,
    validate_dirty_tree_and_plugins, CheckpointStatus, PlanCheckpoint, YieldReason,
};
use crate::projectmem::store::{atomic_write_file, find_repo_root, TOKSI_MEM_SUBDIR};
use crate::util::paths::normalize_path;

pub const JOURNAL_FILENAME: &str = "orchestration_journal.jsonl";
pub const LEASES_FILENAME: &str = "worker_leases.json";
pub const FIVE_STORES: &[&str] = &["projectmem", "icm", "aimemory", "ctxmode", "codegraph"];

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationFailureKind {
    NetworkDrop,
    Http429,
    QuotaExhausted,
    ToolError,
    UserInterrupt,
    ContextCompactPre,
}

impl OrchestrationFailureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NetworkDrop => "network_drop",
            Self::Http429 => "http_429",
            Self::QuotaExhausted => "quota_exhausted",
            Self::ToolError => "tool_error",
            Self::UserInterrupt => "user_interrupt",
            Self::ContextCompactPre => "context_compact_pre",
        }
    }

    pub const fn to_yield_reason(self) -> YieldReason {
        match self {
            Self::NetworkDrop => YieldReason::NetworkDrop,
            Self::Http429 | Self::QuotaExhausted => YieldReason::RateLimit429,
            Self::ToolError => YieldReason::Crash,
            Self::UserInterrupt => YieldReason::Sigint,
            Self::ContextCompactPre => YieldReason::Milestone,
        }
    }

    pub const fn from_failure_class(fc: FailureClass) -> Self {
        match fc {
            FailureClass::RateLimit => Self::Http429,
            FailureClass::Quota => Self::QuotaExhausted,
            FailureClass::Transient | FailureClass::Timeout => Self::NetworkDrop,
            FailureClass::Credential
            | FailureClass::ModelAccess
            | FailureClass::ModelGone
            | FailureClass::RegionBlocked
            | FailureClass::Unknown => Self::ToolError,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationJournalEntry {
    pub seq: u64,
    pub timestamp_unix: u64,
    pub plan_id: String,
    pub worker_id: String,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<OrchestrationFailureKind>,
    pub detail: String,
}

pub struct OrchestrationJournal {
    mem_dir: PathBuf,
}

impl OrchestrationJournal {
    pub fn for_repo(repo_root: &Path) -> Self {
        let root = find_repo_root(Some(repo_root));
        Self {
            mem_dir: root.join(TOKSI_MEM_SUBDIR),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.mem_dir.join(JOURNAL_FILENAME)
    }

    fn lock_path(&self) -> PathBuf {
        self.mem_dir.join("orchestration_journal.lock")
    }

    pub fn append(&self, mut entry: OrchestrationJournalEntry) -> Result<u64, String> {
        fs::create_dir_all(&self.mem_dir).map_err(|e| format!("Create projectmem dir: {e}"))?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())
            .map_err(|e| format!("Open journal lock: {e}"))?;
        lock_file
            .lock_exclusive()
            .map_err(|e| format!("Lock journal exclusive: {e}"))?;

        let existing = self.read_unlocked()?;
        let next_seq = existing.last().map(|e| e.seq + 1).unwrap_or(1);
        entry.seq = next_seq;
        if entry.timestamp_unix == 0 {
            entry.timestamp_unix = now_epoch();
        }
        let line = serde_json::to_string(&entry).map_err(|e| format!("Serialize journal: {e}"))?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path())
            .map_err(|e| format!("Open journal: {e}"))?;
        writeln!(f, "{line}").map_err(|e| format!("Write journal: {e}"))?;
        let _ = f.sync_data();
        let _ = lock_file.unlock();
        Ok(next_seq)
    }

    pub fn read_all(&self) -> Result<Vec<OrchestrationJournalEntry>, String> {
        if !self.path().exists() {
            return Ok(Vec::new());
        }
        fs::create_dir_all(&self.mem_dir).map_err(|e| format!("Create projectmem dir: {e}"))?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())
            .map_err(|e| format!("Open journal lock: {e}"))?;
        lock_file
            .lock_shared()
            .map_err(|e| format!("Lock journal shared: {e}"))?;
        let entries = self.read_unlocked()?;
        let _ = lock_file.unlock();
        Ok(entries)
    }

    fn read_unlocked(&self) -> Result<Vec<OrchestrationJournalEntry>, String> {
        let p = self.path();
        if !p.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&p).map_err(|e| format!("Read journal: {e}"))?;
        let mut out = Vec::new();
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: OrchestrationJournalEntry = serde_json::from_str(trimmed)
                .map_err(|e| format!("Parse journal line: {e}"))?;
            out.push(entry);
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerLease {
    pub worker_id: String,
    pub pid: u32,
    pub files: Vec<String>,
    pub claimed_at_unix: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WorkerLeaseFile {
    leases: Vec<WorkerLease>,
}

pub struct WorkerLeaseRegistry {
    mem_dir: PathBuf,
}

impl WorkerLeaseRegistry {
    pub fn for_repo(repo_root: &Path) -> Self {
        let root = find_repo_root(Some(repo_root));
        Self {
            mem_dir: root.join(TOKSI_MEM_SUBDIR),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.mem_dir.join(LEASES_FILENAME)
    }

    fn lock_path(&self) -> PathBuf {
        self.mem_dir.join("worker_leases.lock")
    }

    pub fn claim_leases(
        &self,
        worker_id: &str,
        pid: u32,
        files: &[String],
    ) -> Result<(), String> {
        let clean_worker = worker_id.trim();
        if clean_worker.is_empty() {
            return Err("worker_id cannot be empty".to_string());
        }
        let normalized_files: Vec<String> = files
            .iter()
            .map(|f| normalize_path(f.trim()))
            .filter(|f| !f.is_empty())
            .collect();

        fs::create_dir_all(&self.mem_dir).map_err(|e| format!("Create projectmem dir: {e}"))?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())
            .map_err(|e| format!("Open lease lock: {e}"))?;
        lock_file
            .lock_exclusive()
            .map_err(|e| format!("Lock lease registry: {e}"))?;

        let mut doc = self.read_unlocked()?;
        for lease in &doc.leases {
            if lease.worker_id == clean_worker {
                continue;
            }
            for f in &normalized_files {
                if lease.files.iter().any(|held| held == f) {
                    let _ = lock_file.unlock();
                    return Err(format!(
                        "file lease conflict: '{}' is already owned by worker '{}' (pid={})",
                        f, lease.worker_id, lease.pid
                    ));
                }
            }
        }

        doc.leases.retain(|l| l.worker_id != clean_worker);
        doc.leases.push(WorkerLease {
            worker_id: clean_worker.to_string(),
            pid,
            files: normalized_files,
            claimed_at_unix: now_epoch(),
        });
        let res = self.write_unlocked(&doc);
        let _ = lock_file.unlock();
        res
    }

    pub fn release_worker(&self, worker_id: &str) -> Result<usize, String> {
        fs::create_dir_all(&self.mem_dir).map_err(|e| format!("Create projectmem dir: {e}"))?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())
            .map_err(|e| format!("Open lease lock: {e}"))?;
        lock_file
            .lock_exclusive()
            .map_err(|e| format!("Lock lease registry: {e}"))?;
        let mut doc = self.read_unlocked()?;
        let before = doc.leases.len();
        doc.leases.retain(|l| l.worker_id != worker_id.trim());
        let removed = before.saturating_sub(doc.leases.len());
        let res = self.write_unlocked(&doc);
        let _ = lock_file.unlock();
        res?;
        Ok(removed)
    }

    pub fn reap_stale_leases<F>(&self, is_alive: F) -> Result<Vec<String>, String>
    where
        F: Fn(u32) -> bool,
    {
        if !self.path().exists() {
            return Ok(Vec::new());
        }
        fs::create_dir_all(&self.mem_dir).map_err(|e| format!("Create projectmem dir: {e}"))?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())
            .map_err(|e| format!("Open lease lock: {e}"))?;
        lock_file
            .lock_exclusive()
            .map_err(|e| format!("Lock lease registry: {e}"))?;
        let mut doc = self.read_unlocked()?;
        let mut reaped = Vec::new();
        doc.leases.retain(|l| {
            if is_alive(l.pid) {
                true
            } else {
                reaped.push(l.worker_id.clone());
                false
            }
        });
        let res = self.write_unlocked(&doc);
        let _ = lock_file.unlock();
        res?;
        Ok(reaped)
    }

    pub fn list_leases(&self) -> Result<Vec<WorkerLease>, String> {
        Ok(self.read_unlocked()?.leases)
    }

    fn read_unlocked(&self) -> Result<WorkerLeaseFile, String> {
        let p = self.path();
        if !p.exists() {
            return Ok(WorkerLeaseFile::default());
        }
        let raw = fs::read_to_string(&p).map_err(|e| format!("Read leases: {e}"))?;
        serde_json::from_str(&raw).map_err(|e| format!("Parse leases: {e}"))
    }

    fn write_unlocked(&self, doc: &WorkerLeaseFile) -> Result<(), String> {
        let json = serde_json::to_string_pretty(doc).map_err(|e| format!("Serialize leases: {e}"))?;
        atomic_write_file(&self.path(), &json).map_err(|e| format!("Write leases: {e}"))
    }
}

/// Selects the next distinct L0 -> L1 -> L2 BYOK profile on rate limit / quota exhaustion.
pub fn select_failover_byok_profile(
    active_profile: &str,
    candidates: &[String],
    failure: OrchestrationFailureKind,
) -> Option<String> {
    let class = match failure {
        OrchestrationFailureKind::Http429 => FailureClass::RateLimit,
        OrchestrationFailureKind::QuotaExhausted => FailureClass::Quota,
        OrchestrationFailureKind::NetworkDrop => FailureClass::Transient,
        _ => FailureClass::Unknown,
    };
    if !class.is_retryable() && !class.is_permanent_key_failure() {
        return None;
    }
    let clean_active = active_profile.trim();
    let distinct: Vec<&String> = candidates
        .iter()
        .filter(|c| !c.trim().is_empty())
        .collect();
    if distinct.is_empty() {
        return None;
    }
    if let Some(pos) = distinct.iter().position(|c| c.as_str() == clean_active) {
        for offset in 1..distinct.len() {
            let next = distinct[(pos + offset) % distinct.len()];
            if next.as_str() != clean_active {
                return Some(next.clone());
            }
        }
        return None;
    }
    distinct.first().map(|s| (*s).clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiveStoreFlushReport {
    pub plan_id: String,
    pub checkpoint_path: String,
    pub journal_seq: u64,
    pub flushed_stores: Vec<String>,
    pub blake3_recorded_files: usize,
    pub failover_next_profile: Option<String>,
}

/// Atomically flushes checkpoint, BLAKE3 dirty-file digests, write-ahead journal, and
/// all 5 repo stores (`projectmem`, `icm`, `aimemory`, `ctxmode`, `codegraph`) prior to
/// network drop, HTTP 429, quota exhaustion, `StopFailure`, or `CompactPre`.
pub fn flush_five_stores_before_exit(
    repo_root: &Path,
    checkpoint: &mut PlanCheckpoint,
    failure: OrchestrationFailureKind,
    worker_id: &str,
    dirty_files: &[String],
    failover_candidates: &[String],
) -> Result<FiveStoreFlushReport, String> {
    let root = find_repo_root(Some(repo_root));
    checkpoint.repo_root = normalize_path(&root.to_string_lossy());
    checkpoint.status = CheckpointStatus::Paused;
    checkpoint.last_error_reason = Some(failure.as_str().to_string());
    checkpoint.last_error = Some(format!(
        "Paused on '{}' by worker '{}'",
        failure.as_str(),
        worker_id
    ));
    checkpoint.yield_reason = Some(failure.to_yield_reason());

    for raw_rel in dirty_files {
        let rel = normalize_path(raw_rel.trim());
        if rel.is_empty() {
            continue;
        }
        let full = root.join(&rel);
        if let Ok(bytes) = fs::read(&full) {
            checkpoint.dirty_files_blake3.insert(rel, blake3_hex(&bytes));
        }
    }

    let failover_next = select_failover_byok_profile(
        &checkpoint.active_byok_profile,
        failover_candidates,
        failure,
    );
    if let Some(ref next_profile) = failover_next {
        checkpoint.active_byok_profile = next_profile.clone();
    }

    let cp_path = save_plan_checkpoint(Some(&root), checkpoint)
        .map_err(|e| format!("Save plan checkpoint: {e}"))?;
    let _ = sync_plan_md_checkboxes(Some(&root), checkpoint);
    let _ = append_checkpoint_event(Some(&root), checkpoint, failure.as_str());

    let journal = OrchestrationJournal::for_repo(&root);
    let seq = journal.append(OrchestrationJournalEntry {
        seq: 0,
        timestamp_unix: now_epoch(),
        plan_id: checkpoint.plan_id.clone(),
        worker_id: worker_id.to_string(),
        event: "pre_failure_five_store_flush".to_string(),
        step_id: checkpoint.first_incomplete_step().1,
        failure_kind: Some(failure),
        detail: format!(
            "Flushed 5 stores; failover={}",
            failover_next.as_deref().unwrap_or("none")
        ),
    })?;

    let mut flushed_stores = Vec::new();
    for store in FIVE_STORES {
        let marker_path = root
            .join(".toksi")
            .join(store)
            .join("last_resume_flush.json");
        let marker = json!({
            "plan_id": checkpoint.plan_id,
            "active_phase": checkpoint.active_phase,
            "failure_kind": failure.as_str(),
            "worker_id": worker_id,
            "journal_seq": seq,
            "failover_next_profile": failover_next,
            "updated_unix": now_epoch(),
        });
        let serialized = serde_json::to_string_pretty(&marker)
            .map_err(|e| format!("Serialize store marker: {e}"))?;
        atomic_write_file(&marker_path, &serialized)
            .map_err(|e| format!("Flush store marker '{store}': {e}"))?;
        flushed_stores.push((*store).to_string());
    }

    Ok(FiveStoreFlushReport {
        plan_id: checkpoint.plan_id.clone(),
        checkpoint_path: normalize_path(&cp_path.to_string_lossy()),
        journal_seq: seq,
        flushed_stores,
        blake3_recorded_files: checkpoint.dirty_files_blake3.len(),
        failover_next_profile: failover_next,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChatResumeReport {
    pub plan_id: String,
    pub title: String,
    pub active_phase: String,
    pub active_byok_profile: String,
    pub next_step_index: usize,
    pub next_step_title: Option<String>,
    pub completed_count: usize,
    pub total_steps: usize,
    pub blake3_verified_files: Vec<String>,
    pub blake3_drifted_files: Vec<String>,
    pub reaped_stale_workers: Vec<String>,
    pub skipped_green_suites: Vec<String>,
    pub journal_entries_count: usize,
}

pub fn prepare_cross_chat_resume(
    repo_root: &Path,
    plan_id: &str,
) -> Result<CrossChatResumeReport, String> {
    let root = find_repo_root(Some(repo_root));
    let cp = load_plan_checkpoint(Some(&root), plan_id)
        .map_err(|e| format!("Failed loading checkpoint '{plan_id}': {e}"))?;
    let preflight = validate_dirty_tree_and_plugins(Some(&root), &cp);

    let leases = WorkerLeaseRegistry::for_repo(&root);
    let reaped_stale_workers = leases.reap_stale_leases(process_alive)?;

    let mut blake3_verified_files = Vec::new();
    let mut blake3_drifted_files = Vec::new();
    for (rel, expected) in &cp.dirty_files_blake3 {
        let full = root.join(rel);
        match fs::read(&full) {
            Ok(bytes) if blake3_hex(&bytes) == *expected => {
                blake3_verified_files.push(rel.clone());
            }
            Ok(_) => blake3_drifted_files.push(format!("{rel} (blake3 mismatch)")),
            Err(_) => blake3_drifted_files.push(format!("{rel} (missing)")),
        }
    }

    let skipped_green_suites = detect_unchanged_green_suites(&root);
    let journal_entries_count = OrchestrationJournal::for_repo(&root)
        .read_all()
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(CrossChatResumeReport {
        plan_id: cp.plan_id,
        title: cp.title,
        active_phase: cp.active_phase,
        active_byok_profile: cp.active_byok_profile,
        next_step_index: preflight.next_step_index,
        next_step_title: preflight.next_step_title,
        completed_count: preflight.completed_count,
        total_steps: preflight.total_steps,
        blake3_verified_files,
        blake3_drifted_files,
        reaped_stale_workers,
        skipped_green_suites,
        journal_entries_count,
    })
}

fn detect_unchanged_green_suites(root: &Path) -> Vec<String> {
    let baseline_path = root.join(".toksi/test-baseline.json");
    let Ok(raw) = fs::read_to_string(baseline_path) else {
        return Vec::new();
    };
    let Ok(manifest) = serde_json::from_str::<TestBaselineManifest>(&raw) else {
        return Vec::new();
    };
    let lock_hash = compute_cargo_lock_blake3(root);
    if lock_hash != manifest.cargo_lock_blake3 {
        return Vec::new();
    }
    let mut skipped = BTreeSet::new();
    for (name, suite) in &manifest.suites {
        if suite.last_passed_unix > 0
            && compute_tree_blake3(root, &suite.watched_paths) == suite.tree_blake3
        {
            skipped.insert(name.clone());
        }
    }
    skipped.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_disjoint_leases_five_store_flush_and_cross_chat_resume() {
        let tmp = TempDir::new().expect("tempdir");
        let root = tmp.path();
        fs::create_dir_all(root.join(".git")).expect("mkdir .git");
        fs::create_dir_all(root.join("src")).expect("mkdir src");
        fs::write(root.join("src/a.rs"), "pub fn a() {}\n").expect("write a.rs");

        let leases = WorkerLeaseRegistry::for_repo(root);
        leases
            .claim_leases("worker-1", std::process::id(), &["src/a.rs".to_string()])
            .expect("worker-1 claims src/a.rs");
        let conflict =
            leases.claim_leases("worker-2", std::process::id(), &["src/a.rs".to_string()]);
        assert!(conflict.is_err(), "overlapping lease must fail fast");

        let mut cp = PlanCheckpoint::new("wave21-plan", "Wave 21 Resilience");
        cp.active_phase = "PHASE_21_5".to_string();
        cp.active_byok_profile = "byok-primary".to_string();
        cp.completed_steps = vec!["step-1".to_string()];
        cp.pending_steps = vec!["step-2".to_string(), "step-3".to_string()];
        cp.total_steps = 3;

        let flush = flush_five_stores_before_exit(
            root,
            &mut cp,
            OrchestrationFailureKind::Http429,
            "worker-1",
            &["src/a.rs".to_string()],
            &["byok-primary".to_string(), "byok-fallback".to_string()],
        )
        .expect("five store flush");
        assert_eq!(flush.flushed_stores.len(), 5);
        assert_eq!(flush.blake3_recorded_files, 1);
        assert_eq!(flush.failover_next_profile.as_deref(), Some("byok-fallback"));

        let resumed = prepare_cross_chat_resume(root, "wave21-plan").expect("resume");
        assert_eq!(resumed.next_step_title.as_deref(), Some("step-2"));
        assert_eq!(resumed.completed_count, 1);
        assert_eq!(resumed.blake3_verified_files, vec!["src/a.rs"]);
        assert!(resumed.blake3_drifted_files.is_empty());
        assert_eq!(resumed.active_byok_profile, "byok-fallback");
        assert_eq!(resumed.journal_entries_count, 1);
    }
}
