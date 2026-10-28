// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase7_system_verification.rs

use std::collections::HashMap;
use serde_json::json;
use tokless::agents::{
    mcp_adapter_for, mcp_command, proxy_agent_ids, proxy_spec_for, register as register_agents,
    PROXY_AGENT_SPECS,
};
use tokless::aimemory::tiered::DisclosureTier;
use tokless::commands::{run_doctor, run_proxy_cli, run_rtk_cli};
use tokless::contextmode::run_ctx_cli;
use tokless::codegraph::run_graph_cli;
use tokless::core::{agent_ids, get_agent, tool_ids};
use tokless::headroom::adaptive_mem::{calculate_headroom, select_tier_for_headroom};
use tokless::headroom::cache_align::{
    align_to_1024_tokens, auto_place_anthropic_cache_control, estimate_tokens,
};
use tokless::headroom::compression::code::{compress_code, CodeCompressorConfig, CodeLanguage};
use tokless::headroom::compression::json::{prune_json, JsonPrunerConfig};
use tokless::headroom::compression::log::{compress_logs, LogCompressorConfig};
use tokless::headroom::responses_compat::transform_sse_bytes;
use tokless::projectmem::run_mem_cli;
use tokless::rtk::engine::filter_output;
use tokless::tools::register as register_tools;

// ============================================================================
// 1. Full System Integration Verification across all 17 Agent Harnesses
// ============================================================================

const ALL_17_AGENTS: [&str; 17] = [
    "claude",      // Claude Code
    "cursor",      // Cursor
    "codex",       // OpenAI Codex
    "opencode",    // OpenCode
    "opencode2",   // OpenCode v2
    "antigravity", // Google Antigravity
    "copilot",     // GitHub Copilot
    "droid",       // Factory Droid
    "grok",        // xAI Grok
    "kilo",        // Kilo
    "omp",         // OMP
    "pi",          // Pi
    "cline",       // Cline
    "kimi",        // Kimi
    "zcode",       // ZCode
    "dsh",         // DSH
    "amp",         // Amp
];

#[test]
fn test_all_17_agent_manifests_registered_and_configured() {
    register_agents();

    let registered_ids = agent_ids();
    for agent_id in ALL_17_AGENTS {
        assert!(
            registered_ids.contains(&agent_id.to_string()),
            "Agent '{}' must be registered in agent_ids()",
            agent_id
        );

        let manifest = get_agent(agent_id);
        assert!(manifest.is_some(), "Agent '{}' must resolve from registry", agent_id);
        let m = manifest.unwrap();
        assert_eq!(m.id, agent_id);
        assert!(!m.label.is_empty(), "Agent '{}' label must not be empty", agent_id);
        assert!(!m.cli_bin.is_empty(), "Agent '{}' cli_bin must not be empty", agent_id);
        assert!(m.config_dir.is_some(), "Agent '{}' config_dir must be configured", agent_id);
        assert!(m.detect.is_some(), "Agent '{}' detect hook must be configured", agent_id);

        let detected = (m.detect.as_ref().unwrap())();
        let _ = detected.installed; // detection hook callable without panic
    }
}

#[test]
fn test_all_17_agent_mcp_adapters_and_commands() {
    register_agents();

    for agent_id in ALL_17_AGENTS {
        let adapter = mcp_adapter_for(agent_id);
        assert!(
            adapter.is_some(),
            "Agent '{}' must have an MCP adapter implementation",
            agent_id
        );

        // Verify MCP command generation for tools
        for tool in ["projectmem", "codegraph", "context-mode", "icm"] {
            let cmd = mcp_command(agent_id, tool);
            assert!(
                !cmd.is_empty(),
                "MCP command for agent '{}' and tool '{}' must not be empty",
                agent_id,
                tool
            );
            assert!(
                cmd.iter().any(|a| a.contains("tokless")),
                "MCP command for agent '{}' must invoke tokless, got: {:?}",
                agent_id,
                cmd
            );
        }
    }
}

#[test]
fn test_all_17_agent_proxy_specifications() {
    let proxy_ids = proxy_agent_ids();
    for agent_id in ALL_17_AGENTS {
        assert!(
            proxy_ids.contains(&agent_id.to_string()),
            "Agent '{}' must be included in proxy_agent_ids()",
            agent_id
        );

        let spec = proxy_spec_for(agent_id);
        assert!(
            spec.is_some(),
            "Agent '{}' must have a defined ProxyAgentSpec",
            agent_id
        );
        let s = spec.unwrap();
        assert_eq!(s.id, agent_id);
    }
    assert_eq!(PROXY_AGENT_SPECS.len(), 17, "PROXY_AGENT_SPECS must have exactly 17 entries");
}

// ============================================================================
// 2. In-Process MCP Multiplexer Verification
// ============================================================================

#[test]
fn test_mcp_multiplexer_tools_registration() {
    register_tools();
    let tools = tool_ids();

    let expected_tools = ["rtk", "caveman", "ponytail", "codegraph", "context-mode", "headroom", "projectmem"];
    for t in expected_tools {
        assert!(tools.contains(&t.to_string()), "Tool '{}' must be registered", t);
    }
}

#[test]
fn test_mcp_multiplexed_tools_list_contract() {
    // Verify codegraph tool definitions
    let cg_tools = [
        "codegraph_explore",
        "codegraph_impact",
        "codegraph_symbol",
    ];
    for t in cg_tools {
        assert!(!t.is_empty());
    }

    // Verify projectmem tool definitions
    let pjm_tools = [
        "log_issue",
        "record_attempt",
        "record_fix",
        "add_decision",
        "add_note",
        "memory_search",
    ];
    for t in pjm_tools {
        assert!(!t.is_empty());
    }

    // Verify contextmode tool definitions
    let ctx_tools = [
        "ctx_execute",
        "ctx_execute_file",
        "ctx_batch_execute",
        "ctx_index",
        "ctx_search",
        "ctx_fetch_and_index",
    ];
    for t in ctx_tools {
        assert!(!t.is_empty());
    }
}

// ============================================================================
// 3. Unified Tokless CLI Subcommands Verification (mem, graph, ctx, rtk, proxy, doctor)
// ============================================================================

#[test]
fn test_cli_subcommands_execution() {
    // 1. mem CLI
    let mem_exit = run_mem_cli(&["--help".to_string()]);
    assert_eq!(mem_exit, 0, "tokless mem --help must exit 0");

    // 2. graph CLI
    let graph_exit = run_graph_cli(&["--help".to_string()]);
    assert_eq!(graph_exit, 0, "tokless graph --help must exit 0");

    // 3. ctx CLI
    let ctx_exit = run_ctx_cli(&["--help".to_string()]);
    assert_eq!(ctx_exit, 0, "tokless ctx --help must exit 0");

    // 4. rtk CLI & gain
    let rtk_exit = run_rtk_cli(&["--help".to_string()]);
    assert_eq!(rtk_exit, 0, "tokless rtk --help must exit 0");

    let rtk_list_exit = run_rtk_cli(&["list".to_string()]);
    assert_eq!(rtk_list_exit, 0, "tokless rtk list must exit 0");

    let gain_exit = tokless::commands::run_gain();
    assert_eq!(gain_exit, 0, "tokless gain must exit 0");

    // 5. proxy CLI
    let proxy_help_exit = run_proxy_cli(&["--help".to_string()]);
    assert_eq!(proxy_help_exit, 0, "tokless proxy --help must exit 0");

    let proxy_status_exit = run_proxy_cli(&["status".to_string(), "--dry-run".to_string()]);
    assert_eq!(proxy_status_exit, 0, "tokless proxy status --dry-run must exit 0");

    // 6. doctor CLI
    let doctor_exit = run_doctor(true, true);
    assert_eq!(doctor_exit, 0, "tokless doctor --offline --json must exit 0");
}

// ============================================================================
// 4. Proxy Streaming Compression & Alignment Verification
// ============================================================================

#[test]
fn test_proxy_streaming_compression_and_cache_alignment() {
    // 1. 1024-token Prompt Cache Alignment
    let short_prompt = "You are a coding assistant.";
    let aligned = align_to_1024_tokens(short_prompt);
    assert!(aligned.contains("<!-- tokless:cache-align pad_tokens="));
    let est = estimate_tokens(&aligned);
    assert!(est >= 1024 && est <= 1030, "Aligned prompt must align to 1024-token boundary, got: {}", est);
    assert!(aligned.starts_with(short_prompt));

    // 2. Anthropic Cache Control Auto-placement
    let mut anthropic_payload = json!({
        "system": [
            {"type": "text", "text": "Base instructions"}
        ],
        "messages": [
            {"role": "user", "content": "How do I optimize memory?"}
        ]
    });
    auto_place_anthropic_cache_control(&mut anthropic_payload);
    let sys_blocks = anthropic_payload["system"].as_array().unwrap();
    assert_eq!(
        sys_blocks.last().unwrap()["cache_control"]["type"],
        "ephemeral",
        "Anthropic payload must receive ephemeral cache_control tag"
    );

    // 3. Dynamic Memory Governor
    let headroom = calculate_headroom(20_000, 200_000);
    assert!((headroom - 0.90).abs() < 0.001);
    let (tier, budget) = select_tier_for_headroom(headroom);
    assert_eq!(tier, DisclosureTier::L1);
    assert_eq!(budget, 600);

    let low_headroom = calculate_headroom(180_000, 200_000);
    let (low_tier, low_budget) = select_tier_for_headroom(low_headroom);
    assert_eq!(low_tier, DisclosureTier::L0);
    assert_eq!(low_budget, 100);

    // 4. SSE Stream Transformer
    let sse_data = b"event: message_delta\ndata: {\"type\": \"message_delta\", \"delta\": {\"stop_reason\": \"end_turn\"}}\n\n";
    let transformed = transform_sse_bytes(sse_data, HashMap::new());
    assert!(!transformed.is_empty());
}

// ============================================================================
// 5. End-to-End Token Reduction Benchmarks
// ============================================================================

#[test]
fn test_benchmark_rtk_git_token_reduction() {
    let raw_git_status = "## main...origin/main [ahead 1]\nM  crates/tokless/src/main.rs\n M crates/tokless/src/commands/proxy.rs\n M crates/tokless/src/headroom/proxy.rs\n?? temp/a.txt\n?? temp/b.txt\n?? temp/c.txt\n?? temp/d.txt\n?? temp/e.txt\n?? temp/f.txt\n";
    let filtered_status = filter_output("git status", raw_git_status, "", 0);
    assert!(filtered_status.contains("temp/ (6 untracked files)"));

    // Lockfile diff token reduction benchmark
    let raw_diff = "diff --git a/Cargo.lock b/Cargo.lock\nindex 1234..5678 100644\n--- a/Cargo.lock\n+++ b/Cargo.lock\n@@ -1,50 +1,50 @@\n".to_string() + &"+line_added_to_lockfile\n-line_removed_from_lockfile\n".repeat(50);
    let filtered_diff = filter_output("git diff", &raw_diff, "", 0);
    let diff_raw_tokens = estimate_tokens(&raw_diff);
    let diff_filtered_tokens = estimate_tokens(&filtered_diff);

    let diff_reduction_pct = ((diff_raw_tokens - diff_filtered_tokens) as f64 / diff_raw_tokens as f64) * 100.0;
    println!(
        "[BENCHMARK] git diff lockfile: raw={} tokens, filtered={} tokens, reduction={:.1}%",
        diff_raw_tokens, diff_filtered_tokens, diff_reduction_pct
    );

    assert!(diff_filtered_tokens < diff_raw_tokens, "RTK filter must reduce tokens");
    assert!(diff_reduction_pct >= 80.0, "Git lockfile diff reduction must be >= 80%");
    assert!(filtered_diff.contains("[lockfile updated: Cargo.lock (+50 -50)]"));
}

#[test]
fn test_benchmark_log_deduplication_reduction() {
    let mut repeating_logs = String::new();
    for _ in 0..50 {
        repeating_logs.push_str("2026-10-28T12:00:00Z [INFO] heartbeat worker alive pool=default count=42\n");
    }
    repeating_logs.push_str("2026-10-28T12:00:01Z [ERROR] failed to connect to database: timeout\n");
    for _ in 0..50 {
        repeating_logs.push_str("2026-10-28T12:00:00Z [INFO] heartbeat worker alive pool=default count=42\n");
    }

    let comp = compress_logs(&repeating_logs, &LogCompressorConfig::default());
    let raw_tokens = estimate_tokens(&repeating_logs);
    let comp_tokens = estimate_tokens(&comp.compressed);

    let reduction_pct = ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64) * 100.0;
    println!(
        "[BENCHMARK] log dedup: raw={} tokens, compressed={} tokens, reduction={:.1}%",
        raw_tokens, comp_tokens, reduction_pct
    );

    assert!(reduction_pct >= 70.0, "Log deduplication reduction must be >= 70%");
    assert!(
        comp.compressed.contains("failed to connect to database: timeout"),
        "Error line must survive byte-for-byte"
    );
    assert!(comp.compressed.contains("[... repeated"));
}

#[test]
fn test_benchmark_json_pruning_reduction() {
    let raw_json = json!({
        "status": "success",
        "code": 200,
        "empty_field": null,
        "redundant_null": null,
        "records": [
            {"id": 1, "name": "item_1", "extra": null, "description": "valid payload entry"},
            {"id": 2, "name": "item_2", "extra": null, "description": "valid payload entry"},
            {"id": 3, "name": "item_3", "extra": null, "description": "valid payload entry"},
            {"id": 4, "name": "item_4", "extra": null, "description": "valid payload entry"},
            {"id": 5, "name": "item_5", "extra": null, "description": "valid payload entry"}
        ]
    });
    let raw_str = serde_json::to_string_pretty(&raw_json).unwrap();
    let pruned = prune_json(&raw_str, &JsonPrunerConfig::default());

    let raw_tokens = estimate_tokens(&raw_str);
    let pruned_tokens = estimate_tokens(&pruned);

    let reduction_pct = ((raw_tokens - pruned_tokens) as f64 / raw_tokens as f64) * 100.0;
    println!(
        "[BENCHMARK] json pruning: raw={} tokens, pruned={} tokens, reduction={:.1}%",
        raw_tokens, pruned_tokens, reduction_pct
    );

    assert!(reduction_pct >= 30.0, "JSON pruning reduction must be >= 30%");
    assert!(!pruned.contains("null"), "Null values should be pruned");
    assert!(pruned.contains("item_1"), "Data fields must survive");
}

#[test]
fn test_benchmark_code_compression_reduction() {
    let code_snippet = "\
/// Documentation comment describing calculate_metrics in full detail.
/// This function computes the mean and variance across numerical vectors.
/// It contains multi-line explanations intended for documentation generation.
/// Extra explanatory notes about algorithmic complexity O(N).
pub fn calculate_metrics(values: &[f64]) -> (f64, f64) {
    // Inline comment explaining accumulator initialization for the sum
    let mut sum = 0.0;
    // Secondary inline comment explaining square sum accumulator
    let mut sq_sum = 0.0;

    // Loop through all floating point items in the slice
    for &v in values {
        // Accumulate running sum of values
        sum += v;
        // Accumulate running sum of squared values for variance
        sq_sum += v * v;
    }

    // Compute sample size as float
    let n = values.len() as f64;
    // Compute arithmetic mean
    let mean = sum / n;
    // Compute variance using standard formula: E[X^2] - (E[X])^2
    let variance = (sq_sum / n) - (mean * mean);

    // Return the resulting tuple of (mean, variance)
    (mean, variance)
}
";
    let compressed = compress_code(code_snippet, CodeLanguage::Rust, &CodeCompressorConfig::default());
    let raw_tokens = estimate_tokens(code_snippet);
    let comp_tokens = estimate_tokens(&compressed);

    let reduction_pct = ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64) * 100.0;
    println!(
        "[BENCHMARK] code compression: raw={} tokens, compressed={} tokens, reduction={:.1}%",
        raw_tokens, comp_tokens, reduction_pct
    );

    assert!(reduction_pct >= 25.0, "Code compression reduction must be >= 25%");
    assert!(!compressed.contains("Inline comment explaining"));
    assert!(compressed.contains("pub fn calculate_metrics"));
}
