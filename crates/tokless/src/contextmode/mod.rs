// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/mod.rs
// created: 2026-10-27

pub mod cli;
pub mod fetch;
pub mod filter;
pub mod fts;
pub mod mcp;
pub mod sandbox;
pub mod security;
pub mod stats;

#[cfg(test)]
mod tests;

pub use cli::run_ctx_cli;
pub use fetch::{html_to_markdown, ssrf_guard, WebFetcher};
pub use filter::{filter_and_pipe_rtk, process_sandbox_output, slice_lines};
pub use fts::{chunk_json, chunk_markdown, chunk_plain_text, ContentStore, IndexResult, SearchOptions, SearchResult};
pub use mcp::{run_mcp_server, ContextMcpServer};
pub use sandbox::{detect_runtimes, ExecRequest, ExecResult, PolyglotSandbox};
pub use stats::{get_stats, ContextStats};

