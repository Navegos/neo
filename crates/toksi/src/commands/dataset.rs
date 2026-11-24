// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/dataset.rs
// created: 2026-11-24

//! CLI handler for `toksi dataset <inspect|validate|ingest|ontology> ...`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::training::dataset::{
    build_ontology_from_fields, inspect_dataset_manifest, liteparse_extract_chunks,
    validate_dataset_manifest,
};

pub fn run_dataset_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("inspect");
    match sub {
        "inspect" | "validate" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: toksi dataset {sub} <path>");
                return 2;
            };
            match inspect_dataset_manifest(Path::new(path)) {
                Ok(m) => {
                    if sub == "validate"
                        && let Err(errs) = validate_dataset_manifest(&m)
                    {
                        eprintln!("Invalid dataset: {}", errs.join("; "));
                        return 1;
                    }
                    println!("{}", serde_json::to_string_pretty(&m).unwrap_or_default());
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            }
        }
        "ingest" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: toksi dataset ingest <path> [max_chars]");
                return 2;
            };
            let max_chars = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(512);
            let Ok(raw) = std::fs::read_to_string(path) else {
                eprintln!("Failed to read dataset source: {path}");
                return 1;
            };
            let chunks = liteparse_extract_chunks(&raw, max_chars);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "source": path,
                    "chunks": chunks.len(),
                }))
                .unwrap_or_default()
            );
            0
        }
        "ontology" => {
            let domain = args.get(1).map(String::as_str).unwrap_or("default");
            let fields: Vec<&str> = args
                .get(2)
                .map(|s| s.split(',').map(str::trim).filter(|x| !x.is_empty()).collect())
                .unwrap_or_else(|| vec!["id", "text", "label"]);
            let ont = build_ontology_from_fields(domain, &fields);
            println!("{}", serde_json::to_string_pretty(&ont).unwrap_or_default());
            0
        }
        _ => {
            eprintln!("Usage: toksi dataset <inspect|validate|ingest|ontology> [args...]");
            2
        }
    }
}
