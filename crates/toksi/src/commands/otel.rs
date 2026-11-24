// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/otel.rs
// created: 2026-11-24

//! `toksi otel` CLI command (`status`, `traces`, `export`, `collector-config`, `operator-crd`).

#![deny(unsafe_code)]

use crate::cluster::{render_otel_collector_yaml, render_otel_operator_crd};
use crate::db::service_dbs::{service_db_path, ServiceDbKind};
use crate::telemetry::otel::OtlpJsonExporter;
use crate::util::paths::normalize_path;

fn flag_val<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].as_str())
}

pub fn run_otel_cli(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.trim()).unwrap_or("status");
    let json_mode = args.iter().any(|a| a == "--json");

    match sub {
        "status" => {
            let db_path = normalize_path(&service_db_path(ServiceDbKind::Otel).to_string_lossy());
            let count = OtlpJsonExporter::span_count();
            if json_mode {
                println!(
                    "{}",
                    serde_json::json!({
                        "service_db_path": db_path,
                        "span_count": count,
                        "gen_ai_semconv": "1.29",
                        "fail_open": true,
                    })
                );
            } else {
                println!("OTel LLMOps Status");
                println!("  Service DB : {}", db_path);
                println!("  Spans      : {}", count);
                println!("  SemConv    : gen_ai.* (1.29) + toksi.*");
                println!("  Mode       : fail-open non-blocking OTLP/HTTP JSON");
            }
            0
        }
        "traces" => {
            let limit = flag_val(args, "--limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(20);
            let spans = OtlpJsonExporter::recent_spans(limit);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "count": spans.len(),
                    "spans": spans,
                }))
                .unwrap_or_else(|_| "{}".to_string())
            );
            0
        }
        "export" => {
            let limit = flag_val(args, "--limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(50);
            let endpoint = flag_val(args, "--endpoint");
            let spans = OtlpJsonExporter::recent_spans(limit);
            let res = OtlpJsonExporter::export_non_blocking_fail_open(endpoint, &spans);
            println!(
                "{}",
                serde_json::to_string_pretty(&res).unwrap_or_else(|_| "{}".to_string())
            );
            0
        }
        "collector-config" => {
            let otlp_port = flag_val(args, "--otlp-port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(4318);
            let prom_port = flag_val(args, "--prom-port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(9464);
            print!("{}", render_otel_collector_yaml(otlp_port, prom_port));
            0
        }
        "operator-crd" => {
            let name = flag_val(args, "--name").unwrap_or("toksi-otel");
            let namespace = flag_val(args, "--namespace").unwrap_or("toksi");
            let mode = flag_val(args, "--mode").unwrap_or("deployment");
            print!("{}", render_otel_operator_crd(name, namespace, mode));
            0
        }
        _ => {
            eprintln!(
                "Usage: toksi otel <status|traces|export|collector-config|operator-crd> [flags]"
            );
            2
        }
    }
}
