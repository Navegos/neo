// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/mod.rs
// created: 2026-10-27

pub mod cli;
pub mod heatmap;
pub mod mcp;
pub mod precheck;
pub mod redaction;
pub mod schema;
pub mod store;
pub mod summary;

pub use cli::run_mem_cli;
pub use heatmap::{calculate_score, generate_heatmap, HeatmapResult, ScoreResult};
pub use mcp::{handle_mcp_message, run_mcp_server};
pub use precheck::run_precheck;
pub use redaction::{is_redaction_enabled, redact, redact_event_fields};
pub use schema::{
    auto_resolve_location, default_event_id, location_to_file, normalize_timestamp,
    parse_location_coordinate, resolve_event_ref, resolve_location_symbol, superseded_ids,
    utc_now_iso, Attempt, Decision, Event, Fix, Issue, Note, VALID_CAPTURE_SOURCES,
    VALID_CONFIDENCE_LEVELS, VALID_EVENT_TYPES, VALID_OUTCOMES,
};
pub use store::{
    append_event, discover_mem_dir, initialize, read_events, require_mem_dir,
};
pub use summary::{build_summary, regenerate_summary};

#[cfg(test)]
mod tests;

