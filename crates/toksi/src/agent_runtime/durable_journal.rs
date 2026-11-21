// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/durable_journal.rs
// created: 2026-11-21

//! Restate-style durable execution journal with idempotent activity records and awakeables.

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::projectmem::store::atomic_write_file;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AwakeableStatus {
    Pending,
    Resolved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DurableAwakeable {
    pub awakeable_id: String,
    pub step_id: String,
    pub status: AwakeableStatus,
    pub payload: Option<String>,
    pub created_at_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournaledActivity {
    pub seq: u64,
    pub step_id: String,
    pub idempotency_key: String,
    pub result_digest: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DurableJournal {
    pub journal_id: String,
    pub activities: Vec<JournaledActivity>,
    pub awakeables: BTreeMap<String, DurableAwakeable>,
}

impl DurableJournal {
    pub fn new(journal_id: impl Into<String>) -> Self {
        Self {
            journal_id: journal_id.into(),
            activities: Vec::new(),
            awakeables: BTreeMap::new(),
        }
    }

    pub fn record_activity(
        &mut self,
        step_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        result_digest: impl Into<String>,
        completed: bool,
    ) -> u64 {
        let key = idempotency_key.into();
        if let Some(existing) = self.activities.iter().find(|a| a.idempotency_key == key) {
            return existing.seq;
        }
        let seq = self.activities.len() as u64 + 1;
        self.activities.push(JournaledActivity {
            seq,
            step_id: step_id.into(),
            idempotency_key: key,
            result_digest: result_digest.into(),
            completed,
        });
        seq
    }

    pub fn register_awakeable(
        &mut self,
        awakeable_id: impl Into<String>,
        step_id: impl Into<String>,
    ) {
        let id = awakeable_id.into();
        self.awakeables.insert(
            id.clone(),
            DurableAwakeable {
                awakeable_id: id,
                step_id: step_id.into(),
                status: AwakeableStatus::Pending,
                payload: None,
                created_at_unix: chrono::Utc::now().timestamp(),
            },
        );
    }

    pub fn resolve_awakeable(&mut self, awakeable_id: &str, payload: Option<String>) -> bool {
        if let Some(aw) = self.awakeables.get_mut(awakeable_id) {
            aw.status = AwakeableStatus::Resolved;
            aw.payload = payload;
            true
        } else {
            false
        }
    }

    pub fn save_to_repo(&self, repo_root: &Path) -> Result<(), String> {
        let dir = repo_root
            .join(".toksi")
            .join("projectmem")
            .join("plan_checkpoints");
        fs::create_dir_all(&dir).map_err(|e| format!("durable_journal mkdir failed: {e}"))?;
        let path = dir.join(format!("journal_{}.json", self.journal_id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("durable_journal serialize failed: {e}"))?;
        atomic_write_file(&path, &json).map_err(|e| format!("durable_journal write failed: {e}"))
    }
}
