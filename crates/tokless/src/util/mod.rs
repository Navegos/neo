// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/mod.rs
// created: 2026-10-05
// lastModified: 2026-10-30

pub mod byok;
pub mod colors;
pub mod deps;
pub mod exec;
pub mod grok;
pub mod headroom;
pub mod headroom_proxy;
pub mod install;
pub mod instructions;
pub mod isatty;
pub mod jsonc;
pub mod logger;
pub mod mcpspawn;
pub mod paths;
pub mod pathsetup;
pub mod preflight;
pub mod progress;
pub mod prompt;
pub mod routing_coordinator;
pub mod routing_lock;
pub mod routing_plan;
pub mod routing_state;
pub mod routing_store;
pub mod rtk_commands;
pub mod selection;
pub mod toml;
pub mod version;
pub mod versions;

pub use byok::*;
pub use colors::{
    colors_enabled, glyphs_enabled, interactive_tty, pick, rule, running_inside_claude_code,
    set_vt_ready, stdin_is_tty, stdout_ansi, stdout_is_tty, vt_ready, Colors, Sym, Symbols, C,
    SYM,
};
pub use deps::{detect_runtimes, pick_js_runner, report_runtimes, RuntimeReport};
pub use exec::*;
pub use grok::*;
pub use headroom::{
    headroom_bin, headroom_env, headroom_installed, headroom_installed_version,
    headroom_paths_resolved, HeadroomPaths,
};
pub use headroom_proxy::*;
pub use install::*;
pub use instructions::*;
pub use isatty::is_terminal;
pub use jsonc::*;
pub use logger::{quiet, set_quiet, set_verbose, with_quiet, Logger, L};
pub use mcpspawn::*;
pub use paths::{
    amp_paths_resolved, antigravity_paths_resolved, claude_code_paths, clear_home_override,
    clear_write_file_override, cline_paths_resolved, codex_paths_resolved, copilot_paths_resolved,
    copy_dir_merge, cursor_global_mcp_path, cursor_paths_resolved, ensure_dir, exists, home,
    is_go_test_executable, is_test_executable, is_win, kilo_paths_resolved, kimi_paths_resolved,
    open_code_paths_resolved, opencode_config_dir_v2, opencode_paths_resolved, opencode_paths_v2,
    persisted_tokless_command, read_file_safe, set_home_override, set_write_file_override,
    sh_quote, split_command, stable_executable, temp_dir, tokless_abs, tokless_abs_strict,
    tokless_data_dir, tokless_persisted_abs, which, write_file, write_file_atomic,
    write_file_mode, zcode_paths_resolved, AmpPaths, AntigravityPaths, ClaudePaths, ClinePaths,
    CodexPaths, CopilotPaths, CursorPaths, KiloPaths, KimiPaths, OpenCodePaths, ZcodePaths, IS_WIN,
};
pub use pathsetup::*;
pub use preflight::*;
pub use progress::{
    capture_logs, erase_styled_line, new_progress, new_root_section_progress,
    new_section_progress, pad_end, tree_close, tree_corner, tree_corner_styled, tree_footer,
    tree_leaf, tree_stem, tree_top, visible_len, with_silenced_logs, NewProgress,
    NewRootSectionProgress, NewSectionProgress, Progress, RootSectionProgress, SectionProgress,
};
pub use prompt::*;
pub use routing_coordinator::*;
pub use routing_lock::*;
pub use routing_plan::*;
pub use routing_state::*;
pub use routing_store::*;
pub use rtk_commands::*;
pub use selection::*;
pub use toml::*;
pub use version::tokless_version;
pub use versions::*;
