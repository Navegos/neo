// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/validator/validate.rs
// created: 2026-11-21

//! Native CVE JSON 5.1 (`cve-schema`, `cve-validation-library`, `cve-services`),
//! OSV 1.7.4 (`osv-schema`, `cve-bin-tool`, `cve-lite-cli`), and zero-plaintext
//! `Navegos/advisory` validator.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdvisoryValidationResult {
    pub valid: bool,
    pub schema_kind: String,
    pub errors: Vec<String>,
}

const FORBIDDEN_PLAINTEXT_KEYS: &[&str] = &["raw_exploit", "poc_code", "raw_regex", "exploit_payload"];

/// Validates an OSV 1.7.4 or CVE JSON 5.1 advisory document and enforces zero-plaintext hygiene.
pub fn validate_advisory_json(doc: &Value) -> AdvisoryValidationResult {
    let mut errors = Vec::new();
    let schema_kind = if doc.get("dataType").and_then(Value::as_str) == Some("CVE_RECORD") {
        if doc.get("cveMetadata").and_then(|m| m.get("cveId")).and_then(Value::as_str).is_none() {
            errors.push("CVE 5.1 missing cveMetadata.cveId".to_string());
        }
        if doc.get("containers").is_none() {
            errors.push("CVE 5.1 missing containers".to_string());
        }
        "cve-5.1".to_string()
    } else if doc.get("id").and_then(Value::as_str).is_some() {
        if doc.get("schema_version").and_then(Value::as_str).is_none() {
            errors.push("OSV 1.7.4 missing schema_version".to_string());
        }
        "osv-1.7.4".to_string()
    } else {
        errors.push("Unrecognized advisory schema (expected CVE 5.1 or OSV 1.7.4)".to_string());
        "unknown".to_string()
    };

    check_no_plaintext_keys(doc, &mut errors);

    AdvisoryValidationResult {
        valid: errors.is_empty(),
        schema_kind,
        errors,
    }
}

fn check_no_plaintext_keys(val: &Value, errors: &mut Vec<String>) {
    match val {
        Value::Object(map) => {
            for (k, v) in map {
                let low = k.to_ascii_lowercase();
                if FORBIDDEN_PLAINTEXT_KEYS.contains(&low.as_str()) {
                    errors.push(format!("Zero-plaintext violation: forbidden key '{k}'"));
                }
                check_no_plaintext_keys(v, errors);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                check_no_plaintext_keys(item, errors);
            }
        }
        _ => {}
    }
}
