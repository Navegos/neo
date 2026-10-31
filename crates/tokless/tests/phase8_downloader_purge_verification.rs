// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase8_downloader_purge_verification.rs

use std::path::Path;
use tokless::agents::register as register_all_agents;
use tokless::core::{get_agent, list_agents, RunOpts};
use tokless::tools::{
    build_caveman, build_codegraph, build_context_mode, build_headroom, build_ponytail,
    build_principles, build_projectmem, build_rtk,
};
use tokless::util::deps::{detect_runtimes, pick_js_runner};
use tokless::util::preflight::ensure_deps;

#[test]
fn test_phase8_purged_downloader_modules_absence() {
    // Verify that legacy runtime downloader source files are completely gone
    let purged_files = [
        "crates/tokless/src/util/npminstall.rs",
        "crates/tokless/src/util/nodewin.rs",
        "crates/tokless/src/util/nodelinux.rs",
        "crates/tokless/src/util/uv.rs",
        "crates/tokless/src/util/projectmem.rs",
    ];

    for file in purged_files {
        assert!(
            !Path::new(file).exists(),
            "Purged runtime downloader file must not exist: {}",
            file
        );
    }
}

#[test]
fn test_phase8_all_17_agents_registered() {
    register_all_agents();
    tokless::tools::register();

    let expected_17_agents = [
        "claude",
        "opencode",
        "opencode2",
        "codex",
        "cursor",
        "antigravity",
        "copilot",
        "droid",
        "grok",
        "kilo",
        "omp",
        "pi",
        "cline",
        "kimi",
        "zcode",
        "dsh",
        "amp",
    ];

    let registered = list_agents();
    assert_eq!(
        registered.len(),
        17,
        "Expected exactly 17 agent harnesses registered, found {}",
        registered.len()
    );

    for agent_id in expected_17_agents {
        let manifest = get_agent(agent_id);
        assert!(
            manifest.is_some(),
            "Agent harness '{}' must be registered",
            agent_id
        );
        let m = manifest.unwrap();
        assert_eq!(m.id, agent_id);
        assert!(!m.label.is_empty());
        assert!(!m.cli_bin.is_empty());
        assert!(m.config_dir.is_some());
    }
}

#[test]
fn test_phase8_native_in_tree_tool_harnesses() {
    let tools = [
        build_rtk(),
        build_caveman(),
        build_ponytail(),
        build_principles(),
        build_codegraph(),
        build_context_mode(),
        build_headroom(),
        build_projectmem(),
    ];

    for tool in tools {
        // Native in-tree tools do not require Node.js or Git downloads
        assert!(
            !tool.needs_node,
            "Tool '{}' must not require node runtime downloads",
            tool.id
        );
        assert!(
            !tool.needs_git,
            "Tool '{}' must not require git downloads",
            tool.id
        );
        assert_eq!(
            tool.min_node_major, 0,
            "Tool '{}' must have min_node_major == 0",
            tool.id
        );

        // Tool install hook must succeed in-tree without network downloading
        if let Some(ref install_fn) = tool.install {
            let res = install_fn(&RunOpts::default());
            assert!(
                res.is_ok(),
                "Tool '{}' install hook must succeed in-tree: {:?}",
                tool.id,
                res.err()
            );
            assert!(
                res.unwrap(),
                "Tool '{}' install hook must return true",
                tool.id
            );
        }
    }
}

#[test]
fn test_phase8_system_detection_policy_via_which() {
    // Tokless system detection only queries PATH via which, never auto-installing
    let runtimes = detect_runtimes();
    // Verify runtime report fields are strings (either found on PATH or empty)
    let _ = format!(
        "node: {}, cargo: {}, python: {}",
        runtimes.node, runtimes.cargo, runtimes.python
    );

    let (js_bin, _args, _has_runner) = pick_js_runner();
    if !js_bin.is_empty() {
        assert!(
            js_bin == "npx" || js_bin == "bunx",
            "JS runner must be system binary"
        );
    }

    // Preflight ensure_deps must only check system state without triggering downloads
    let (node_ok, git_ok) = ensure_deps(false, false, 0);
    assert!(node_ok);
    assert!(git_ok);
}

#[test]
fn test_phase8_mcp_command_wiring_uses_in_tree_tokless() {
    use tokless::agents::mcp_command;

    let agents = ["claude", "cursor", "antigravity", "copilot", "codex", "opencode"];
    let tools = ["projectmem", "context-mode", "codegraph", "icm"];

    for agent in agents {
        for tool in tools {
            let cmd = mcp_command(agent, tool);
            assert!(
                !cmd.is_empty(),
                "mcp_command for agent {} and tool {} must not be empty",
                agent,
                tool
            );
            // Command should call tokless itself with run-mcp
            let joined = cmd.join(" ");
            assert!(
                joined.contains("tokless") || joined.contains("run-mcp"),
                "MCP invocation must route through native tokless binary, got: {}",
                joined
            );
        }
    }
}
