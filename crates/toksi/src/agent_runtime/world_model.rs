// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/world_model.rs
// created: 2026-11-21

//! `<toksi-siagent>` Component #27: World Model unifying CodeGraph structural facts,
//! bi-temporal knowledge graph state, and `physicsnemo`/`deepagents` state transitions.

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldEntity {
    pub id: String,
    pub kind: String,
    pub state: String,
    pub updated_at_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldTransition {
    pub action: String,
    pub target_id: String,
    pub pre_state: String,
    pub post_state: String,
    pub confidence_pct: u8,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WorldModel {
    pub entities: BTreeMap<String, WorldEntity>,
    pub transitions: Vec<WorldTransition>,
}

impl WorldModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_entity(
        &mut self,
        id: impl Into<String>,
        kind: impl Into<String>,
        state: impl Into<String>,
    ) {
        let key = id.into();
        self.entities.insert(
            key.clone(),
            WorldEntity {
                id: key,
                kind: kind.into(),
                state: state.into(),
                updated_at_unix: chrono::Utc::now().timestamp(),
            },
        );
    }

    pub fn record_transition(
        &mut self,
        action: impl Into<String>,
        target_id: impl Into<String>,
        post_state: impl Into<String>,
        confidence_pct: u8,
    ) -> WorldTransition {
        let tid = target_id.into();
        let pre_state = self
            .entities
            .get(&tid)
            .map(|e| e.state.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let post = post_state.into();
        if let Some(ent) = self.entities.get_mut(&tid) {
            ent.state = post.clone();
            ent.updated_at_unix = chrono::Utc::now().timestamp();
        }
        let tr = WorldTransition {
            action: action.into(),
            target_id: tid,
            pre_state,
            post_state: post,
            confidence_pct: confidence_pct.min(100),
        };
        self.transitions.push(tr.clone());
        tr
    }

    pub fn predict_impact(&self, target_id: &str) -> Vec<&WorldTransition> {
        self.transitions
            .iter()
            .filter(|t| t.target_id == target_id)
            .collect()
    }
}
