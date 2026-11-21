// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/model_recipes.rs
// created: 2026-11-21

//! Deployment recipe registry (`recipes`, `deepseek-recipe`, `Megatron-Bridge`,
//! `vllm-omni`, `sglang-omni`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRecipeDescriptor {
    pub id: &'static str,
    pub engine: &'static str,
    pub quant: &'static str,
    pub tensor_parallel: u8,
}

pub const BUILTIN_MODEL_RECIPES: &[ModelRecipeDescriptor] = &[
    ModelRecipeDescriptor { id: "deepseek-r1-fp8", engine: "sglang", quant: "FP8_DYNAMIC", tensor_parallel: 8 },
    ModelRecipeDescriptor { id: "llama3-70b-nvfp4", engine: "vllm", quant: "NVFP4", tensor_parallel: 4 },
    ModelRecipeDescriptor { id: "qwen25-coder-32b-awq", engine: "vllm", quant: "AWQ", tensor_parallel: 2 },
    ModelRecipeDescriptor { id: "mistral-small-gguf", engine: "mistral.rs", quant: "GGUF_Q4_K_M", tensor_parallel: 1 },
];

pub fn find_model_recipe(id: &str) -> Option<&'static ModelRecipeDescriptor> {
    BUILTIN_MODEL_RECIPES
        .iter()
        .find(|r| r.id.eq_ignore_ascii_case(id.trim()))
}
