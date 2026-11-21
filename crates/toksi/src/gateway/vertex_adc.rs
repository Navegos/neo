// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/vertex_adc.rs
// created: 2026-11-21

//! `python-aiplatform` regional Vertex AI endpoint & `AGY_ADC_AUTH=true` adapter.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAdcEndpoint {
    pub project: String,
    pub location: String,
    pub model: String,
    pub adc_enabled: bool,
}

impl VertexAdcEndpoint {
    pub fn from_env_or_config(project: &str, location: &str, model: &str) -> Self {
        let env_adc = std::env::var("AGY_ADC_AUTH")
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        let loc = if location.trim().is_empty() {
            "us-central1"
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

    /// Builds regional Vertex AI `generateContent` / `streamGenerateContent` URL.
    pub fn publisher_url(&self, stream: bool) -> Result<String, String> {
        if self.project.is_empty() {
            return Err("Vertex ADC requires non-empty gcp.project".to_string());
        }
        let method = if stream {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };
        Ok(format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:{}",
            self.location, self.project, self.location, self.model, method
        ))
    }
}
