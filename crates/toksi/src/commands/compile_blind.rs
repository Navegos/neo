// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/compile_blind.rs
// created: 2026-11-21

//! CLI dispatcher for `toksi compile-blind <input> [--output <path>] [--taxonomy <tax>]`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::security::blind_compiler::{compile_blind_bundle, DEFAULT_BLIND_KEY};
use crate::util::paths::write_file_atomic;

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].as_str());
        }
        if let Some(rest) = args[i].strip_prefix(&format!("{flag}=")) {
            return Some(rest);
        }
        i += 1;
    }
    None
}

pub fn run_compile_blind_cli(args: &[String]) -> i32 {
    let input_path = args
        .first()
        .filter(|s| !s.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("");
    if input_path.is_empty() {
        eprintln!("Usage: toksi compile-blind <input> [--output <path>] [--taxonomy <tax>]");
        return 1;
    }

    let taxonomy = flag_value(args, "--taxonomy").unwrap_or("generic");
    let raw = match std::fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read '{}': {e}", input_path);
            return 1;
        }
    };

    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    let bundle = match compile_blind_bundle(&lines, &[], taxonomy, "CLEAR", &DEFAULT_BLIND_KEY) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("compile-blind failed: {e}");
            return 1;
        }
    };

    let json_out = serde_json::to_string_pretty(&bundle).unwrap_or_else(|_| "{}".to_string());
    if let Some(out_path) = flag_value(args, "--output") {
        if let Err(e) = write_file_atomic(Path::new(out_path), &json_out) {
            eprintln!("Failed to write '{}': {e}", out_path);
            return 1;
        }
        println!("Compiled zero-plaintext blind signatures to '{}'", out_path);
    } else {
        println!("{json_out}");
    }
    0
}
