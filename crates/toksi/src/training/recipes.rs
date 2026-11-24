// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/recipes.rs
// created: 2026-11-24

//! Effort-aware declarative training recipe generator across `unsloth`, `axolotl`,
//! `LLaMA-Factory`, `torchtune`, `trl`, `Megatron-Bridge`, `nemo-rl`, and `OpenRLHF`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::headroom::effort::EffortLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrainingBackend {
    Unsloth,
    Axolotl,
    LlamaFactory,
    Torchtune,
    Trl,
    MegatronBridge,
    NemoRl,
    OpenRlhf,
}

impl TrainingBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unsloth => "unsloth",
            Self::Axolotl => "axolotl",
            Self::LlamaFactory => "llama-factory",
            Self::Torchtune => "torchtune",
            Self::Trl => "trl",
            Self::MegatronBridge => "megatron-bridge",
            Self::NemoRl => "nemo-rl",
            Self::OpenRlhf => "openrlhf",
        }
    }

    pub const fn cli_binary(self) -> &'static str {
        match self {
            Self::Unsloth => "python3",
            Self::Axolotl => "axolotl",
            Self::LlamaFactory => "llamafactory-cli",
            Self::Torchtune => "tune",
            Self::Trl => "trl",
            Self::MegatronBridge => "torchrun",
            Self::NemoRl => "nemo_rl",
            Self::OpenRlhf => "openrlhf",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "unsloth" => Some(Self::Unsloth),
            "axolotl" => Some(Self::Axolotl),
            "llama-factory" | "llamafactory" => Some(Self::LlamaFactory),
            "torchtune" | "tune" => Some(Self::Torchtune),
            "trl" => Some(Self::Trl),
            "megatron-bridge" | "megatron" => Some(Self::MegatronBridge),
            "nemo-rl" | "rl" => Some(Self::NemoRl),
            "openrlhf" => Some(Self::OpenRlhf),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrainingMethod {
    Sft,
    Lora,
    Qlora,
    Dpo,
    Grpo,
}

impl TrainingMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sft => "sft",
            Self::Lora => "lora",
            Self::Qlora => "qlora",
            Self::Dpo => "dpo",
            Self::Grpo => "grpo",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "sft" => Some(Self::Sft),
            "lora" => Some(Self::Lora),
            "qlora" => Some(Self::Qlora),
            "dpo" => Some(Self::Dpo),
            "grpo" => Some(Self::Grpo),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingRecipe {
    pub backend: TrainingBackend,
    pub method: TrainingMethod,
    pub base_model: String,
    pub dataset_path: String,
    pub effort: EffortLevel,
    pub lora_rank: u16,
    pub learning_rate: f64,
    pub max_steps: u32,
}

pub fn build_training_recipe(
    backend: TrainingBackend,
    method: TrainingMethod,
    base_model: &str,
    dataset_path: &str,
    effort: EffortLevel,
) -> Result<TrainingRecipe, String> {
    if base_model.trim().is_empty() || dataset_path.trim().is_empty() {
        return Err("TrainingRecipe requires non-empty base_model and dataset_path".to_string());
    }
    let budget_scale = (effort.anthropic_budget_tokens() / 1024).max(1) as u32;
    let lora_rank = (8 * budget_scale.min(16)) as u16;
    let max_steps = 100 * budget_scale.min(32);
    Ok(TrainingRecipe {
        backend,
        method,
        base_model: base_model.trim().to_string(),
        dataset_path: crate::util::paths::normalize_path(dataset_path.trim()),
        effort,
        lora_rank,
        learning_rate: if matches!(method, TrainingMethod::Dpo | TrainingMethod::Grpo) {
            5e-6
        } else {
            2e-4
        },
        max_steps,
    })
}

pub fn render_recipe_yaml(recipe: &TrainingRecipe) -> String {
    format!(
        "backend: {}\nmethod: {}\nbase_model: {}\ndataset_path: {}\nreasoning_effort: {}\nlora_rank: {}\nlearning_rate: {:.6}\nmax_steps: {}\n",
        recipe.backend.as_str(),
        recipe.method.as_str(),
        recipe.base_model,
        recipe.dataset_path,
        recipe.effort.as_str(),
        recipe.lora_rank,
        recipe.learning_rate,
        recipe.max_steps
    )
}
