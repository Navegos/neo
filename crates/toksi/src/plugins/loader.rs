// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/loader.rs
// created: 2026-11-21

//! Multi-format plugin manifest discovery across `[plugins].plugin_dirs`
//! (`~/.toksi/plugins`, `<repo>/.toksi/plugins`, `<repo>/.agents/plugins`).

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};

use crate::util::paths::{home, normalize_path};

use super::manifest::parse_manifest_file;
use super::PluginManifest;

const MANIFEST_NAMES: &[&str] = &["plugin.toml", "plugin.yaml", "openclaw.plugin.json"];

#[derive(Debug, Clone)]
pub struct DiscoveredManifest {
    pub manifest: PluginManifest,
    pub manifest_path: String,
}

/// Returns the default `[plugins].plugin_dirs` list normalized with forward slashes.
pub fn default_plugin_dirs(repo_root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let user_home = home();
    if !user_home.is_empty() {
        dirs.push(PathBuf::from(normalize_path(&format!(
            "{}/.toksi/plugins",
            user_home.trim_end_matches('/')
        ))));
    }
    dirs.push(repo_root.join(".toksi").join("plugins"));
    dirs.push(repo_root.join(".agents").join("plugins"));
    dirs
}

/// Discovers and parses `plugin.toml`, `plugin.yaml`, and `openclaw.plugin.json`
/// across the supplied `plugin_dirs`.
pub fn discover_plugin_manifests(plugin_dirs: &[PathBuf]) -> Vec<DiscoveredManifest> {
    let mut found = Vec::new();
    for dir in plugin_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            for fname in MANIFEST_NAMES {
                let candidate = p.join(fname);
                if candidate.is_file()
                    && let Ok(manifest) = parse_manifest_file(&candidate) {
                        found.push(DiscoveredManifest {
                            manifest,
                            manifest_path: normalize_path(&candidate.to_string_lossy()),
                        });
                        break;
                    }
            }
        }
    }
    found
}
