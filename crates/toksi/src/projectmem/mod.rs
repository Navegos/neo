// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/mod.rs
// created: 2026-10-27
// lastModified: 2026-11-18

#![deny(unsafe_code)]

pub mod cli;
pub mod error;
pub mod global;
pub mod heatmap;
pub mod mcp;
pub mod plan_checkpoint;
pub mod precheck;
pub mod redaction;
pub mod schema;
pub mod slash;
pub mod staleness;
pub mod store;
pub mod structure;
pub mod summary;
pub mod wiki_sync;
pub mod wire;

pub use error::{ProjectMemError, Result};
pub use cli::run_mem_cli;
pub use global::{
    add_gotcha, add_pattern, auto_promote_event, build_inherited_instructions, detect_stack,
    get_relevant_entries, global_dir, read_gotchas, read_patterns, DetectedStack, GlobalGotcha,
    GlobalPattern, ProjectRecord, Registry, list_registered_projects, register_project, resolve_project,
};
pub use heatmap::{calculate_score, generate_heatmap, DrilldownEvent, FileDrilldown, HeatmapResult, ScoreResult};
pub use mcp::{handle_mcp_message, run_mcp_server};
pub use plan_checkpoint::{
    checkpoints_dir, delete_plan_checkpoint, latest_resumable_checkpoint, list_plan_checkpoints,
    load_plan_checkpoint, save_plan_checkpoint, CheckpointStatus, PlanCheckpoint,
};
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
    append_event, consolidate_nested_mem_dirs, derive_subproject_id, discover_mem_dir, initialize,
    read_events, read_last_failure_trace, require_mem_dir, write_last_failure_trace,
    LastFailureTrace,
};
pub use structure::{
    build_structure, generate_and_save_structure, StructureNode, StructureRelationship,
    StructureResult, StructureStats,
};
pub use summary::{build_summary, regenerate_summary};
pub use wiki_sync::sync_project_map_from_structure;
pub use wire::{DirectWireBus, WireMessage};

#[cfg(test)]
mod tests;
