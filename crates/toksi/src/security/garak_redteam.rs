// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/garak_redteam.rs
// created: 2026-11-21

//! `garak` defensive self-evaluation harness under `<enforce>`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::security::guardrails_validators::{defensive_garak_probes, validate_and_anonymize};
use crate::security::llm_guard::{scan_llm_guard, LlmGuardConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GarakProbeResult {
    pub probe_id: String,
    pub blocked: bool,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GarakSelfEvalReport {
    pub total_probes: usize,
    pub blocked_count: usize,
    pub pass_rate: f32,
    pub results: Vec<GarakProbeResult>,
}

/// Runs defensive Garak probes against Toksi's Guardrails + LLM-Guard pipeline.
pub fn run_defensive_garak_eval() -> GarakSelfEvalReport {
    let probes = defensive_garak_probes();
    let cfg = LlmGuardConfig::default();
    let mut results = Vec::with_capacity(probes.len());
    let mut blocked_count = 0usize;

    for (probe_id, prompt) in probes {
        let g = validate_and_anonymize(prompt);
        let lg = scan_llm_guard(prompt, &cfg);
        let blocked = !g.passed || !lg.is_valid;
        if blocked {
            blocked_count += 1;
        }
        let mut details = g.findings;
        details.extend(lg.triggered_scanners);
        results.push(GarakProbeResult {
            probe_id: probe_id.to_string(),
            blocked,
            details,
        });
    }

    let total_probes = results.len();
    let pass_rate = if total_probes == 0 {
        1.0
    } else {
        blocked_count as f32 / total_probes as f32
    };

    GarakSelfEvalReport {
        total_probes,
        blocked_count,
        pass_rate,
        results,
    }
}
