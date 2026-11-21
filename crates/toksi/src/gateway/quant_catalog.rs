// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/quant_catalog.rs
// created: 2026-11-21

//! Quantization format catalog (`llm-compressor`, `compressed-tensors`, `optimum-quanto`,
//! `neural-compressor`, `ao`, `FP8`, `NVFP4`, `AWQ`, `GPTQ`, `GGUF`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantSchemeDescriptor {
    pub name: &'static str,
    pub bits_per_weight: u8,
    pub vault: &'static str,
    pub lossless_metadata: bool,
}

pub const SUPPORTED_QUANT_SCHEMES: &[QuantSchemeDescriptor] = &[
    QuantSchemeDescriptor { name: "FP8_DYNAMIC", bits_per_weight: 8, vault: "llm-compressor", lossless_metadata: true },
    QuantSchemeDescriptor { name: "NVFP4", bits_per_weight: 4, vault: "compressed-tensors", lossless_metadata: true },
    QuantSchemeDescriptor { name: "AWQ", bits_per_weight: 4, vault: "optimum-quanto", lossless_metadata: true },
    QuantSchemeDescriptor { name: "GPTQ", bits_per_weight: 4, vault: "neural-compressor", lossless_metadata: true },
    QuantSchemeDescriptor { name: "GGUF_Q4_K_M", bits_per_weight: 4, vault: "vllm-gguf-plugin", lossless_metadata: true },
    QuantSchemeDescriptor { name: "TORCH_AO_INT4", bits_per_weight: 4, vault: "ao", lossless_metadata: true },
];

pub fn lookup_quant_scheme(name: &str) -> Option<&'static QuantSchemeDescriptor> {
    let needle = name.trim().to_ascii_lowercase();
    SUPPORTED_QUANT_SCHEMES
        .iter()
        .find(|q| q.name.to_ascii_lowercase() == needle)
}
