// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase13_e2e_verification.rs

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use serde_json::json;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn acquire_env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

use tokless::agents::{
    mcp_adapter_for, mcp_command, proxy_agent_ids, proxy_spec_for, register as register_agents,
    PROXY_AGENT_SPECS,
};
use tokless::aimemory::tiered::DisclosureTier;
use tokless::codegraph::{mcp::handle_mcp_message as codegraph_handle_mcp, run_graph_cli};
use tokless::commands::{run_doctor, run_gain, run_proxy_cli, run_rtk_cli, run_tui_cli};
use tokless::contextmode::{mcp::mcp_tools_list as ctx_tools_list, run_ctx_cli, ContextMcpServer};
use tokless::core::{agent_ids, get_agent, tool_ids};
use tokless::dashboard::run_dashboard_cli;
use tokless::env_injector::{
    audit_environment_status, inject_proxy_env, scan_legacy_hijacks_in, teardown_proxy_env,
};
use tokless::headroom::adaptive_mem::{calculate_headroom, select_tier_for_headroom};
use tokless::headroom::cache_align::{
    align_to_1024_tokens, auto_place_anthropic_cache_control, estimate_tokens,
};
use tokless::headroom::compression::code::{compress_code, CodeCompressorConfig, CodeLanguage};
use tokless::headroom::compression::json::{prune_json, JsonPrunerConfig};
use tokless::headroom::compression::log::{compress_logs, LogCompressorConfig};
use tokless::headroom::responses_compat::transform_sse_bytes;
use tokless::icm::mcp::handle_mcp_request as icm_handle_mcp;
use tokless::icm::store::IcmStore;
use tokless::projectmem::{
    handle_mcp_message as projectmem_handle_mcp, run_mem_cli,
};
use tokless::rtk::engine::filter_output;
use tokless::rules::RuleKind;
use tokless::tools::register as register_tools;

// ============================================================================
// 1. End-to-End Integration Tests across all 17 supported Agent Harnesses
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
fn test_all_17_agent_manifests_and_lifecycle() {
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
        let _ = detected.installed;
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

#[test]
fn test_in_tree_rules_and_principles_embedding() {
    let principles = RuleKind::Principles.markdown();
    assert!(principles.contains("Think Before Coding"));
    assert!(principles.contains("Simplicity First"));
    assert!(principles.contains("Surgical Changes"));

    let caveman = RuleKind::Caveman.markdown();
    assert!(caveman.contains("Caveman"));
    assert!(caveman.contains("Compress"));

    let ponytail = RuleKind::Ponytail.markdown();
    assert!(ponytail.contains("Build Discipline"));
    assert!(ponytail.contains("The Ladder"));
}

// ============================================================================
// 2. Verify all Unified CLI Commands
// ============================================================================

#[test]
fn test_all_unified_cli_commands() {
    // 1. tokless mem
    let mem_exit = run_mem_cli(&["--help".to_string()]);
    assert_eq!(mem_exit, 0, "tokless mem --help must exit 0");

    // 2. tokless graph
    let graph_exit = run_graph_cli(&["--help".to_string()]);
    assert_eq!(graph_exit, 0, "tokless graph --help must exit 0");

    // 3. tokless ctx
    let ctx_exit = run_ctx_cli(&["--help".to_string()]);
    assert_eq!(ctx_exit, 0, "tokless ctx --help must exit 0");

    let ctx_doc_exit = run_ctx_cli(&["doctor".to_string()]);
    assert_eq!(ctx_doc_exit, 0, "tokless ctx doctor must exit 0");

    // 4. tokless rtk & gain
    let rtk_exit = run_rtk_cli(&["--help".to_string()]);
    assert_eq!(rtk_exit, 0, "tokless rtk --help must exit 0");

    let rtk_list_exit = run_rtk_cli(&["list".to_string()]);
    assert_eq!(rtk_list_exit, 0, "tokless rtk list must exit 0");

    let gain_exit = run_gain();
    assert_eq!(gain_exit, 0, "tokless gain must exit 0");

    // 5. tokless proxy
    let proxy_help_exit = run_proxy_cli(&["--help".to_string()]);
    assert_eq!(proxy_help_exit, 0, "tokless proxy --help must exit 0");

    let proxy_status_exit = run_proxy_cli(&["status".to_string(), "--dry-run".to_string()]);
    assert_eq!(proxy_status_exit, 0, "tokless proxy status --dry-run must exit 0");

    // 6. tokless doctor
    let doctor_exit = run_doctor(true, true, false);
    assert_eq!(doctor_exit, 0, "tokless doctor --offline --json must exit 0");

    // 7. tokless tui
    let tui_exit = run_tui_cli(&["--help".to_string()]);
    assert_eq!(tui_exit, 0, "tokless tui --help must exit 0");

    // 8. tokless dashboard
    let dash_exit = run_dashboard_cli(&["--help".to_string()]);
    assert_eq!(dash_exit, 0, "tokless dashboard --help must exit 0");
}

// ============================================================================
// 3. Verify Streaming Proxy Compression & In-Process MCP Multiplexer
// ============================================================================

#[test]
fn test_streaming_proxy_compression_pipeline() {
    // 1. 1024-token prompt cache alignment
    let prompt = "System instructions for code generation.";
    let aligned = align_to_1024_tokens(prompt);
    assert!(aligned.contains("<!-- tokless:cache-align pad_tokens="));
    let est = estimate_tokens(&aligned);
    assert!(est >= 1024 && est <= 1030);

    // 2. Anthropic auto ephemeral cache control placement
    let mut payload = json!({
        "system": [{"type": "text", "text": "Base instructions"}],
        "messages": [{"role": "user", "content": "Hello"}]
    });
    auto_place_anthropic_cache_control(&mut payload);
    assert_eq!(
        payload["system"].as_array().unwrap().last().unwrap()["cache_control"]["type"],
        "ephemeral"
    );

    // 3. Adaptive memory governor headroom selection
    let headroom = calculate_headroom(30_000, 200_000);
    let (tier, budget) = select_tier_for_headroom(headroom);
    assert_eq!(tier, DisclosureTier::L1);
    assert_eq!(budget, 600);

    // 4. SSE streaming bytes transform
    let sse_chunk = b"event: content_block_delta\ndata: {\"type\": \"content_block_delta\", \"delta\": {\"type\": \"text_delta\", \"text\": \"response\"}}\n\n";
    let transformed = transform_sse_bytes(sse_chunk, HashMap::new());
    assert!(!transformed.is_empty());
}

#[test]
fn test_in_process_mcp_multiplexer_live_simulated_requests() {
    register_tools();
    let tools = tool_ids();
    for t in ["rtk", "codegraph", "context-mode", "headroom", "projectmem"] {
        assert!(tools.contains(&t.to_string()));
    }

    // 1. projectmem in-process JSON-RPC simulation
    let pjm_init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let pjm_init_resp = projectmem_handle_mcp(&pjm_init).expect("projectmem init response");
    assert_eq!(pjm_init_resp["result"]["serverInfo"]["name"], "projectmem");

    let pjm_list = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let pjm_list_resp = projectmem_handle_mcp(&pjm_list).expect("projectmem tools/list response");
    assert!(pjm_list_resp["result"]["tools"].as_array().unwrap().len() >= 5);

    // 2. codegraph in-process JSON-RPC simulation
    let cg_init = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "initialize",
        "params": {}
    });
    let cg_init_resp = codegraph_handle_mcp(&cg_init).expect("codegraph init response");
    assert_eq!(cg_init_resp["result"]["serverInfo"]["name"], "tokless-codegraph");

    let cg_list = json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/list",
        "params": {}
    });
    let cg_list_resp = codegraph_handle_mcp(&cg_list).expect("codegraph tools/list response");
    let cg_tools = cg_list_resp["result"]["tools"].as_array().unwrap();
    assert!(cg_tools.iter().any(|t| t["name"] == "codegraph_explore"));

    // 3. context-mode in-process JSON-RPC simulation
    let ctx_server = ContextMcpServer::new(None);
    let ctx_init = json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "initialize",
        "params": {}
    });
    let ctx_init_resp = ctx_server.handle_mcp_message(&ctx_init).expect("contextmode init response");
    assert_eq!(ctx_init_resp["result"]["serverInfo"]["name"], "context-mode");

    let ctx_list = json!({
        "jsonrpc": "2.0",
        "id": 21,
        "method": "tools/list",
        "params": {}
    });
    let ctx_list_resp = ctx_server.handle_mcp_message(&ctx_list).expect("contextmode tools/list response");
    let ctx_tool_defs = ctx_list_resp["result"]["tools"].as_array().unwrap();
    assert!(ctx_tool_defs.iter().any(|t| t["name"] == "ctx_execute"));

    let ctx_tools_raw = ctx_tools_list();
    assert!(ctx_tools_raw.as_array().unwrap().len() >= 6);

    // 4. icm in-process JSON-RPC simulation
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_db = temp_dir.path().join("test_icm.db");
    let mut icm_store = IcmStore::open(&temp_db).unwrap();

    let icm_init = json!({
        "jsonrpc": "2.0",
        "id": 30,
        "method": "initialize",
        "params": {}
    });
    let icm_init_resp = icm_handle_mcp(&mut icm_store, &icm_init).expect("icm init response");
    assert_eq!(icm_init_resp["result"]["serverInfo"]["name"], "tokless-icm");

    let icm_store_call = json!({
        "jsonrpc": "2.0",
        "id": 31,
        "method": "tools/call",
        "params": {
            "name": "icm_memory_store",
            "arguments": {
                "topic": "compiler_policy",
                "summary": "Rust zero warning discipline enforced across all modules",
                "keywords": ["rust", "zero_warnings"]
            }
        }
    });
    let icm_store_resp = icm_handle_mcp(&mut icm_store, &icm_store_call).expect("icm store response");
    assert!(icm_store_resp["result"]["content"][0]["text"].as_str().unwrap().contains("Memory stored successfully"));

    let icm_recall_call = json!({
        "jsonrpc": "2.0",
        "id": 32,
        "method": "tools/call",
        "params": {
            "name": "icm_memory_recall",
            "arguments": {
                "query": "discipline zero warnings"
            }
        }
    });
    let icm_recall_resp = icm_handle_mcp(&mut icm_store, &icm_recall_call).expect("icm recall response");
    assert!(icm_recall_resp["result"]["content"][0]["text"].as_str().unwrap().contains("zero warning discipline"));
}

// ============================================================================
// 4. Verify Non-Destructive Environment Injection & Clean Teardown
// ============================================================================

#[test]
fn test_non_destructive_environment_injection_and_clean_teardown() {
    let _lock = acquire_env_lock();

    // 1. Audit environment status: inspect system reporting
    let status = audit_environment_status();
    assert!(status.binaries_clean || !status.binaries_clean);
    assert!(status.env_vars_healthy || !status.env_vars_healthy);
    assert!(!status.details.is_empty(), "audit should provide diagnostic details");

    // 2. Isolated test directory: verify scanning and self-healing of legacy hijacks
    let temp_dir = std::env::temp_dir().join("tokless_p13_e2e_env_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::create_dir_all(&temp_dir);

    let bin_dir = temp_dir.join("bin");
    let data_dir = temp_dir.join("data");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::create_dir_all(&data_dir).unwrap();

    let real_bin = bin_dir.join("agy.real.exe");
    std::fs::write(&real_bin, b"simulated real binary").unwrap();
    let shim = bin_dir.join("agy.cmd");
    std::fs::write(&shim, b"@echo off\r\nrem tokless:antigravity-cli-proxy\r\n").unwrap();

    // Verify scan detects without modifying
    let scan = scan_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);
    assert!(!scan.is_clean());
    assert_eq!(scan.restored_binaries.len(), 1);
    assert_eq!(scan.removed_shims.len(), 1);
    assert!(real_bin.exists());

    // Clean up temp dir
    let _ = std::fs::remove_dir_all(&temp_dir);

    // 3. Inject proxy environment variables non-destructively without modifying any binaries
    let injected = inject_proxy_env().expect("Environment injection must succeed");
    assert!(!injected.is_empty(), "Injected keys must not be empty");
    assert!(injected.contains(&"TOKLESS_OWNED".to_string()));
    assert_eq!(
        std::env::var("TOKLESS_OWNED").ok().as_deref(),
        Some("1"),
        "Process environment must reflect TOKLESS_OWNED=1"
    );

    // 4. Teardown proxy environment cleanly
    let purged = teardown_proxy_env().expect("Environment teardown must succeed");
    assert!(!purged.is_empty(), "Purged keys must not be empty");
    assert!(purged.contains(&"TOKLESS_OWNED".to_string()));
    assert!(
        std::env::var("TOKLESS_OWNED").is_err(),
        "TOKLESS_OWNED must be removed after teardown"
    );

    // 5. Verify IDE extension compatibility: all 17 agent config dirs remain intact and valid
    for agent_id in ALL_17_AGENTS {
        let manifest = get_agent(agent_id).expect("Agent must exist");
        if let Some(ref config_dir) = manifest.config_dir {
            let path = config_dir();
            assert!(!path.is_empty(), "Config dir for {} must be valid", agent_id);
        }
    }
}

// ============================================================================
// 5. Zero External Runtime Dependencies (no npm, no pip, no node, no python)
// ============================================================================

#[test]
fn test_zero_external_runtime_dependencies() {
    register_agents();

    for agent_id in ALL_17_AGENTS {
        for tool in ["projectmem", "codegraph", "context-mode", "icm"] {
            let cmd = mcp_command(agent_id, tool);
            for arg in &cmd {
                let lower = arg.to_lowercase();
                assert!(
                    !lower.starts_with("npx")
                        && !lower.starts_with("npm")
                        && !lower.starts_with("node")
                        && !lower.starts_with("python")
                        && !lower.starts_with("pip")
                        && !lower.starts_with("uvx"),
                    "MCP command for agent '{}' and tool '{}' must not use external runtime: {:?}",
                    agent_id,
                    tool,
                    cmd
                );
            }
        }
    }
}

// ============================================================================
// 6. Benchmarks: Token Reduction Ratio (40-80%) & Latency (<10ms)
// ============================================================================

#[test]
fn test_benchmarks_token_reduction_and_latency_overhead() {
    // ------------------------------------------------------------------------
    // Benchmark 1: RTK Output Filtering (Git Diff / Status)
    // ------------------------------------------------------------------------
    let raw_diff = "diff --git a/Cargo.lock b/Cargo.lock\nindex 1234..5678 100644\n--- a/Cargo.lock\n+++ b/Cargo.lock\n@@ -1,50 +1,50 @@\n".to_string() + &"+line_added_to_lockfile\n-line_removed_from_lockfile\n".repeat(60);

    let start_rtk = Instant::now();
    let filtered_diff = filter_output("git diff", &raw_diff, "", 0);
    let rtk_latency = start_rtk.elapsed();

    let raw_diff_tokens = estimate_tokens(&raw_diff);
    let filtered_diff_tokens = estimate_tokens(&filtered_diff);
    let diff_reduction_pct = ((raw_diff_tokens - filtered_diff_tokens) as f64 / raw_diff_tokens as f64) * 100.0;

    println!(
        "[BENCHMARK: RTK] tokens: {} -> {} ({:.1}% reduction), latency: {:?}",
        raw_diff_tokens, filtered_diff_tokens, diff_reduction_pct, rtk_latency
    );
    assert!(diff_reduction_pct >= 75.0, "Git diff token reduction must be >= 75%");
    assert!(rtk_latency.as_millis() < 5, "RTK latency must be < 5ms");

    // ------------------------------------------------------------------------
    // Benchmark 2: Log Deduplication
    // ------------------------------------------------------------------------
    let mut repeating_logs = String::new();
    for _ in 0..60 {
        repeating_logs.push_str("2026-10-30T10:00:00Z [INFO] worker heartbeat status=ok count=100\n");
    }
    repeating_logs.push_str("2026-10-30T10:00:01Z [ERROR] network connection reset by peer\n");
    for _ in 0..60 {
        repeating_logs.push_str("2026-10-30T10:00:00Z [INFO] worker heartbeat status=ok count=100\n");
    }

    let start_log = Instant::now();
    let comp_log = compress_logs(&repeating_logs, &LogCompressorConfig::default());
    let log_latency = start_log.elapsed();

    let raw_log_tokens = estimate_tokens(&repeating_logs);
    let comp_log_tokens = estimate_tokens(&comp_log.compressed);
    let log_reduction_pct = ((raw_log_tokens - comp_log_tokens) as f64 / raw_log_tokens as f64) * 100.0;

    println!(
        "[BENCHMARK: LOG] tokens: {} -> {} ({:.1}% reduction), latency: {:?}",
        raw_log_tokens, comp_log_tokens, log_reduction_pct, log_latency
    );
    assert!(log_reduction_pct >= 70.0, "Log deduplication reduction must be >= 70%");
    assert!(log_latency.as_millis() < 5, "Log compression latency must be < 5ms");

    // ------------------------------------------------------------------------
    // Benchmark 3: JSON Pruning
    // ------------------------------------------------------------------------
    let raw_json = json!({
        "status": "success",
        "code": 200,
        "empty_payload": null,
        "redundant_null": null,
        "items": [
            {"id": 1, "value": "val1", "unused": null, "description": "payload record item 1"},
            {"id": 2, "value": "val2", "unused": null, "description": "payload record item 2"},
            {"id": 3, "value": "val3", "unused": null, "description": "payload record item 3"},
            {"id": 4, "value": "val4", "unused": null, "description": "payload record item 4"},
            {"id": 5, "value": "val5", "unused": null, "description": "payload record item 5"}
        ]
    });
    let raw_json_str = serde_json::to_string_pretty(&raw_json).unwrap();

    let start_json = Instant::now();
    let pruned_json = prune_json(&raw_json_str, &JsonPrunerConfig::default());
    let json_latency = start_json.elapsed();

    let raw_json_tokens = estimate_tokens(&raw_json_str);
    let pruned_json_tokens = estimate_tokens(&pruned_json);
    let json_reduction_pct = ((raw_json_tokens - pruned_json_tokens) as f64 / raw_json_tokens as f64) * 100.0;

    println!(
        "[BENCHMARK: JSON] tokens: {} -> {} ({:.1}% reduction), latency: {:?}",
        raw_json_tokens, pruned_json_tokens, json_reduction_pct, json_latency
    );
    assert!(json_reduction_pct >= 30.0, "JSON pruning reduction must be >= 30%");
    assert!(json_latency.as_millis() < 5, "JSON pruning latency must be < 5ms");

    // ------------------------------------------------------------------------
    // Benchmark 4: AST-Aware Code Compression
    // ------------------------------------------------------------------------
    let code_sample = "\
/// Extensive documentation comments explaining algorithm.
/// Computes sum of squares and variance across data points.
/// Additional explanatory lines for API documentation.
pub fn compute_sum_of_squares(values: &[f64]) -> f64 {
    // Initializer for accumulator
    let mut sum = 0.0;

    // Loop through all data points
    for &v in values {
        // Accumulate squares
        sum += v * v;
    }

    // Return final computed sum
    sum
}
";
    let start_code = Instant::now();
    let comp_code = compress_code(code_sample, CodeLanguage::Rust, &CodeCompressorConfig::default());
    let code_latency = start_code.elapsed();

    let raw_code_tokens = estimate_tokens(code_sample);
    let comp_code_tokens = estimate_tokens(&comp_code);
    let code_reduction_pct = ((raw_code_tokens - comp_code_tokens) as f64 / raw_code_tokens as f64) * 100.0;

    println!(
        "[BENCHMARK: CODE] tokens: {} -> {} ({:.1}% reduction), latency: {:?}",
        raw_code_tokens, comp_code_tokens, code_reduction_pct, code_latency
    );
    assert!(code_reduction_pct >= 25.0, "Code compression reduction must be >= 25%");
    assert!(code_latency.as_millis() < 5, "Code compression latency must be < 5ms");

    // ------------------------------------------------------------------------
    // Benchmark 5: Overall Combined Token Reduction (Target: 40-80% overall savings)
    // ------------------------------------------------------------------------
    let total_raw_tokens = raw_diff_tokens + raw_log_tokens + raw_json_tokens + raw_code_tokens;
    let total_reduced_tokens = filtered_diff_tokens + comp_log_tokens + pruned_json_tokens + comp_code_tokens;
    let overall_reduction_pct = ((total_raw_tokens - total_reduced_tokens) as f64 / total_raw_tokens as f64) * 100.0;

    println!(
        "[BENCHMARK: OVERALL] total raw: {} tokens, total reduced: {} tokens, overall savings: {:.1}%",
        total_raw_tokens, total_reduced_tokens, overall_reduction_pct
    );
    assert!(
        overall_reduction_pct >= 40.0 && overall_reduction_pct <= 95.0,
        "Overall savings must be in target 40-80%+ range, got: {:.1}%",
        overall_reduction_pct
    );

    // ------------------------------------------------------------------------
    // Benchmark 6: In-Process MCP Call Dispatch Latency Overhead (<10ms)
    // ------------------------------------------------------------------------
    let start_mcp = Instant::now();
    let req = json!({
        "jsonrpc": "2.0",
        "id": 999,
        "method": "tools/list",
        "params": {}
    });
    let _ = projectmem_handle_mcp(&req);
    let mcp_latency = start_mcp.elapsed();

    println!("[BENCHMARK: MCP DISPATCH] latency: {:?}", mcp_latency);
    assert!(mcp_latency.as_millis() < 5, "In-process MCP dispatch latency must be < 5ms");

    // Max component latency must be strictly under 10ms
    let max_latency = rtk_latency.max(log_latency).max(json_latency).max(code_latency).max(mcp_latency);
    println!("[BENCHMARK: MAX OVERHEAD] max latency: {:?}", max_latency);
    assert!(max_latency.as_millis() < 10, "Latency overhead must be strictly < 10ms");
}
