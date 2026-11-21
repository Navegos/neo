// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/schema.rs
// created: 2026-11-21

//! Canonical zero-plaintext security schemas: OSV 1.7.4, CVE JSON 5.1, STIX 2.1,
//! SARIF 2.1.0, `BlindSignatures`, `TargetKind`, `SourceOrigin`, `DistributionChannel`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Package,
    SourceRepo,
    Agent,
    Skill,
    Rule,
    Command,
    Script,
    CodeSnippet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOrigin {
    GithubRepo,
    ExternalGitRepo,
    PackageRegistry,
    Distributed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionChannel {
    OciRegistry,
    ReleaseAsset,
    ArchiveBundle,
    RawUrl,
    LocalDir,
    SingleFile,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlindSignatures {
    #[serde(default)]
    pub blake3_ngrams: Vec<String>,
    #[serde(default)]
    pub sparse_dfas_b64: Vec<String>,
    #[serde(default)]
    pub taxonomy: String,
    #[serde(default)]
    pub tlp_level: String,
}

impl BlindSignatures {
    /// Embeds blind signatures into an OSV 1.7.4 document (`database_specific.blind_signatures`).
    pub fn embed_into_osv(&self, osv: &mut Value) {
        if let Some(obj) = osv.as_object_mut() {
            let db_spec = obj
                .entry("database_specific")
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Some(db_map) = db_spec.as_object_mut() {
                db_map.insert(
                    "blind_signatures".to_string(),
                    serde_json::to_value(self).unwrap_or(Value::Null),
                );
            }
        }
    }

    /// Embeds blind signatures into CVE JSON 5.1 (`containers.adp[].x_blind_signatures`).
    pub fn embed_into_cve51(&self, cve: &mut Value) {
        if let Some(containers) = cve.get_mut("containers").and_then(Value::as_object_mut) {
            let adp = containers
                .entry("adp")
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Some(arr) = adp.as_array_mut() {
                arr.push(json!({
                    "providerMetadata": { "shortName": "toksi" },
                    "x_blind_signatures": self
                }));
            }
        }
    }

    /// Embeds blind signatures into a STIX 2.1 SDO (`x_blind_signatures`).
    pub fn embed_into_stix21(&self, stix_sdo: &mut Value) {
        if let Some(obj) = stix_sdo.as_object_mut() {
            obj.insert(
                "x_blind_signatures".to_string(),
                serde_json::to_value(self).unwrap_or(Value::Null),
            );
        }
    }
}
