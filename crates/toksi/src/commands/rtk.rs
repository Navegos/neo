// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/rtk.rs
// created: 2026-10-27
// lastModified: 2026-11-04

use crate::rtk::engine::{list_supported_commands, run_command_filtered};
use crate::rtk::hook::run_rtk_rewrite;
use crate::util::colors::C;

pub fn rtk_help_text() -> String {
    let cy = |s: &str| C.cyan(s);
    format!(
        "{} — embedded Rust Token Killer filter engine\n\n\
        {}\n  {} <cmd> [args...]   Execute command and apply token-reduction filters\n\
          {} rewrite <cmd>       Rewrite raw command to toksi rtk equivalent\n\
          {} list                List 100+ supported command filters\n\
          {} help                Show this help message\n\n\
        {}\n  Filters 100+ commands: git, cargo, tsc, pytest, vitest, go, docker, kubectl, etc.\n\
          Zero-overhead in-process filtering and zero-allocation ANSI stripping.\n",
        C.bold(&cy("toksi rtk")),
        C.bold("Usage:"),
        cy("toksi rtk"),
        cy("toksi rtk"),
        cy("toksi rtk"),
        cy("toksi rtk"),
        C.bold("Features:"),
    )
}

pub fn run_rtk_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        println!("{}", rtk_help_text());
        return 0;
    }

    let first = &args[0];

    if first == "--help" || first == "-h" || first == "help" {
        println!("{}", rtk_help_text());
        return 0;
    }

    if first == "list" {
        println!("{}", C.bold("Supported RTK command filters (100+):"));
        for cmd in list_supported_commands() {
            println!("  {}", C.cyan(cmd));
        }
        return 0;
    }

    if first == "rewrite" {
        return run_rtk_rewrite();
    }

    let actual_args = if first == "--" && args.len() > 1 {
        &args[1..]
    } else {
        args
    };

    let cmd_line = actual_args.join(" ");
    match run_command_filtered(&cmd_line) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("toksi rtk: error executing '{}': {}", cmd_line, e);
            1
        }
    }
}

