// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/orchestrator.rs
// created: 2026-11-24

//! CLI handler for `toksi orchestrator <modes|route|set|reset>`.

#![deny(unsafe_code)]

use crate::agent_runtime::mode_router::{
    resolve_mode_spec, OrchestratorMode, ALL_ORCHESTRATOR_MODES,
};
use crate::db::{open_default_db, OrchestratorModeRouteRecord};

pub fn run_orchestrator_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("modes");
    match sub {
        "modes" => {
            for mode in ALL_ORCHESTRATOR_MODES {
                let spec = resolve_mode_spec(mode);
                println!(
                    "  * {:<22} [{:<28}] primary={:<24} effort={} ({})",
                    spec.mode_id,
                    spec.domain.label(),
                    spec.primary_model,
                    spec.reasoning_effort.as_str(),
                    spec.source
                );
            }
            println!("Total modes: {} across 7 Cognitive Domains", ALL_ORCHESTRATOR_MODES.len());
            0
        }
        "route" => {
            let Some(mode_name) = args.get(1) else {
                eprintln!("Usage: toksi orchestrator route <mode>");
                return 1;
            };
            let Some(mode) = OrchestratorMode::from_str_loose(mode_name) else {
                eprintln!("Unknown orchestrator mode '{mode_name}'");
                return 1;
            };
            let spec = resolve_mode_spec(mode);
            println!("{}", serde_json::to_string_pretty(&spec).unwrap_or_default());
            0
        }
        "set" => {
            let (Some(mode_name), Some(model)) = (args.get(1), args.get(2)) else {
                eprintln!("Usage: toksi orchestrator set <mode> <primary_model> [effort]");
                return 1;
            };
            let Some(mode) = OrchestratorMode::from_str_loose(mode_name) else {
                eprintln!("Unknown orchestrator mode '{mode_name}'");
                return 1;
            };
            let effort = args
                .get(3)
                .map(String::as_str)
                .unwrap_or_else(|| mode.default_spec().reasoning_effort.as_str());
            let Ok(mut db) = open_default_db() else {
                eprintln!("Failed to open ~/.toksi/toksi.db");
                return 1;
            };
            let def = mode.default_spec();
            let rec = OrchestratorModeRouteRecord {
                mode_id: mode.as_str().to_string(),
                domain: mode.domain().key().to_string(),
                primary_model: model.clone(),
                fallback_model: def.fallback_model,
                reasoning_effort: effort.to_ascii_lowercase(),
                temperature: def.temperature,
                top_p: def.top_p,
                top_k: def.top_k.map(i64::from),
                updated_at: chrono::Utc::now().timestamp(),
            };
            if let Err(e) = db.upsert_orchestrator_mode_route(&rec) {
                eprintln!("Failed persisting orchestrator route: {e}");
                return 1;
            }
            println!("Updated mode '{}' -> primary='{}' effort='{}'", rec.mode_id, rec.primary_model, rec.reasoning_effort);
            0
        }
        "reset" => {
            let Some(mode_name) = args.get(1) else {
                eprintln!("Usage: toksi orchestrator reset <mode>");
                return 1;
            };
            let Ok(mut db) = open_default_db() else {
                eprintln!("Failed to open ~/.toksi/toksi.db");
                return 1;
            };
            let _ = db.delete_orchestrator_mode_route(mode_name);
            println!("Reset orchestrator mode '{mode_name}' override.");
            0
        }
        _ => {
            eprintln!("Usage: toksi orchestrator <modes|route <mode>|set <mode> <model> [effort]|reset <mode>>");
            1
        }
    }
}
