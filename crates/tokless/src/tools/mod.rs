// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/mod.rs
// created: 2026-10-05
// lastModified: 2026-10-14

pub mod autoindex;
pub mod caveman;
pub mod codegraph;
pub mod contextmode;
pub mod copilot_transaction;
pub mod headroom;
pub mod helpers;
pub mod ponytail;
pub mod principles;
pub mod projectmem;
pub mod rtk;
pub mod separators;
pub mod tokless_block;

use crate::core::register_tool;

pub use autoindex::*;
pub use caveman::*;
pub use codegraph::*;
pub use contextmode::*;
pub use copilot_transaction::*;
pub use headroom::*;
pub use helpers::*;
pub use ponytail::*;
pub use principles::*;
pub use projectmem::*;
pub use rtk::*;
pub use separators::*;

pub fn register() {
    register_tool(build_rtk());
    register_tool(build_caveman());
    register_tool(build_ponytail());
    register_tool(build_principles());
    register_tool(build_codegraph());
    register_tool(build_context_mode());
    register_tool(build_headroom());
    register_tool(build_projectmem());
}
