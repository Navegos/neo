// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/manifest.rs
// created: 2026-11-21

//! Multi-format plugin manifest parser supporting `plugin.toml`, `plugin.yaml`,
//! and `openclaw.plugin.json`.

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{PluginCapability, PluginManifest};

/// Parses a `plugin.toml`, `plugin.yaml`, or `openclaw.plugin.json` manifest from disk.
pub fn parse_manifest_file(path: &Path) -> Result<PluginManifest, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read manifest {}: {e}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if file_name.ends_with(".toml") {
        toml::from_str::<PluginManifest>(&raw)
            .map_err(|e| format!("Invalid plugin.toml {}: {e}", path.display()))
    } else if file_name.ends_with(".json") {
        parse_openclaw_or_json_manifest(&raw)
    } else if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
        parse_simple_plugin_yaml(&raw)
    } else {
        Err(format!("Unsupported plugin manifest extension: {file_name}"))
    }
}

fn parse_openclaw_or_json_manifest(raw: &str) -> Result<PluginManifest, String> {
    if let Ok(m) = serde_json::from_str::<PluginManifest>(raw) {
        return Ok(m);
    }
    let val: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("Invalid plugin JSON: {e}"))?;
    let id = val
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "openclaw.plugin.json missing 'id'".to_string())?;
    let name = val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(id)
        .to_string();
    let version = val
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();
    let description = val
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let entry_path = val
        .get("entry")
        .or_else(|| val.get("entry_path"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(PluginManifest {
        id: id.to_string(),
        name,
        version,
        description,
        entry_path,
        capabilities: vec![PluginCapability::Tool],
        protocols: vec!["mcp".to_string()],
        slash_commands: Vec::new(),
    })
}

/// Deterministic zero-dependency parser for flat `plugin.yaml` key-value descriptors.
fn parse_simple_plugin_yaml(raw: &str) -> Result<PluginManifest, String> {
    let mut id = String::new();
    let mut name = String::new();
    let mut version = "0.1.0".to_string();
    let mut description = String::new();
    let mut entry_path = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'');
            match key {
                "id" => id = val.to_string(),
                "name" => name = val.to_string(),
                "version" => version = val.to_string(),
                "description" => description = val.to_string(),
                "entry" | "entry_path" => entry_path = val.to_string(),
                _ => {}
            }
        }
    }

    if id.is_empty() {
        return Err("plugin.yaml missing required 'id'".to_string());
    }
    if name.is_empty() {
        name = id.clone();
    }
    Ok(PluginManifest {
        id,
        name,
        version,
        description,
        entry_path,
        capabilities: vec![PluginCapability::Tool],
        protocols: vec!["mcp".to_string()],
        slash_commands: Vec::new(),
    })
}

/// `NeMo-Relay-Plugins` (`relay-plugin.toml`) verified relay manifest contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayPluginManifest {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_relay_version")]
    pub version: String,
    #[serde(default)]
    pub entrypoint: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub protocols: Vec<String>,
}

fn default_relay_version() -> String {
    "0.1.0".to_string()
}

pub fn parse_relay_plugin_toml(raw: &str) -> Result<RelayPluginManifest, String> {
    let m = toml::from_str::<RelayPluginManifest>(raw)
        .map_err(|e| format!("Invalid relay-plugin.toml: {e}"))?;
    if m.id.trim().is_empty() {
        return Err("relay-plugin.toml requires non-empty 'id'".to_string());
    }
    Ok(m)
}

/// Constant-style lowercase SHA-256 hex verification for downloaded/staged plugin artifacts.
pub fn verify_sha256_hex(payload: &[u8], expected_hex: &str) -> bool {
    let clean = expected_hex.trim().to_ascii_lowercase();
    if clean.len() != 64 || !clean.bytes().all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    let digest = Sha256::digest(payload);
    hex::encode(digest) == clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_plugin_toml_and_sha256() {
        let payload = b"toksi-relay-plugin";
        let sum = hex::encode(Sha256::digest(payload));
        assert!(verify_sha256_hex(payload, &sum));
        assert!(!verify_sha256_hex(payload, "deadbeef"));

        let raw = format!(
            "id = \"nemo.relay.tools\"\nname = \"NeMo Relay\"\nversion = \"1.2.0\"\nsha256 = \"{}\"\nprotocols = [\"mcp\", \"a2a\"]\n",
            sum
        );
        let parsed = parse_relay_plugin_toml(&raw).expect("parse relay-plugin.toml");
        assert_eq!(parsed.id, "nemo.relay.tools");
        assert!(verify_sha256_hex(payload, &parsed.sha256));
    }
}
