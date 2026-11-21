// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/byok.rs
// created: 2026-11-21

//! BYOK profile helpers for Google Cloud ADC (`auth_mode = 'adc'`) and multi-instance routing.

#![deny(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use super::operations::ToksiConfigDb;
use super::schema::ByokProfileRecord;

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Inserts or updates a Google Cloud ADC BYOK profile in `byok_profiles`
/// with `auth_mode = "adc"`, `gcp_project`, and `gcp_location` (default `us-central1`).
pub fn upsert_adc_byok_profile(
    db: &mut ToksiConfigDb,
    profile_id: &str,
    model_id: &str,
    gcp_project: &str,
    gcp_location: &str,
    thinking_effort: &str,
) -> Result<ByokProfileRecord, rusqlite::Error> {
    let loc = if gcp_location.trim().is_empty() {
        "us-central1".to_string()
    } else {
        gcp_location.trim().to_string()
    };
    let proj = gcp_project.trim().to_string();
    let effort = if thinking_effort.trim().is_empty() {
        "medium".to_string()
    } else {
        thinking_effort.trim().to_ascii_lowercase()
    };
    let now = now_unix();
    let base_url = format!(
        "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/publishers/google",
        loc, proj, loc
    );
    let record = ByokProfileRecord {
        id: profile_id.trim().to_string(),
        provider_id: "vertex-adc".to_string(),
        instance_label: format!("adc-{}", proj),
        model_id: model_id.trim().to_string(),
        thinking_effort: effort,
        base_url,
        secret_ref: "adc://google-cloud".to_string(),
        auth_mode: "adc".to_string(),
        gcp_project: proj,
        gcp_location: loc,
        priority: 110,
        weight: 100,
        headroom_enabled: true,
        is_active: true,
        created_at: now,
        updated_at: now,
    };
    db.upsert_byok_profile(&record)?;
    Ok(record)
}

/// Lists all active `auth_mode = 'adc'` profiles from `byok_profiles`.
pub fn list_adc_byok_profiles(db: &ToksiConfigDb) -> Result<Vec<ByokProfileRecord>, rusqlite::Error> {
    let all = db.list_byok_profiles()?;
    Ok(all
        .into_iter()
        .filter(|p| p.is_active && p.auth_mode.eq_ignore_ascii_case("adc"))
        .collect())
}

/// Returns the highest-priority active ADC profile if configured.
pub fn resolve_active_adc_profile(db: &ToksiConfigDb) -> Option<ByokProfileRecord> {
    list_adc_byok_profiles(db).ok()?.into_iter().next()
}
