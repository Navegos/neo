// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/openspec.rs
// created: 2026-11-24

//! Pure-Rust spec-driven development contract & lifecycle validator (`OpenSpec`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecLifecycle {
    Draft,
    Reviewed,
    Approved,
    Executing,
    Verified,
}

impl SpecLifecycle {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Reviewed)
                | (Self::Reviewed, Self::Approved)
                | (Self::Approved, Self::Executing)
                | (Self::Executing, Self::Verified)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecContract {
    pub id: String,
    pub goal: String,
    pub acceptance_criteria: Vec<String>,
    pub files: Vec<String>,
    pub lifecycle: SpecLifecycle,
}

impl SpecContract {
    pub fn new(id: &str, goal: &str, acceptance_criteria: Vec<String>, files: Vec<String>) -> Self {
        Self {
            id: id.trim().to_string(),
            goal: goal.trim().to_string(),
            acceptance_criteria,
            files: files.into_iter().map(|f| normalize_path(&f)).collect(),
            lifecycle: SpecLifecycle::Draft,
        }
    }

    /// Fail-closed contract validator requiring id, goal, acceptance criteria, and safe relative paths.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("OpenSpec contract missing non-empty id".to_string());
        }
        if self.goal.is_empty() {
            return Err("OpenSpec contract missing non-empty goal".to_string());
        }
        if self.acceptance_criteria.is_empty()
            || self.acceptance_criteria.iter().all(|c| c.trim().is_empty())
        {
            return Err("OpenSpec fail-closed: at least one acceptance criterion is required".to_string());
        }
        for file in &self.files {
            if file.contains("..") || file.starts_with('/') {
                return Err(format!("OpenSpec unsafe file scope path rejected: '{file}'"));
            }
        }
        Ok(())
    }

    pub fn transition_to(&mut self, next: SpecLifecycle) -> Result<(), String> {
        self.validate()?;
        if !self.lifecycle.can_transition_to(next) {
            return Err(format!(
                "Invalid OpenSpec transition {:?} -> {:?}",
                self.lifecycle, next
            ));
        }
        self.lifecycle = next;
        Ok(())
    }
}
