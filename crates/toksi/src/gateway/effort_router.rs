// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/effort_router.rs
// created: 2026-11-21

//! Thinking-effort-aware router (`Low` / `Medium` / `High`) uniting CLI `--effort`,
//! `/effort`, `x-toksi-effort`, and provider wire parameters with context-budget policies.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::headroom::effort::{
    apply_effort_wire_translation, detect_effort_level, global_effort_metrics,
    EffortCompressionStats, EffortLevel, HeadroomEffortCompressor,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffortContextStrategy {
    pub effort: EffortLevel,
    pub scip_multi_hop: bool,
    pub blast_radius_depth: usize,
    pub ast_top_k_slices: usize,
    pub rtk_max_compaction: bool,
    pub icm_summary_cache: bool,
    pub lossless_constraints_and_traces: bool,
}

impl EffortContextStrategy {
    pub fn for_effort(effort: EffortLevel) -> Self {
        match effort {
            EffortLevel::Low => Self {
                effort,
                scip_multi_hop: false,
                blast_radius_depth: 0,
                ast_top_k_slices: 3,
                rtk_max_compaction: true,
                icm_summary_cache: false,
                lossless_constraints_and_traces: false,
            },
            EffortLevel::Medium => Self {
                effort,
                scip_multi_hop: false,
                blast_radius_depth: 1,
                ast_top_k_slices: 8,
                rtk_max_compaction: false,
                icm_summary_cache: true,
                lossless_constraints_and_traces: false,
            },
            EffortLevel::High => Self {
                effort,
                scip_multi_hop: true,
                blast_radius_depth: 3,
                ast_top_k_slices: 24,
                rtk_max_compaction: false,
                icm_summary_cache: true,
                lossless_constraints_and_traces: true,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortRouteDecision {
    pub effort: EffortLevel,
    pub wire_translated: bool,
    pub compression_stats: EffortCompressionStats,
    pub context_strategy: EffortContextStrategy,
}

/// Parses `--effort <low|medium|high>` or `--effort=<tier>` from CLI arguments.
pub fn parse_effort_cli_flag(args: &[String]) -> Option<EffortLevel> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--effort" && i + 1 < args.len() {
            return EffortLevel::from_str_loose(&args[i + 1]);
        }
        if let Some(val) = args[i].strip_prefix("--effort=") {
            return EffortLevel::from_str_loose(val);
        }
        i += 1;
    }
    None
}

/// Routes a mutable JSON request through effort detection, provider wire translation,
/// and `HeadroomEffortCompressor` in one unified step.
pub fn route_and_translate(
    path: &str,
    headers: &[(String, String)],
    payload: &mut Value,
    cli_or_slash_effort: Option<EffortLevel>,
) -> EffortRouteDecision {
    let mut effective_headers = headers.to_vec();
    let mut explicit = cli_or_slash_effort.is_some()
        || headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("x-toksi-effort"));

    if let Some(level) = cli_or_slash_effort {
        effective_headers.retain(|(k, _)| !k.eq_ignore_ascii_case("x-toksi-effort"));
        effective_headers.push(("x-toksi-effort".to_string(), level.as_str().to_string()));
        explicit = true;
    }

    let effort = detect_effort_level(&effective_headers, Some(payload));
    global_effort_metrics().set_active_effort(effort);

    let wire_translated = apply_effort_wire_translation(path, payload, effort, explicit);
    let compression_stats = HeadroomEffortCompressor::gateway_default().compress_json_payload(
        path,
        &effective_headers,
        payload,
    );

    EffortRouteDecision {
        effort,
        wire_translated,
        compression_stats,
        context_strategy: EffortContextStrategy::for_effort(effort),
    }
}
