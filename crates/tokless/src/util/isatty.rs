// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/isatty.rs
// created: 2026-10-05
// lastModified: 2026-10-14

pub fn is_terminal(fd: u32) -> bool {
    match fd {
        0 => crossterm::tty::IsTty::is_tty(&std::io::stdin()),
        1 => crossterm::tty::IsTty::is_tty(&std::io::stdout()),
        2 => crossterm::tty::IsTty::is_tty(&std::io::stderr()),
        _ => false,
    }
}

pub fn is_interactive() -> bool {
    is_terminal(0) && is_terminal(1)
}

pub fn stdout_is_tty() -> bool {
    is_terminal(1)
}

