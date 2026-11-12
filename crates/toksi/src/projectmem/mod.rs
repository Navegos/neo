// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/mod.rs
// created: 2026-10-27
// lastModified: 2026-11-12

#![deny(unsafe_code)]

pub mod cli;
pub mod error;
pub mod global;
pub mod heatmap;
pub mod mcp;
pub mod precheck;
pub mod redaction;
pub mod schema;
pub mod slash;
pub mod staleness;
pub mod store;
pub mod structure;
pub mod summary;

pub use error::{ProjectMemError, Result};
pub use cli::run_mem_cli;
pub use global::{
    add_gotcha, add_pattern, auto_promote_event, build_inherited_instructions, detect_stack,
    get_relevant_entries, global_dir, read_gotchas, read_patterns, DetectedStack, GlobalGotcha,
    GlobalPattern, ProjectRecord, Registry, list_registered_projects, register_project, resolve_project,
};
pub use heatmap::{calculate_score, generate_heatmap, DrilldownEvent, FileDrilldown, HeatmapResult, ScoreResult};
pub use mcp::{handle_mcp_message, run_mcp_server};
pub use precheck::run_precheck;
pub use redaction::{is_redaction_enabled, redact, redact_event_fields};
pub use schema::{
    auto_resolve_location, default_event_id, location_to_file, normalize_timestamp,
    parse_location_coordinate, resolve_event_ref, resolve_location_symbol, superseded_ids,
    utc_now_iso, Attempt, Decision, Event, Fix, Issue, Note, VALID_CAPTURE_SOURCES,
    VALID_CONFIDENCE_LEVELS, VALID_EVENT_TYPES, VALID_OUTCOMES,
};
pub use slash::handle_slash_command;
pub use staleness::{commits_touching_since, find_stale_events, stale_label, StaleEvent, STALE_COMMIT_THRESHOLD};
pub use store::{
    append_event, discover_mem_dir, initialize, read_events, read_last_failure_trace,
    require_mem_dir, write_last_failure_trace, LastFailureTrace,
};
pub use structure::{
    build_structure, generate_and_save_structure, StructureNode, StructureRelationship,
    StructureResult, StructureStats,
};
pub use summary::{build_summary, regenerate_summary};

#[cfg(test)]
mod tests;
