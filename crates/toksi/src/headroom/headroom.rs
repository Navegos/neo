// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/headroom.rs
// created: 2026-10-11
// lastModified: 2026-10-31

use std::borrow::Borrow;
use crate::core::RunOpts;
use crate::util::logger::L;

pub fn ensure_installed<O: Borrow<RunOpts>>(
    opts: O,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let opts = opts.borrow();
    if opts.dry_run {
        L.sub("[dry-run] native toksi headroom engine embedded (no Python required)");
        return Ok(true);
    }
    // Fully native Rust headroom embedded into toksi. Python headroom-py dropped.
    Ok(true)
}
