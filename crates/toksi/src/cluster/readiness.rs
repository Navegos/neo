// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/readiness.rs
// created: 2026-11-24

//! NVIDIA AI Factory NVCRE (`cluster-readiness-engine`) and `NVSentinel`
//! auto-quarantine state machine.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::topology::{ClusterNode, ClusterNodeHealth};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessGate {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub ready: bool,
    pub healthy_nodes: usize,
    pub total_nodes: usize,
    pub gates: Vec<ReadinessGate>,
}

/// Evaluates cluster nodes against NVCRE readiness gates.
pub fn evaluate_nvcre_readiness(nodes: &[ClusterNode]) -> ReadinessReport {
    let total_nodes = nodes.len();
    let healthy_nodes = nodes
        .iter()
        .filter(|n| matches!(n.health, ClusterNodeHealth::Healthy))
        .count();

    let non_empty_gate = ReadinessGate {
        name: "topology_registered".to_string(),
        passed: total_nodes > 0,
        detail: format!("{total_nodes} node(s) registered"),
    };
    let health_gate = ReadinessGate {
        name: "nvcre_zero_faulted_nodes".to_string(),
        passed: nodes
            .iter()
            .all(|n| !matches!(n.health, ClusterNodeHealth::Faulted | ClusterNodeHealth::Quarantined)),
        detail: format!("{healthy_nodes}/{total_nodes} healthy"),
    };
    let ready = non_empty_gate.passed && health_gate.passed;
    ReadinessReport {
        ready,
        healthy_nodes,
        total_nodes,
        gates: vec![non_empty_gate, health_gate],
    }
}

pub struct NvSentinelStateMachine;

impl NvSentinelStateMachine {
    /// Evaluates GPU Xid/thermal telemetry and returns `(new_health, cordon_recommended)`.
    pub fn ingest_health_event(
        node: &mut ClusterNode,
        xid_error: Option<u32>,
        temp_c: u32,
    ) -> (ClusterNodeHealth, bool) {
        let fatal_xid = matches!(xid_error, Some(31 | 48 | 63 | 74 | 79 | 95));
        let (next, cordon) = if fatal_xid || temp_c >= 95 {
            (ClusterNodeHealth::Quarantined, true)
        } else if temp_c >= 88 {
            (ClusterNodeHealth::Faulted, true)
        } else if xid_error.is_some() || temp_c >= 80 {
            (ClusterNodeHealth::Warning, false)
        } else {
            (ClusterNodeHealth::Healthy, false)
        };
        node.health = next;
        (next, cordon)
    }
}
