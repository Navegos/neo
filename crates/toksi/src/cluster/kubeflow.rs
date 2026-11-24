// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/kubeflow.rs
// created: 2026-11-24

//! Pure-Rust Kubeflow (`trainer`, `pipelines`, `hub`, `sdk`, `kale`, `notebooks`,
//! `semantic-operator`, archival `katib` HPO) and `Run` (`nemo_run`) contracts.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainJobSpec {
    pub name: String,
    pub image: String,
    pub num_nodes: u32,
    pub gpus_per_node: u32,
    pub command: Vec<String>,
}

impl TrainJobSpec {
    pub fn render_yaml(&self) -> String {
        format!(
            "apiVersion: trainer.kubeflow.org/v1alpha1\nkind: TrainJob\nmetadata:\n  name: {}\nspec:\n  trainer:\n    image: {}\n    numNodes: {}\n    resourcesPerNode:\n      nvidia.com/gpu: {}\n",
            self.name, self.image, self.num_nodes.max(1), self.gpus_per_node
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineRunSpec {
    pub pipeline_name: String,
    pub params: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRegistryEntry {
    pub name: String,
    pub version: String,
    pub uri: String,
    pub stage: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KatibHpoTrialSpec {
    pub experiment: String,
    pub algorithm: String,
    pub max_trials: u32,
    pub objective_metric: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NemoRunExperimentSpec {
    pub name: String,
    pub executor: String,
    pub task_command: String,
    pub gpus: u32,
}
