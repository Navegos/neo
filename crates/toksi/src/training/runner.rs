// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/runner.rs
// created: 2026-11-24

//! PATH-probed training job planner and launcher (zero bundled Python/CUDA).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::commands::test_runner::check_executable_in_path;

use super::recipes::{render_recipe_yaml, TrainingBackend, TrainingRecipe};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrainingJobPlan {
    pub backend: TrainingBackend,
    pub binary: String,
    pub args: Vec<String>,
    pub binary_available: bool,
    pub dry_run: bool,
    pub recipe_yaml: String,
}

pub fn plan_training_job(recipe: &TrainingRecipe, dry_run: bool) -> TrainingJobPlan {
    let binary = recipe.backend.cli_binary().to_string();
    let binary_available = check_executable_in_path(&binary);
    let args = match recipe.backend {
        TrainingBackend::Axolotl => vec!["train".to_string(), "recipe.yaml".to_string()],
        TrainingBackend::LlamaFactory => vec!["train".to_string(), "recipe.yaml".to_string()],
        TrainingBackend::Torchtune => {
            vec!["run".to_string(), recipe.method.as_str().to_string(), "--config".to_string(), "recipe.yaml".to_string()]
        }
        TrainingBackend::Trl => vec![recipe.method.as_str().to_string(), "--model_name_or_path".to_string(), recipe.base_model.clone()],
        TrainingBackend::Unsloth | TrainingBackend::MegatronBridge | TrainingBackend::NemoRl | TrainingBackend::OpenRlhf => {
            vec!["-m".to_string(), recipe.backend.as_str().replace('-', "_"), "--model".to_string(), recipe.base_model.clone()]
        }
    };
    TrainingJobPlan {
        backend: recipe.backend,
        binary,
        args,
        binary_available,
        dry_run,
        recipe_yaml: render_recipe_yaml(recipe),
    }
}
