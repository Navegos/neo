// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/audit.rs
// created: 2026-11-21

//! Append-only JSONL `defenseclaw` audit evidence logger with zero-plaintext redaction.

#![deny(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{LazyLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use fs2::FileExt;
use serde::{Deserialize, Serialize};

use crate::projectmem::redaction::redact;
use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvidenceRecord {
    pub timestamp_unix: i64,
    pub actor: String,
    pub component: String,
    pub action: String,
    pub outcome: String,
    pub reason: String,
    pub resource: String,
}

static RECENT_AUDIT_EVIDENCE: LazyLock<RwLock<Vec<AuditEvidenceRecord>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Records an immutable audit evidence entry in memory and optionally appends to `<repo>/.toksi/security/audit_evidence.jsonl`.
pub fn record_audit_evidence(
    repo_root: Option<&Path>,
    actor: &str,
    component: &str,
    action: &str,
    outcome: &str,
    reason: &str,
    resource: &str,
) -> Result<AuditEvidenceRecord, String> {
    let (clean_reason, _) = redact(reason);
    let (clean_resource, _) = redact(&normalize_path(resource));
    let rec = AuditEvidenceRecord {
        timestamp_unix: now_unix(),
        actor: actor.to_string(),
        component: component.to_string(),
        action: action.to_string(),
        outcome: outcome.to_string(),
        reason: clean_reason,
        resource: clean_resource,
    };

    if let Ok(mut guard) = RECENT_AUDIT_EVIDENCE.write() {
        guard.push(rec.clone());
        if guard.len() > 500 {
            let drain = guard.len() - 500;
            guard.drain(..drain);
        }
    }

    if let Some(root) = repo_root {
        let dir = root.join(".toksi").join("security");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create security audit dir: {e}"))?;
        let path = dir.join("audit_evidence.jsonl");
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open audit_evidence.jsonl: {e}"))?;
        file.lock_exclusive()
            .map_err(|e| format!("Failed to lock audit_evidence.jsonl: {e}"))?;
        let mut writer = std::io::BufWriter::new(&file);
        let line = serde_json::to_string(&rec)
            .map_err(|e| format!("Failed to serialize audit record: {e}"))?;
        writeln!(writer, "{line}").map_err(|e| format!("Failed to write audit record: {e}"))?;
        writer.flush().map_err(|e| format!("Failed to flush audit record: {e}"))?;
        let _ = file.unlock();
    }

    Ok(rec)
}

/// Returns recent audit evidence entries from memory (and disk if `repo_root` is provided).
pub fn list_audit_evidence(repo_root: Option<&Path>) -> Vec<AuditEvidenceRecord> {
    if let Some(root) = repo_root {
        let path = root.join(".toksi").join("security").join("audit_evidence.jsonl");
        if let Ok(raw) = std::fs::read_to_string(&path) {
            let mut out = Vec::new();
            for line in raw.lines().filter(|l| !l.trim().is_empty()) {
                if let Ok(rec) = serde_json::from_str::<AuditEvidenceRecord>(line) {
                    out.push(rec);
                }
            }
            if !out.is_empty() {
                return out;
            }
        }
    }
    RECENT_AUDIT_EVIDENCE
        .read()
        .map(|g| g.clone())
        .unwrap_or_default()
}
