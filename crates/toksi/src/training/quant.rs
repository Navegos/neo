// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/quant.rs
// created: 2026-11-24

//! Quantization plans (`GGUF`, `AWQ`, `GPTQ`, `FP8`, `NVFP4`, `bitsandbytes`) and
//! fail-closed pure-Rust `.safetensors` binary header verifier.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_SAFETENSORS_HEADER_BYTES: usize = 100 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuantFormat {
    GgufQ4KM,
    GgufQ8_0,
    Awq4Bit,
    Gptq4Bit,
    Fp8Dynamic,
    Nvfp4,
    BnbNf4,
    BnbInt8,
}

impl QuantFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GgufQ4KM => "gguf-q4_k_m",
            Self::GgufQ8_0 => "gguf-q8_0",
            Self::Awq4Bit => "awq-4bit",
            Self::Gptq4Bit => "gptq-4bit",
            Self::Fp8Dynamic => "fp8-dynamic",
            Self::Nvfp4 => "nvfp4",
            Self::BnbNf4 => "bnb-nf4",
            Self::BnbInt8 => "bnb-int8",
        }
    }

    pub const fn bits_per_weight(self) -> f64 {
        match self {
            Self::GgufQ4KM => 4.5,
            Self::GgufQ8_0 | Self::Fp8Dynamic | Self::BnbInt8 => 8.0,
            Self::Awq4Bit | Self::Gptq4Bit | Self::Nvfp4 | Self::BnbNf4 => 4.0,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "q4_k_m" | "gguf-q4_k_m" | "gguf" => Some(Self::GgufQ4KM),
            "q8_0" | "gguf-q8_0" => Some(Self::GgufQ8_0),
            "awq" | "awq-4bit" => Some(Self::Awq4Bit),
            "gptq" | "gptq-4bit" => Some(Self::Gptq4Bit),
            "fp8" | "fp8-dynamic" => Some(Self::Fp8Dynamic),
            "nvfp4" => Some(Self::Nvfp4),
            "nf4" | "bnb-nf4" => Some(Self::BnbNf4),
            "int8" | "bnb-int8" => Some(Self::BnbInt8),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantPlan {
    pub format: QuantFormat,
    pub model_id: String,
    pub output_path: String,
    pub estimated_compression_ratio: f64,
}

pub fn build_quant_plan(format: QuantFormat, model_id: &str, output_path: &str) -> QuantPlan {
    QuantPlan {
        format,
        model_id: model_id.trim().to_string(),
        output_path: crate::util::paths::normalize_path(output_path.trim()),
        estimated_compression_ratio: (16.0 / format.bits_per_weight() * 100.0).round() / 100.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetensorsHeaderInfo {
    pub header_len: usize,
    pub tensor_count: usize,
    pub tensor_names: Vec<String>,
    pub has_metadata: bool,
}

/// Fail-closed validator for `.safetensors` headers (rejects truncated/pickle/empty-tensor blobs).
pub fn verify_safetensors_header(bytes: &[u8]) -> Result<SafetensorsHeaderInfo, String> {
    if bytes.len() < 8 {
        return Err("Invalid safetensors file: shorter than 8-byte header length".to_string());
    }
    if bytes.len() >= 2 && bytes[0] == 0x80 && matches!(bytes[1], 2..=5) {
        return Err("Safetensors security rejection: pickle opcode header detected".to_string());
    }
    let header_len_u64 = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let header_len = usize::try_from(header_len_u64)
        .map_err(|_| "Safetensors header length overflows platform usize".to_string())?;
    if header_len == 0 || header_len > MAX_SAFETENSORS_HEADER_BYTES {
        return Err(format!("Invalid safetensors header length: {header_len}"));
    }
    let end = 8usize
        .checked_add(header_len)
        .ok_or_else(|| "Safetensors header size overflow".to_string())?;
    if end > bytes.len() {
        return Err(format!(
            "Truncated safetensors header: declared {header_len} bytes, file is {} bytes",
            bytes.len()
        ));
    }
    let header_slice = &bytes[8..end];
    let header_str = std::str::from_utf8(header_slice)
        .map_err(|_| "Safetensors header is not valid UTF-8 JSON".to_string())?;
    if header_str.contains("__reduce__") || header_str.contains("cposix\nsystem") {
        return Err("Safetensors security rejection: unsafe serialized callable token".to_string());
    }
    let Value::Object(map) = serde_json::from_str::<Value>(header_str)
        .map_err(|e| format!("Invalid safetensors JSON header: {e}"))?
    else {
        return Err("Safetensors header must be a JSON object".to_string());
    };

    let has_metadata = map.contains_key("__metadata__");
    let mut tensor_names = Vec::new();
    for (k, v) in &map {
        if k == "__metadata__" {
            continue;
        }
        let Some(obj) = v.as_object() else {
            return Err(format!("Tensor '{k}' entry must be a JSON object"));
        };
        if !obj.contains_key("dtype") || !obj.contains_key("shape") || !obj.contains_key("data_offsets") {
            return Err(format!("Tensor '{k}' missing dtype/shape/data_offsets"));
        }
        tensor_names.push(k.clone());
    }
    if tensor_names.is_empty() {
        return Err("Invalid safetensors header: contains 0 tensors".to_string());
    }

    Ok(SafetensorsHeaderInfo {
        header_len,
        tensor_count: tensor_names.len(),
        tensor_names,
        has_metadata,
    })
}
