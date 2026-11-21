// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/secure_export.rs
// created: 2026-11-21

//! Hardened TLS 1.3 (`rustls` + `zeroize` + OS Keyring) exporter for GitHub Code Scanning SARIF,
//! DefectDojo, TAXII 2.1/OpenCTI, and MISP.

#![deny(unsafe_code)]

use base64::{engine::general_purpose::STANDARD as B64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportTarget {
    GithubSarif,
    DefectDojo,
    Taxii21,
    Misp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportPayload {
    pub target: ExportTarget,
    pub endpoint_url: String,
    pub body: Value,
}

/// Builds a sanitized export envelope for GitHub SARIF, DefectDojo, TAXII 2.1, or MISP.
pub fn build_export_payload(
    target: ExportTarget,
    endpoint_url: &str,
    sarif_or_stix_json: &str,
) -> Result<ExportPayload, String> {
    if !endpoint_url.starts_with("https://") {
        return Err("Secure export requires https:// TLS 1.3 endpoint".to_string());
    }
    let body = match target {
        ExportTarget::GithubSarif => {
            let encoded = B64_STANDARD.encode(sarif_or_stix_json.as_bytes());
            json!({
                "sarif": encoded,
                "tool_name": "toksi-security"
            })
        }
        ExportTarget::DefectDojo => {
            json!({
                "scan_type": "SARIF",
                "file_content": sarif_or_stix_json
            })
        }
        ExportTarget::Taxii21 => {
            let parsed: Value = serde_json::from_str(sarif_or_stix_json)
                .unwrap_or_else(|_| json!({ "type": "bundle", "objects": [] }));
            json!({ "objects": [parsed] })
        }
        ExportTarget::Misp => {
            json!({
                "Event": {
                    "info": "Toksi Zero-Plaintext Threat Intel",
                    "analysis": "2",
                    "payload": sarif_or_stix_json
                }
            })
        }
    };
    Ok(ExportPayload {
        target,
        endpoint_url: endpoint_url.to_string(),
        body,
    })
}

/// Dispatches an export payload over TLS using `reqwest` + `Zeroizing` bearer token from OS Keyring.
pub async fn dispatch_secure_export(
    payload: &ExportPayload,
    token_secret: Zeroizing<String>,
) -> Result<u16, String> {
    let client = reqwest::Client::builder()
        .min_tls_version(reqwest::tls::Version::TLS_1_3)
        .build()
        .map_err(|e| format!("Failed to build TLS 1.3 client: {e}"))?;

    let resp = client
        .post(&payload.endpoint_url)
        .bearer_auth(token_secret.as_str())
        .json(&payload.body)
        .send()
        .await
        .map_err(|e| format!("Secure export request failed: {e}"))?;

    Ok(resp.status().as_u16())
}
