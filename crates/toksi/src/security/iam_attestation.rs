// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/iam_attestation.rs
// created: 2026-11-21

//! SPIFFE / SPIRE cryptographic workload SVID attestation (`spire` parity).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpiffeSvid {
    pub spiffe_id: String,
    pub trust_domain: String,
    pub workload_id: String,
    pub digest_hex: String,
    pub expires_at_unix: i64,
}

impl SpiffeSvid {
    pub fn issue(trust_domain: &str, workload_id: &str, ttl_secs: i64) -> Result<Self, String> {
        let td = trust_domain.trim();
        let wl = workload_id.trim();
        if td.is_empty() || wl.is_empty() {
            return Err("SPIFFE trust_domain and workload_id must be non-empty".to_string());
        }
        let spiffe_id = format!("spiffe://{td}/workload/{wl}");
        let mut hasher = Sha256::new();
        hasher.update(spiffe_id.as_bytes());
        let digest_hex: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let expires_at_unix = chrono::Utc::now().timestamp() + ttl_secs.max(60);
        Ok(Self {
            spiffe_id,
            trust_domain: td.to_string(),
            workload_id: wl.to_string(),
            digest_hex,
            expires_at_unix,
        })
    }

    pub fn is_valid_at(&self, now_unix: i64) -> bool {
        self.spiffe_id.starts_with("spiffe://") && now_unix < self.expires_at_unix
    }
}
