// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/catalog.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpec {
    /// "binary" | "archive" | "git"
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSpec {
    pub builder: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configure_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub install_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compiler_definitions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRecipe {
    pub source: SourceSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildSpec>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env_exports: BTreeMap<String, Vec<String>>,
}

fn default_version() -> String {
    "latest".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCatalog {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rawversion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abiversion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soversion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// "compiler" | "sdk" | "tool" | "library"
    pub category: String,
    /// "system" (puts in triple root) | "user" (puts in triple/usr)
    pub target_level: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_subpath: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_hints: Option<BTreeMap<String, BTreeMap<String, Vec<String>>>>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env_exports: BTreeMap<String, Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpkg_port: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vcpkg_headers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vcpkg_libs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vcpkg_tools: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,

    /// Map of target triple or platform name -> PlatformRecipe
    #[serde(default)]
    pub targets: BTreeMap<String, PlatformRecipe>,
}

impl PackageCatalog {
    pub fn get_remote_repo(&self) -> Option<&str> {
        self.remote_repo.as_deref().or(self.github_repo.as_deref())
    }

    pub fn get_releases_url(&self) -> Option<&str> {
        self.releases_url.as_deref()
    }

    pub fn get_asset_hints(&self, platform: &str, arch: &str) -> Option<&Vec<String>> {
        self.asset_hints.as_ref()
            .and_then(|p| p.get(platform))
            .and_then(|a| a.get(arch))
    }

    pub fn is_vcpkg(&self) -> bool {
        self.manager.as_deref() == Some("vcpkg")
            || self.targets.values().any(|t| {
                t.source.source_type == "vcpkg"
                    || t.build.as_ref().map_or(false, |b| b.builder == "vcpkg")
            })
    }
}

pub struct CatalogRegistry {
    pub catalogs: BTreeMap<String, PackageCatalog>,
    pub search_paths: Vec<PathBuf>,
}

impl CatalogRegistry {
    pub fn new() -> Self {
        Self {
            catalogs: BTreeMap::new(),
            search_paths: Vec::new(),
        }
    }

    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref().to_path_buf();
        if !self.search_paths.contains(&p) {
            self.search_paths.push(p);
        }
    }

    /// Load all JSON / YAML catalogs from search paths
    pub fn load_all(&mut self) -> Result<()> {
        for search_dir in &self.search_paths.clone() {
            if search_dir.is_dir() {
                self.load_dir(search_dir)?;
            }
        }
        Ok(())
    }

    pub fn load_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.load_dir(&path)?;
            } else if let Some(ext) = path.extension() {
                if ext == "json" {
                    match Self::load_file(&path) {
                        Ok(cat) => {
                            self.catalogs.insert(cat.name.clone(), cat);
                        }
                        Err(e) => {
                            eprintln!("[-] Warning: Failed to parse catalog {}: {:#}", path.display(), e);
                        }
                    }
                } else if ext == "yaml" || ext == "yml" {
                    match Self::load_yaml_file(&path) {
                        Ok(cat) => {
                            self.catalogs.insert(cat.name.clone(), cat);
                        }
                        Err(e) => {
                            eprintln!("[-] Warning: Failed to parse catalog {}: {:#}", path.display(), e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_file(path: &Path) -> Result<PackageCatalog> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Reading catalog file {}", path.display()))?;
        let cat: PackageCatalog = serde_json::from_str(&content)
            .with_context(|| format!("Parsing JSON catalog {}", path.display()))?;
        Ok(cat)
    }

    pub fn load_yaml_file(path: &Path) -> Result<PackageCatalog> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Reading YAML catalog file {}", path.display()))?;
        let cat: PackageCatalog = serde_yaml::from_str(&content)
            .with_context(|| format!("Parsing YAML catalog {}", path.display()))?;
        Ok(cat)
    }

    pub fn get(&self, name: &str) -> Option<&PackageCatalog> {
        self.catalogs.get(name)
    }

    pub fn list(&self) -> Vec<&PackageCatalog> {
        self.catalogs.values().collect()
    }
}

