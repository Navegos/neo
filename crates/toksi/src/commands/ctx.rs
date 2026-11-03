// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/ctx.rs
// created: 2026-10-29
// lastModified: 2026-10-31

use std::io::{self, IsTerminal, Read};
use crate::util::paths::ensure_process_path;
use crate::util::jsonc::try_parse_jsonc;

pub fn run_ctx_command(args: &[String]) -> i32 {
    crate::contextmode::run_ctx_cli(args)
}

pub fn run_ctx_hook(args: &[String]) -> i32 {
    ensure_process_path();
    let mut input = Vec::new();
    if !io::stdin().is_terminal() {
        let _ = io::stdin().read_to_end(&mut input);
    }

    if !input.is_empty() {
        let raw = String::from_utf8_lossy(&input);
        let _ = try_parse_jsonc(&raw);
    }

    let _agent = args.first().map(|s| s.as_str()).unwrap_or("generic");
    let _event = args.get(1).map(|s| s.as_str()).unwrap_or("pretooluse");

    // Exit 0 with clean passthrough
    0
}


