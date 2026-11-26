// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/vertex_adc.rs
// created: 2026-11-21

//! Regional Vertex AI `v1beta1` endpoint & `AGY_ADC_AUTH=true` native OAuth2 signer.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::headroom::effort::{apply_effort_wire_translation, EffortLevel};
use crate::keyring::adc::{
    format_missing_adc_guidance, get_or_refresh_adc_token, inspect_adc_status,
    is_agy_adc_auth_enabled, read_vscode_gcp_settings, DEFAULT_GCP_LOCATION,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAdcEndpoint {
    pub project: String,
    pub location: String,
    pub model: String,
    pub adc_enabled: bool,
}

impl VertexAdcEndpoint {
    pub fn from_env_or_config(project: &str, location: &str, model: &str) -> Self {
        let env_adc = is_agy_adc_auth_enabled();
        let loc = if location.trim().is_empty() {
            DEFAULT_GCP_LOCATION
        } else {
            location.trim()
        };
        Self {
            project: project.trim().to_string(),
            location: loc.to_string(),
            model: model.trim().to_string(),
            adc_enabled: env_adc,
        }
    }

    /// Resolves Vertex AI endpoint from env (`AGY_ADC_AUTH`, `GOOGLE_CLOUD_PROJECT`),
    /// `~/.toksi/config.toml`, VS Code `"gcp.project"` / `"gcp.location"`, or `byok_profiles`.
    pub fn resolve_from_system(model: &str) -> Result<Self, String> {
        let (mut project, mut location) = read_vscode_gcp_settings();
        let adc_enabled = is_agy_adc_auth_enabled();

        if project.is_none()
            && let Ok(db) = crate::db::open_default_db()
                && let Some(prof) = crate::db::resolve_active_adc_profile(&db) {
                    if !prof.gcp_project.trim().is_empty() {
                        project = Some(prof.gcp_project);
                    }
                    if !prof.gcp_location.trim().is_empty() {
                        location = prof.gcp_location;
                    }
                }

        let Some(proj) = project.filter(|p| !p.trim().is_empty()) else {
            let status = inspect_adc_status();
            return Err(format_missing_adc_guidance(None, &status.adc_path));
        };

        Ok(Self {
            project: proj,
            location,
            model: model.trim().to_string(),
            adc_enabled,
        })
    }

    /// Builds regional Vertex AI `v1beta1` `generateContent` / `streamGenerateContent` URL.
    pub fn publisher_url(&self, stream: bool) -> Result<String, String> {
        if self.project.trim().is_empty() {
            let status = inspect_adc_status();
            return Err(format_missing_adc_guidance(None, &status.adc_path));
        }
        let method = if stream {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };
        Ok(format!(
            "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/publishers/google/models/{}:{}",
            self.location, self.project, self.location, self.model, method
        ))
    }

    /// Canonical `v1beta1` `:streamGenerateContent` URL without query parameters.
    pub fn stream_generate_content_url(&self) -> Result<String, String> {
        if self.project.trim().is_empty() {
            let status = inspect_adc_status();
            return Err(format_missing_adc_guidance(None, &status.adc_path));
        }
        Ok(format!(
            "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/publishers/google/models/{}:streamGenerateContent",
            self.location, self.project, self.location, self.model
        ))
    }

    /// Returns signed HTTP headers (`Authorization: Bearer ...` + `x-goog-user-project`).
    pub fn sign_vertex_headers(&self, bearer_token: &str) -> Vec<(String, String)> {
        vec![
            (
                "Authorization".to_string(),
                format!("Bearer {}", bearer_token),
            ),
            ("x-goog-user-project".to_string(), self.project.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ]
    }

    /// Resolves cached/refreshed ADC OAuth2 token, translates effort budget into `payload`,
    /// and returns `(url, signed_headers)`.
    pub async fn build_signed_vertex_request(
        &self,
        payload: &mut Value,
        effort: EffortLevel,
        stream: bool,
    ) -> Result<(String, Vec<(String, String)>), String> {
        let url = self.publisher_url(stream)?;
        let _ = apply_effort_wire_translation(&url, payload, effort, true);
        let tok = get_or_refresh_adc_token(None).await?;
        let headers = self.sign_vertex_headers(&tok.access_token);
        Ok((url, headers))
    }
}
