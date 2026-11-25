// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/swarm/mod.rs
// created: 2026-11-25

//! `agent-orchestrator` (`inspect-vault/agent-orchestrator`) pure-Rust swarm fleet engine:
//! per-task isolated git worktree & branch planning (`toksi/task/<id>`), Kanban fleet lane
//! state machine (`Working`, `NeedsYou`, `InReview`, `ReadyToMerge`), and automated CI
//! failure / PR review comment routing back to the owning worker branch.

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KanbanLane {
    Working,
    NeedsYou,
    InReview,
    ReadyToMerge,
}

impl KanbanLane {
    pub const ALL: &'static [KanbanLane] = &[
        Self::Working,
        Self::NeedsYou,
        Self::InReview,
        Self::ReadyToMerge,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Working => "Working",
            Self::NeedsYou => "Needs you",
            Self::InReview => "In review",
            Self::ReadyToMerge => "Ready to merge",
        }
    }

    pub const fn can_transition_to(self, next: KanbanLane) -> bool {
        match (self, next) {
            (a, b) if matches!((a, b), (Self::Working, Self::Working) | (Self::NeedsYou, Self::NeedsYou) | (Self::InReview, Self::InReview) | (Self::ReadyToMerge, Self::ReadyToMerge)) => true,
            (Self::Working, Self::NeedsYou | Self::InReview) => true,
            (Self::NeedsYou, Self::Working | Self::InReview) => true,
            (Self::InReview, Self::Working | Self::NeedsYou | Self::ReadyToMerge) => true,
            (Self::ReadyToMerge, Self::Working | Self::InReview) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeTaskSpec {
    pub task_id: String,
    pub branch_name: String,
    pub worktree_path: String,
    pub base_branch: String,
    pub agent_adapter: String,
    pub lane: KanbanLane,
    pub ci_failures: Vec<String>,
    pub review_comments: Vec<String>,
}

impl WorktreeTaskSpec {
    pub fn git_worktree_add_args(&self) -> Vec<String> {
        vec![
            "worktree".to_string(),
            "add".to_string(),
            "-b".to_string(),
            self.branch_name.clone(),
            self.worktree_path.clone(),
            self.base_branch.clone(),
        ]
    }
}

pub fn sanitize_worktree_task_id(raw: &str) -> Result<String, String> {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-').to_string();
    if trimmed.is_empty() {
        Err("Worktree task_id cannot be empty".to_string())
    } else {
        Ok(trimmed)
    }
}

pub fn plan_isolated_worktree(
    repo_root: &Path,
    task_id: &str,
    base_branch: &str,
    agent_adapter: &str,
) -> Result<WorktreeTaskSpec, String> {
    let id = sanitize_worktree_task_id(task_id)?;
    let base = if base_branch.trim().is_empty() {
        "main"
    } else {
        base_branch.trim()
    };
    let adapter = if agent_adapter.trim().is_empty() {
        "toksi-siagent"
    } else {
        agent_adapter.trim()
    };
    let wt = repo_root.join(".toksi").join("worktrees").join(&id);
    Ok(WorktreeTaskSpec {
        task_id: id.clone(),
        branch_name: format!("toksi/task/{id}"),
        worktree_path: normalize_path(&wt.to_string_lossy()),
        base_branch: base.to_string(),
        agent_adapter: adapter.to_string(),
        lane: KanbanLane::Working,
        ci_failures: Vec::new(),
        review_comments: Vec::new(),
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KanbanFleetBoard {
    tasks: BTreeMap<String, WorktreeTaskSpec>,
}

impl KanbanFleetBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_task(&mut self, task: WorktreeTaskSpec) {
        self.tasks.insert(task.task_id.clone(), task);
    }

    pub fn get_task(&self, task_id: &str) -> Option<&WorktreeTaskSpec> {
        self.tasks.get(task_id)
    }

    pub fn transition_lane(&mut self, task_id: &str, next: KanbanLane) -> Result<KanbanLane, String> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Unknown Kanban task '{task_id}'"))?;
        if !task.lane.can_transition_to(next) {
            return Err(format!(
                "Invalid Kanban lane transition {:?} -> {:?} for '{}'",
                task.lane, next, task_id
            ));
        }
        task.lane = next;
        Ok(next)
    }

    pub fn route_ci_failure(
        &mut self,
        task_id: &str,
        failure_summary: &str,
    ) -> Result<KanbanLane, String> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Unknown Kanban task '{task_id}'"))?;
        task.ci_failures.push(failure_summary.trim().to_string());
        task.lane = KanbanLane::Working;
        Ok(task.lane)
    }

    pub fn route_pr_review_comment(
        &mut self,
        task_id: &str,
        reviewer: &str,
        comment: &str,
        requires_human: bool,
    ) -> Result<KanbanLane, String> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Unknown Kanban task '{task_id}'"))?;
        task.review_comments
            .push(format!("[{}] {}", reviewer.trim(), comment.trim()));
        task.lane = if requires_human {
            KanbanLane::NeedsYou
        } else {
            KanbanLane::Working
        };
        Ok(task.lane)
    }

    pub fn tasks_in_lane(&self, lane: KanbanLane) -> Vec<&WorktreeTaskSpec> {
        self.tasks.values().filter(|t| t.lane == lane).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_planning_and_kanban_routing() {
        let spec = plan_isolated_worktree(Path::new("/tmp/neo"), "Fix-10048", "main", "codex")
            .expect("plan worktree");
        assert_eq!(spec.branch_name, "toksi/task/fix-10048");
        assert_eq!(spec.worktree_path, "/tmp/neo/.toksi/worktrees/fix-10048");
        assert_eq!(
            spec.git_worktree_add_args(),
            vec![
                "worktree",
                "add",
                "-b",
                "toksi/task/fix-10048",
                "/tmp/neo/.toksi/worktrees/fix-10048",
                "main"
            ]
        );

        let mut board = KanbanFleetBoard::new();
        board.upsert_task(spec);
        board
            .transition_lane("fix-10048", KanbanLane::InReview)
            .expect("in review");
        assert_eq!(board.tasks_in_lane(KanbanLane::InReview).len(), 1);

        let lane_after_ci = board
            .route_ci_failure("fix-10048", "clippy failed")
            .expect("route ci");
        assert_eq!(lane_after_ci, KanbanLane::Working);
        assert_eq!(
            board.get_task("fix-10048").expect("task").ci_failures,
            vec!["clippy failed"]
        );

        let lane_after_pr = board
            .route_pr_review_comment("fix-10048", "reviewer", "please rebase", true)
            .expect("route review");
        assert_eq!(lane_after_pr, KanbanLane::NeedsYou);
    }
}
