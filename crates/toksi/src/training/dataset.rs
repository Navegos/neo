// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/dataset.rs
// created: 2026-11-24

//! Pillar 1 Dataset & Ingestion contracts (`Megatron-Energon`, `structured-data-models`,
//! `ossie`, `ossie-converters`, `auto-ontology`, `liteparse`).

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFormat {
    Jsonl,
    Parquet,
    WebDataset,
    EnergonShard,
    StructuredSchema,
    OssieDocument,
}

impl DatasetFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Jsonl => "jsonl",
            Self::Parquet => "parquet",
            Self::WebDataset => "webdataset",
            Self::EnergonShard => "energon_shard",
            Self::StructuredSchema => "structured_schema",
            Self::OssieDocument => "ossie_document",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "parquet" | "pq" => Self::Parquet,
            "webdataset" | "wds" | "tar" => Self::WebDataset,
            "energon" | "energon_shard" | "megatron-energon" => Self::EnergonShard,
            "structured" | "structured_schema" | "schema" => Self::StructuredSchema,
            "ossie" | "ossie_document" | "doc" => Self::OssieDocument,
            _ => Self::Jsonl,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetManifest {
    pub name: String,
    pub format: DatasetFormat,
    pub path: String,
    pub num_samples: usize,
    pub schema_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyEdge {
    pub subject: String,
    pub relation: String,
    pub object: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyGraph {
    pub domain: String,
    pub entities: Vec<String>,
    pub edges: Vec<OntologyEdge>,
}

/// Inspects a dataset file or manifest path and constructs a `DatasetManifest`.
pub fn inspect_dataset_manifest(path: &Path) -> Result<DatasetManifest, String> {
    if !path.exists() {
        return Err(format!("Dataset path does not exist: {}", path.display()));
    }
    let norm = normalize_path(&path.to_string_lossy());
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dataset")
        .to_string();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jsonl")
        .to_ascii_lowercase();
    let format = match ext.as_str() {
        "parquet" => DatasetFormat::Parquet,
        "tar" | "wds" => DatasetFormat::WebDataset,
        "energon" => DatasetFormat::EnergonShard,
        "md" | "txt" => DatasetFormat::OssieDocument,
        _ => DatasetFormat::Jsonl,
    };

    let (num_samples, schema_fields) = if matches!(format, DatasetFormat::Jsonl | DatasetFormat::OssieDocument) {
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        let non_empty: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
        let mut fields = Vec::new();
        if let Some(first) = non_empty.first()
            && let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(first)
        {
            fields = map.keys().cloned().collect();
        }
        if fields.is_empty() && format == DatasetFormat::OssieDocument {
            fields = vec!["chunk_text".to_string(), "source_path".to_string()];
        }
        (non_empty.len(), fields)
    } else {
        (0, Vec::new())
    };

    Ok(DatasetManifest {
        name,
        format,
        path: norm,
        num_samples,
        schema_fields,
    })
}

/// Validates structural invariants on a `DatasetManifest`.
pub fn validate_dataset_manifest(m: &DatasetManifest) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    if m.name.trim().is_empty() {
        errs.push("DatasetManifest requires non-empty name".to_string());
    }
    if m.path.trim().is_empty() {
        errs.push("DatasetManifest requires non-empty path".to_string());
    }
    if normalize_path(&m.path).contains("..") {
        errs.push(format!("Unsafe parent traversal in dataset path '{}'", m.path));
    }
    if m.schema_fields.iter().any(|f| f.trim().is_empty()) {
        errs.push("DatasetManifest contains empty schema field name".to_string());
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Builds an `auto-ontology` graph tying schema fields to a domain entity root.
pub fn build_ontology_from_fields(domain: &str, fields: &[&str]) -> OntologyGraph {
    let clean_domain = if domain.trim().is_empty() {
        "toksi_dataset".to_string()
    } else {
        domain.trim().to_string()
    };
    let mut entities = vec![clean_domain.clone()];
    let mut edges = Vec::new();
    for f in fields {
        let trimmed = f.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ent = trimmed.to_string();
        if !entities.contains(&ent) {
            entities.push(ent.clone());
        }
        edges.push(OntologyEdge {
            subject: clean_domain.clone(),
            relation: "has_field".to_string(),
            object: ent,
        });
    }
    OntologyGraph {
        domain: clean_domain,
        entities,
        edges,
    }
}

/// Pure-Rust `liteparse`-compatible document chunker splitting on paragraphs/sentences.
pub fn liteparse_extract_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let cap = max_chars.max(32);
    let mut chunks = Vec::new();
    let mut buf = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !buf.trim().is_empty() {
                chunks.push(buf.trim().to_string());
                buf.clear();
            }
            continue;
        }
        if !buf.is_empty() && buf.len() + trimmed.len() + 1 > cap {
            chunks.push(buf.trim().to_string());
            buf.clear();
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(trimmed);
    }
    if !buf.trim().is_empty() {
        chunks.push(buf.trim().to_string());
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_inspect_ontology_and_liteparse() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let ds_path = tmp.path().join("samples.jsonl");
        std::fs::write(
            &ds_path,
            "{\"prompt\":\"hello\",\"response\":\"world\"}\n{\"prompt\":\"rust\",\"response\":\"safe\"}\n",
        )
        .expect("write jsonl");

        let manifest = inspect_dataset_manifest(&ds_path).expect("inspect");
        assert_eq!(manifest.format, DatasetFormat::Jsonl);
        assert_eq!(manifest.num_samples, 2);
        assert!(validate_dataset_manifest(&manifest).is_ok());

        let ont = build_ontology_from_fields("coding", &["prompt", "response"]);
        assert_eq!(ont.edges.len(), 2);

        let chunks = liteparse_extract_chunks("Para one line.\n\nPara two line.", 64);
        assert_eq!(chunks.len(), 2);
    }
}
