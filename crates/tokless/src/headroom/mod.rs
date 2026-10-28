// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/mod.rs
// created: 2026-10-05
// lastModified: 2026-10-14

pub mod adaptive_mem;
pub mod autostart;
pub mod byok_gateway;
pub mod cache_align;
pub mod cloudcode_patch;
pub mod compression;
pub mod copilot_daemon;
pub mod grok_daemon;
pub mod headroom;
pub mod process_identity;
pub mod proxy;
pub mod responses_compat;
pub mod router;
pub mod watchdog;

pub use adaptive_mem::*;
pub use autostart::*;
pub use byok_gateway::*;
pub use cache_align::*;
pub use cloudcode_patch::*;
pub use compression::*;
pub use copilot_daemon::{
    copilot_proxy_args, copilot_proxy_files, copilot_proxy_live_z, copilot_proxy_owned,
    copilot_proxy_ownership_valid, copilot_proxy_port, copilot_proxy_running,
    rollback_copilot_proxy, start_copilot_proxy, stop_copilot_proxy, DEFAULT_COPILOT_PROXY_PORT,
};
pub use grok_daemon::*;
pub use headroom::*;
pub use process_identity::*;
pub use proxy::*;
pub use responses_compat::*;
pub use router::*;
pub use watchdog::{
    cleanup_supervised_process, cleanup_supervised_process_after_wait, clear_proxy_stop_request,
    proxy_stop_request_file, proxy_stop_requested, remove_supervised_records, request_proxy_stop,
    run_headroom_supervised,
};

#[cfg(test)]
mod tests;
