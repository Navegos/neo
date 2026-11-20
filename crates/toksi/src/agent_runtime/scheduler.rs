// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/scheduler.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Single-sleep CronService and `HEARTBEAT.md` task loader.

#![deny(unsafe_code)]

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub interval_secs: u64,
    pub command: String,
    pub last_run_secs: u64,
    pub enabled: bool,
}

impl CronJob {
    pub fn next_due_in(&self, now: u64) -> Duration {
        if !self.enabled || self.interval_secs == 0 {
            return Duration::from_secs(u64::MAX / 2);
        }
        let next = self.last_run_secs.saturating_add(self.interval_secs);
        if next <= now {
            Duration::ZERO
        } else {
            Duration::from_secs(next - now)
        }
    }
}

/// Single-sleep cron scheduler: computes the minimum sleep duration across all jobs.
#[derive(Debug, Default)]
pub struct CronService {
    jobs: Vec<CronJob>,
}

impl CronService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_job(&mut self, job: CronJob) {
        self.jobs.push(job);
    }

    pub fn jobs(&self) -> &[CronJob] {
        &self.jobs
    }

    pub fn next_sleep_duration(&self) -> Option<Duration> {
        let now = now_secs();
        self.jobs
            .iter()
            .filter(|j| j.enabled && j.interval_secs > 0)
            .map(|j| j.next_due_in(now))
            .min()
    }

    pub fn collect_due_jobs(&mut self) -> Vec<CronJob> {
        let now = now_secs();
        let mut due = Vec::new();
        for job in &mut self.jobs {
            if job.enabled && job.next_due_in(now).is_zero() {
                job.last_run_secs = now;
                due.push(job.clone());
            }
        }
        due
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatTask {
    pub title: String,
    pub completed: bool,
}

/// Parses `HEARTBEAT.md` checklist items (`- [ ] ...` / `- [x] ...`).
pub fn load_heartbeat_tasks(workspace_root: &Path) -> Vec<HeartbeatTask> {
    let candidates = [
        workspace_root.join("HEARTBEAT.md"),
        workspace_root.join(".toksi").join("HEARTBEAT.md"),
    ];
    for path in &candidates {
        if let Ok(raw) = std::fs::read_to_string(path) {
            return parse_heartbeat_markdown(&raw);
        }
    }
    Vec::new()
}

pub fn parse_heartbeat_markdown(raw: &str) -> Vec<HeartbeatTask> {
    let mut tasks = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            tasks.push(HeartbeatTask {
                title: rest.trim().to_string(),
                completed: false,
            });
        } else if let Some(rest) = trimmed
            .strip_prefix("- [x] ")
            .or_else(|| trimmed.strip_prefix("- [X] "))
        {
            tasks.push(HeartbeatTask {
                title: rest.trim().to_string(),
                completed: true,
            });
        }
    }
    tasks
}
