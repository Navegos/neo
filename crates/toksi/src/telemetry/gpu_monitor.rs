// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/gpu_monitor.rs
// created: 2026-11-21

//! Pure-Rust GPU & inference node telemetry (`vmon`, `cuda-rust` `cuda-core`/`cuda-async`,
//! `gpustack*`, `gpu-operator`, `cuda-python`) without bundling CUDA runtime binaries.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuDeviceMetrics {
    pub index: u32,
    pub name: String,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub utilization_pct: f32,
    pub temperature_c: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GpuNodeTelemetry {
    pub node_id: String,
    pub driver_source: String, // vmon | cuda-rust | gpustack | none
    pub devices: Vec<GpuDeviceMetrics>,
}

impl GpuNodeTelemetry {
    pub fn empty(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            driver_source: "none".to_string(),
            devices: Vec::new(),
        }
    }

    pub fn total_vram_used_mb(&self) -> u64 {
        self.devices.iter().map(|d| d.vram_used_mb).sum()
    }

    pub fn total_vram_mb(&self) -> u64 {
        self.devices.iter().map(|d| d.vram_total_mb).sum()
    }
}
