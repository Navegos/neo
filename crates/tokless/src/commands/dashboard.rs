// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/dashboard.rs

use crate::dashboard::run_dashboard_cli;

pub fn run_dashboard(argv: &[String]) -> i32 {
    run_dashboard_cli(argv)
}

