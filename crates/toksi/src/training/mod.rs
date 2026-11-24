// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/mod.rs
// created: 2026-11-24

//! Pure-Rust training recipe, quantization/safetensors verification, and
//! local multimodal serving plane (zero bundled Python or CUDA).

#![deny(unsafe_code)]

pub mod compress;
pub mod dataset;
pub mod quant;
pub mod recipes;
pub mod runner;
pub mod serving;

pub use compress::{
    plan_bnb_quant, plan_kvpress, BnbQuantConfig, BnbQuantMethod, BnbQuantPlan, KvPressPlan,
    KvPressPolicy,
};
pub use dataset::{
    build_ontology_from_fields, inspect_dataset_manifest, liteparse_extract_chunks,
    validate_dataset_manifest, DatasetFormat, DatasetManifest, OntologyEdge, OntologyGraph,
};
pub use quant::{verify_safetensors_header, QuantFormat, QuantPlan, SafetensorsHeaderInfo};
pub use recipes::{
    build_training_recipe, render_recipe_yaml, TrainingBackend, TrainingMethod, TrainingRecipe,
};
pub use runner::{plan_training_job, TrainingJobPlan};
pub use serving::{build_serving_profile, LlguidanceJsonMask, ServingProfile};
