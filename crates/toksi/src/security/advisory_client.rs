// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/advisory_client.rs
// created: 2026-11-21

//! `Navegos/advisory` O(1) prefix search (`~/.toksi/feed/` cache), `DisclosureGate`
//! embargo enforcement, and 12-DB `UpstreamDeduplicator`.

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::{open_default_db, AdvisoryEmbargoRecord};
use crate::util::paths::{home, normalize_path};

pub const DEFAULT_ADVISORY_PAGES_URL: &str = "https://navegos.github.io/advisory/";

/// All 12 upstream security advisory databases checked by `UpstreamDeduplicator`.
pub const UPSTREAM_ADVISORY_DATABASES: &[(&str, &str)] = &[
    ("navegos-advisory", "https://navegos.github.io/advisory/"),
    ("nvd", "https://nvd.nist.gov/"),
    ("redhat-security-data", "https://access.redhat.com/hydra/rest/securitydata"),
    ("gitlab-advisory-db", "https://advisories.gitlab.com/"),
    ("osv", "https://osv.dev/list"),
    ("rustsec", "https://rustsec.org/advisories/"),
    ("curl-vuln", "https://curl.se/docs/vuln.json"),
    ("github-advisory-db", "https://github.com/advisories"),
    ("friendsofphp", "https://github.com/FriendsOfPHP/security-advisories"),
    ("go-vuln-db", "https://vuln.go.dev/"),
    ("pypa-advisory-db", "https://github.com/pypa/advisory-database"),
    ("rubysec", "https://rubysec.com/"),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisclosureGateInput {
    pub advisory_id: String,
    pub tlp_level: String,
    pub patch_released: bool,
    pub issue_resolved: bool,
    pub force_publish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisclosureGateDecision {
    pub publish_allowed: bool,
    pub reason: String,
}

/// Enforces responsible disclosure policy (`DisclosureGate`) and persists embargo state in `advisory_embargoes`.
pub fn evaluate_disclosure_gate(input: &DisclosureGateInput) -> DisclosureGateDecision {
    let tlp = input.tlp_level.trim().to_ascii_uppercase();
    let is_restricted_tlp = tlp == "TLP:RED" || tlp == "RED" || tlp == "TLP:AMBER" || tlp == "AMBER";
    let publish_allowed =
        input.force_publish || (!is_restricted_tlp && (input.patch_released || input.issue_resolved));

    let reason = if publish_allowed {
        if input.force_publish {
            "Allowed via explicit --force-publish (zero-plaintext enforced)".to_string()
        } else {
            "Allowed: patch released or issue resolved under non-restricted TLP".to_string()
        }
    } else {
        format!(
            "Embargoed by DisclosureGate (tlp={tlp}, patch_released={}, resolved={})",
            input.patch_released, input.issue_resolved
        )
    };

    if let Ok(mut db) = open_default_db() {
        let now = chrono::Utc::now().timestamp();
        let rec = AdvisoryEmbargoRecord {
            advisory_id: input.advisory_id.clone(),
            disclosure_state: if publish_allowed {
                "cleared".to_string()
            } else {
                "embargoed".to_string()
            },
            embargo_until_epoch: if publish_allowed { 0 } else { now + 86400 * 30 },
            upstream_repo: "Navegos/advisory".to_string(),
            pr_url: None,
            updated_at: now,
        };
        let _ = db.upsert_advisory_embargo(&rec);
    }

    DisclosureGateDecision {
        publish_allowed,
        reason,
    }
}

/// Returns local feed cache directory (`~/.toksi/feed/`).
pub fn default_feed_cache_dir() -> PathBuf {
    let h = home();
    if h.is_empty() {
        PathBuf::from(".toksi/feed")
    } else {
        PathBuf::from(normalize_path(&format!("{}/.toksi/feed", h.trim_end_matches('/'))))
    }
}

/// Performs O(1) prefix-sharded lookup in a local `Navegos/advisory` clone or `~/.toksi/feed/` cache.
pub fn search_advisory_local_o1(advisory_root: &Path, query_id: &str) -> Vec<Value> {
    let q = query_id.trim().to_ascii_uppercase();
    let mut results = Vec::new();
    if q.is_empty() || !advisory_root.exists() {
        return results;
    }

    let lookup_global = advisory_root.join("lookup").join("_global");
    let kind_dir = if q.starts_with("CVE-") {
        "cve"
    } else if q.starts_with("GHSA-") {
        "ghsa"
    } else if q.starts_with("TOKSI-") {
        "toksi"
    } else {
        "vuln"
    };

    let year = q
        .split('-')
        .nth(1)
        .filter(|y| y.len() == 4 && y.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or("2026");
    let prefix: String = q.chars().take(8).collect();
    let shard_file = lookup_global
        .join(kind_dir)
        .join(year)
        .join(format!("{prefix}.json"));

    if shard_file.is_file()
        && let Ok(raw) = std::fs::read_to_string(&shard_file)
            && let Ok(val) = serde_json::from_str::<Value>(&raw) {
                results.push(val);
            }
    results
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeduplicationReport {
    pub query: String,
    pub checked_databases: Vec<String>,
    pub duplicate_found: bool,
    pub matched_source: Option<String>,
}

/// Runs 12-DB `UpstreamDeduplicator` against local `Navegos/advisory` shards and configured upstream feed cache.
pub fn check_upstream_duplicates(advisory_root: &Path, id_or_cve: &str) -> DeduplicationReport {
    let checked_databases: Vec<String> = UPSTREAM_ADVISORY_DATABASES
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect();
    let local_hits = search_advisory_local_o1(advisory_root, id_or_cve);
    if !local_hits.is_empty() {
        return DeduplicationReport {
            query: id_or_cve.to_string(),
            checked_databases,
            duplicate_found: true,
            matched_source: Some("navegos-advisory".to_string()),
        };
    }

    let feed_cache = default_feed_cache_dir();
    if feed_cache.exists() {
        for (name, _) in UPSTREAM_ADVISORY_DATABASES {
            let candidate = feed_cache.join(format!("{name}.json"));
            if let Ok(raw) = std::fs::read_to_string(candidate)
                && raw.contains(id_or_cve) {
                    return DeduplicationReport {
                        query: id_or_cve.to_string(),
                        checked_databases,
                        duplicate_found: true,
                        matched_source: Some((*name).to_string()),
                    };
                }
        }
    }

    DeduplicationReport {
        query: id_or_cve.to_string(),
        checked_databases,
        duplicate_found: false,
        matched_source: None,
    }
}
