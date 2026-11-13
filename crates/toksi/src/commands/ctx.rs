// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/ctx.rs
// created: 2026-10-29
// lastModified: 2026-11-13

use std::io::{self, IsTerminal, Read};
use crate::util::paths::ensure_process_path;

pub fn run_ctx_command(args: &[String]) -> i32 {
    crate::contextmode::run_ctx_cli(args)
}

pub fn run_ctx_hook(args: &[String]) -> i32 {
    ensure_process_path();
    let mut input = Vec::new();
    if !io::stdin().is_terminal() {
        let _ = io::stdin().read_to_end(&mut input);
    }

    crate::contextmode::hooks::run_context_hook(args, &input)
}


