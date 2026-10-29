// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/ctx.rs
// created: 2026-10-27

pub fn run_ctx_command(args: &[String]) -> i32 {
    crate::contextmode::run_ctx_cli(args)
}

