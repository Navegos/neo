// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/model_spec_verifier.rs
// created: 2026-11-24

//! Pure-Rust cryptographic model weight/config provenance verifier (`model-spec`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::commands::test_runner::blake3_hex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelArtifactChecksum {
    pub path: String,
    pub sha256_hex: String,
    pub blake3_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelSpecManifest {
    pub model_id: String,
    pub revision: String,
    pub license: String,
    pub artifacts: Vec<ModelArtifactChecksum>,
}

fn compute_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// Validates manifest structure and verifies concrete artifact byte digests (fail-closed).
pub fn verify_model_spec_artifact(
    manifest: &ModelSpecManifest,
    artifact_path: &str,
    bytes: &[u8],
) -> Result<(), String> {
    if manifest.model_id.trim().is_empty() || manifest.revision.trim().is_empty() {
        return Err("ModelSpec fail-closed: unpinned model_id or revision".to_string());
    }
    if manifest.artifacts.is_empty() {
        return Err("ModelSpec fail-closed: empty artifact checksum table".to_string());
    }
    let entry = manifest
        .artifacts
        .iter()
        .find(|a| a.path == artifact_path)
        .ok_or_else(|| format!("ModelSpec fail-closed: artifact '{artifact_path}' not attested"))?;

    if entry.sha256_hex.len() != 64 || entry.blake3_hex.len() != 64 {
        return Err(format!("ModelSpec fail-closed: invalid digest length for '{artifact_path}'"));
    }

    let actual_sha = compute_sha256_hex(bytes);
    if !actual_sha.eq_ignore_ascii_case(&entry.sha256_hex) {
        return Err(format!(
            "ModelSpec SHA-256 mismatch on '{artifact_path}': expected {}, got {actual_sha}",
            entry.sha256_hex
        ));
    }

    let actual_b3 = blake3_hex(bytes);
    if !actual_b3.eq_ignore_ascii_case(&entry.blake3_hex) {
        return Err(format!(
            "ModelSpec BLAKE3 mismatch on '{artifact_path}': expected {}, got {actual_b3}",
            entry.blake3_hex
        ));
    }
    Ok(())
}
