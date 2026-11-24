// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/topology.rs
// created: 2026-11-24

//! Multi-backend distributed cluster topology (`Local | LanPair | Kubernetes | Slurm | Ssh`)
//! persisted in `toksi.db` (`cluster_nodes`) and `~/.toksi/services/cluster.db`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::db::{ClusterNodeRecord, ServiceDbKind, ServiceTelemetryDb, ToksiConfigDb};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterBackendKind {
    Local,
    LanPair,
    Kubernetes,
    Slurm,
    Ssh,
}

impl ClusterBackendKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::LanPair => "lan_pair",
            Self::Kubernetes => "kubernetes",
            Self::Slurm => "slurm",
            Self::Ssh => "ssh",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "lan" | "lan_pair" | "pair" | "lan-pair" => Self::LanPair,
            "k8s" | "kubernetes" => Self::Kubernetes,
            "slurm" | "srt-slurm" => Self::Slurm,
            "ssh" => Self::Ssh,
            _ => Self::Local,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterNodeHealth {
    Healthy,
    Warning,
    Faulted,
    Quarantined,
}

impl ClusterNodeHealth {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Warning => "warning",
            Self::Faulted => "faulted",
            Self::Quarantined => "quarantined",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "warning" | "warn" => Self::Warning,
            "faulted" | "fault" => Self::Faulted,
            "quarantined" | "cordon" => Self::Quarantined,
            _ => Self::Healthy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterNode {
    pub node_id: String,
    pub backend: ClusterBackendKind,
    pub endpoint: String,
    pub gpu_model: String,
    pub gpu_count: u32,
    pub vram_gb: u32,
    pub health: ClusterNodeHealth,
    pub updated_at: i64,
}

impl ClusterNode {
    pub fn to_db_record(&self) -> ClusterNodeRecord {
        ClusterNodeRecord {
            node_id: self.node_id.clone(),
            backend: self.backend.as_str().to_string(),
            endpoint: self.endpoint.clone(),
            gpu_model: self.gpu_model.clone(),
            gpu_count: i64::from(self.gpu_count),
            vram_gb: i64::from(self.vram_gb),
            health: self.health.as_str().to_string(),
            updated_at: self.updated_at,
        }
    }

    pub fn from_db_record(r: &ClusterNodeRecord) -> Self {
        Self {
            node_id: r.node_id.clone(),
            backend: ClusterBackendKind::from_str_loose(&r.backend),
            endpoint: r.endpoint.clone(),
            gpu_model: r.gpu_model.clone(),
            gpu_count: u32::try_from(r.gpu_count.max(0)).unwrap_or(0),
            vram_gb: u32::try_from(r.vram_gb.max(0)).unwrap_or(0),
            health: ClusterNodeHealth::from_str_loose(&r.health),
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClusterTopology {
    pub nodes: Vec<ClusterNode>,
}

impl ClusterTopology {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn upsert_node(&mut self, node: ClusterNode) {
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.node_id == node.node_id) {
            *existing = node;
        } else {
            self.nodes.push(node);
        }
    }

    pub fn sync_to_config_db(&self, db: &mut ToksiConfigDb) -> Result<(), rusqlite::Error> {
        for n in &self.nodes {
            db.upsert_cluster_node(&n.to_db_record())?;
        }
        Ok(())
    }

    pub fn load_from_config_db(db: &ToksiConfigDb) -> Result<Self, rusqlite::Error> {
        let recs = db.list_cluster_nodes()?;
        let nodes = recs.iter().map(ClusterNode::from_db_record).collect();
        Ok(Self { nodes })
    }

    pub fn record_telemetry_event(
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<i64, rusqlite::Error> {
        let mut srv = ServiceTelemetryDb::open(ServiceDbKind::Cluster)?;
        srv.record_event(event_type, payload)
    }
}
