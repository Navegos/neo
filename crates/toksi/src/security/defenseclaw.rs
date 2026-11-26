// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/defenseclaw.rs
// created: 2026-11-21

//! `defenseclaw` runtime governance: fail-closed hookguard (`defenseclaw_fail_closed = true`),
//! HMAC-SHA256 gateway auth, `codesig` BLAKE3 verification, YARA/MCP, and JSONL audit evidence.

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::commands::test_runner::blake3_hex;
use crate::security::audit::record_audit_evidence;
use crate::security::hookguard::{evaluate_hookguard, HookGuardVerdict};
use crate::security::yara_mcp::{scan_yara_mcp, YaraMcpMatch};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefenseClawConfig {
    pub fail_closed: bool,
    pub require_codesig: bool,
    pub require_hmac_auth: bool,
}

impl Default for DefenseClawConfig {
    fn default() -> Self {
        Self {
            fail_closed: true,
            require_codesig: true,
            require_hmac_auth: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeSigRecord {
    pub path: String,
    pub blake3_hex: String,
}

/// Computes pure-Rust RFC 2104 HMAC-SHA256 hex digest for gateway auth.
pub fn hmac_sha256_hex(secret: &[u8], message: &[u8]) -> String {
    const BLOCK_SIZE: usize = 64;
    let mut key_block = [0u8; BLOCK_SIZE];
    if secret.len() > BLOCK_SIZE {
        let hashed = Sha256::digest(secret);
        key_block[..32].copy_from_slice(&hashed);
    } else {
        key_block[..secret.len()].copy_from_slice(secret);
    }

    let mut ipad = [0x36u8; BLOCK_SIZE];
    let mut opad = [0x5cu8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    hex::encode(outer.finalize())
}

/// Constant-time HMAC-SHA256 verification for gateway requests.
pub fn verify_hmac_sha256(secret: &[u8], message: &[u8], provided_hex: &str) -> bool {
    let expected = hmac_sha256_hex(secret, message);
    if expected.len() != provided_hex.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in expected.bytes().zip(provided_hex.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Computes and verifies a `codesig` BLAKE3 digest for a file on disk.
pub fn verify_file_codesig(path: &Path, expected_blake3: Option<&str>) -> Result<CodeSigRecord, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("codesig failed to read '{}': {e}", path.display()))?;
    let actual = blake3_hex(&bytes);
    if let Some(exp) = expected_blake3
        && !actual.eq_ignore_ascii_case(exp.trim()) {
            return Err(format!(
                "codesig mismatch for '{}': expected {}, got {}",
                path.display(),
                exp,
                actual
            ));
        }
    Ok(CodeSigRecord {
        path: crate::util::paths::normalize_path(&path.to_string_lossy()),
        blake3_hex: actual,
    })
}

/// Evaluates a DefenseClaw governance gate combining hookguard, YARA-MCP, and audit evidence.
pub fn enforce_defenseclaw_gate(
    repo_root: Option<&Path>,
    actor: &str,
    tool_name: &str,
    payload: &Value,
    prior_tools: &[String],
    cfg: &DefenseClawConfig,
) -> HookGuardVerdict {
    let verdict = evaluate_hookguard(
        repo_root,
        actor,
        tool_name,
        payload,
        prior_tools,
        cfg.fail_closed,
    );
    let yara_hits: Vec<YaraMcpMatch> = scan_yara_mcp(&payload.to_string());
    if !yara_hits.is_empty() && cfg.fail_closed && verdict.allowed {
        let _ = record_audit_evidence(
            repo_root,
            actor,
            "defenseclaw",
            tool_name,
            "blocked_fail_closed",
            "YARA-MCP match blocked under fail-closed policy",
            tool_name,
        );
        return HookGuardVerdict {
            allowed: false,
            fail_closed: true,
            reasons: yara_hits
                .into_iter()
                .map(|m| format!("{}: {}", m.rule_name, m.description))
                .collect(),
        };
    }
    verdict
}
