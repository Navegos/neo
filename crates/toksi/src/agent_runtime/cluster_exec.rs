// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/cluster_exec.rs
// created: 2026-11-21

//! Cluster execution & scheduling contracts for `OSMO`, `srt-slurm`, and `slurm`
//! without bundling external cluster binaries.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterBackend {
    Osmo,
    SrtSlurm,
    Slurm,
    LocalFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterJobSpec {
    pub job_name: String,
    pub backend: ClusterBackend,
    pub nodes: u32,
    pub gpus_per_node: u32,
    pub partition: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterJobSubmission {
    pub job_id: String,
    pub backend: ClusterBackend,
    pub dry_run: bool,
    pub rendered_script: String,
}

/// Validates and renders cluster job submissions (`OSMO`, `srt-slurm`, `slurm`).
pub fn render_cluster_submission(spec: &ClusterJobSpec, dry_run: bool) -> Result<ClusterJobSubmission, String> {
    if spec.job_name.trim().is_empty() || spec.command.trim().is_empty() {
        return Err("ClusterJobSpec requires non-empty job_name and command".to_string());
    }
    let rendered = match spec.backend {
        ClusterBackend::Slurm | ClusterBackend::SrtSlurm => format!(
            "#!/bin/bash\n#SBATCH --job-name={}\n#SBATCH --nodes={}\n#SBATCH --gres=gpu:{}\n#SBATCH --partition={}\n{}\n",
            spec.job_name, spec.nodes.max(1), spec.gpus_per_node, spec.partition, spec.command
        ),
        ClusterBackend::Osmo => format!(
            "workflow:\n  name: {}\n  resources:\n    gpu: {}\n  command: {}\n",
            spec.job_name, spec.gpus_per_node, spec.command
        ),
        ClusterBackend::LocalFallback => spec.command.clone(),
    };
    Ok(ClusterJobSubmission {
        job_id: format!("job-{}", spec.job_name.to_ascii_lowercase()),
        backend: spec.backend,
        dry_run,
        rendered_script: rendered,
    })
}
