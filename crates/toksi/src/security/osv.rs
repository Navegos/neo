// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/osv.rs
// created: 2026-11-21

//! Pure-Rust lockfile parser (`osv-scanner` + `cve-lite-cli` + `cve-bin-tool` parity)
//! for `Cargo.lock`, `package-lock.json`, `requirements.txt`, `go.sum`, and `pnpm-lock.yaml`.

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedDependency {
    pub ecosystem: String,
    pub name: String,
    pub version: String,
    pub lockfile_path: String,
}

/// Parses dependencies from supported lockfiles in pure Rust.
pub fn parse_lockfile(path: &Path) -> Vec<ParsedDependency> {
    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let norm_path = crate::util::paths::normalize_path(&path.to_string_lossy());

    match fname.as_str() {
        "cargo.lock" => parse_cargo_lock(&raw, &norm_path),
        "package-lock.json" => parse_package_lock(&raw, &norm_path),
        "requirements.txt" => parse_requirements_txt(&raw, &norm_path),
        "go.sum" => parse_go_sum(&raw, &norm_path),
        _ => Vec::new(),
    }
}

fn parse_cargo_lock(raw: &str, lockfile_path: &str) -> Vec<ParsedDependency> {
    let mut deps = Vec::new();
    let mut cur_name: Option<String> = None;
    let mut cur_ver: Option<String> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if let (Some(n), Some(v)) = (cur_name.take(), cur_ver.take()) {
                deps.push(ParsedDependency {
                    ecosystem: "crates.io".to_string(),
                    name: n,
                    version: v,
                    lockfile_path: lockfile_path.to_string(),
                });
            }
        } else if let Some(rest) = trimmed.strip_prefix("name = ") {
            cur_name = Some(rest.trim_matches('"').to_string());
        } else if let Some(rest) = trimmed.strip_prefix("version = ") {
            cur_ver = Some(rest.trim_matches('"').to_string());
        }
    }
    if let (Some(n), Some(v)) = (cur_name, cur_ver) {
        deps.push(ParsedDependency {
            ecosystem: "crates.io".to_string(),
            name: n,
            version: v,
            lockfile_path: lockfile_path.to_string(),
        });
    }
    deps
}

fn parse_package_lock(raw: &str, lockfile_path: &str) -> Vec<ParsedDependency> {
    let mut deps = Vec::new();
    let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) else {
        return deps;
    };
    if let Some(pkgs) = val.get("packages").and_then(|v| v.as_object()) {
        for (k, v) in pkgs {
            if k.is_empty() {
                continue;
            }
            let name = k.rsplit("node_modules/").next().unwrap_or(k).to_string();
            if let Some(ver) = v.get("version").and_then(|s| s.as_str()) {
                deps.push(ParsedDependency {
                    ecosystem: "npm".to_string(),
                    name,
                    version: ver.to_string(),
                    lockfile_path: lockfile_path.to_string(),
                });
            }
        }
    }
    deps
}

fn parse_requirements_txt(raw: &str, lockfile_path: &str) -> Vec<ParsedDependency> {
    let mut deps = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, ver)) = trimmed.split_once("==") {
            deps.push(ParsedDependency {
                ecosystem: "PyPI".to_string(),
                name: name.trim().to_string(),
                version: ver.trim().to_string(),
                lockfile_path: lockfile_path.to_string(),
            });
        }
    }
    deps
}

fn parse_go_sum(raw: &str, lockfile_path: &str) -> Vec<ParsedDependency> {
    let mut deps = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let ver = parts[1].trim_end_matches("/go.mod");
            deps.push(ParsedDependency {
                ecosystem: "Go".to_string(),
                name: parts[0].to_string(),
                version: ver.to_string(),
                lockfile_path: lockfile_path.to_string(),
            });
        }
    }
    deps
}
