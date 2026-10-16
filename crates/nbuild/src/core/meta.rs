// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/meta.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub platform: String,
    pub triple: String,
    pub arch: String,
    /// "system" (compilers, OS SDKs, CRT) or "user" (libraries, dependencies, dev tools)
    pub target_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compiler_definitions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configure_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub install_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMeta {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkRecord {
    pub src: String,
    pub dest: String,
    /// "exec" | "dll" | "header" | "lib" | "pkgconfig" | "data" | "dir"
    pub symlink_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

/// Package receipt metadata stored at $TOOLCHAINS_DIR/<platform>/<triple>/distro/<pkg>/pkg_meta.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgMeta {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub name: String,
    pub version: String,
    pub rawversion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abiversion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soversion: Option<String>,
    /// "source_build" | "prebuilt_binary" | "sdk"
    #[serde(rename = "type")]
    pub pkg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    pub target: TargetInfo,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildInfo>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<DependencyMeta>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env_exports: BTreeMap<String, Vec<String>>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symlinks: Vec<SymlinkRecord>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub installed_files: Vec<FileRecord>,
}

impl PkgMeta {
    pub fn new(name: &str, version: &str, pkg_type: &str, target: TargetInfo) -> Self {
        Self {
            schema: Some("https://neo.build/schemas/pkg_meta.v1.json".to_string()),
            name: name.to_string(),
            version: version.to_string(),
            rawversion: version.to_string(),
            abiversion: None,
            soversion: None,
            pkg_type: pkg_type.to_string(),
            tag_name: None,
            commit: None,
            date: Some(Utc::now().format("%Y-%m-%d").to_string()),
            updated_at: Some(Utc::now().to_rfc3339()),
            url: None,
            sha256: None,
            target,
            build: None,
            dependencies: Vec::new(),
            env_exports: BTreeMap::new(),
            symlinks: Vec::new(),
            installed_files: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;
        let meta = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON in {}", path.as_ref().display()))?;
        Ok(meta)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(self)?;
        fs::write(p, json_str)?;
        Ok(())
    }
}

