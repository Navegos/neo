// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/lan_pair.rs
// created: 2026-11-24

//! `Personal-AI-Router` (`PAIR`) LAN GPU mesh peer registry & failover selector.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::topology::ClusterNodeHealth;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanPairPeer {
    pub node_id: String,
    pub endpoint: String,
    pub vram_gb: u32,
    pub health: ClusterNodeHealth,
    pub active_models: Vec<String>,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanPairMesh {
    pub peers: Vec<LanPairPeer>,
}

impl LanPairMesh {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn register_peer(&mut self, peer: LanPairPeer) {
        if let Some(existing) = self.peers.iter_mut().find(|p| p.node_id == peer.node_id) {
            *existing = peer;
        } else {
            self.peers.push(peer);
        }
    }

    pub fn mark_health(&mut self, node_id: &str, health: ClusterNodeHealth) {
        if let Some(peer) = self.peers.iter_mut().find(|p| p.node_id == node_id) {
            peer.health = health;
        }
    }

    /// Selects the lowest-latency healthy/warning peer with sufficient VRAM, preferring
    /// peers that already hold `model` hot in VRAM.
    pub fn select_best_peer(&self, required_vram_gb: u32, model: &str) -> Option<&LanPairPeer> {
        let m_low = model.trim().to_ascii_lowercase();
        self.peers
            .iter()
            .filter(|p| {
                matches!(p.health, ClusterNodeHealth::Healthy | ClusterNodeHealth::Warning)
                    && p.vram_gb >= required_vram_gb
            })
            .min_by_key(|p| {
                let model_hit = !m_low.is_empty()
                    && p.active_models
                        .iter()
                        .any(|am| am.to_ascii_lowercase() == m_low);
                let health_penalty = match p.health {
                    ClusterNodeHealth::Healthy => 0u32,
                    _ => 500u32,
                };
                let cold_penalty = if model_hit { 0u32 } else { 200u32 };
                (health_penalty + cold_penalty + p.latency_ms, std::cmp::Reverse(p.vram_gb))
            })
    }
}
