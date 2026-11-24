// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/compress.rs
// created: 2026-11-24

//! CLI handler for `toksi compress <plan|kvpress|quant> ...`.

#![deny(unsafe_code)]

use crate::training::compress::{
    plan_bnb_quant, plan_kvpress, BnbQuantConfig, BnbQuantMethod, KvPressPolicy,
};

pub fn run_compress_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("plan");
    match sub {
        "plan" | "kvpress" => {
            let policy = args
                .get(1)
                .and_then(|s| KvPressPolicy::from_str_loose(s))
                .unwrap_or(KvPressPolicy::SnapKv);
            let ratio = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.5);
            let ctx = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(32768);
            let plan = plan_kvpress(policy, ratio, ctx);
            println!("{}", serde_json::to_string_pretty(&plan).unwrap_or_default());
            0
        }
        "quant" => {
            let model = args.get(1).map(String::as_str).unwrap_or("llama-3.1-8b");
            let params_b = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(8.0);
            let method = match args.get(3).map(String::as_str) {
                Some("int8") => BnbQuantMethod::Int8Llm,
                Some("fp4") => BnbQuantMethod::Fp4,
                _ => BnbQuantMethod::Nf4,
            };
            let cfg = BnbQuantConfig {
                method,
                ..Default::default()
            };
            let plan = plan_bnb_quant(model, params_b, &cfg);
            println!("{}", serde_json::to_string_pretty(&plan).unwrap_or_default());
            0
        }
        _ => {
            eprintln!("Usage: toksi compress <plan|kvpress|quant> [args...]");
            2
        }
    }
}
