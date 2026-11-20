// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/plugin.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! CLI dispatcher for `toksi plugin <list|load|unload|reload|info|scan|enable|disable>`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::plugins::{global_plugin_registry, PluginSecurityStatus};
use crate::util::colors::C;

pub fn run_plugin_cli(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
    let target = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let reg = global_plugin_registry();

    match sub {
        "list" | "ls" => {
            let list = reg.list();
            println!("{}", C.bold("Toksi Unified Plugin Registry:"));
            if list.is_empty() {
                println!("  {}", C.gray("No plugins registered."));
            } else {
                for p in list {
                    println!(
                        "  * {}@{} [{}] scan={}",
                        C.cyan(&p.manifest.id),
                        p.manifest.version,
                        if p.is_enabled { "enabled" } else { "disabled" },
                        p.security_status.as_str()
                    );
                }
            }
            0
        }
        "load" if !target.is_empty() => match reg.load(target) {
            Ok(p) => {
                println!("Loaded plugin '{}' (status={}).", p.manifest.id, p.security_status.as_str());
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        "unload" if !target.is_empty() => match reg.unload(target) {
            Ok(p) => {
                println!("Unloaded plugin '{}'.", p.manifest.id);
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        "reload" if !target.is_empty() => match reg.reload(target) {
            Ok(p) => {
                println!("Reloaded plugin '{}'.", p.manifest.id);
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        "enable" if !target.is_empty() => match reg.enable(target) {
            Ok(p) => {
                println!("Enabled plugin '{}'.", p.manifest.id);
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        "disable" if !target.is_empty() => match reg.disable(target) {
            Ok(p) => {
                println!("Disabled plugin '{}'.", p.manifest.id);
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        "info" if !target.is_empty() => match reg.get(target) {
            Some(p) => {
                println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
                0
            }
            None => {
                eprintln!("Plugin '{}' not found.", target);
                1
            }
        },
        "scan" if !target.is_empty() => match reg.scan_path(Path::new(target)) {
            Ok(rep) => {
                println!(
                    "Plugin '{}' scan={} findings={}",
                    rep.plugin_id,
                    rep.status.as_str(),
                    rep.findings.len()
                );
                if rep.status == PluginSecurityStatus::BlockedFailClosed {
                    1
                } else {
                    0
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        },
        _ => {
            eprintln!("Usage: toksi plugin <list|load <id|path>|unload <id>|reload <id>|info <id>|scan <path>|enable <id>|disable <id>>");
            1
        }
    }
}
