// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/serve.rs
// created: 2026-11-24

//! CLI dispatcher for `toksi serve <list|profile> [engine] [model] [effort]`.

#![deny(unsafe_code)]

use crate::gateway::local_engines::SECTION_2_7_LOCAL_ENGINES;
use crate::headroom::effort::EffortLevel;
use crate::training::build_serving_profile;

pub fn run_serve_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => {
            for e in SECTION_2_7_LOCAL_ENGINES {
                println!(
                    "  * {} (port={}, pure_rust={}, openai_compat={})",
                    e.id, e.default_port, e.pure_rust, e.openai_compat
                );
            }
            0
        }
        "profile" => {
            let engine = args.get(1).map(String::as_str).unwrap_or("oxillama");
            let model = args.get(2).map(String::as_str).unwrap_or("local-model");
            let effort = args
                .get(3)
                .and_then(|s| EffortLevel::from_str_loose(s))
                .unwrap_or_default();
            match build_serving_profile(engine, model, effort) {
                Ok(p) => {
                    println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
                    0
                }
                Err(e) => {
                    eprintln!("serve profile error: {e}");
                    1
                }
            }
        }
        _ => {
            eprintln!("Usage: toksi serve <list|profile [engine] [model] [effort]>");
            1
        }
    }
}
