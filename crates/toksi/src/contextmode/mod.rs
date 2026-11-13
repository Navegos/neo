// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/mod.rs
// created: 2026-10-27
// lastModified: 2026-11-13

pub mod auto_memory;
pub mod cli;
pub mod compaction;
pub mod error;
pub mod fetch;
pub mod filter;
pub mod fts;
pub mod hooks;
pub mod journal;
pub mod mcp;
pub mod policy;
pub mod sandbox;
pub mod security;
pub mod stats;
pub mod store;
pub mod wire;

#[cfg(test)]
mod tests;

pub use auto_memory::{query_auto_memory, read_auto_memory, MemoryFile, MEMORY_LABEL_PREFIX};
pub use cli::run_ctx_cli;
pub use error::ContextModeError;
pub use fetch::{html_to_markdown, ssrf_guard, WebFetcher};
pub use filter::{filter_and_pipe_rtk, get_replace_trigger_bytes, process_sandbox_output, slice_lines};
pub use fts::{
    build_recall_query, chunk_json, chunk_markdown, chunk_plain_text, resolve_content_store_path,
    ContentStore, DirectoryIndexResult, IndexResult, SearchOptions, SearchResult,
    DEFAULT_INDEX_EXTENSIONS,
};
pub use hooks::run_context_hook;
pub use mcp::{handle_slash_command, run_mcp_server, ContextMcpServer};
pub use sandbox::{detect_runtimes, ExecRequest, ExecResult, PolyglotSandbox};
pub use security::{check_boundary, check_command_deny, check_path_deny, is_own_file, own_dirs_of, read_deny_message};
pub use stats::{get_stats, ContextStats};
pub use store::{find_repo_root_and_subproject, resolve_context_db_path, resolve_context_dir};
pub use wire::DirectWireBus;

