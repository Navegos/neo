// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/ide_bridge.rs
// created: 2026-11-25

//! CLI handler for `toksi ide-bridge --ide <id> [--transport stdio|pipe|uds|sse|http]`.

#![deny(unsafe_code)]

use std::io::{self, BufRead, Write};

use serde_json::json;

use crate::ipc::WorkerRole;
use crate::tools::ide_plugin_host::{IdePluginHost, IdePluginKind, IdeTransportKind};

fn ide_bridge_help() -> String {
    "toksi ide-bridge — External 13-IDE Plugin Host Bridge\n\n\
     Usage:\n  \
       toksi ide-bridge --list\n  \
       toksi ide-bridge --ide <id> [--manifest] [--transport stdio|pipe|uds|sse|http]\n\n\
     Supported IDEs (13):\n  \
       vscode, visual-studio, jetbrains, android-studio, zed, xcode,\n  \
       eclipse, cursor, windsurf, apache-netbeans, kiro, qoder, lapce"
        .to_string()
}

pub fn run_ide_bridge_cli(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", ide_bridge_help());
        return 0;
    }

    let host = IdePluginHost::new();
    if args.iter().any(|a| a == "--list") {
        let payload = json!({
            "count": IdePluginKind::ALL.len(),
            "external_package_only": true,
            "ides": host.all_manifests(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return 0;
    }

    let mut ide_raw = "vscode".to_string();
    let mut transport_raw: Option<String> = None;
    let mut manifest_only = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--ide" if i + 1 < args.len() => {
                ide_raw = args[i + 1].clone();
                i += 2;
            }
            "--transport" if i + 1 < args.len() => {
                transport_raw = Some(args[i + 1].clone());
                i += 2;
            }
            "--manifest" => {
                manifest_only = true;
                i += 1;
            }
            s if s.starts_with("--ide=") => {
                ide_raw = s.trim_start_matches("--ide=").to_string();
                i += 1;
            }
            s if s.starts_with("--transport=") => {
                transport_raw = Some(s.trim_start_matches("--transport=").to_string());
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    let ide = match ide_raw.parse::<IdePluginKind>() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let transport = match transport_raw {
        Some(raw) => match raw.parse::<IdeTransportKind>() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        },
        None => ide.default_transport(),
    };

    let manifest = host.manifest_for(ide);
    if manifest_only || transport != IdeTransportKind::Stdio {
        let ports = crate::config::resolved_ports();
        let descriptor = json!({
            "status": "ready",
            "ide": ide.as_str(),
            "transport": transport.as_str(),
            "ipc_endpoint": WorkerRole::Port9098.ipc_endpoint(),
            "http_base": format!("http://127.0.0.1:{}/ide", ports.protocol_bus),
            "sse_stream": format!("http://127.0.0.1:{}/sse", ports.protocol_bus),
            "manifest": manifest,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&descriptor).unwrap_or_else(|_| "{}".to_string())
        );
        return 0;
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(raw) = line else {
            break;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(req) => host.handle_jsonrpc(&req),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": serde_json::Value::Null,
                "error": { "code": -32700, "message": format!("Invalid JSON: {e}") }
            }),
        };
        let _ = writeln!(
            stdout,
            "{}",
            serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string())
        );
        let _ = stdout.flush();
    }
    0
}
